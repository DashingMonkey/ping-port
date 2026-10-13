use crate::scripting::engine::ScriptEngine;
use crate::scripting::pm_api::{PmApi, PmRequest, PmResponse, PmVariables, TestResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
struct ExecuteScriptInput {
    script_type: String,
    script: String,
    #[serde(default)]
    request: Option<PmRequest>,
    #[serde(default)]
    response: Option<PmResponse>,
    #[serde(default)]
    variables: HashMap<String, String>,
}

#[derive(Debug, Serialize)]
pub struct ExecuteScriptOutput {
    pub variables: HashMap<String, String>,
    pub test_results: Vec<TestResult>,
    pub console_logs: Vec<String>,
    pub modified_request: Option<PmRequest>,
}

#[tauri::command]
pub async fn execute_script(input: String) -> Result<String, String> {
    let input: ExecuteScriptInput =
        serde_json::from_str(&input).map_err(|e| format!("Failed to parse input JSON: {}", e))?;
    let pm_api = PmApi {
        variables: PmVariables {
            data: input.variables,
        },
        request: input.request,
        response: input.response,
        ..Default::default()
    };
    // Script execution waits on a dedicated JS thread; keep that wait off the
    // async runtime so a slow script cannot block unrelated commands.
    let result = tauri::async_runtime::spawn_blocking(move || match input.script_type.as_str() {
        "pre_request" => ScriptEngine::execute_pre_request(&input.script, pm_api),
        "test" => ScriptEngine::execute_test(&input.script, pm_api),
        _ => Err(format!("Unknown script type: {}", input.script_type)),
    })
    .await
    .map_err(|e| e.to_string())??;

    let output = ExecuteScriptOutput {
        variables: result.variables.data,
        test_results: result.test_results,
        console_logs: result.console_logs,
        modified_request: result.request,
    };
    serde_json::to_string(&output).map_err(|e| format!("Failed to serialize output: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn pre_request_round_trips_body_headers_params_and_variables() {
        let result = execute_script(json!({
            "script_type": "pre_request",
            "script": "if (pp.request.body !== 'original') throw new Error('body lost'); pp.request.method = 'PUT'; pp.request.body = ''; pp.request.headers['X-Test'] = 'new'; delete pp.request.headers['X-Old']; pp.request.addQueryParam('added', 'yes'); pp.request.removeQueryParam('old'); pp.request.setUrl('http://localhost/{{token}}'); pp.variables.set('token', 'new-token');",
            "request": { "method": "POST", "url": "http://localhost", "body": "original", "headers": { "X-Old": "old" }, "query_params": { "old": "old" } },
            "variables": { "token": "old-token" }
        }).to_string()).await.unwrap();
        let result: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(result["variables"]["token"], "new-token");
        let request = &result["modified_request"];
        assert_eq!(request["method"], "PUT");
        assert_eq!(request["body"], "");
        assert_eq!(request["url"], "http://localhost/{{token}}");
        assert_eq!(request["headers"], json!({ "X-Test": "new" }));
        assert_eq!(request["query_params"], json!({ "added": "yes" }));
    }

    #[tokio::test]
    async fn tests_receive_response_headers_and_return_variable_changes() {
        let result = execute_script(json!({
            "script_type": "test",
            "script": "if (pp.response.headers['content-type'] !== 'application/json') throw new Error('header lost'); if (pp.response.headers['set-cookie'].length !== 2) throw new Error('cookies lost'); if (pp.request.getQueryParam('page') !== '2') throw new Error('params lost'); pp.variables.set('next', 'ready'); pp.variables.unset('old'); pp.test('status', () => pp.expect(pp.response.status).toEqual(200));",
            "request": { "method": "GET", "url": "http://localhost", "query_params": { "page": "2" } },
            "response": { "status": 200, "time": 5, "body": "{}", "headers": { "content-type": ["application/json"], "set-cookie": ["a=1", "b=2"] } },
            "variables": { "old": "gone" }
        }).to_string()).await.unwrap();
        let result: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(result["variables"], json!({ "next": "ready" }));
        assert_eq!(result["test_results"][0]["passed"], true, "{}", result);
    }
}
