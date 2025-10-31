use once_cell::sync::Lazy;
use rusqlite::{Connection, Result};
use std::path::Path;
use std::sync::Mutex;

// Global default DB path
static DEFAULT_DB_PATH: Lazy<Mutex<String>> = Lazy::new(|| Mutex::new("devbox.db".to_string()));

pub struct DB {
    db_path: String,
}

impl DB {
    pub fn new(db_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let db = DB {
            db_path: db_path.to_string(),
        };
        if !Path::new(db_path).exists() {
            db.db_init()?;
        }
        Ok(db)
    }

    fn db_init(&self) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.get_connection()?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS containers (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                provider TEXT NOT NULL,
                status TEXT NOT NULL, 
                task_id TEXT NOT NULL,
                resource_id TEXT NOT NULL
            )",
            [],
        )?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS providers (
                name TEXT NOT NULL, 
                config TEXT NOT NULL
            )",
            [],
        )?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS tasks (
                id TEXT PRIMARY KEY,
                provider TEXT NOT NULL,
                status TEXT NOT NULL, 
                request TEXT NOT NULL
            )",
            [],
        )?;
        Ok(())
    }

    pub fn get_connection(&self) -> Result<Connection, Box<dyn std::error::Error>> {
        let conn = Connection::open(&self.db_path)?;
        Ok(conn)
    }
}

pub struct DefaultDB;

impl DefaultDB {
    pub fn get_db() -> Result<Connection, Box<dyn std::error::Error>> {
        let path = DEFAULT_DB_PATH.lock().unwrap().clone();
        let db = DB::new(&path)?;
        let conn = db.get_connection()?;
        Ok(conn)
    }
}
