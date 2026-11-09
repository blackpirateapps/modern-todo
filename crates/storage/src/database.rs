use std::path::Path;
use std::sync::{Arc, Mutex};
use rusqlite::Connection;
use crate::error::StorageError;
use crate::migrations::run_migrations;

#[derive(Clone)]
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, StorageError> {
        let mut conn = Connection::open(path)?;
        Self::configure(&mut conn)?;
        run_migrations(&mut conn)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn in_memory() -> Result<Self, StorageError> {
        let mut conn = Connection::open_in_memory()?;
        Self::configure_in_memory(&mut conn)?;
        run_migrations(&mut conn)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    fn configure(conn: &mut Connection) -> Result<(), StorageError> {
        conn.execute_batch(
            r#"
            PRAGMA foreign_keys = ON;
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            PRAGMA busy_timeout = 5000;
            "#,
        )?;
        Ok(())
    }

    fn configure_in_memory(conn: &mut Connection) -> Result<(), StorageError> {
        conn.execute_batch(
            r#"
            PRAGMA foreign_keys = ON;
            PRAGMA busy_timeout = 5000;
            "#,
        )?;
        Ok(())
    }

    pub fn lock(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.conn.lock().expect("Database lock poisoned")
    }
}
