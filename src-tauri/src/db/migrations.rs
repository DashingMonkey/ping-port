use rusqlite::{Connection, Result};

use crate::db::schema::{
    CREATE_COLLECTIONS_TABLE, CREATE_ENVIRONMENTS_TABLE, CREATE_REQUESTS_TABLE,
};

/// Run all database migrations
pub fn run(conn: &Connection) -> Result<()> {
    // Enable foreign keys
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    // Create tables
    conn.execute(CREATE_COLLECTIONS_TABLE, [])?;
    conn.execute(CREATE_REQUESTS_TABLE, [])?;
    conn.execute(CREATE_ENVIRONMENTS_TABLE, [])?;

    // Create indexes for better query performance
    create_indexes(conn)?;

    Ok(())
}

fn create_indexes(conn: &Connection) -> Result<()> {
    // Index for requests by collection
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_requests_collection_id ON requests(collection_id)",
        [],
    )?;

    // Index for ordering
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_collections_position ON collections(position)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_requests_position ON requests(position)",
        [],
    )?;

    Ok(())
}
