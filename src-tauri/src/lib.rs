mod commands;
mod db;
mod http;
pub mod scripting;

use commands::collections::{
    create_collection, delete_collection, get_collections, reorder_collections, update_collection,
};
use commands::dialog::pick_file;
use commands::environments::{
    create_environment, delete_environment, delete_environment_variable, get_environments,
    update_environment,
};
use commands::http::send_request;
use commands::requests::{
    create_request, delete_request, get_requests, reorder_requests, update_request,
};
use commands::script::execute_script;
use commands::workspace::{
    create_workspace, exit_app, get_current_workspace_name, init_workspace, rename_workspace,
    scan_workspaces, switch_workspace,
};
use db::Database;
use std::sync::{Arc, Mutex};
use tauri::Manager;

pub struct AppState {
    pub db: Arc<Database>,
    pub current_workspace: Mutex<Option<String>>,
    pub http_client: crate::http::client::HttpClient,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();
    log::info!("Starting PingPort application");

    check_webview2();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            get_collections,
            pick_file,
            create_collection,
            update_collection,
            delete_collection,
            reorder_collections,
            get_requests,
            create_request,
            update_request,
            delete_request,
            reorder_requests,
            send_request,
            get_environments,
            create_environment,
            update_environment,
            delete_environment,
            delete_environment_variable,
            execute_script,
            scan_workspaces,
            switch_workspace,
            rename_workspace,
            create_workspace,
            get_current_workspace_name,
            exit_app,
            init_workspace,
        ])
        .setup(|app: &mut tauri::App| {
            // Get exe directory using std::env::current_exe for reliable path
            let exe_dir = std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|p| p.to_path_buf()))
                .unwrap_or_else(|| {
                    app.path().executable_dir().unwrap_or_else(|_| {
                        std::env::current_exe()
                            .ok()
                            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
                            .unwrap_or_else(|| std::env::temp_dir())
                    })
                });

            log::info!("exe_dir: {:?}", exe_dir);

            // Create database directory if it doesn't exist
            if let Err(e) = std::fs::create_dir_all(&exe_dir) {
                let msg = format!("无法创建数据库目录 / Failed to create database directory: {}", e);
                log::error!("{}", msg);
                rfd::MessageDialog::new()
                    .set_title("PingPort Error")
                    .set_description(&msg)
                    .set_level(rfd::MessageLevel::Error)
                    .set_buttons(rfd::MessageButtons::Ok)
                    .show();
                return Err(Box::new(e));
            }

            log::info!("PingPort setup complete, waiting for frontend init");
            Ok(())
        })
        .run(tauri::generate_context!())
        .unwrap_or_else(|e| {
            let msg = format!("Error while running tauri application: {}", e);
            log::error!("{}", msg);
            rfd::MessageDialog::new()
                .set_title("PingPort Error")
                .set_description(&msg)
                .set_level(rfd::MessageLevel::Error)
                .set_buttons(rfd::MessageButtons::Ok)
                .show();
        });
}

#[cfg(target_os = "windows")]
fn check_webview2() {
    let webview2_download_url = "https://go.microsoft.com/fwlink/p/?LinkId=2124703";

    // Strategy 1: Registry check
    let detected = detect_webview2_registry();
    if detected.is_none() {
        // Strategy 2: File system check (known install paths)
        log::info!("Registry check failed, falling back to file system check");
        if let Some(path) = detect_webview2_fs() {
            log::info!("WebView2 found at: {}", path.display());
            return;
        }
        // Strategy 3: Check loader DLLs in system paths
        log::info!("File system check failed, falling back to loader DLL check");
        if detect_webview2_loader() {
            log::info!("WebView2 loader DLL detected");
            return;
        }
    } else {
        log::info!("WebView2 Runtime detected via registry");
        return;
    }

    // All strategies failed
    let msg = format!(
        "未检测到 WebView2 运行时，PingPort 依赖此组件才能运行。\nWebView2 Runtime is not installed. PingPort requires it to run.\n\n请从以下地址下载安装 / Download from:\n{}",
        webview2_download_url
    );
    log::error!("{}", msg);
    rfd::MessageDialog::new()
        .set_title("PingPort - 缺少依赖 Missing Dependency")
        .set_description(&msg)
        .set_level(rfd::MessageLevel::Error)
        .set_buttons(rfd::MessageButtons::Ok)
        .show();
    std::process::exit(1);
}

#[cfg(target_os = "windows")]
fn detect_webview2_registry() -> Option<String> {
    use winreg::enums::*;
    use winreg::RegKey;

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let key_paths = [
        r"SOFTWARE\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}",
    ];

    for key_path in &key_paths {
        if let Ok(key) = hklm.open_subkey_with_flags(key_path, KEY_READ) {
            if let Ok(pv) = key.get_value::<String, _>("pv") {
                return Some(pv);
            }
        }
    }

    // Also try WOW6432Node for 32-bit registry view
    let wow_paths = [
        r"SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}",
    ];
    for key_path in &wow_paths {
        if let Ok(key) = hklm.open_subkey_with_flags(key_path, KEY_READ) {
            if let Ok(pv) = key.get_value::<String, _>("pv") {
                return Some(pv);
            }
        }
    }

    None
}

#[cfg(target_os = "windows")]
fn detect_webview2_fs() -> Option<std::path::PathBuf> {
    use std::path::PathBuf;

    let candidates = [
        PathBuf::from(r"C:\Program Files (x86)\Microsoft\EdgeWebView\Application"),
        PathBuf::from(r"C:\Program Files\Microsoft\EdgeWebView\Application"),
    ];

    // Also check env vars
    if let Ok(pf) = std::env::var("ProgramFiles(x86)") {
        let p = PathBuf::from(&pf).join(r"Microsoft\EdgeWebView\Application");
        if p.exists() && p.is_dir() {
            // Check if any version subfolder contains msedgewebview2.exe
            if has_webview2_content(&p) {
                return Some(p);
            }
        }
    }
    if let Ok(pf) = std::env::var("ProgramFiles") {
        let p = PathBuf::from(&pf).join(r"Microsoft\EdgeWebView\Application");
        if p.exists() && p.is_dir() && has_webview2_content(&p) {
            return Some(p);
        }
    }

    for p in &candidates {
        if p.exists() && p.is_dir() && has_webview2_content(p) {
            return Some(p.clone());
        }
    }

    None
}

#[cfg(target_os = "windows")]
fn has_webview2_content(dir: &std::path::Path) -> bool {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if path.join("msedgewebview2.exe").exists()
                    || path.join("EBWebView").exists()
                {
                    return true;
                }
            }
        }
    }
    false
}

#[cfg(target_os = "windows")]
fn detect_webview2_loader() -> bool {
    // Check common locations for webview2loader.dll
    let dll_paths = [
        // System32 (64-bit on 64-bit systems)
        r"C:\Windows\System32\webview2loader.dll",
        // SysWOW64 (32-bit on 64-bit systems)
        r"C:\Windows\SysWOW64\webview2loader.dll",
    ];

    for path in &dll_paths {
        if std::path::Path::new(path).exists() {
            return true;
        }
    }

    // Try to find via PATH
    if let Ok(path_var) = std::env::var("PATH") {
        for dir in path_var.split(';') {
            let dll_path = std::path::Path::new(dir).join("webview2loader.dll");
            if dll_path.exists() {
                return true;
            }
        }
    }

    false
}

#[cfg(not(target_os = "windows"))]
fn check_webview2() {}
