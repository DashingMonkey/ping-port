//! stdio ↔ HTTP bridge for MCP clients.
//!
//! Runs via `PingPort.exe mcp-bridge`: reads newline-delimited JSON-RPC
//! messages from stdin, forwards each one to the MCP HTTP server inside the
//! running PingPort app, and prints responses back to stdout. If PingPort is
//! not running, the bridge (re)starts it.

use std::io::{BufRead, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

use serde_json::json;

use super::APP_IDENTIFIER;

pub fn run_bridge() {
    let config_path = bridge_config_path();
    let (mut port, mut token) = match wait_for_config(&config_path) {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("PingPort MCP bridge: {}", e);
            std::process::exit(1);
        }
    };

    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut out = stdout.lock();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        if line.trim().is_empty() {
            continue;
        }

        let id: Option<serde_json::Value> =
            serde_json::from_str(&line).ok().and_then(|v: serde_json::Value| v.get("id").cloned());

        match post(&mut port, &mut token, &config_path, &line) {
            Ok(Some(body)) => {
                if id.is_some() {
                    let _ = writeln!(out, "{}", body);
                    let _ = out.flush();
                }
            }
            // Notification acknowledged without a response body
            Ok(None) => {}
            Err(message) => {
                if let Some(id) = id {
                    let err = json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "error": { "code": -32000, "message": message }
                    });
                    let _ = writeln!(out, "{}", err);
                    let _ = out.flush();
                } else {
                    eprintln!("PingPort MCP bridge: {}", message);
                }
            }
        }
    }
}

/// Location of the connection info written by the running app. Mirrors the
/// Tauri `app_data_dir` resolution; overridable via PINGPORT_MCP_CONFIG.
fn bridge_config_path() -> PathBuf {
    if let Ok(p) = std::env::var("PINGPORT_MCP_CONFIG") {
        return PathBuf::from(p);
    }
    #[cfg(target_os = "windows")]
    {
        if let Ok(base) = std::env::var("APPDATA") {
            return PathBuf::from(base).join(APP_IDENTIFIER).join("mcp.json");
        }
    }
    #[cfg(target_os = "macos")]
    {
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home)
                .join("Library")
                .join("Application Support")
                .join(APP_IDENTIFIER)
                .join("mcp.json");
        }
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        if let Ok(xdg) = std::env::var("XDG_DATA_HOME") {
            return PathBuf::from(xdg).join(APP_IDENTIFIER).join("mcp.json");
        }
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home)
                .join(".local")
                .join("share")
                .join(APP_IDENTIFIER)
                .join("mcp.json");
        }
    }
    PathBuf::from("mcp.json")
}

fn read_config(path: &PathBuf) -> Option<(u16, String)> {
    let content = std::fs::read_to_string(path).ok()?;
    let value: serde_json::Value = serde_json::from_str(&content).ok()?;
    let port = value.get("port")?.as_u64()? as u16;
    let token = value.get("token")?.as_str()?.to_string();
    Some((port, token))
}

/// Read the config, waiting briefly. If it never appears (app not running),
/// start the app and wait for it to come up.
fn wait_for_config(path: &PathBuf) -> Result<(u16, String), String> {
    for _ in 0..20 {
        if let Some(cfg) = read_config(path) {
            return Ok(cfg);
        }
        thread::sleep(Duration::from_millis(150));
    }

    if launch_app().is_ok() {
        for _ in 0..200 {
            if let Some(cfg) = read_config(path) {
                return Ok(cfg);
            }
            thread::sleep(Duration::from_millis(250));
        }
    }

    Err("PingPort is not running and could not be started. Start the PingPort app, then retry.".to_string())
}

fn launch_app() -> std::io::Result<()> {
    let exe = std::env::current_exe()?;
    // Detach the GUI process from the bridge's stdio: the MCP client detects
    // bridge exit via EOF on stdout, and an inherited handle would keep the
    // pipe open after the bridge itself has exited.
    Command::new(exe)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map(|_| ())
}

