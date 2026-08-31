use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Collection {
    pub id: String,
    pub name: String,
    #[serde(rename = "kind")]
    pub kind: Option<String>,
    pub parent_id: Option<String>,
    pub position: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateCollectionInput {
    pub id: Option<String>,
    pub name: String,
    #[serde(rename = "kind")]
    pub kind: Option<String>,
    pub parent_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCollectionInput {
    pub id: String,
    #[serde(rename = "kind")]
    pub kind: Option<String>,
    pub name: Option<String>,
    pub parent_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ReorderCollectionsInput {
    pub source_id: String,
    pub target_id: String,
}

pub fn list_collections(conn: &Connection) -> Result<Vec<Collection>, String> {
    let mut stmt = conn
        .prepare("SELECT id, name, type, parent_id, COALESCE(position, 0), created_at, updated_at FROM collections ORDER BY position ASC, created_at DESC")
        .map_err(|e| e.to_string())?;

    let collections = stmt
        .query_map([], |row| {
            Ok(Collection {
                id: row.get(0)?,
                name: row.get(1)?,
                kind: row.get(2)?,
                parent_id: row.get(3)?,
                position: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(collections)
}

/// Name of the fallback root collection used by external writers (the MCP
/// server) when a destination collection is not specified.
pub const DEFAULT_COLLECTION_NAME: &str = "Default";

/// Find a root-level collection by name, creating it if it does not exist.
/// Used as the implicit destination for MCP create_request calls that omit
/// collection_id. Only root-level collections (parent_id IS NULL) match, so
/// a same-named sub-collection is never chosen. Returns the collection and
/// whether it was newly created (so the caller can notify the UI).
pub fn find_or_create_root_by_name(conn: &Connection, name: &str) -> Result<(Collection, bool), String> {
    let existing = conn
        .query_row(
            "SELECT id, name, type, parent_id, COALESCE(position, 0), created_at, updated_at FROM collections WHERE name = ?1 AND parent_id IS NULL ORDER BY position ASC, created_at ASC LIMIT 1",
            params![name],
            |row| {
                Ok(Collection {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    kind: row.get(2)?,
                    parent_id: row.get(3)?,
                    position: row.get(4)?,
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                })
            },
        )
        .map(Some)
        .or_else(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => Ok(None),
            other => Err(other.to_string()),
        })?;

    match existing {
        Some(collection) => Ok((collection, false)),
        None => Ok((
            create_collection(
                conn,
                CreateCollectionInput {
                    id: None,
                    name: name.to_string(),
                    kind: None,
                    parent_id: None,
                },
            )?,
            true,
        )),
    }
}

pub fn exists(conn: &Connection, id: &str) -> Result<bool, String> {
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM collections WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    Ok(count > 0)
}

pub fn create_collection(
    conn: &Connection,
    input: CreateCollectionInput,
) -> Result<Collection, String> {
    let now = chrono::Utc::now().to_rfc3339();
    let id = input.id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    // Get max position for this parent
    let max_pos: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(position), -1) + 1 FROM collections WHERE parent_id IS ?1",
            params![input.parent_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO collections (id, name, type, parent_id, position, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![id, input.name, input.kind, input.parent_id, max_pos, now, now],
    )
    .map_err(|e| e.to_string())?;

    Ok(Collection {
        id,
        name: input.name,
        kind: input.kind,
        parent_id: input.parent_id,
        position: max_pos,
        created_at: now.clone(),
        updated_at: now,
    })
}

pub fn update_collection(conn: &Connection, input: UpdateCollectionInput) -> Result<(), String> {
    let now = chrono::Utc::now().to_rfc3339();

    let current = conn.query_row(
        "SELECT name, type, parent_id FROM collections WHERE id = ?1",
        params![input.id],
        |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, Option<String>>(2)?,
            ))
        },
    )
    .map_err(|e| format!("Collection not found: {}", e))?;

    let name = input.name.unwrap_or(current.0);
    let kind = input.kind.or(current.1);
    let parent_id = input.parent_id.or(current.2);

    conn.execute(
        "UPDATE collections SET name = ?1, type = ?2, parent_id = ?3, updated_at = ?4 WHERE id = ?5",
        params![name, kind, parent_id, now, input.id],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

pub fn delete_collection(conn: &Connection, id: &str) -> Result<(), String> {
    conn.execute("DELETE FROM collections WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn reorder_collections(
    conn: &Connection,
    input: ReorderCollectionsInput,
) -> Result<(), String> {
    // Get source and target positions
    let source_pos: i32 = conn
        .query_row(
            "SELECT COALESCE(position, 0) FROM collections WHERE id = ?1",
            params![input.source_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let target_pos: i32 = conn
        .query_row(
            "SELECT COALESCE(position, 0) FROM collections WHERE id = ?1",
            params![input.target_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let now = chrono::Utc::now().to_rfc3339();

    // Move source to target position
    conn.execute(
        "UPDATE collections SET position = ?1, updated_at = ?2 WHERE id = ?3",
        params![target_pos, now, input.source_id],
    )
    .map_err(|e| e.to_string())?;

    // Shift other collections
    if source_pos < target_pos {
        conn.execute(
            "UPDATE collections SET position = position - 1, updated_at = ?1 WHERE position > ?2 AND position <= ?3 AND id != ?4 AND parent_id IS (SELECT parent_id FROM collections WHERE id = ?4)",
            params![now, source_pos, target_pos, input.source_id],
        )
        .map_err(|e| e.to_string())?;
    } else {
        conn.execute(
            "UPDATE collections SET position = position + 1, updated_at = ?1 WHERE position >= ?2 AND position < ?3 AND id != ?4 AND parent_id IS (SELECT parent_id FROM collections WHERE id = ?4)",
            params![now, target_pos, source_pos, input.source_id],
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(())
}
