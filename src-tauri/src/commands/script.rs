use crate::scripting::engine::ScriptEngine;
use crate::scripting::pm_api::{PmApi, PmRequest, PmResponse, PmVariables, TestResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
struct ExecuteScriptInput {
    script_type: String,
    script: String,
    #[serde(default)]
    request: Option<serde_json::Value>,
    #[serde(default)]
    response: Option<serde_json::Value>,
    #[serde(default)]
    variables: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestData {
    pub method: String,
    pub url: String,
    #[serde(default)]
    pub headers: Vec<HeaderItem>,
    #[serde(default)]
    pub body: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeaderItem {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Serialize)]
pub struct ExecuteScriptOutput {
    #[serde(default)]
    pub variables: HashMap<String, String>,
    #[serde(default)]
    pub test_results: Vec<TestResult>,
    #[serde(default)]
    pub console_logs: Vec<String>,
    pub modified_request: Option<RequestData>,
}

#[tauri::command]
pub async fn execute_script(input: String) -> Result<String, String> {
    log::info!(
        "Executing script with input: {}",
        &input[..input.len().min(100)]
    );

    // Parse JSON input manually
    let input: ExecuteScriptInput =
        serde_json::from_str(&input).map_err(|e| format!("Failed to parse input JSON: {}", e))?;

    // Parse request from JSON value
    let request = if let Some(req_val) = &input.request {
        match req_val {
            serde_json::Value::Object(map) => {
                let method = map
                    .get("method")
                    .and_then(|v| v.as_str())
                    .unwrap_or("GET")
                    .to_string();
                let url = map
                    .get("url")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let body = map.get("body").and_then(|v| v.as_str()).map(String::from);

                // Parse headers
                let mut headers = HashMap::new();
                if let Some(headers_val) = map.get("headers") {
                    if let Some(arr) = headers_val.as_array() {
                        for item in arr {
                            if let Some(obj) = item.as_object() {
                                if let (Some(k), Some(v)) = (obj.get("key"), obj.get("value")) {
                                    if let (Some(key), Some(value)) = (k.as_str(), v.as_str()) {
                                        headers.insert(key.to_string(), value.to_string());
                                    }
                                }
                            }
                        }
                    }
                }

                // Parse query_params
                let mut query_params = HashMap::new();
                if let Some(params_val) = map.get("params") {
                    if let Some(arr) = params_val.as_array() {
                        for item in arr {
                            if let Some(obj) = item.as_object() {
                                if let (Some(k), Some(v)) = (obj.get("key"), obj.get("value")) {
                                    if let (Some(key), Some(value)) = (k.as_str(), v.as_str()) {
                                        query_params.insert(key.to_string(), value.to_string());
                                    }
                                }
                            }
                        }
                    }
                }

                Some(PmRequest {
                    method,
                    url,
                    headers,
                    body,
                    query_params,
                })
            }
            _ => None,
        }
    } else {
        None
    };

    // Parse response from JSON value
    let response = if let Some(resp_val) = &input.response {
        match resp_val {
            serde_json::Value::Object(map) => {
                let status = map.get("status").and_then(|v| v.as_u64()).unwrap_or(0) as u16;
                let time = map.get("time").and_then(|v| v.as_u64()).unwrap_or(0);
                let body = map.get("body").and_then(|v| v.as_str()).map(String::from);

                // Parse headers
                let mut headers = HashMap::new();
                if let Some(headers_val) = map.get("headers") {
                    if let Some(arr) = headers_val.as_array() {
                        for item in arr {
                            if let Some(obj) = item.as_object() {
                                if let (Some(k), Some(v)) = (obj.get("key"), obj.get("value")) {
                                    if let (Some(key), Some(value)) = (k.as_str(), v.as_str()) {
                                        headers.insert(key.to_string(), value.to_string());
                                    }
                                }
                            }
                        }
                    }
                }

                Some(PmResponse {
                    status,
                    headers,
                    body,
                    time,
                })
            }
            _ => None,
        }
    } else {
        None
    };

    let mut pm_api = PmApi {
        variables: PmVariables::new(),
        request,
        response,
        test_results: Vec::new(),
        console_logs: Vec::new(),
    };

    // Set initial variables from JSON value
    if let serde_json::Value::Object(map) = &input.variables {
        for (key, value) in map {
            if let serde_json::Value::String(v) = value {
                pm_api.variables.set(key.clone(), v.clone());
            }
        }
    }

    let result = match input.script_type.as_str() {
        "pre_request" => ScriptEngine::execute_pre_request(&input.script, pm_api),
        "test" => ScriptEngine::execute_test(&input.script, pm_api),
        _ => {
            return Err(format!("Unknown script type: {}", input.script_type));
        }
    };

    match result {
        Ok(pm_api) => {
            let variables = pm_api.variables.data;

            let modified_request = pm_api.request.map(|r| RequestData {
                method: r.method,
                url: r.url,
                headers: r
                    .headers
                    .into_iter()
                    .map(|(k, v)| HeaderItem { key: k, value: v })
                    .collect(),
                body: r.body,
            });

            let output = ExecuteScriptOutput {
                variables,
                test_results: pm_api.test_results,
                console_logs: pm_api.console_logs,
                modified_request,
            };

            serde_json::to_string(&output).map_err(|e| format!("Failed to serialize output: {}", e))
        }
        Err(e) => {
            log::error!("Script execution error: {}", e);
            Err(e.to_string())
        }
    }
}
