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
        let mut current_path = self.path.lock().map_err(|e| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Mutex poisoned: {}", e),
            )))
        })?;
        let mut guard = self.conn.lock().map_err(|e| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Mutex poisoned: {}", e),
            )))
        })?;
        let conn = Connection::open(path.as_ref())?;
        migrations::run(&conn)?;
        *guard = Some(conn);
        *current_path = path.as_ref().to_path_buf();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn switching_updates_path_and_keeps_original_on_failure() {
        let dir = std::env::current_dir().unwrap().join("target");
        std::fs::create_dir_all(&dir).unwrap();
        let a = dir.join(format!("review-{}.db", uuid::Uuid::new_v4()));
        let b = dir.join(format!("review-{}.db", uuid::Uuid::new_v4()));
        let db = Database::new(&a).unwrap();
        db.switch_db(&b).unwrap();
        assert_eq!(*db.path.lock().unwrap(), b);
        assert!(db
            .switch_db(dir.join("missing-directory").join("bad.db"))
            .is_err());
        assert_eq!(*db.path.lock().unwrap(), b);
        let value: i32 = db
            .conn
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .query_row("SELECT 1", [], |row| row.get(0))
            .unwrap();
        assert_eq!(value, 1);
        let renamed = b.with_extension("renamed.db");
        db.close().unwrap();
        std::fs::rename(&b, &renamed).unwrap();
        db.switch_db(&renamed).unwrap();
        assert_eq!(*db.path.lock().unwrap(), renamed);
        db.close().unwrap();
        std::fs::remove_file(a).unwrap();
        std::fs::remove_file(renamed).unwrap();
    }
}
