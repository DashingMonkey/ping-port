use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::Manager;

// Note: regex's \p{L} requires the unicode-gencat feature which may not
// always be enabled. We validate characters manually in validate_workspace_name
// to reliably support Unicode letters (Chinese, Japanese, etc.).

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorkspaceInfo {
    pub name: String,
    pub path: String,
    pub is_temporary: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScanResult {
    pub workspaces: Vec<WorkspaceInfo>,
}

/// Helper function to get exe directory
fn get_exe_dir(app_handle: &tauri::AppHandle) -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| {
            app_handle.path().executable_dir().unwrap_or_else(|_| {
                std::env::current_dir().expect("Failed to get current directory")
            })
        })
}

/// Validate workspace name to prevent path traversal
fn validate_workspace_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("Workspace name cannot be empty".to_string());
    }
    if name.chars().count() > 64 {
        return Err("Workspace name too long (max 64 characters)".to_string());
    }
    // Reject path traversal patterns early
    if name.contains("..") || name.contains('/') || name.contains('\\') {
        return Err("Workspace name contains invalid characters".to_string());
    }
    // Allow Unicode letters/numbers (Chinese, Japanese, etc.), plus
    // underscore, hyphen, dot, and space. Reject everything else.
    for ch in name.chars() {
        if !ch.is_alphanumeric() && ch != '_' && ch != '-' && ch != '.' && ch != ' ' {
            return Err(
                "Workspace name can only contain letters, numbers, spaces, underscores, hyphens, and dots"
                    .to_string(),
            );
        }
    }
    Ok(())
}

/// Helper function to find workspace path
fn find_workspace_path(app_handle: &tauri::AppHandle, name: &str) -> Option<PathBuf> {
    // Check exe directory first
    let exe_dir = get_exe_dir(app_handle);

    // Check exe root directory
    let exe_db_path = exe_dir.join(format!("{}.db", name));
    if exe_db_path.exists() {
        return Some(exe_db_path);
    }

    // Check exe/workspaces subdirectory
    let workspaces_dir = exe_dir.join("workspaces");
    let shared_db_path = workspaces_dir.join(format!("{}.db", name));
    if shared_db_path.exists() {
        return Some(shared_db_path);
    }

    None
}

/// Scan all workspaces from exe directories
fn scan_workspaces_list(app_handle: &tauri::AppHandle) -> Vec<WorkspaceInfo> {
    let mut workspaces = Vec::new();
    let mut seen_paths = std::collections::HashSet::new();

    // Scan exe directory
    let exe_dir = get_exe_dir(app_handle);

    // Scan for *.db files directly in exe directory
    if let Ok(entries) = std::fs::read_dir(&exe_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "db").unwrap_or(false) {
                if let Some(name) = path.file_stem() {
                    let name_str = name.to_string_lossy().to_string();
                    let is_temp = name_str.starts_with("temp-");
                    let path_str = path.to_string_lossy().to_string();
                    if !seen_paths.contains(&path_str) {
                        seen_paths.insert(path_str.clone());
                        workspaces.push(WorkspaceInfo {
                            name: name_str,
                            path: path_str,
                            is_temporary: is_temp,
                        });
                    }
                }
            }
        }
    }

    // Also scan exe/workspaces subdirectory
    let workspaces_dir = exe_dir.join("workspaces");
    if let Ok(entries) = std::fs::read_dir(&workspaces_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "db").unwrap_or(false) {
                if let Some(name) = path.file_stem() {
                    let name_str = name.to_string_lossy().to_string();
                    let path_str = path.to_string_lossy().to_string();
                    if !seen_paths.contains(&path_str) {
                        seen_paths.insert(path_str.clone());
                        workspaces.push(WorkspaceInfo {
                            name: name_str,
                            path: path_str,
                            is_temporary: false,
                        });
                    }
                }
            }
        }
    }

    workspaces
}

/// Scan all workspaces from exe directories
#[tauri::command]
pub async fn scan_workspaces(app_handle: tauri::AppHandle) -> Result<ScanResult, String> {
    Ok(ScanResult {
        workspaces: scan_workspaces_list(&app_handle),
    })
}

