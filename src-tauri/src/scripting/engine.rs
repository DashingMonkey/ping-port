use crate::scripting::isolation::ScriptIsolation;
use crate::scripting::pm_api::{PmApi, TestResult};
use rquickjs::{CaughtError, CatchResultExt, Context, Ctx, Function, Object, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::{Arc, LazyLock, Mutex};
use std::time::{Duration, Instant};

/// Thread-safe container for test results collected during script execution
#[derive(Clone)]
struct TestResults(Arc<Mutex<Vec<TestResult>>>);

/// Console log storage
#[derive(Clone)]
struct ConsoleLogs(Arc<Mutex<Vec<String>>>);

/// Variables storage with get/set/unset/replace methods
#[derive(Clone)]
struct JsVariables {
    #[allow(clippy::arc_with_non_send_sync)]
    data: Arc<RefCell<HashMap<String, String>>>,
}

impl JsVariables {
    fn new() -> Self {
        Self {
            #[allow(clippy::arc_with_non_send_sync)]
            data: Arc::new(RefCell::new(HashMap::new())),
        }
    }
}

/// Request object exposed to JS
#[derive(Clone)]
struct JsRequest {
    method: String,
    url: RefCell<String>,
    body: Option<String>,
    query_params: RefCell<std::collections::HashMap<String, String>>,
}

impl JsRequest {
    fn new(
        method: String,
        url: String,
        body: Option<String>,
        query_params: std::collections::HashMap<String, String>,
    ) -> Self {
        Self {
            method,
            url: RefCell::new(url),
            body,
            query_params: RefCell::new(query_params),
        }
    }
}

/// Response object exposed to JS
#[derive(Clone)]
struct JsResponse {
    status: u16,
    body: Option<String>,
    time: u64,
    headers: HashMap<String, String>,
}

impl JsResponse {
    fn new(status: u16, body: Option<String>, time: u64, headers: HashMap<String, String>) -> Self {
        Self {
            status,
            body,
            time,
            headers,
        }
    }
}

/// Main pp API object exposed to JavaScript
struct JsPmApi {
    variables: JsVariables,
    #[allow(clippy::arc_with_non_send_sync)]
    request: Arc<RefCell<Option<JsRequest>>>,
    #[allow(clippy::arc_with_non_send_sync)]
    response: Arc<RefCell<Option<JsResponse>>>,
    test_results: TestResults,
    console_logs: ConsoleLogs,
}

impl JsPmApi {
    fn new() -> Self {
        Self {
            variables: JsVariables::new(),
            #[allow(clippy::arc_with_non_send_sync)]
            request: Arc::new(RefCell::new(None)),
            #[allow(clippy::arc_with_non_send_sync)]
            response: Arc::new(RefCell::new(None)),
            test_results: TestResults(Arc::new(Mutex::new(Vec::new()))),
            console_logs: ConsoleLogs(Arc::new(Mutex::new(Vec::new()))),
        }
    }

    fn from_pm_api(pm_api: &PmApi) -> Self {
        let js_pm = Self::new();

        *js_pm.variables.data.borrow_mut() = pm_api.variables.data.clone();

        if let Some(ref req) = pm_api.request {
            *js_pm.request.borrow_mut() = Some(JsRequest::new(
                req.method.clone(),
                req.url.clone(),
                req.body.clone(),
                req.query_params.clone(),
            ));
        }

        if let Some(ref resp) = pm_api.response {
            *js_pm.response.borrow_mut() = Some(JsResponse::new(
                resp.status,
                resp.body.clone(),
                resp.time,
                resp.headers.clone(),
            ));
        }

        js_pm
    }
}

/// Per-tab Runtime manager for script isolation
struct RuntimeManager {
    #[allow(dead_code)]
    runtime: rquickjs::Runtime,
    context: Context,
    isolation: ScriptIsolation,
    deadline: Arc<Mutex<Option<Instant>>>,
}

impl RuntimeManager {
    fn new() -> Result<Self, rquickjs::Error> {
        let runtime = rquickjs::Runtime::new()?;

        // Set up interrupt handler for timeout enforcement
        let deadline = Arc::new(Mutex::new(None::<Instant>));
        let deadline_clone = deadline.clone();
        runtime.set_interrupt_handler(Some(Box::new(move || {
            if let Ok(guard) = deadline_clone.lock() {
                if let Some(deadline) = *guard {
                    return Instant::now() > deadline;
                }
            }
            false
        })));

        // Apply sandbox resource limits
        runtime.set_memory_limit(10 * 1024 * 1024); // 10 MB
        runtime.set_max_stack_size(64 * 1024); // 64 KB

        let context = Context::full(&runtime)?;
        Ok(Self {
            runtime,
            context,
            isolation: ScriptIsolation::new(),
            deadline,
        })
    }

    fn execute_pre_request(&mut self, script: &str, mut pm_api: PmApi) -> Result<PmApi, String> {
        if script.trim().is_empty() {
            return Ok(pm_api);
        }

        log::info!(
            "Executing pre-request script: {}",
            script.chars().take(200).collect::<String>()
        );
        self.isolation.start();
        let deadline = Instant::now() + Duration::from_millis(self.isolation.max_time_ms);
        if let Ok(mut guard) = self.deadline.lock() {
            *guard = Some(deadline);
        }
        let js_pm = JsPmApi::from_pm_api(&pm_api);

        let js_pm_result: Result<JsPmApi, String> = self.context.with(|ctx| {
            log::info!("Creating pm object...");
            let js_pm =
                create_pm_object(&ctx, js_pm).map_err(|e| format!("PM object error: {}", e))?;
            log::info!("PM object created successfully");
            log::info!("Evaluating script...");
            let scoped_script = format!("(function() {{ {} }})();", script);
            let _: () = ctx.eval(scoped_script.as_bytes()).catch(&ctx).map_err(|e| {
                let msg = format_caught_error(&e);
                log::error!("Script eval error: {}", msg);
                format!("Script error: {}", msg)
            })?;
            log::info!("Script evaluated successfully");
            Ok(js_pm)
        });
        let js_pm = js_pm_result.map_err(|e| {
            log::error!("JS execution failed: {}", e);
            e
        })?;

        if let Some(ref mut req) = pm_api.request {
            if let Some(js_req) = js_pm.request.borrow().as_ref() {
                req.url = js_req.url.borrow().clone();
                req.method = js_req.method.clone();
                if let Some(ref body) = js_req.body {
                    req.body = Some(body.clone());
                }
                req.query_params = js_req.query_params.borrow().clone();
            }
        }
        pm_api.variables.data = js_pm.variables.data.borrow().clone();

        let logs = js_pm
            .console_logs
            .0
            .lock()
            .map_err(|e| format!("Internal error: {}", e))?;
        pm_api.console_logs = logs.clone();
        for log in logs.iter() {
            log::info!("[JS] {}", log);
        }

        Ok(pm_api)
    }

    fn execute_test(&mut self, script: &str, mut pm_api: PmApi) -> Result<PmApi, String> {
        if script.trim().is_empty() {
            return Ok(pm_api);
        }

        self.isolation.start();
        let deadline = Instant::now() + Duration::from_millis(self.isolation.max_time_ms);
        if let Ok(mut guard) = self.deadline.lock() {
            *guard = Some(deadline);
        }
        let js_pm = JsPmApi::from_pm_api(&pm_api);

        let js_pm_result: Result<JsPmApi, String> = self.context.with(|ctx| {
            let js_pm =
                create_pm_object(&ctx, js_pm).map_err(|e| format!("PM object error: {}", e))?;
            let scoped_script = format!("(function() {{ {} }})();", script);
            ctx.eval::<(), _>(scoped_script.as_bytes())
                .catch(&ctx)
                .map_err(|e| format!("Script error: {}", format_caught_error(&e)))?;
            Ok(js_pm)
        });

        let js_pm = js_pm_result?;

        let results = js_pm
            .test_results
            .0
            .lock()
            .map_err(|e| format!("Internal error: {}", e))?;
        pm_api.test_results = results.clone();

        let logs = js_pm
            .console_logs
            .0
            .lock()
            .map_err(|e| format!("Internal error: {}", e))?;
        pm_api.console_logs = logs.clone();
        for log in logs.iter() {
            log::info!("[JS] {}", log);
        }

        Ok(pm_api)
    }
}

/// Command sent to the script thread
enum ScriptCommand {
    PreRequest {
        script: String,
        pm_api: PmApi,
    },
    Test {
        script: String,
        pm_api: PmApi,
    },
    #[allow(dead_code)]
    Shutdown,
}

struct ScriptResult(Result<PmApi, String>);

/// Script engine backed by a dedicated thread that owns the JS runtime.
/// The runtime is `!Send`, so it must stay on a single thread.
pub struct ScriptEngine {
    sender: mpsc::Sender<(ScriptCommand, mpsc::Sender<ScriptResult>)>,
}

fn script_thread(
    rx: mpsc::Receiver<(ScriptCommand, mpsc::Sender<ScriptResult>)>,
) {
    let mut manager = match RuntimeManager::new() {
        Ok(m) => m,
        Err(e) => {
            log::error!("Failed to create script runtime: {}", e);
            // Drain remaining commands and return errors
            for (cmd, reply) in rx {
                let msg = format!("Script runtime unavailable: {}", e);
                let result = Err(msg);
                let _ = reply.send(ScriptResult(result));
                if matches!(cmd, ScriptCommand::Shutdown) {
                    break;
                }
            }
            return;
        }
    };

    for (cmd, reply) in rx {
        let result = match cmd {
            ScriptCommand::PreRequest { script, pm_api } => {
                manager.execute_pre_request(&script, pm_api)
            }
            ScriptCommand::Test { script, pm_api } => {
                manager.execute_test(&script, pm_api)
            }
            ScriptCommand::Shutdown => break,
        };
        let _ = reply.send(ScriptResult(result));
    }
}

impl ScriptEngine {
    fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        std::thread::Builder::new()
            .name("pingport-script-engine".into())
            .spawn(move || script_thread(rx))
            .expect("Failed to spawn script engine thread");
        ScriptEngine { sender: tx }
    }

    fn execute(&self, cmd: ScriptCommand) -> Result<PmApi, String> {
        let (reply_tx, reply_rx) = mpsc::channel();
        self.sender
            .send((cmd, reply_tx))
            .map_err(|e| format!("Script engine unavailable: {}", e))?;
        let ScriptResult(result) = reply_rx
            .recv()
            .map_err(|e| format!("Script engine unavailable: {}", e))?;
        result
    }

    pub fn execute_pre_request(script: &str, pm_api: PmApi) -> Result<PmApi, String> {
        get_engine().execute(ScriptCommand::PreRequest {
            script: script.to_string(),
            pm_api,
        })
    }

    pub fn execute_test(script: &str, pm_api: PmApi) -> Result<PmApi, String> {
        get_engine().execute(ScriptCommand::Test {
            script: script.to_string(),
            pm_api,
        })
    }
}

