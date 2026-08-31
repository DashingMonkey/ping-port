use std::collections::HashMap;
use std::sync::Mutex;

use rusqlite::Connection;
use serde_json::{json, Value};
use tauri::{AppHandle, Emitter, Manager};

use crate::services::{collections as col_svc, requests as req_svc};
use crate::AppState;

const TOOL_NAMES: [&str; 7] = [
    "list_collections",
    "list_requests",
    "create_collection",
    "create_request",
    "update_request",
    "delete_request",
    "delete_collection",
];

pub fn is_known(name: &str) -> bool {
    TOOL_NAMES.contains(&name)
}

pub fn definitions() -> Vec<Value> {
    let mut update_props = match request_props() {
        Value::Object(map) => map,
        _ => serde_json::Map::new(),
    };
    update_props.insert(
        "id".to_string(),
        json!({ "type": "string", "description": "Request ID to update" }),
    );
    update_props.insert(
        "collection_id".to_string(),
        json!({ "type": "string", "description": "Optional: move the request into this collection ID" }),
    );
    let mut create_props = match request_props() {
        Value::Object(map) => map,
        _ => serde_json::Map::new(),
    };
    create_props.insert(
        "collection_id".to_string(),
        json!({ "type": "string", "description": "Optional parent collection ID. Omit to file the request under the 'Default' root collection (created automatically if missing)." }),
    );
    vec![
        json!({
            "name": "list_collections",
            "description": "List all collections and saved requests (with IDs) in the current PingPort workspace. Call this first to get collection IDs before creating requests.",
            "inputSchema": { "type": "object", "properties": {} }
        }),
        json!({
            "name": "list_requests",
            "description": "List saved requests with parsed params/headers/body/auth. Optionally filter by collection_id.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "collection_id": { "type": "string", "description": "Optional: only return requests in this collection" }
                }
            }
        }),
        json!({
            "name": "create_collection",
            "description": "Create a new collection (folder) in the current PingPort workspace.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "name": { "type": "string", "description": "Collection name" },
                    "parent_id": { "type": "string", "description": "Optional parent collection ID to create a nested folder" },
                    "kind": { "type": "string", "description": "Optional collection kind, e.g. 'folder' (default)" }
                },
                "required": ["name"]
            }
        }),
        json!({
            "name": "create_request",
            "description": "Create a new HTTP request. Pass collection_id (from list_collections) to choose the destination collection, or omit it to file the request under the 'Default' root collection, which is created automatically when missing.",
            "inputSchema": {
                "type": "object",
                "properties": Value::Object(create_props),
                "required": ["name"]
            }
        }),
        json!({
            "name": "update_request",
            "description": "Update an existing saved request by ID. Only provided fields are changed. Note: params/headers/body/auth replace the whole previous value; pass null to clear a field.",
            "inputSchema": {
                "type": "object",
                "properties": Value::Object(update_props),
                "required": ["id"]
            }
        }),
        json!({
            "name": "delete_request",
            "description": "Delete a saved request by ID.",
            "inputSchema": {
                "type": "object",
                "properties": { "id": { "type": "string", "description": "Request ID" } },
                "required": ["id"]
            }
        }),
        json!({
            "name": "delete_collection",
            "description": "Delete a collection by ID. Its requests and sub-collections are removed recursively.",
            "inputSchema": {
                "type": "object",
                "properties": { "id": { "type": "string", "description": "Collection ID" } },
                "required": ["id"]
            }
        }),
    ]
}

/// Properties shared by create_request / update_request tool schemas.
/// Field formats match the PingPort frontend serialization:
/// - params / headers: array of {key, value, enabled}
/// - body: {type: none|json|form-data|x-www-form-urlencoded, content: string}
/// - auth: {type: none|basic|bearer|api-key, ...credentials}
fn request_props() -> Value {
    let kv_item = json!({
        "type": "object",
        "properties": {
            "key": { "type": "string" },
            "value": { "type": "string" },
            "enabled": { "type": "boolean", "default": true }
        }
    });
    json!({
        "collection_id": { "type": "string", "description": "Parent collection ID" },
        "name": { "type": "string", "description": "Request name" },
        "method": { "type": "string", "enum": ["GET", "POST", "PUT", "DELETE", "PATCH", "OPTIONS", "HEAD"], "description": "HTTP method (default GET)" },
        "url": { "type": "string", "description": "Request URL" },
        "params": { "type": "array", "items": kv_item.clone(), "description": "Query parameters" },
        "headers": { "type": "array", "items": kv_item, "description": "HTTP headers" },
        "body": {
            "type": "object",
            "properties": {
                "type": { "type": "string", "enum": ["none", "json", "form-data", "x-www-form-urlencoded"] },
                "content": { "type": "string", "description": "Raw body content (JSON text, form entries, etc.)" }
            },
            "description": "Request body"
        },
        "auth": {
            "type": "object",
            "properties": {
                "type": { "type": "string", "enum": ["none", "basic", "bearer", "api-key"] },
                "basic": { "type": "object", "properties": { "username": { "type": "string" }, "password": { "type": "string" } } },
                "bearer": { "type": "object", "properties": { "token": { "type": "string" } } },
                "apiKey": { "type": "object", "properties": { "key": { "type": "string" }, "value": { "type": "string" }, "in": { "type": "string", "enum": ["header", "query"] } } }
            },
            "required": ["type"],
            "description": "Auth configuration"
        },
        "pre_request_script": { "type": "string", "description": "Optional pre-request script (JavaScript)" },
        "test_script": { "type": "string", "description": "Optional test script (JavaScript)" }
    })
}