/// Switch the active database to the given workspace (shared by command / init / MCP)
pub fn switch_to_workspace(app_handle: &tauri::AppHandle, name: &str) -> Result<(), String> {
    validate_workspace_name(name)?;
    let db_path = find_workspace_path(app_handle, name)
        .ok_or_else(|| format!("Workspace '{}' not found", name))?;

    let state = app_handle.state::<std::sync::Mutex<crate::AppState>>();
    let guard = state.lock().map_err(|e| e.to_string())?;
    guard
        .db
        .switch_db(&db_path)
        .map_err(|e| format!("Failed to switch database: {}", e))?;
    *guard
        .current_workspace
        .lock()
        .map_err(|e| format!("Internal error: {}", e))? = Some(name.to_string());

    log::info!("Switched to workspace: {} at {:?}", name, db_path);
    Ok(())
}

/// Switch to a different workspace
#[tauri::command]
pub async fn switch_workspace(app_handle: tauri::AppHandle, name: String) -> Result<(), String> {
    switch_to_workspace(&app_handle, &name)
}

/// Get current workspace name
#[tauri::command]
pub fn get_current_workspace_name(
    state: tauri::State<'_, std::sync::Mutex<crate::AppState>>,
) -> Result<Option<String>, String> {
    let db = state.lock().map_err(|e| e.to_string())?;
    let current = db
        .current_workspace
        .lock()
        .map_err(|e| format!("Internal error: {}", e))?
        .clone();
    Ok(current)
}

/// Rename a workspace (rename the db file)
#[tauri::command]
pub async fn rename_workspace(
    state: tauri::State<'_, std::sync::Mutex<crate::AppState>>,
    app_handle: tauri::AppHandle,
    old_name: String,
    new_name: String,
) -> Result<(), String> {
    validate_workspace_name(&old_name)?;
    validate_workspace_name(&new_name)?;
    let exe_dir = get_exe_dir(&app_handle);

    let old_path = find_workspace_path(&app_handle, &old_name)
        .ok_or_else(|| format!("Workspace '{}' not found", old_name))?;

    let new_path = if old_path.parent() == Some(&exe_dir) {
        exe_dir.join(format!("{}.db", new_name))
    } else {
        exe_dir.join("workspaces").join(format!("{}.db", new_name))
    };

    // Check if new_name already exists
    if new_path.exists() {
        return Err(format!("Workspace '{}' already exists", new_name));
    }

    // Keep the state lock throughout the rename so no command can switch
    // databases while the active connection is temporarily closed.
    let app_state = state.lock().map_err(|e| e.to_string())?;
    let is_current = *app_state.db.path.lock().map_err(|e| e.to_string())? == old_path;
    if is_current {
        app_state
            .db
            .close()
            .map_err(|e| format!("Failed to close database: {}", e))?;
    } else {
        // Checkpoint an inactive database too, so all data is in the main file.
        let db = crate::db::Database::new(&old_path).map_err(|e| e.to_string())?;
        db.close().map_err(|e| e.to_string())?;
    }

    if let Err(error) = std::fs::rename(&old_path, &new_path) {
        if is_current {
            app_state.db.switch_db(&old_path).map_err(|restore| {
                format!(
                    "Failed to rename workspace: {}; failed to reopen original: {}",
                    error, restore
                )
            })?;
        }
        return Err(format!("Failed to rename workspace: {}", error));
    }

    if is_current {
        if let Err(error) = app_state.db.switch_db(&new_path) {
            std::fs::rename(&new_path, &old_path).map_err(|rollback| {
                format!(
                    "Failed to reopen renamed workspace: {}; rollback failed: {}",
                    error, rollback
                )
            })?;
            app_state.db.switch_db(&old_path).map_err(|restore| {
                format!(
                    "Failed to reopen renamed workspace: {}; restore failed: {}",
                    error, restore
                )
            })?;
            return Err(format!("Failed to reopen renamed workspace: {}", error));
        }
        *app_state
            .current_workspace
            .lock()
            .map_err(|e| e.to_string())? = Some(new_name.clone());
    }

    log::info!("Renamed workspace '{}' to '{}'", old_name, new_name);
    Ok(())
}

