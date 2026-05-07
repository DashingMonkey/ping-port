pub mod migrations;
pub mod schema;

use rusqlite::{Connection, Result};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

pub struct Database {
    pub conn: Mutex<Connection>,
    pub path: Mutex<PathBuf>, // Record current db path
}

impl Database {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path.as_ref())?;
        let db = Database {
            conn: Mutex::new(conn),
            path: Mutex::new(path.as_ref().to_path_buf()),
        };
        db.init()?;
        Ok(db)
    }

    fn init(&self) -> Result<()> {
        let conn = self.conn.lock().map_err(|e| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Mutex poisoned: {}", e),
            )))
        })?;
        migrations::run(&conn)
    }

    pub fn switch_db<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let mut conn = self.conn.lock().map_err(|e| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Mutex poisoned: {}", e),
            )))
        })?;
        *conn = Connection::open(path.as_ref())?;
        migrations::run(&conn)?;
        Ok(())
    }

    /// Close the current database connection and release file locks.
    /// After calling this, the caller should switch to a different workspace.
    pub fn close(&self) -> Result<()> {
        let mut conn = self.conn.lock().map_err(|e| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Mutex poisoned: {}", e),
            )))
        })?;
        conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE); PRAGMA journal_mode=DELETE;")?;
        *conn = Connection::open_in_memory()?;
        Ok(())
    }
}
