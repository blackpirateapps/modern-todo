use rusqlite::Connection;
use time::OffsetDateTime;
use crate::error::StorageError;

pub struct Migration {
    pub version: i32,
    pub description: &'static str,
    pub sql: &'static str,
}

pub const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        description: "Initial schema: tasks, lists, tags, task_tags, settings",
        sql: r#"
            CREATE TABLE IF NOT EXISTS lists (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                icon TEXT,
                color TEXT,
                position INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS tasks (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                description TEXT,
                priority TEXT NOT NULL,
                completed INTEGER NOT NULL DEFAULT 0,
                list_id TEXT REFERENCES lists(id) ON DELETE SET NULL,
                due_date TEXT,
                reminder_at TEXT,
                parent_task_id TEXT REFERENCES tasks(id) ON DELETE CASCADE,
                position INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                completed_at TEXT,
                deleted_at TEXT
            );

            CREATE TABLE IF NOT EXISTS tags (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                color TEXT
            );

            CREATE TABLE IF NOT EXISTS task_tags (
                task_id TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
                tag_id TEXT NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
                PRIMARY KEY (task_id, tag_id)
            );

            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_tasks_due_date ON tasks(due_date);
            CREATE INDEX IF NOT EXISTS idx_tasks_list_completed ON tasks(list_id, completed);
            CREATE INDEX IF NOT EXISTS idx_tasks_completed_deleted ON tasks(completed, deleted_at);
            CREATE INDEX IF NOT EXISTS idx_tasks_updated_at ON tasks(updated_at);
            CREATE INDEX IF NOT EXISTS idx_task_tags_tag_task ON task_tags(tag_id, task_id);
        "#,
    },
];

pub fn run_migrations(conn: &mut Connection) -> Result<(), StorageError> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            applied_at TEXT NOT NULL
        );
        "#,
    )?;

    for migration in MIGRATIONS {
        let already_applied: bool = conn
            .query_row(
                "SELECT COUNT(1) FROM schema_migrations WHERE version = ?1",
                [migration.version],
                |row| row.get::<_, i64>(0),
            )? > 0;

        if !already_applied {
            tracing::info!(
                version = migration.version,
                description = migration.description,
                "Applying database migration"
            );

            let tx = conn.transaction()?;
            tx.execute_batch(migration.sql)?;
            let now = OffsetDateTime::now_utc().to_string();
            tx.execute(
                "INSERT INTO schema_migrations (version, applied_at) VALUES (?1, ?2)",
                rusqlite::params![migration.version, now],
            )?;
            tx.commit()?;
        }
    }

    Ok(())
}
