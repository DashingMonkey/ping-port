pub mod bridge;
pub mod tools;

use std::sync::Arc;

use axum::body::Bytes;
use axum::extract::State;
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Json, Response};
use serde_json::{json, Value};
use tauri::{AppHandle, Manager};

/// Tauri app identifier (must match tauri.conf.json) used to locate the
/// app data dir where the bridge connection info is written.
pub const APP_IDENTIFIER: &str = "com.pingport.app";

/// Start the local MCP HTTP server on 127.0.0.1 with a random port and
/// write the connection info (port + token) for the stdio bridge.
pub fn start(app: AppHandle) -> Result<(), String> {
    let std_listener = std::net::TcpListener::bind("127.0.0.1:0")
        .map_err(|e| format!("MCP server bind failed: {}", e))?;
    let port = std_listener
        .local_addr()
        .map_err(|e| format!("MCP server local_addr failed: {}", e))?
        .port();
    std_listener
        .set_nonblocking(true)
        .map_err(|e| format!("MCP server nonblocking failed: {}", e))?;

    let token = uuid::Uuid::new_v4().to_string();
    write_config(&app, port, &token)?;

    let server = Arc::new(McpServer { app, token });
    let router = build_router(server);

    tauri::async_runtime::spawn(async move {
        match tokio::net::TcpListener::from_std(std_listener) {
            Ok(listener) => {
                if let Err(e) = axum::serve(listener, router).await {
                    log::error!("MCP server error: {}", e);
                }
            }
            Err(e) => log::error!("MCP listener conversion failed: {}", e),
        }
    });

    log::info!("MCP server listening on 127.0.0.1:{}", port);
    Ok(())
}

fn write_config(app: &AppHandle, port: u16, token: &str) -> Result<(), String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to resolve app data dir: {}", e))?;
    std::fs::create_dir_all(&dir).map_err(|e| format!("Failed to create app data dir: {}", e))?;
    let cfg = json!({ "port": port, "token": token });
    std::fs::write(dir.join("mcp.json"), cfg.to_string())
        .map_err(|e| format!("Failed to write MCP config: {}", e))?;
    Ok(())
}

/// Remove the bridge connection file on app exit so the stdio bridge can't
/// pick up a stale port/token from a previous run.
pub fn cleanup_config(app: &AppHandle) {
    let dir = match app.path().app_data_dir() {
        Ok(dir) => dir,
        Err(e) => {
            log::warn!("Failed to resolve app data dir for MCP cleanup: {}", e);
            return;
        }
    };
    if let Err(e) = std::fs::remove_file(dir.join("mcp.json")) {
        if e.kind() != std::io::ErrorKind::NotFound {
            log::warn!("Failed to remove MCP config: {}", e);
        }
    }
}

struct McpServer {
    app: AppHandle,
    token: String,
}

fn build_router(server: Arc<McpServer>) -> axum::Router {
    axum::Router::new()
        .route("/mcp", axum::routing::post(handle_mcp))
        .with_state(server)
}

async fn handle_mcp(
    State(server): State<Arc<McpServer>>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    // Bearer token check
    let expected = format!("Bearer {}", server.token);
    let authorized = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .map(|v| v == expected)
        .unwrap_or(false);
    if !authorized {
        return (StatusCode::UNAUTHORIZED, "unauthorized").into_response();
    }

    // Only allow loopback hosts (DNS rebinding mitigation)
    let host_ok = headers
        .get(header::HOST)
        .and_then(|v| v.to_str().ok())
        .map(|h| {
            let host = h.rsplit_once(':').map(|x| x.0).unwrap_or(h);
            host == "127.0.0.1" || host == "localhost" || host == "[::1]"
        })
        .unwrap_or(false);
    if !host_ok {
        return (StatusCode::FORBIDDEN, "forbidden host").into_response();
    }

    let msg: Value = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(e) => return rpc_error_response(Value::Null, -32700, &format!("Parse error: {}", e)),
    };
    // JSON-RPC batches are not supported; reject explicitly instead of
    // silently treating the array as a notification (which would return 204).
    if msg.is_array() {
        return rpc_error_response(Value::Null, -32600, "Batch requests are not supported");
    }

    let handler = server.clone();
    let result =
        tokio::task::spawn_blocking(move || handler.handle_message(msg)).await;

    match result {
        Ok(Some(response)) => (StatusCode::OK, Json(response)).into_response(),
        // Notifications (no id) are acknowledged without a body
        Ok(None) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => rpc_error_response(Value::Null, -32603, &format!("Internal error: {}", e)),
    }
}

fn rpc_error_response(id: Value, code: i64, message: &str) -> Response {
    (
        StatusCode::OK,
        Json(json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": { "code": code, "message": message }
        })),
    )
        .into_response()
}

impl McpServer {
    /// Handle one JSON-RPC message. Returns None for notifications.
    fn handle_message(&self, msg: Value) -> Option<Value> {
        let id = msg.get("id").cloned();
        let method = msg
            .get("method")
            .and_then(|m| m.as_str())
            .unwrap_or("")
            .to_string();
        let params = msg.get("params").cloned().unwrap_or(Value::Null);

        let result: Result<Value, (i64, String)> = match method.as_str() {
            "initialize" => Ok(self.handle_initialize(&params)),
            "ping" => Ok(json!({})),
            "tools/list" => Ok(json!({ "tools": tools::definitions() })),
            "tools/call" => self.handle_call_tool(&params),
            other => {
                if id.is_some() {
                    Err((-32601, format!("Method not found: {}", other)))
                } else {
                    // Notifications (e.g. notifications/initialized) get no response
                    return None;
                }
            }
        };

        Some(match (id, result) {
            (Some(id), Ok(result)) => json!({ "jsonrpc": "2.0", "id": id, "result": result }),
            (Some(id), Err((code, message))) => json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": { "code": code, "message": message }
            }),
            (None, _) => return None,
        })
    }

    fn handle_initialize(&self, params: &Value) -> Value {
        const SUPPORTED: [&str; 3] = ["2024-11-05", "2025-03-26", "2025-06-18"];
        let requested = params
            .get("protocolVersion")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let protocol_version = if SUPPORTED.contains(&requested) {
            requested
        } else {
            "2025-06-18"
        };
        json!({
            "protocolVersion": protocol_version,
            "capabilities": { "tools": { "listChanged": false } },
            "serverInfo": { "name": "pingport", "version": env!("CARGO_PKG_VERSION") },
            "instructions": "PingPort local MCP server. Call list_collections first to discover collection IDs. For create_request, pass collection_id to choose the destination, or omit it to file the request under the 'Default' root collection (auto-created when missing). Use create_collection for new folders."
        })
    }

    fn handle_call_tool(&self, params: &Value) -> Result<Value, (i64, String)> {
        let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
        let args = params.get("arguments").cloned().unwrap_or(json!({}));
        if name.is_empty() {
            return Err((-32602, "Missing tool name".to_string()));
        }
        if !tools::is_known(name) {
            return Err((-32602, format!("Unknown tool: {}", name)));
        }
        match tools::call_tool(&self.app, name, &args) {
            Ok(result) => {
                let text = serde_json::to_string_pretty(&result)
                    .unwrap_or_else(|_| "{}".to_string());
                Ok(json!({
                    "content": [ { "type": "text", "text": text } ]
                }))
            }
            // Tool execution failure is reported as a tool result, not a protocol error
            Err(message) => Ok(json!({
                "content": [ { "type": "text", "text": message } ],
                "isError": true
            })),
        }
    }
}
