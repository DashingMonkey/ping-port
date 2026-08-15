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
        if !ch.is_alphanumeric()
            && ch != '_'
            && ch != '-'
            && ch != '.'
            && ch != ' '
        {
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
#[tauri::command]
pub async fn scan_workspaces(app_handle: tauri::AppHandle) -> Result<ScanResult, String> {
    let mut workspaces = Vec::new();
    let mut seen_paths = std::collections::HashSet::new();

    // Scan exe directory
    let exe_dir = get_exe_dir(&app_handle);

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

    Ok(ScanResult { workspaces })
}

/// Switch to a different workspace
#[tauri::command]
pub async fn switch_workspace(
    state: tauri::State<'_, std::sync::Mutex<crate::AppState>>,
    app_handle: tauri::AppHandle,
    name: String,
) -> Result<(), String> {
    let db_path = find_workspace_path(&app_handle, &name)
        .ok_or_else(|| format!("Workspace '{}' not found", name))?;

    // Switch database
    {
        let db = state.lock().map_err(|e| e.to_string())?;
        db.db
            .switch_db(&db_path)
            .map_err(|e| format!("Failed to switch database: {}", e))?;
        *db.current_workspace
            .lock()
            .map_err(|e| format!("Internal error: {}", e))? = Some(name.clone());
    }

    log::info!("Switched to workspace: {} at {:?}", name, db_path);
    Ok(())
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

    // Check if renaming the currently active workspace
    let is_current = {
        let app_state = state.lock().map_err(|e| e.to_string())?;
        let is_current = *app_state
            .db
            .path
            .lock()
            .map_err(|e| format!("Internal error: {}", e))?
            == old_path;

        if is_current {
            app_state
                .db
                .close()
                .map_err(|e| format!("Failed to close database: {}", e))?;
        }
        is_current
    };

    if is_current {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    std::fs::rename(&old_path, &new_path)
        .map_err(|e| format!("Failed to rename workspace: {}", e))?;

    // Move WAL and SHM files
    let wal_path = old_path.with_extension("db-wal");
    let shm_path = old_path.with_extension("db-shm");
    if wal_path.exists() {
        std::fs::rename(&wal_path, new_path.with_extension("db-wal"))
            .map_err(|e| format!("Failed to rename wal file: {}", e))?;
    }
    if shm_path.exists() {
        std::fs::rename(&shm_path, new_path.with_extension("db-shm"))
            .map_err(|e| format!("Failed to rename shm file: {}", e))?;
    }

    // Update current workspace in app state
    let app_state = state.lock().map_err(|e| e.to_string())?;
    {
        let mut cw = app_state
            .current_workspace
            .lock()
            .map_err(|e| format!("Internal error: {}", e))?;
        if cw.as_ref() == Some(&old_name) {
            *cw = Some(new_name.clone());
        }
    }
    // Reconnect to the renamed database so subsequent operations work correctly
    if is_current {
        *app_state.db.path.lock().map_err(|e| format!("Internal error: {}", e))? = new_path.clone();
        app_state
            .db
            .switch_db(&new_path)
            .map_err(|e| format!("Failed to reconnect to renamed workspace: {}", e))?;
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
    let exe_dir = get_exe_dir(&app_handle);

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

    // Determine which db to use
    let (db_path, workspace_name) = if db_files.is_empty() {
        // No db files exist, create default.db
        let path = exe_dir.join("default.db");
        log::info!("No existing database found, creating default.db");
        (path, "default".to_string())
    } else {
        // Sort to get deterministic order
        db_files.sort();

        if let Some(ref last) = last_workspace_name {
            // Try to find the last workspace
            let last_db_path = exe_dir.join(format!("{}.db", last));
            if last_db_path.exists() {
                log::info!("Using last workspace: {}", last);
                (last_db_path, last.clone())
            } else {
                // Last workspace not found, use first available
                let first_path = db_files.remove(0);
                let first_name = first_path
                    .file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_else(|| "default".to_string());
                log::info!(
                    "Last workspace '{}' not found, using first: {}",
                    last,
                    first_name
                );
                (first_path, first_name)
            }
        } else {
            // No last workspace specified, use first available
            let first_path = db_files.remove(0);
            let first_name = first_path
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "default".to_string());
            log::info!("Using first available workspace: {}", first_name);
            (first_path, first_name)
        }
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

    // Build workspaces list from db_files (remaining ones)
    let workspaces: Vec<WorkspaceInfo> = db_files
        .into_iter()
        .map(|path| {
            let name = path
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();
            WorkspaceInfo {
                name,
                path: path.to_string_lossy().to_string(),
                is_temporary: false,
            }
        })
        .collect();

    log::info!(
        "Workspace initialized: {}, found {} workspaces total",
        workspace_name,
        workspaces.len() + 1
    );

    Ok(InitResult {
        current_workspace: workspace_name,
        workspaces,
    })
}