/// Create a new workspace database file
#[tauri::command]
pub async fn create_workspace(app_handle: tauri::AppHandle, name: String) -> Result<(), String> {
    validate_workspace_name(&name)?;
    let exe_dir = get_exe_dir(&app_handle);
    let db_path = exe_dir.join(format!("{}.db", name));

    if db_path.exists() {
        return Err(format!("Workspace '{}' already exists", name));
    }

    crate::db::Database::new(&db_path).map_err(|e| format!("Failed to create database: {}", e))?;

    if !db_path.exists() {
        return Err("Failed to create workspace database file".to_string());
    }

    log::info!("Created new workspace: {} at {:?}", name, db_path);
    Ok(())
}

/// Exit the application
#[tauri::command]
pub async fn exit_app(app_handle: tauri::AppHandle) -> Result<(), String> {
    log::info!("Exiting application via user request");
    app_handle.exit(0);
    Ok(())
}

/// Ensure backend AppState is initialized: scan db files, pick default workspace,
/// open database and register state. Idempotent. Called from app setup (before
/// frontend loads) so backend-only consumers (MCP server) have a DB to work with.
pub fn init_state(app_handle: &tauri::AppHandle) -> Result<String, String> {
    if let Some(state) = app_handle.try_state::<std::sync::Mutex<crate::AppState>>() {
        let guard = state.lock().map_err(|e| e.to_string())?;
        if let Some(name) = guard
            .current_workspace
            .lock()
            .map_err(|e| e.to_string())?
            .clone()
        {
            return Ok(name);
        }
        return Err("Workspace state initialized without a workspace name".to_string());
    }

    let exe_dir = get_exe_dir(app_handle);

    // Scan for all .db files in exe directory
    let mut db_files: Vec<PathBuf> = std::fs::read_dir(&exe_dir)
        .map(|entries| {
            entries
                .flatten()
                .filter(|e| e.path().extension().map(|ext| ext == "db").unwrap_or(false))
                .map(|e| e.path())
                .collect()
        })
        .unwrap_or_default();

    log::info!("Found {} db files in {:?}", db_files.len(), exe_dir);

    // No last workspace known at setup time: create default.db or use first available
    let (db_path, workspace_name) = if db_files.is_empty() {
        let path = exe_dir.join("default.db");
        log::info!("No existing database found, creating default.db");
        (path, "default".to_string())
    } else {
        db_files.sort();
        let first_path = db_files.remove(0);
        let first_name = first_path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "default".to_string());
        log::info!("Using first available workspace: {}", first_name);
        (first_path, first_name)
    };

    log::info!("Opening database: {:?}", db_path);

    // Open the database
    let db = crate::db::Database::new(&db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;

    // Register AppState
    let http_client = crate::http::client::HttpClient::new()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;
    let state = crate::AppState {
        db: Arc::new(db),
        current_workspace: Mutex::new(Some(workspace_name.clone())),
        http_client,
    };
    app_handle.manage(Mutex::new(state));

    log::info!("Backend workspace initialized: {}", workspace_name);
    Ok(workspace_name)
}

/// Initialize workspace: scan for db files, decide which to use, and register AppState
#[derive(Debug, Serialize, Deserialize)]
pub struct InitResult {
    pub current_workspace: String,
    pub workspaces: Vec<WorkspaceInfo>,
}

#[tauri::command]
pub async fn init_workspace(
    app_handle: tauri::AppHandle,
    last_workspace_name: Option<String>,
) -> Result<InitResult, String> {
    // Backend setup normally registers AppState already; retry here covers
    // the case where setup failed (e.g. db locked at startup).
    let mut current = init_state(&app_handle)?;

    // Restore the last workspace remembered by the frontend if it still exists
    if let Some(last) = &last_workspace_name {
        if *last != current && find_workspace_path(&app_handle, last).is_some() {
            log::info!("Restoring last workspace: {}", last);
            switch_to_workspace(&app_handle, last)?;
            current = last.clone();
        }
    }

    // The selector needs the current workspace too, to display its selected
    // state on the first render just as it does after scan_workspaces.
    let workspaces = scan_workspaces_list(&app_handle);

    log::info!(
        "Workspace initialized: {}, found {} workspaces total",
        current,
        workspaces.len()
    );

    Ok(InitResult {
        current_workspace: current,
        workspaces,
    })
}
