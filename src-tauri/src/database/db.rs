use rusqlite::{Connection, Result};
use std::path::PathBuf;
use std::sync::Mutex;

pub struct Database {
    pub conn: Mutex<Connection>,
}

impl Database {
    pub fn new(app_data_dir: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&app_data_dir).ok();
        let db_path = app_data_dir.join("bongocat.db");
        let conn = Connection::open(&db_path)?;
        let db = Database {
            conn: Mutex::new(conn),
        };
        db.init_tables()?;
        Ok(db)
    }

    fn init_tables(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        // Add task_date column if it doesn't exist (migration for existing databases)
        conn.execute_batch("ALTER TABLE todos ADD COLUMN task_date TEXT").ok();

        // Create analytics tables if they don't exist (migration for existing databases)
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS analytics_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                event_name TEXT NOT NULL,
                event_value TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS app_settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_analytics_events_name ON analytics_events(event_name);
            CREATE INDEX IF NOT EXISTS idx_analytics_events_created ON analytics_events(created_at);
        ").ok();

        // Set task_date = date(created_at) for existing rows that have NULL task_date
        conn.execute("UPDATE todos SET task_date = date(created_at) WHERE task_date IS NULL", []).ok();

        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS todos (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                description TEXT,
                status TEXT,
                priority INTEGER,
                deadline DATETIME,
                created_at DATETIME,
                completed_at DATETIME,
                task_date TEXT
            );

            CREATE TABLE IF NOT EXISTS activity_logs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                action TEXT NOT NULL,
                task_id INTEGER,
                timestamp DATETIME
            );

            CREATE TABLE IF NOT EXISTS ai_settings (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                provider TEXT NOT NULL DEFAULT 'ollama',
                model_name TEXT NOT NULL DEFAULT 'qwen2.5:7b',
                api_key TEXT NOT NULL DEFAULT '',
                base_url TEXT NOT NULL DEFAULT '',
                is_active INTEGER NOT NULL DEFAULT 1
            );

            CREATE TABLE IF NOT EXISTS daily_reports (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                report_date TEXT NOT NULL UNIQUE,
                content TEXT NOT NULL,
                created_at DATETIME
            );
            ",
        )?;
        Ok(())
    }
}