pub fn call_tool(app: &AppHandle, name: &str, args: &Value) -> Result<Value, String> {
    match name {
        "list_collections" => list_collections(app),
        "list_requests" => list_requests(app, args),
        "create_collection" => create_collection(app, args),
        "create_request" => create_request(app, args),
        "update_request" => update_request(app, args),
        "delete_request" => delete_request(app, args),
        "delete_collection" => delete_collection(app, args),
        _ => Err(format!("Unknown tool: {}", name)),
    }
}

fn with_conn<T>(
    app: &AppHandle,
    f: impl FnOnce(&Connection, &AppState) -> Result<T, String>,
) -> Result<T, String> {
    let state = app
        .try_state::<Mutex<AppState>>()
        .ok_or("Workspace is not initialized")?;
    let guard = state.lock().map_err(|e| format!("Internal error: {}", e))?;
    let conn_guard = guard.db.conn.lock().map_err(|e| e.to_string())?;
    let conn = conn_guard
        .as_ref()
        .ok_or("Database is closed".to_string())?;
    f(conn, &guard)
}

fn emit_changed(app: &AppHandle, event: &str, ids: &[String]) {
    let payload = json!({ "source": "mcp", "ids": ids });
    if let Err(e) = app.emit(event, payload) {
        log::warn!("Failed to emit {}: {}", event, e);
    }
}

fn required_str(args: &Value, field: &str) -> Result<String, String> {
    args.get(field)
        .and_then(|v| v.as_str())
        .map(str::to_string)
        .ok_or_else(|| format!("Missing required field: {}", field))
}

fn optional_str(args: &Value, field: &str) -> Option<String> {
    args.get(field)
        .and_then(|v| v.as_str())
        .map(str::to_string)
}

fn list_collections(app: &AppHandle) -> Result<Value, String> {
    with_conn(app, |conn, state| {
        let collections = col_svc::list_collections(conn)?;
        let requests = req_svc::list_requests(conn)?;

        let mut by_collection: HashMap<String, Vec<Value>> = HashMap::new();
        for r in &requests {
            by_collection
                .entry(r.collection_id.clone())
                .or_default()
                .push(json!({
                    "id": r.id,
                    "name": r.name,
                    "method": r.method,
                    "url": r.url
                }));
        }

        let items: Vec<Value> = collections
            .iter()
            .map(|c| {
                json!({
                    "id": c.id,
                    "name": c.name,
                    "type": c.kind,
                    "parent_id": c.parent_id,
                    "requests": by_collection.remove(&c.id).unwrap_or_default(),
                })
            })
            .collect();

        let workspace = state
            .current_workspace
            .lock()
            .map_err(|e| e.to_string())?
            .clone();

        Ok(json!({ "workspace": workspace, "collections": items }))
    })
}

fn list_requests(app: &AppHandle, args: &Value) -> Result<Value, String> {
    let filter = args
        .get("collection_id")
        .and_then(|v| v.as_str())
        .map(str::to_string);
    with_conn(app, |conn, _| {
        let requests = req_svc::list_requests(conn)?;
        let items: Vec<Value> = requests
            .iter()
            .filter(|r| filter.as_deref().map_or(true, |f| r.collection_id == f))
            .map(request_detail)
            .collect();
        Ok(json!({ "requests": items, "count": items.len() }))
    })
}

