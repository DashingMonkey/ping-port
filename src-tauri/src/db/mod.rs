pub mod migrations;
pub mod schema;

use rusqlite::{Connection, Result};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

pub struct Database {
    pub conn: Mutex<Option<Connection>>,
    pub path: Mutex<PathBuf>, // Record current db path
}

impl Database {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path.as_ref())?;
        let db = Database {
            conn: Mutex::new(Some(conn)),
            path: Mutex::new(path.as_ref().to_path_buf()),
        };
        db.init()?;
        Ok(db)
    }

    fn init(&self) -> Result<()> {
        let guard = self.conn.lock().map_err(|e| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Mutex poisoned: {}", e),
            )))
        })?;
        let conn = guard.as_ref().ok_or_else(|| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Database is closed",
            )))
        })?;
        migrations::run(conn)
    }

    pub fn switch_db<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let mut guard = self.conn.lock().map_err(|e| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Mutex poisoned: {}", e),
            )))
        })?;
        let conn = Connection::open(path.as_ref())?;
        migrations::run(&conn)?;
        *guard = Some(conn);
        Ok(())
    }

    /// Close the current database connection and release file locks.
    /// After calling this, the caller should switch to a different workspace.
    pub fn close(&self) -> Result<()> {
        let mut guard = self.conn.lock().map_err(|e| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Mutex poisoned: {}", e),
            )))
        })?;
        if let Some(ref conn) = *guard {
            conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE); PRAGMA journal_mode=DELETE;")?;
        }
        *guard = None;
        Ok(())
    }
}