enum PostOutcome {
    Response(String),
    NoContent,
}

enum PostError {
    /// Connection-level failure: the app may be starting or was closed.
    Unreachable(String),
    /// The server rejected our token: the app may have restarted with a new one.
    Unauthorized,
    /// Waiting or restarting will not help.
    Fatal(String),
}

fn post_once(port: u16, token: &str, body: &str) -> Result<PostOutcome, PostError> {
    let url = format!("http://127.0.0.1:{}/mcp", port);
    match ureq::post(&url)
        .set("Authorization", &format!("Bearer {}", token))
        .timeout(Duration::from_secs(120))
        .send_string(body)
    {
        Ok(resp) => {
            let status = resp.status();
            if status == 204 {
                return Ok(PostOutcome::NoContent);
            }
            if (200..300).contains(&status) {
                let text = resp
                    .into_string()
                    .map_err(|e| PostError::Fatal(format!("Failed to read response: {}", e)))?;
                return Ok(PostOutcome::Response(text));
            }
            if status == 401 {
                return Err(PostError::Unauthorized);
            }
            Err(PostError::Fatal(format!("HTTP status {}", status)))
        }
        Err(ureq::Error::Status(code, _)) => {
            if code == 401 {
                Err(PostError::Unauthorized)
            } else {
                Err(PostError::Fatal(format!("HTTP status {}", code)))
            }
        }
        Err(ureq::Error::Transport(t)) => Err(PostError::Unreachable(format!(
            "Cannot reach PingPort: {}",
            t
        ))),
    }
}

/// Post a message, with recovery: if the app was closed or restarted, refresh
/// the connection info (restarting the app if needed) and retry.
fn post(
    port: &mut u16,
    token: &mut String,
    config_path: &PathBuf,
    body: &str,
) -> Result<Option<String>, String> {
    let mut attempt = 0u32;
    let mut recoveries = 0u32;
    loop {
        match post_once(*port, token, body) {
            Ok(PostOutcome::Response(text)) => return Ok(Some(text)),
            Ok(PostOutcome::NoContent) => return Ok(None),
            Err(PostError::Fatal(message)) => return Err(message),
            Err(PostError::Unauthorized) => {
                // The app restarted with a new token; refresh and retry.
                if let Some((p, t)) = read_config(config_path) {
                    if p != *port || t != *token {
                        *port = p;
                        *token = t;
                        attempt = 0;
                        continue;
                    }
                }
                return Err(
                    "MCP authorization failed: another PingPort instance may have taken over the connection.".to_string(),
                );
            }
            Err(PostError::Unreachable(message)) => {
                if attempt < 5 {
                    attempt += 1;
                    thread::sleep(Duration::from_millis(300));
                    continue;
                }
                if recoveries >= 2 {
                    return Err(message);
                }
                recoveries += 1;

                // The app may have restarted with a new port/token
                if let Some((p, t)) = read_config(config_path) {
                    if p != *port || t != *token {
                        *port = p;
                        *token = t;
                        attempt = 0;
                        continue;
                    }
                }

                // Config unchanged: the app is closed → (re)start it
                if let Some((p, t)) = restart_and_wait(config_path) {
                    *port = p;
                    *token = t;
                    attempt = 0;
                    continue;
                }
                return Err(message);
            }
        }
    }
}

/// Start the app and wait until it rewrites the connection info.
fn restart_and_wait(config_path: &PathBuf) -> Option<(u16, String)> {
    let before = std::fs::read_to_string(config_path).unwrap_or_default();
    let _ = launch_app();
    for _ in 0..200 {
        let now = std::fs::read_to_string(config_path).unwrap_or_default();
        if !now.is_empty() && now != before {
            if let Some(cfg) = read_config(config_path) {
                return Some(cfg);
            }
        }
        thread::sleep(Duration::from_millis(250));
    }
    None
}