fn request_detail(r: &req_svc::Request) -> Value {
    json!({
        "id": r.id,
        "collection_id": r.collection_id,
        "name": r.name,
        "method": r.method,
        "url": r.url,
        "params": parse_json_field(&r.params),
        "headers": parse_json_field(&r.headers),
        "body": parse_json_field(&r.body),
        "auth": parse_json_field(&r.auth),
        "pre_request_script": r.pre_request_script,
        "test_script": r.test_script,
        "created_at": r.created_at,
        "updated_at": r.updated_at,
    })
}

/// Parse a stored JSON-string field into a Value (Null when empty/missing).
fn parse_json_field(value: &Option<String>) -> Value {
    match value.as_deref() {
        None | Some("") => Value::Null,
        Some(s) => serde_json::from_str(s).unwrap_or(Value::Null),
    }
}

fn create_collection(app: &AppHandle, args: &Value) -> Result<Value, String> {
    let name = required_str(args, "name")?;
    let input = col_svc::CreateCollectionInput {
        id: None,
        name,
        kind: optional_str(args, "kind"),
        parent_id: optional_str(args, "parent_id"),
    };
    let created = with_conn(app, |conn, _| col_svc::create_collection(conn, input))?;
    emit_changed(app, "collections-changed", &[created.id.clone()]);
    serde_json::to_value(&created).map_err(|e| e.to_string())
}

fn create_request(app: &AppHandle, args: &Value) -> Result<Value, String> {
    let provided_collection = optional_str(args, "collection_id");
    let name = required_str(args, "name")?;
    let method = normalize_method(args.get("method").and_then(|v| v.as_str()))?;
    let url = args
        .get("url")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let params = encode_kv_list(args.get("params"), "params")?;
    let headers = encode_kv_list(args.get("headers"), "headers")?;
    let body = encode_body(args.get("body"))?;
    let auth = encode_auth(args.get("auth"))?;
    let pre_request_script = optional_str(args, "pre_request_script");
    let test_script = optional_str(args, "test_script");

    let mut new_collection_id: Option<String> = None;
    let created = with_conn(app, |conn, _| {
        // Resolve the destination: an explicitly provided collection must
        // exist; when omitted, the request is filed under the root-level
        // "Default" collection (created on first use), so a caller never
        // hits a dead end on an empty workspace.
        let collection_id = match &provided_collection {
            Some(id) => {
                if !col_svc::exists(conn, id)? {
                    return Err(format!(
                        "Collection '{}' not found. Call list_collections to get valid collection IDs.",
                        id
                    ));
                }
                id.clone()
            }
            None => {
                let (col, created) =
                    col_svc::find_or_create_root_by_name(conn, col_svc::DEFAULT_COLLECTION_NAME)?;
                if created {
                    new_collection_id = Some(col.id.clone());
                }
                col.id
            }
        };
        req_svc::create_request(
            conn,
            req_svc::CreateRequestInput {
                id: None,
                collection_id: collection_id.clone(),
                name: name.clone(),
                method: method.clone(),
                url: Some(url.clone()),
                params: params.clone(),
                headers: headers.clone(),
                body: body.clone(),
                auth: auth.clone(),
                pre_request_script: pre_request_script.clone(),
                test_script: test_script.clone(),
            },
        )
    })?;

    // If the fallback collection was auto-created, the sidebar must learn
    // about the new node before/with the request that references it.
    if let Some(col_id) = new_collection_id {
        emit_changed(app, "collections-changed", &[col_id]);
    }
    emit_changed(app, "requests-changed", &[created.id.clone()]);
    serde_json::to_value(&created).map_err(|e| e.to_string())
}

fn update_request(app: &AppHandle, args: &Value) -> Result<Value, String> {
    let id = required_str(args, "id")?;
    let input = req_svc::UpdateRequestInput {
        id: id.clone(),
        collection_id: optional_str(args, "collection_id"),
        name: optional_str(args, "name"),
        method: match args.get("method").and_then(|v| v.as_str()) {
            Some(m) => Some(normalize_method(Some(m))?),
            None => None,
        },
        url: optional_str(args, "url"),
        params: encode_kv_list(args.get("params"), "params")?,
        headers: encode_kv_list(args.get("headers"), "headers")?,
        body: encode_body(args.get("body"))?,
        auth: encode_auth(args.get("auth"))?,
        pre_request_script: optional_str(args, "pre_request_script"),
        test_script: optional_str(args, "test_script"),
    };

    with_conn(app, |conn, _| {
        if let Some(target) = &input.collection_id {
            if !col_svc::exists(conn, target)? {
                return Err(format!(
                    "Collection '{}' not found. Call list_collections to get valid collection IDs.",
                    target
                ));
            }
        }
        req_svc::update_request(conn, input)
    })?;

    emit_changed(app, "requests-changed", &[id]);
    Ok(json!({ "success": true }))
}

