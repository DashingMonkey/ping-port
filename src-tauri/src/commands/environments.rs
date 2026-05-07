use crate::db::Database;
use crate::AppState;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct Environment {
    pub id: String,
    pub name: String,
    pub variables: HashMap<String, String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateEnvironmentInput {
    pub id: Option<String>,
    pub name: String,
    pub variables: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateEnvironmentInput {
    pub id: String,
    pub name: Option<String>,
    pub variables: Option<HashMap<String, String>>,
}

fn get_db<'a>(
    state: &'a State<'a, Mutex<AppState>>,
) -> Result<std::sync::MutexGuard<'a, AppState>, String> {
    state.lock().map_err(|e| format!("Internal error: {}", e))
}

fn parse_variables(variables_json: &str) -> HashMap<String, String> {
    serde_json::from_str(variables_json).unwrap_or_default()
}

fn serialize_variables(variables: &HashMap<String, String>) -> String {
    serde_json::to_string(variables).unwrap_or_else(|_| "{}".to_string())
}

#[tauri::command]
pub fn get_environments(state: State<'_, Mutex<AppState>>) -> Result<Vec<Environment>, String> {
    let app_state = get_db(&state)?;
    let db: &Database = &app_state.db;
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT id, name, COALESCE(variables, '{}'), created_at, updated_at FROM environments ORDER BY created_at DESC")
        .map_err(|e| e.to_string())?;

    let environments = stmt
        .query_map([], |row| {
            let variables_json: String = row.get(2)?;
            Ok(Environment {
                id: row.get(0)?,
                name: row.get(1)?,
                variables: parse_variables(&variables_json),
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(environments)
}

#[tauri::command]
pub fn create_environment(
    input: CreateEnvironmentInput,
    state: State<'_, Mutex<AppState>>,
) -> Result<Environment, String> {
    let app_state = get_db(&state)?;
    let db: &Database = &app_state.db;
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let now = chrono::Utc::now().to_rfc3339();
    let id = input.id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let variables = input.variables.unwrap_or_default();
    let variables_json = serialize_variables(&variables);

    conn.execute(
        "INSERT INTO environments (id, name, variables, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![id, input.name, variables_json, now, now],
    )
    .map_err(|e| e.to_string())?;

    Ok(Environment {
        id,
        name: input.name,
        variables,
        created_at: now.clone(),
        updated_at: now,
    })
}

#[tauri::command]
pub fn update_environment(
    input: UpdateEnvironmentInput,
    state: State<'_, Mutex<AppState>>,
) -> Result<(), String> {
    let app_state = get_db(&state)?;
    let db: &Database = &app_state.db;
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let now = chrono::Utc::now().to_rfc3339();

    // Get current environment to merge variables
    let mut stmt = conn
        .prepare("SELECT variables FROM environments WHERE id = ?1")
        .map_err(|e| e.to_string())?;
    let current_variables: String = stmt
        .query_row(params![input.id], |row| row.get(0))
        .map_err(|e| e.to_string())?;

    let mut variables = parse_variables(&current_variables);

    // Update name if provided
    let name = if let Some(name) = input.name {
        name
    } else {
        let mut stmt = conn
            .prepare("SELECT name FROM environments WHERE id = ?1")
            .map_err(|e| e.to_string())?;
        stmt.query_row(params![input.id], |row| row.get(0))
            .map_err(|e| e.to_string())?
    };

    // Merge variables if provided
    if let Some(new_vars) = input.variables {
        variables.extend(new_vars);
    }

    let variables_json = serialize_variables(&variables);

    conn.execute(
        "UPDATE environments SET name = ?1, variables = ?2, updated_at = ?3 WHERE id = ?4",
        params![name, variables_json, now, input.id],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn delete_environment(id: String, state: State<'_, Mutex<AppState>>) -> Result<(), String> {
    let app_state = get_db(&state)?;
    let db: &Database = &app_state.db;
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    conn.execute("DELETE FROM environments WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn delete_environment_variable(
    env_id: String,
    key: String,
    state: State<'_, Mutex<AppState>>,
) -> Result<(), String> {
    let app_state = get_db(&state)?;
    let db: &Database = &app_state.db;
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    // Get current variables
    let mut stmt = conn
        .prepare("SELECT variables FROM environments WHERE id = ?1")
        .map_err(|e| e.to_string())?;
    let current_variables: String = stmt
        .query_row(params![env_id], |row| row.get(0))
        .map_err(|e| e.to_string())?;

    let mut variables = parse_variables(&current_variables);
    variables.remove(&key);

    let variables_json = serialize_variables(&variables);
    let now = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "UPDATE environments SET variables = ?1, updated_at = ?2 WHERE id = ?3",
        params![variables_json, now, env_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}
