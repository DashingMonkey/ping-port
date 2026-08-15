use crate::http::client::{Body, HttpAuth, KeyValuePair};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::{command, Manager};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum RequestBody {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "json")]
    Json(String),
    #[serde(rename = "form-data")]
    FormData(String), // JSON string of KeyValuePair[]
    #[serde(rename = "x-www-form-urlencoded")]
    UrlEncoded(String), // JSON string of KeyValuePair[]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendRequestInput {
    pub method: String,
    pub url: String,
    pub headers: HashMap<String, String>,
    pub query_params: HashMap<String, String>,
    pub body: Option<RequestBody>,
    pub auth: Option<Auth>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Auth {
    pub kind: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub token: Option<String>,
    pub key: Option<String>,
    pub value: Option<String>,
    pub in_header: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendRequestResponse {
    pub success: bool,
    pub error: Option<String>,
    pub status: Option<u16>,
    pub status_text: Option<String>,
    pub headers: Option<HashMap<String, String>>,
    pub body: Option<String>,
    pub time_ms: Option<u64>,
    pub size_bytes: Option<usize>,
}

#[command]
pub async fn send_request(
    input: SendRequestInput,
    app_handle: tauri::AppHandle,
) -> Result<SendRequestResponse, String> {
    let client = {
        let state = app_handle.state::<std::sync::Mutex<crate::AppState>>();
        let app_state = state
            .lock()
            .map_err(|e| format!("Internal error: {}", e))?;
        app_state.http_client.clone()
    };

    let auth = input.auth.map(|a| match a.kind.as_str() {
        "basic" => HttpAuth::Basic {
            username: a.username.unwrap_or_default(),
            password: a.password.unwrap_or_default(),
        },
        "bearer" => HttpAuth::Bearer {
            token: a.token.unwrap_or_default(),
        },
        "apikey" => HttpAuth::ApiKey {
            key: a.key.unwrap_or_default(),
            value: a.value.unwrap_or_default(),
            in_header: a.in_header.unwrap_or(true),
        },
        _ => HttpAuth::None,
    });

    let body = match &input.body {
        None => Body::None,
        Some(RequestBody::None) => Body::None,
        Some(RequestBody::Json(content)) => Body::Raw(content.clone()),
        Some(RequestBody::FormData(json_content)) => {
            match serde_json::from_str::<Vec<KeyValuePair>>(json_content) {
                Ok(items) => Body::FormData(items),
                Err(e) => return Err(format!("Failed to parse form data: {}", e)),
            }
        }
        Some(RequestBody::UrlEncoded(json_content)) => {
            match serde_json::from_str::<Vec<KeyValuePair>>(json_content) {
                Ok(items) => Body::UrlEncoded(items),
                Err(e) => return Err(format!("Failed to parse form data: {}", e)),
            }
        }
    };

    let request = crate::http::client::HttpRequest {
        method: input.method,
        url: input.url,
        headers: input.headers,
        query_params: input.query_params,
        body,
        auth,
    };

    match client.send(request).await {
        Ok(response) => Ok(SendRequestResponse {
            success: true,
            error: None,
            status: Some(response.status),
            status_text: Some(response.status_text),
            headers: Some(response.headers),
            body: Some(response.body),
            time_ms: Some(response.time_ms),
            size_bytes: Some(response.size_bytes),
        }),
        Err(e) => Ok(SendRequestResponse {
            success: false,
            error: Some(e),
            status: None,
            status_text: None,
            headers: None,
            body: None,
            time_ms: None,
            size_bytes: None,
        }),
    }
}