fn delete_request(app: &AppHandle, args: &Value) -> Result<Value, String> {
    let id = required_str(args, "id")?;
    with_conn(app, |conn, _| req_svc::delete_request(conn, &id))?;
    emit_changed(app, "requests-changed", &[id.clone()]);
    Ok(json!({ "success": true, "id": id }))
}

fn delete_collection(app: &AppHandle, args: &Value) -> Result<Value, String> {
    let id = required_str(args, "id")?;
    with_conn(app, |conn, _| col_svc::delete_collection(conn, &id))?;
    emit_changed(app, "collections-changed", &[id.clone()]);
    // Requests under this collection (and its sub-collections) are cascaded
    emit_changed(app, "requests-changed", &[]);
    Ok(json!({ "success": true, "id": id }))
}

const HTTP_METHODS: [&str; 7] = ["GET", "POST", "PUT", "DELETE", "PATCH", "OPTIONS", "HEAD"];

fn normalize_method(value: Option<&str>) -> Result<String, String> {
    let method = value.unwrap_or("GET").to_uppercase();
    if HTTP_METHODS.contains(&method.as_str()) {
        Ok(method)
    } else {
        Err(format!(
            "Invalid method '{}', must be one of: {}",
            method,
            HTTP_METHODS.join(", ")
        ))
    }
}

/// Encode an MCP array of {key, value, enabled} into the JSON string the
/// frontend expects for params/headers fields.
fn encode_kv_list(value: Option<&Value>, field: &str) -> Result<Option<String>, String> {
    match value {
        // Field omitted: keep the current value (default on create)
        None => Ok(None),
        // Explicit null: clear the field (empty list, matches the UI's "[]")
        Some(Value::Null) => Ok(Some("[]".to_string())),
        Some(Value::Array(items)) => {
            let mut normalized: Vec<Value> = Vec::with_capacity(items.len());
            for item in items {
                let obj = item
                    .as_object()
                    .ok_or_else(|| format!("{} entries must be objects", field))?;
                let key = obj.get("key").and_then(|v| v.as_str()).unwrap_or("");
                let val = obj.get("value").and_then(|v| v.as_str()).unwrap_or("");
                let enabled = obj.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true);
                let mut entry = json!({ "key": key, "value": val, "enabled": enabled });
                if let Some(t) = obj.get("type").and_then(|v| v.as_str()) {
                    entry["type"] = json!(t);
                }
                normalized.push(entry);
            }
            Ok(Some(
                serde_json::to_string(&normalized).map_err(|e| e.to_string())?,
            ))
        }
        Some(_) => Err(format!(
            "{} must be an array of {{key, value, enabled}} objects",
            field
        )),
    }
}

const BODY_TYPES: [&str; 4] = ["none", "json", "form-data", "x-www-form-urlencoded"];

fn encode_body(value: Option<&Value>) -> Result<Option<String>, String> {
    match value {
        None => Ok(None),
        // Explicit null: clear the field (matches the UI's empty body)
        Some(Value::Null) => Ok(Some(
            json!({ "type": "none", "content": "" }).to_string(),
        )),
        Some(v) => {
            let obj = v
                .as_object()
                .ok_or("body must be an object {type, content}")?;
            let body_type = obj
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("none")
                .to_string();
            if !BODY_TYPES.contains(&body_type.as_str()) {
                return Err(format!(
                    "Invalid body type '{}', must be one of: {}",
                    body_type,
                    BODY_TYPES.join(", ")
                ));
            }
            let content = obj.get("content").and_then(|v| v.as_str()).unwrap_or("");
            Ok(Some(
                json!({ "type": body_type, "content": content }).to_string(),
            ))
        }
    }
}

const AUTH_TYPES: [&str; 4] = ["none", "basic", "bearer", "api-key"];

fn encode_auth(value: Option<&Value>) -> Result<Option<String>, String> {
    match value {
        None => Ok(None),
        // Explicit null: clear the field (matches the UI's auth-off state)
        Some(Value::Null) => Ok(Some(json!({ "type": "none" }).to_string())),
        Some(v) => {
            let auth_type = v
                .get("type")
                .and_then(|t| t.as_str())
                .ok_or("auth.type is required (none | basic | bearer | api-key)")?;
            if !AUTH_TYPES.contains(&auth_type) {
                return Err(format!(
                    "Invalid auth type '{}', must be one of: {}",
                    auth_type,
                    AUTH_TYPES.join(", ")
                ));
            }
            serde_json::to_string(v).map(Some).map_err(|e| e.to_string())
        }
    }
}
