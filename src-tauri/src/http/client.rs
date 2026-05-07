use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue},
    multipart, Client, Method,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;

#[derive(Debug, Clone)]
pub enum HttpAuth {
    None,
    Basic {
        username: String,
        password: String,
    },
    Bearer {
        token: String,
    },
    ApiKey {
        key: String,
        value: String,
        in_header: bool,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyValuePair {
    pub key: String,
    pub value: String,
    pub enabled: bool,
    #[serde(default)]
    pub pair_type: Option<String>, // "text" or "file"
}

#[derive(Debug, Clone)]
pub enum Body {
    None,
    Raw(String),
    FormData(Vec<KeyValuePair>),
}

#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: String,
    pub url: String,
    pub headers: HashMap<String, String>,
    pub query_params: HashMap<String, String>,
    pub body: Body,
    pub auth: Option<HttpAuth>,
}

#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status: u16,
    pub status_text: String,
    pub headers: HashMap<String, String>,
    pub body: String,
    pub time_ms: u64,
    pub size_bytes: usize,
}

pub struct HttpClient {
    client: Client,
}

impl HttpClient {
    pub fn new() -> Result<Self, String> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        Ok(Self { client })
    }

    pub async fn send(&self, request: HttpRequest) -> Result<HttpResponse, String> {
        let start = Instant::now();

        let method = match request.method.to_uppercase().as_str() {
            "GET" => Method::GET,
            "POST" => Method::POST,
            "PUT" => Method::PUT,
            "DELETE" => Method::DELETE,
            "PATCH" => Method::PATCH,
            "HEAD" => Method::HEAD,
            "OPTIONS" => Method::OPTIONS,
            _ => return Err(format!("Unsupported HTTP method: {}", request.method)),
        };

        let mut url =
            reqwest::Url::parse(&request.url).map_err(|e| format!("Invalid URL: {}", e))?;

        // Add query params
        {
            let mut query_pairs = url.query_pairs_mut();
            for (key, value) in &request.query_params {
                query_pairs.append_pair(key, value);
            }
        }

        let mut headers = HeaderMap::new();
        for (key, value) in &request.headers {
            let header_name = HeaderName::from_bytes(key.as_bytes())
                .map_err(|e| format!("Invalid header name '{}': {}", key, e))?;
            let header_value = HeaderValue::from_str(value)
                .map_err(|e| format!("Invalid header value for '{}': {}", key, e))?;
            headers.insert(header_name, header_value);
        }

        let mut req_builder = self.client.request(method, url).headers(headers);

        // Apply authentication
        if let Some(auth) = &request.auth {
            match auth {
                HttpAuth::None => {}
                HttpAuth::Basic { username, password } => {
                    req_builder = req_builder.basic_auth(username, Some(password));
                }
                HttpAuth::Bearer { token } => {
                    req_builder = req_builder.bearer_auth(token);
                }
                HttpAuth::ApiKey {
                    key,
                    value,
                    in_header,
                } => {
                    if *in_header {
                        let header_name = HeaderName::from_bytes(key.as_bytes())
                            .map_err(|e| format!("Invalid API key header name '{}': {}", key, e))?;
                        let header_value = HeaderValue::from_str(value)
                            .map_err(|e| format!("Invalid API key value: {}", e))?;
                        req_builder = req_builder.header(header_name, header_value);
                    }
                }
            }
        }

        // Add body based on type
        match &request.body {
            Body::None => {}
            Body::Raw(body) => {
                req_builder = req_builder.body(body.clone());
            }
            Body::FormData(fields) => {
                let mut form = multipart::Form::new();

                for field in fields {
                    if !field.enabled {
                        continue;
                    }

                    let pair_type = field.pair_type.as_deref().unwrap_or("text");

                    if pair_type == "file" {
                        // Read file content
                        let file_path = std::path::Path::new(&field.value);

                        // Security: reject path traversal attempts
                        if field.value.contains("..") {
                            return Err(format!(
                                "Invalid file path (path traversal detected): {}",
                                field.value
                            ));
                        }

                        if !file_path.exists() {
                            return Err(format!("File not found: {}", field.value));
                        }

                        if !file_path.is_file() {
                            return Err(format!("Not a regular file: {}", field.value));
                        }

                        let file_name = file_path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("file");

                        let file_content = tokio::fs::read(&field.value)
                            .await
                            .map_err(|e| format!("Failed to read file '{}': {}", field.value, e))?;

                        let part =
                            multipart::Part::bytes(file_content).file_name(file_name.to_string());
                        form = form.part(field.key.clone(), part);
                    } else {
                        // Text field
                        let part = multipart::Part::text(field.value.clone());
                        form = form.part(field.key.clone(), part);
                    }
                }

                req_builder = req_builder.multipart(form);
            }
        }

        let response = req_builder
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        let status = response.status().as_u16();
        let status_text = response
            .status()
            .canonical_reason()
            .unwrap_or("Unknown")
            .to_string();

        let mut response_headers = HashMap::new();
        for (key, value) in response.headers() {
            if let Ok(v) = value.to_str() {
                response_headers.insert(key.to_string(), v.to_string());
            }
        }

        let body = response
            .text()
            .await
            .map_err(|e| format!("Failed to read response body: {}", e))?;
        let size_bytes = body.len();

        let time_ms = start.elapsed().as_millis() as u64;

        Ok(HttpResponse {
            status,
            status_text,
            headers: response_headers,
            body,
            time_ms,
            size_bytes,
        })
    }
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new().expect("Failed to create default HTTP client")
    }
}
