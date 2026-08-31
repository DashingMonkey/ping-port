use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Request {
    pub id: String,
    pub collection_id: String,
    pub name: String,
    pub method: String,
    pub url: String,
    pub params: Option<String>,
    pub headers: Option<String>,
    pub body: Option<String>,
    pub auth: Option<String>,
    pub pre_request_script: Option<String>,
    pub test_script: Option<String>,
    pub position: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateRequestInput {
    pub id: Option<String>,
    pub collection_id: String,
    pub name: String,
    pub method: String,
    pub url: Option<String>,
    pub params: Option<String>,
    pub headers: Option<String>,
    pub body: Option<String>,
    pub auth: Option<String>,
    pub pre_request_script: Option<String>,
    pub test_script: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRequestInput {
    pub id: String,
    pub collection_id: Option<String>,
    pub name: Option<String>,
    pub method: Option<String>,
    pub url: Option<String>,
    pub params: Option<String>,
    pub headers: Option<String>,
    pub body: Option<String>,
    pub auth: Option<String>,
    pub pre_request_script: Option<String>,
    pub test_script: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ReorderRequestsInput {
    pub source_id: String,
    pub target_id: String,
}

pub fn list_requests(conn: &Connection) -> Result<Vec<Request>, String> {
    let mut stmt = conn
        .prepare("SELECT id, collection_id, name, method, COALESCE(url, ''), COALESCE(params, ''), COALESCE(headers, ''), COALESCE(body, ''), COALESCE(auth, ''), COALESCE(pre_request_script, ''), COALESCE(test_script, ''), COALESCE(position, 0), created_at, updated_at FROM requests ORDER BY position ASC, created_at DESC")
        .map_err(|e| e.to_string())?;

    let requests = stmt
        .query_map([], |row| {
            Ok(Request {
                id: row.get(0)?,
                collection_id: row.get(1)?,
                name: row.get(2)?,
                method: row.get(3)?,
                url: row.get(4)?,
                params: row.get(5)?,
                headers: row.get(6)?,
                body: row.get(7)?,
                auth: row.get(8)?,
                pre_request_script: row.get(9)?,
                test_script: row.get(10)?,
                position: row.get(11)?,
                created_at: row.get(12)?,
                updated_at: row.get(13)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(requests)
}

pub fn create_request(conn: &Connection, input: CreateRequestInput) -> Result<Request, String> {
    let now = chrono::Utc::now().to_rfc3339();
    let id = input.id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    // Normalize optional fields to empty string if None
    let url = input.url.unwrap_or_default();
    let params = input.params.unwrap_or_default();
    let headers = input.headers.unwrap_or_default();
    let body = input.body.unwrap_or_default();
    let auth = input.auth.unwrap_or_default();
    let pre_request_script = input.pre_request_script.unwrap_or_default();
    let test_script = input.test_script.unwrap_or_default();

    // Get max position for this collection
    let max_pos: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(position), -1) + 1 FROM requests WHERE collection_id = ?1",
            params![input.collection_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO requests (id, collection_id, name, method, url, params, headers, body, auth, pre_request_script, test_script, position, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
        params![
            id,
            input.collection_id,
            input.name,
            input.method,
            url,
            params,
            headers,
            body,
            auth,
            pre_request_script,
            test_script,
            max_pos,
            now,
            now
        ],
    )
    .map_err(|e| e.to_string())?;

    Ok(Request {
        id,
        collection_id: input.collection_id,
        name: input.name,
        method: input.method,
        url,
        params: Some(params),
        headers: Some(headers),
        body: Some(body),
        auth: Some(auth),
        pre_request_script: Some(pre_request_script),
        test_script: Some(test_script),
        position: max_pos,
        created_at: now.clone(),
        updated_at: now,
    })
}

pub fn update_request(conn: &Connection, input: UpdateRequestInput) -> Result<(), String> {
    let now = chrono::Utc::now().to_rfc3339();

    // Fetch current request to support partial updates
    let current = conn.query_row(
        "SELECT collection_id, name, method, COALESCE(url, ''), COALESCE(params, ''), COALESCE(headers, ''), COALESCE(body, ''), COALESCE(auth, ''), COALESCE(pre_request_script, ''), COALESCE(test_script, '') FROM requests WHERE id = ?1",
        params![input.id],
        |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, String>(7)?,
                row.get::<_, String>(8)?,
                row.get::<_, String>(9)?,
            ))
        },
    )
    .map_err(|e| format!("Request not found: {}", e))?;

    let collection_id = input.collection_id.unwrap_or(current.0);
    let name = input.name.unwrap_or(current.1);
    let method = input.method.unwrap_or(current.2);
    let url = input.url.unwrap_or(current.3);
    let params = input.params.unwrap_or(current.4);
    let headers = input.headers.unwrap_or(current.5);
    let body = input.body.unwrap_or(current.6);
    let auth = input.auth.unwrap_or(current.7);
    let pre_request_script = input.pre_request_script.unwrap_or(current.8);
    let test_script = input.test_script.unwrap_or(current.9);

    conn.execute(
        "UPDATE requests SET collection_id = ?1, name = ?2, method = ?3, url = ?4, params = ?5, headers = ?6, body = ?7, auth = ?8, pre_request_script = ?9, test_script = ?10, updated_at = ?11 WHERE id = ?12",
        params![
            collection_id,
            name,
            method,
            url,
            params,
            headers,
            body,
            auth,
            pre_request_script,
            test_script,
            now,
            input.id
        ],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

pub fn delete_request(conn: &Connection, id: &str) -> Result<(), String> {
    conn.execute("DELETE FROM requests WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn reorder_requests(conn: &Connection, input: ReorderRequestsInput) -> Result<(), String> {
    // Get source and target positions
    let source_pos: i32 = conn
        .query_row(
            "SELECT COALESCE(position, 0) FROM requests WHERE id = ?1",
            params![input.source_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let target_pos: i32 = conn
        .query_row(
            "SELECT COALESCE(position, 0) FROM requests WHERE id = ?1",
            params![input.target_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let now = chrono::Utc::now().to_rfc3339();

    // Move source to target position
    conn.execute(
        "UPDATE requests SET position = ?1, updated_at = ?2 WHERE id = ?3",
        params![target_pos, now, input.source_id],
    )
    .map_err(|e| e.to_string())?;

    // Shift other requests
    if source_pos < target_pos {
        conn.execute(
            "UPDATE requests SET position = position - 1, updated_at = ?1 WHERE position > ?2 AND position <= ?3 AND id != ?4 AND collection_id = (SELECT collection_id FROM requests WHERE id = ?4)",
            params![now, source_pos, target_pos, input.source_id],
        )
        .map_err(|e| e.to_string())?;
    } else {
        conn.execute(
            "UPDATE requests SET position = position + 1, updated_at = ?1 WHERE position >= ?2 AND position < ?3 AND id != ?4 AND collection_id = (SELECT collection_id FROM requests WHERE id = ?4)",
            params![now, target_pos, source_pos, input.source_id],
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(())
}
