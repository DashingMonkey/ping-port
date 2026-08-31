use crate::services::collections as svc;
use crate::AppState;
use rusqlite::Connection;
use std::sync::Mutex;
use tauri::State;

pub use svc::{
    Collection, CreateCollectionInput, ReorderCollectionsInput, UpdateCollectionInput,
};

fn with_conn<T>(
    state: &State<'_, Mutex<AppState>>,
    f: impl FnOnce(&Connection) -> Result<T, String>,
) -> Result<T, String> {
    let app_state = state.lock().map_err(|e| format!("Internal error: {}", e))?;
    let db = &app_state.db;
    let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
    let conn = conn_guard.as_ref().ok_or("Database is closed".to_string())?;
    f(conn)
}

#[tauri::command]
pub fn get_collections(state: State<'_, Mutex<AppState>>) -> Result<Vec<Collection>, String> {
    with_conn(&state, svc::list_collections)
}

#[tauri::command]
pub fn create_collection(
    input: CreateCollectionInput,
    state: State<'_, Mutex<AppState>>,
) -> Result<Collection, String> {
    with_conn(&state, |conn| svc::create_collection(conn, input))
}

#[tauri::command]
pub fn update_collection(
    input: UpdateCollectionInput,
    state: State<'_, Mutex<AppState>>,
) -> Result<(), String> {
    with_conn(&state, |conn| svc::update_collection(conn, input))
}

#[tauri::command]
pub fn delete_collection(id: String, state: State<'_, Mutex<AppState>>) -> Result<(), String> {
    with_conn(&state, |conn| svc::delete_collection(conn, &id))
}

#[tauri::command]
pub fn reorder_collections(
    input: ReorderCollectionsInput,
    state: State<'_, Mutex<AppState>>,
) -> Result<(), String> {
    with_conn(&state, |conn| svc::reorder_collections(conn, input))
}
