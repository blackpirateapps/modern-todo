use std::str::FromStr;
use application::{ApplicationError, TaskRepository};
use domain::{ListId, Priority, Tag, TagId, TagName, Task, TaskId, TaskTitle};
use rusqlite::{params, Row};
use time::format_description::well_known::Rfc3339;
use time::{Date, Month, OffsetDateTime};
use uuid::Uuid;
use crate::database::Database;

#[derive(Clone)]
pub struct SqliteTaskRepository {
    db: Database,
}

impl SqliteTaskRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    fn row_to_task(row: &Row<'_>) -> Result<Task, ApplicationError> {
        let id_str: String = row.get(0).map_err(|e| ApplicationError::Repository(e.to_string()))?;
        let title_str: String = row.get(1).map_err(|e| ApplicationError::Repository(e.to_string()))?;
        let description: Option<String> = row.get(2).map_err(|e| ApplicationError::Repository(e.to_string()))?;
        let priority_str: String = row.get(3).map_err(|e| ApplicationError::Repository(e.to_string()))?;
        let completed_int: i32 = row.get(4).map_err(|e| ApplicationError::Repository(e.to_string()))?;
        let list_id_str: Option<String> = row.get(5).map_err(|e| ApplicationError::Repository(e.to_string()))?;
        let due_date_str: Option<String> = row.get(6).map_err(|e| ApplicationError::Repository(e.to_string()))?;
        let reminder_at_str: Option<String> = row.get(7).map_err(|e| ApplicationError::Repository(e.to_string()))?;
        let parent_id_str: Option<String> = row.get(8).map_err(|e| ApplicationError::Repository(e.to_string()))?;
        let position: i32 = row.get(9).map_err(|e| ApplicationError::Repository(e.to_string()))?;
        let created_at_str: String = row.get(10).map_err(|e| ApplicationError::Repository(e.to_string()))?;
        let updated_at_str: String = row.get(11).map_err(|e| ApplicationError::Repository(e.to_string()))?;
        let completed_at_str: Option<String> = row.get(12).map_err(|e| ApplicationError::Repository(e.to_string()))?;
        let deleted_at_str: Option<String> = row.get(13).map_err(|e| ApplicationError::Repository(e.to_string()))?;

        let id = TaskId::from_uuid(
            Uuid::parse_str(&id_str).map_err(|e| ApplicationError::Repository(e.to_string()))?,
        );
        let title = TaskTitle::new(title_str)?;
        let priority = Priority::from_str(&priority_str)?;
        let completed = completed_int != 0;

        let list_id = if let Some(s) = list_id_str {
            Some(ListId::from_uuid(
                Uuid::parse_str(&s).map_err(|e| ApplicationError::Repository(e.to_string()))?,
            ))
        } else {
            None
        };

        let due_date = if let Some(s) = due_date_str {
            parse_date(&s)
        } else {
            None
        };

        let reminder_at = reminder_at_str.as_deref().and_then(parse_datetime);

        let parent_task_id = if let Some(s) = parent_id_str {
            Some(TaskId::from_uuid(
                Uuid::parse_str(&s).map_err(|e| ApplicationError::Repository(e.to_string()))?,
            ))
        } else {
            None
        };

        let created_at = parse_datetime(&created_at_str).unwrap_or_else(OffsetDateTime::now_utc);
        let updated_at = parse_datetime(&updated_at_str).unwrap_or_else(OffsetDateTime::now_utc);
        let completed_at = completed_at_str.as_deref().and_then(parse_datetime);
        let deleted_at = deleted_at_str.as_deref().and_then(parse_datetime);

        Ok(Task {
            id,
            title,
            description,
            priority,
            completed,
            list_id,
            due_date,
            reminder_at,
            parent_task_id,
            position,
            created_at,
            updated_at,
            completed_at,
            deleted_at,
        })
    }
}

fn format_datetime(dt: OffsetDateTime) -> String {
    dt.format(&Rfc3339).unwrap_or_else(|_| dt.to_string())
}