static SCRIPT_ENGINE: LazyLock<ScriptEngine> =
    LazyLock::new(|| ScriptEngine::new());

fn get_engine() -> &'static ScriptEngine {
    &SCRIPT_ENGINE
}

fn create_pm_object<'js>(ctx: &Ctx<'js>, pm: JsPmApi) -> anyhow::Result<JsPmApi> {
    let pp_obj = Object::new(ctx.clone())?;

    // pp.variables
    {
        let variables_obj = Object::new(ctx.clone())?;
        let vars_data_arc = pm.variables.data.clone();
        variables_obj.set(
            "get",
            Function::new(ctx.clone(), move |name: String| -> Option<String> {
                vars_data_arc.borrow().get(&name).cloned()
            }),
        )?;

        let vars_data_arc = pm.variables.data.clone();
        variables_obj.set(
            "set",
            Function::new(ctx.clone(), move |name: String, value: String| {
                vars_data_arc.borrow_mut().insert(name, value);
            }),
        )?;

        let vars_data_arc = pm.variables.data.clone();
        variables_obj.set(
            "unset",
            Function::new(ctx.clone(), move |name: String| {
                vars_data_arc.borrow_mut().remove(&name);
            }),
        )?;

        let vars_data_arc = pm.variables.data.clone();
        variables_obj.set(
            "replace",
            Function::new(ctx.clone(), move |name: String, value: String| {
                vars_data_arc.borrow_mut().insert(name, value);
            }),
        )?;

        pp_obj.set("variables", variables_obj)?;
    }

    // pp.request
    {
        let request_obj = Object::new(ctx.clone())?;

        let request_arc = pm.request.clone();
        request_obj.set(
            "getUrl",
            Function::new(ctx.clone(), move || -> String {
                if let Some(ref req) = *request_arc.borrow() {
                    req.url.borrow().clone()
                } else {
                    String::new()
                }
            }),
        )?;

        let request_arc = pm.request.clone();
        request_obj.set(
            "setUrl",
            Function::new(ctx.clone(), move |new_url: String| {
                if let Some(ref req) = *request_arc.borrow() {
                    *req.url.borrow_mut() = new_url;
                }
            }),
        )?;

        let request_arc = pm.request.clone();
        request_obj.set(
            "addQueryParam",
            Function::new(ctx.clone(), move |key: String, value: String| {
                if let Some(ref req) = *request_arc.borrow() {
                    req.query_params.borrow_mut().insert(key, value);
                }
            }),
        )?;

        let request_arc = pm.request.clone();
        request_obj.set(
            "removeQueryParam",
            Function::new(ctx.clone(), move |key: String| {
                if let Some(ref req) = *request_arc.borrow() {
                    req.query_params.borrow_mut().remove(&key);
                }
            }),
        )?;

        let request_arc = pm.request.clone();
        request_obj.set(
            "hasQueryParam",
            Function::new(ctx.clone(), move |key: String| -> bool {
                if let Some(ref req) = *request_arc.borrow() {
                    return req.query_params.borrow().contains_key(&key);
                }
                false
            }),
        )?;

        let request_arc = pm.request.clone();
        request_obj.set(
            "getQueryParam",
            Function::new(ctx.clone(), move |key: String| -> Option<String> {
                if let Some(ref req) = *request_arc.borrow() {
                    return req.query_params.borrow().get(&key).cloned();
                }
                None
            }),
        )?;

        if let Some(ref req) = *pm.request.borrow() {
            request_obj.set("method", req.method.clone())?;
            request_obj.set("body", req.body.clone().unwrap_or_default())?;
        } else {
            request_obj.set("method", "GET")?;
            request_obj.set("body", "")?;
        }

        pp_obj.set("request", request_obj)?;
    }

    // pp.response
    {
        let response_obj = Object::new(ctx.clone())?;
        let response = pm.response.clone();

        if let Some(ref resp) = *pm.response.borrow() {
            response_obj.set("status", resp.status)?;
            response_obj.set("body", resp.body.clone().unwrap_or_default())?;
            response_obj.set("time", resp.time)?;

            let headers_obj = Object::new(ctx.clone())?;
            for (k, v) in &resp.headers {
                headers_obj.set(k, v.clone())?;
            }
            response_obj.set("headers", headers_obj)?;

            response_obj.set(
                "json",
                Function::new(ctx.clone(), move || -> Option<String> {
                    if let Some(ref resp) = *response.borrow() {
                        if let Some(ref body) = resp.body {
                            if serde_json::from_str::<serde_json::Value>(body).is_ok() {
                                return Some(body.clone());
                            }
                        }
                    }
                    None
                }),
            )?;
        } else {
            response_obj.set("status", 0)?;
            response_obj.set("body", "")?;
            response_obj.set("time", 0)?;
            response_obj.set("headers", Object::new(ctx.clone())?)?;
            response_obj.set(
                "json",
                Function::new(ctx.clone(), move || -> Option<String> { None }),
            )?;
        }
        pp_obj.set("response", response_obj)?;
    }

    // pp.test
    {
        let test_results = pm.test_results.clone();
        pp_obj.set(
            "test",
            Function::new(ctx.clone(), move |name: String, fn_: Function<'js>| {
                let start = Instant::now();
                let (passed, error_msg) = match fn_.call::<_, ()>(()) {
                    Ok(()) => (true, None),
                    Err(e) => (false, Some(e.to_string())),
                };
                let duration = start.elapsed().as_millis() as u64;
                if let Ok(mut results) = test_results.0.lock() {
                    results.push(TestResult {
                        name,
                        passed,
                        error: error_msg,
                        duration,
                    });
                }
            }),
        )?;
    }

    // pp.sleep (capped at 5000ms to prevent thread blocking DoS)
    {
        pp_obj.set(
            "sleep",
            Function::new(ctx.clone(), move |ms: u32| {
                let capped = std::cmp::min(ms as u64, 5000);
                std::thread::sleep(std::time::Duration::from_millis(capped));
            }),
        )?;
    }

    // Console (log collector)
    {
        let console_obj = Object::new(ctx.clone())?;
        let logs = pm.console_logs.clone();
        let logs_for_log = logs.clone();
        console_obj.set(
            "log",
            Function::new(ctx.clone(), move |args: Value| {
                if let Ok(mut logs) = logs_for_log.0.lock() {
                    logs.push(value_to_string(&args));
                }
            }),
        )?;
        let logs_for_info = logs.clone();
        console_obj.set(
            "info",
            Function::new(ctx.clone(), move |args: Value| {
                if let Ok(mut logs) = logs_for_info.0.lock() {
                    logs.push(format!("[INFO] {}", value_to_string(&args)));
                }
            }),
        )?;
        let logs_for_error = logs.clone();
        console_obj.set(
            "error",
            Function::new(ctx.clone(), move |args: Value| {
                if let Ok(mut logs) = logs_for_error.0.lock() {
                    logs.push(format!("[ERROR] {}", value_to_string(&args)));
                }
            }),
        )?;
        let logs_for_warn = logs.clone();
        console_obj.set(
            "warn",
            Function::new(ctx.clone(), move |args: Value| {
                if let Ok(mut logs) = logs_for_warn.0.lock() {
                    logs.push(format!("[WARN] {}", value_to_string(&args)));
                }
            }),
        )?;
        ctx.globals().set("console", console_obj)?;
    }

    ctx.globals().set("pp", pp_obj)?;

    // Define pp.expect using JavaScript
    let pp_expect_js = r#"
        pp.expect = function(actual) {
            var actualStr = String(actual);
            return {
                toEqual: function(expected) {
                    if (actualStr !== String(expected)) {
                        throw new Error("Expected '" + String(expected) + "' but got '" + actualStr + "'");
                    }
                },
                toBe: function(expected) {
                    if (actualStr !== String(expected)) {
                        throw new Error("Expected '" + String(expected) + "' but got '" + actualStr + "'");
                    }
                },
                toContain: function(expected) {
                    if (!actualStr.includes(String(expected))) {
                        throw new Error("'" + actualStr + "' does not contain '" + String(expected) + "'");
                    }
                },
                toBeTruthy: function() {
                    if (!actualStr || actualStr === "null" || actualStr === "undefined" || actualStr === "0" || actualStr === "") {
                        throw new Error("Expected truthy but got '" + actualStr + "'");
                    }
                },
                toBeFalsy: function() {
                    if (actualStr && actualStr !== "null" && actualStr !== "undefined" && actualStr !== "0" && actualStr !== "") {
                        throw new Error("Expected falsy but got '" + actualStr + "'");
                    }
                },
                toBeGreaterThan: function(expected) {
                    var actualNum = parseFloat(actualStr);
                    var expectedNum = parseFloat(String(expected));
                    if (actualNum <= expectedNum) {
                        throw new Error("Expected " + actualNum + " to be greater than " + expectedNum);
                    }
                },
                toBeLessThan: function(expected) {
                    var actualNum = parseFloat(actualStr);
                    var expectedNum = parseFloat(String(expected));
                    if (actualNum >= expectedNum) {
                        throw new Error("Expected " + actualNum + " to be less than " + expectedNum);
                    }
                }
            };
        };
    "#;
    ctx.eval::<(), _>(pp_expect_js.as_bytes())?;

    Ok(pm)
}

fn format_caught_error(err: &CaughtError) -> String {
    match err {
        CaughtError::Exception(exc) => {
            let msg = exc.message().unwrap_or_default();
            if let Some(stack) = exc.stack() {
                format!("{}\n{}", msg, stack)
            } else {
                msg
            }
        }
        CaughtError::Value(val) => value_to_string(val),
        CaughtError::Error(e) => format!("{}", e),
    }
}

fn value_to_string(value: &Value) -> String {
    if let Some(s) = value.as_string() {
        if let Ok(rs) = s.to_string() {
            return rs;
        }
    }
    if let Some(i) = value.as_int() {
        return i.to_string();
    }
    if let Some(f) = value.as_float() {
        return f.to_string();
    }
    if let Some(b) = value.as_bool() {
        return b.to_string();
    }
    if value.is_null() {
        return "null".to_string();
    }
    if value.is_undefined() {
        return "undefined".to_string();
    }
    if value.is_object() {
        return "[Object]".to_string();
    }
    if value.is_array() {
        return "[Array]".to_string();
    }
    format!("{:?}", value)
}
