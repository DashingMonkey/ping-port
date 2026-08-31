use crate::services::requests as svc;
use crate::AppState;
use rusqlite::Connection;
use std::sync::Mutex;
use tauri::State;

pub use svc::{
    CreateRequestInput, ReorderRequestsInput, Request, UpdateRequestInput,
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
pub fn get_requests(state: State<'_, Mutex<AppState>>) -> Result<Vec<Request>, String> {
    with_conn(&state, svc::list_requests)
}

#[tauri::command]
pub fn create_request(
    input: CreateRequestInput,
    state: State<'_, Mutex<AppState>>,
) -> Result<Request, String> {
    with_conn(&state, |conn| svc::create_request(conn, input))
}

#[tauri::command]
pub fn update_request(
    input: UpdateRequestInput,
    state: State<'_, Mutex<AppState>>,
) -> Result<(), String> {
    with_conn(&state, |conn| svc::update_request(conn, input))
}

#[tauri::command]
pub fn delete_request(id: String, state: State<'_, Mutex<AppState>>) -> Result<(), String> {
    with_conn(&state, |conn| svc::delete_request(conn, &id))
}

#[tauri::command]
pub fn reorder_requests(
    input: ReorderRequestsInput,
    state: State<'_, Mutex<AppState>>,
) -> Result<(), String> {
    with_conn(&state, |conn| svc::reorder_requests(conn, input))
}