fn parse_datetime(s: &str) -> Option<OffsetDateTime> {
    OffsetDateTime::parse(s, &Rfc3339)
        .or_else(|_| OffsetDateTime::parse(s, &time::format_description::well_known::Iso8601::DEFAULT))
        .ok()
}

fn format_date(d: Date) -> String {
    format!("{:04}-{:02}-{:02}", d.year(), u8::from(d.month()), d.day())
}

fn parse_date(s: &str) -> Option<Date> {
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() != 3 {
        return None;
    }
    let year: i32 = parts[0].parse().ok()?;
    let month_u8: u8 = parts[1].parse().ok()?;
    let day: u8 = parts[2].parse().ok()?;
    let month = Month::try_from(month_u8).ok()?;
    Date::from_calendar_date(year, month, day).ok()
}

const SELECT_FIELDS: &str = r#"
    id, title, description, priority, completed, list_id, due_date,
    reminder_at, parent_task_id, position, created_at, updated_at,
    completed_at, deleted_at
"#;

impl TaskRepository for SqliteTaskRepository {
    fn create(&self, task: &Task) -> Result<(), ApplicationError> {
        let conn = self.db.lock();
        conn.execute(
            r#"
            INSERT INTO tasks (
                id, title, description, priority, completed, list_id, due_date,
                reminder_at, parent_task_id, position, created_at, updated_at,
                completed_at, deleted_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
            "#,
            params![
                task.id.0.to_string(),
                task.title.as_str(),
                task.description,
                task.priority.as_str(),
                if task.completed { 1 } else { 0 },
                task.list_id.map(|l| l.0.to_string()),
                task.due_date.map(format_date),
                task.reminder_at.map(format_datetime),
                task.parent_task_id.map(|p| p.0.to_string()),
                task.position,
                format_datetime(task.created_at),
                format_datetime(task.updated_at),
                task.completed_at.map(format_datetime),
                task.deleted_at.map(format_datetime),
            ],
        )
        .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        Ok(())
    }

    fn update(&self, task: &Task) -> Result<(), ApplicationError> {
        let conn = self.db.lock();
        let rows = conn
            .execute(
                r#"
                UPDATE tasks
                SET title = ?2, description = ?3, priority = ?4, completed = ?5,
                    list_id = ?6, due_date = ?7, reminder_at = ?8, parent_task_id = ?9,
                    position = ?10, updated_at = ?11, completed_at = ?12, deleted_at = ?13
                WHERE id = ?1
                "#,
                params![
                    task.id.0.to_string(),
                    task.title.as_str(),
                    task.description,
                    task.priority.as_str(),
                    if task.completed { 1 } else { 0 },
                    task.list_id.map(|l| l.0.to_string()),
                    task.due_date.map(format_date),
                    task.reminder_at.map(format_datetime),
                    task.parent_task_id.map(|p| p.0.to_string()),
                    task.position,
                    format_datetime(task.updated_at),
                    task.completed_at.map(format_datetime),
                    task.deleted_at.map(format_datetime),
                ],
            )
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        if rows == 0 {
            return Err(ApplicationError::NotFound {
                entity: "Task",
                id: task.id.to_string(),
            });
        }
        Ok(())
    }

    fn get_by_id(&self, id: &TaskId) -> Result<Option<Task>, ApplicationError> {
        let conn = self.db.lock();
        let query = format!("SELECT {} FROM tasks WHERE id = ?1", SELECT_FIELDS);
        let mut stmt = conn
            .prepare(&query)
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        let mut rows = stmt
            .query(params![id.0.to_string()])
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        if let Some(row) = rows.next().map_err(|e| ApplicationError::Repository(e.to_string()))? {
            Ok(Some(Self::row_to_task(row)?))
        } else {
            Ok(None)
        }
    }

    fn delete(&self, id: &TaskId) -> Result<(), ApplicationError> {
        let conn = self.db.lock();
        conn.execute("DELETE FROM tasks WHERE id = ?1", params![id.0.to_string()])
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        Ok(())
    }

    fn soft_delete(&self, id: &TaskId) -> Result<(), ApplicationError> {
        let conn = self.db.lock();
        let now = OffsetDateTime::now_utc().to_string();
        conn.execute(
            "UPDATE tasks SET deleted_at = ?2, updated_at = ?2 WHERE id = ?1",
            params![id.0.to_string(), now],
        )
        .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        Ok(())
    }

    fn get_all(&self) -> Result<Vec<Task>, ApplicationError> {
        let conn = self.db.lock();
        let query = format!(
            "SELECT {} FROM tasks WHERE deleted_at IS NULL ORDER BY position ASC, created_at DESC",
            SELECT_FIELDS
        );
        let mut stmt = conn
            .prepare(&query)
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        let mut rows = stmt
            .query([])
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        let mut tasks = Vec::new();
        while let Some(row) = rows.next().map_err(|e| ApplicationError::Repository(e.to_string()))? {
            tasks.push(Self::row_to_task(row)?);
        }
        Ok(tasks)
    }

    fn get_today(&self, date: Date) -> Result<Vec<Task>, ApplicationError> {
        let date_str = format_date(date);
        let conn = self.db.lock();
        let query = format!(
            r#"
            SELECT {} FROM tasks
            WHERE deleted_at IS NULL
              AND (due_date = ?1 OR (due_date < ?1 AND completed = 0))
            ORDER BY completed ASC, priority DESC, due_date ASC, position ASC
            "#,
            SELECT_FIELDS
        );
        let mut stmt = conn
            .prepare(&query)
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        let mut rows = stmt
            .query(params![date_str])
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        let mut tasks = Vec::new();
        while let Some(row) = rows.next().map_err(|e| ApplicationError::Repository(e.to_string()))? {
            tasks.push(Self::row_to_task(row)?);
        }
        Ok(tasks)
    }

    fn get_upcoming(&self, from_date: Date) -> Result<Vec<Task>, ApplicationError> {
        let date_str = format_date(from_date);
        let conn = self.db.lock();
        let query = format!(
            r#"
            SELECT {} FROM tasks
            WHERE deleted_at IS NULL
              AND due_date > ?1
            ORDER BY due_date ASC, priority DESC, position ASC
            "#,
            SELECT_FIELDS
        );
        let mut stmt = conn
            .prepare(&query)
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        let mut rows = stmt
            .query(params![date_str])
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        let mut tasks = Vec::new();
        while let Some(row) = rows.next().map_err(|e| ApplicationError::Repository(e.to_string()))? {
            tasks.push(Self::row_to_task(row)?);
        }
        Ok(tasks)
    }

    fn get_inbox(&self) -> Result<Vec<Task>, ApplicationError> {
        let conn = self.db.lock();
        let query = format!(
            r#"
            SELECT {} FROM tasks
            WHERE deleted_at IS NULL
              AND list_id IS NULL
              AND completed = 0
            ORDER BY priority DESC, position ASC, created_at DESC
            "#,
            SELECT_FIELDS
        );
        let mut stmt = conn
            .prepare(&query)
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        let mut rows = stmt
            .query([])
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        let mut tasks = Vec::new();
        while let Some(row) = rows.next().map_err(|e| ApplicationError::Repository(e.to_string()))? {
            tasks.push(Self::row_to_task(row)?);
        }
        Ok(tasks)
    }

    fn get_by_list(&self, list_id: &ListId) -> Result<Vec<Task>, ApplicationError> {
        let conn = self.db.lock();
        let query = format!(
            r#"
            SELECT {} FROM tasks
            WHERE deleted_at IS NULL
              AND list_id = ?1
            ORDER BY completed ASC, priority DESC, position ASC, created_at DESC
            "#,
            SELECT_FIELDS
        );
        let mut stmt = conn
            .prepare(&query)
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        let mut rows = stmt
            .query(params![list_id.0.to_string()])
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        let mut tasks = Vec::new();
        while let Some(row) = rows.next().map_err(|e| ApplicationError::Repository(e.to_string()))? {
            tasks.push(Self::row_to_task(row)?);
        }
        Ok(tasks)
    }

    fn get_subtasks(&self, parent_id: &TaskId) -> Result<Vec<Task>, ApplicationError> {
        let conn = self.db.lock();
        let query = format!(
            r#"
            SELECT {} FROM tasks
            WHERE deleted_at IS NULL
              AND parent_task_id = ?1
            ORDER BY completed ASC, position ASC, created_at ASC
            "#,
            SELECT_FIELDS
        );
        let mut stmt = conn
            .prepare(&query)
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        let mut rows = stmt
            .query(params![parent_id.0.to_string()])
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        let mut tasks = Vec::new();
        while let Some(row) = rows.next().map_err(|e| ApplicationError::Repository(e.to_string()))? {
            tasks.push(Self::row_to_task(row)?);
        }
        Ok(tasks)
    }

    fn get_completed(&self) -> Result<Vec<Task>, ApplicationError> {
        let conn = self.db.lock();
        let query = format!(
            r#"
            SELECT {} FROM tasks
            WHERE deleted_at IS NULL
              AND completed = 1
            ORDER BY completed_at DESC, updated_at DESC
            "#,
            SELECT_FIELDS
        );
        let mut stmt = conn
            .prepare(&query)
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        let mut rows = stmt
            .query([])
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        let mut tasks = Vec::new();
        while let Some(row) = rows.next().map_err(|e| ApplicationError::Repository(e.to_string()))? {
            tasks.push(Self::row_to_task(row)?);
        }
        Ok(tasks)
    }

    fn search(&self, query: &str) -> Result<Vec<Task>, ApplicationError> {
        let search_pattern = format!("%{}%", query);
        let conn = self.db.lock();
        let sql = format!(
            r#"
            SELECT {} FROM tasks
            WHERE deleted_at IS NULL
              AND (title LIKE ?1 OR (description IS NOT NULL AND description LIKE ?1))
            ORDER BY completed ASC, priority DESC, created_at DESC
            "#,
            SELECT_FIELDS
        );
        let mut stmt = conn
            .prepare(&sql)
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        let mut rows = stmt
            .query(params![search_pattern])
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        let mut tasks = Vec::new();
        while let Some(row) = rows.next().map_err(|e| ApplicationError::Repository(e.to_string()))? {
            tasks.push(Self::row_to_task(row)?);
        }
        Ok(tasks)
    }

    fn assign_tag(&self, task_id: &TaskId, tag_id: &TagId) -> Result<(), ApplicationError> {
        let conn = self.db.lock();
        conn.execute(
            r#"
            INSERT OR IGNORE INTO task_tags (task_id, tag_id)
            VALUES (?1, ?2)
            "#,
            params![task_id.0.to_string(), tag_id.0.to_string()],
        )
        .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        Ok(())
    }

    fn remove_tag(&self, task_id: &TaskId, tag_id: &TagId) -> Result<(), ApplicationError> {
        let conn = self.db.lock();
        conn.execute(
            "DELETE FROM task_tags WHERE task_id = ?1 AND tag_id = ?2",
            params![task_id.0.to_string(), tag_id.0.to_string()],
        )
        .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        Ok(())
    }

    fn get_tags_for_task(&self, task_id: &TaskId) -> Result<Vec<Tag>, ApplicationError> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare(
                r#"
                SELECT t.id, t.name, t.color
                FROM tags t
                INNER JOIN task_tags tt ON t.id = tt.tag_id
                WHERE tt.task_id = ?1
                ORDER BY t.name ASC
                "#,
            )
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        let rows = stmt
            .query_map(params![task_id.0.to_string()], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                ))
            })
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        let mut tags = Vec::new();
        for item in rows {
            let (id_str, name_str, color) = item.map_err(|e| ApplicationError::Repository(e.to_string()))?;
            let uuid = Uuid::parse_str(&id_str).map_err(|e| ApplicationError::Repository(e.to_string()))?;
            let name = TagName::new(name_str)?;
            tags.push(Tag {
                id: TagId::from_uuid(uuid),
                name,
                color,
            });
        }
        Ok(tags)
    }
}
