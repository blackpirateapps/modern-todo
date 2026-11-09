use application::{ApplicationError, ListRepository};
use domain::{List, ListId, ListName};
use rusqlite::params;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;
use uuid::Uuid;
use crate::database::Database;

fn format_datetime(dt: OffsetDateTime) -> String {
    dt.format(&Rfc3339).unwrap_or_else(|_| dt.to_string())
}

fn parse_datetime(s: &str) -> Option<OffsetDateTime> {
    OffsetDateTime::parse(s, &Rfc3339)
        .or_else(|_| OffsetDateTime::parse(s, &time::format_description::well_known::Iso8601::DEFAULT))
        .ok()
}

#[derive(Clone)]
pub struct SqliteListRepository {
    db: Database,
}

impl SqliteListRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

impl ListRepository for SqliteListRepository {
    fn create(&self, list: &List) -> Result<(), ApplicationError> {
        let conn = self.db.lock();
        conn.execute(
            r#"
            INSERT INTO lists (id, name, description, icon, color, position, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            "#,
            params![
                list.id.0.to_string(),
                list.name.as_str(),
                list.description,
                list.icon,
                list.color,
                list.position,
                format_datetime(list.created_at),
                format_datetime(list.updated_at),
            ],
        )
        .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        Ok(())
    }

    fn update(&self, list: &List) -> Result<(), ApplicationError> {
        let conn = self.db.lock();
        let rows_affected = conn
            .execute(
                r#"
                UPDATE lists
                SET name = ?2, description = ?3, icon = ?4, color = ?5, position = ?6, updated_at = ?7
                WHERE id = ?1
                "#,
                params![
                    list.id.0.to_string(),
                    list.name.as_str(),
                    list.description,
                    list.icon,
                    list.color,
                    list.position,
                    format_datetime(list.updated_at),
                ],
            )
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        if rows_affected == 0 {
            return Err(ApplicationError::NotFound {
                entity: "List",
                id: list.id.to_string(),
            });
        }
        Ok(())
    }

    fn get_by_id(&self, id: &ListId) -> Result<Option<List>, ApplicationError> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare(
                r#"
                SELECT id, name, description, icon, color, position, created_at, updated_at
                FROM lists
                WHERE id = ?1
                "#,
            )
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        let mut rows = stmt
            .query(params![id.0.to_string()])
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        if let Some(row) = rows.next().map_err(|e| ApplicationError::Repository(e.to_string()))? {
            let id_str: String = row.get(0).map_err(|e| ApplicationError::Repository(e.to_string()))?;
            let name_str: String = row.get(1).map_err(|e| ApplicationError::Repository(e.to_string()))?;
            let description: Option<String> = row.get(2).map_err(|e| ApplicationError::Repository(e.to_string()))?;
            let icon: Option<String> = row.get(3).map_err(|e| ApplicationError::Repository(e.to_string()))?;
            let color: Option<String> = row.get(4).map_err(|e| ApplicationError::Repository(e.to_string()))?;
            let position: i32 = row.get(5).map_err(|e| ApplicationError::Repository(e.to_string()))?;
            let created_at_str: String = row.get(6).map_err(|e| ApplicationError::Repository(e.to_string()))?;
            let updated_at_str: String = row.get(7).map_err(|e| ApplicationError::Repository(e.to_string()))?;

            let uuid = Uuid::parse_str(&id_str).map_err(|e| ApplicationError::Repository(e.to_string()))?;
            let name = ListName::new(name_str)?;
            let created_at = parse_datetime(&created_at_str).unwrap_or_else(OffsetDateTime::now_utc);
            let updated_at = parse_datetime(&updated_at_str).unwrap_or_else(OffsetDateTime::now_utc);

            Ok(Some(List {
                id: ListId::from_uuid(uuid),
                name,
                description,
                icon,
                color,
                position,
                created_at,
                updated_at,
            }))
        } else {
            Ok(None)
        }
    }

    fn delete(&self, id: &ListId) -> Result<(), ApplicationError> {
        let conn = self.db.lock();
        conn.execute("DELETE FROM lists WHERE id = ?1", params![id.0.to_string()])
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        Ok(())
    }

    fn get_all(&self) -> Result<Vec<List>, ApplicationError> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare(
                r#"
                SELECT id, name, description, icon, color, position, created_at, updated_at
                FROM lists
                ORDER BY position ASC, created_at ASC
                "#,
            )
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, i32>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, String>(7)?,
                ))
            })
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        let mut lists = Vec::new();
        for item in rows {
            let (id_str, name_str, description, icon, color, position, created_str, updated_str) =
                item.map_err(|e| ApplicationError::Repository(e.to_string()))?;

            let uuid = Uuid::parse_str(&id_str).map_err(|e| ApplicationError::Repository(e.to_string()))?;
            let name = ListName::new(name_str)?;
            let created_at = parse_datetime(&created_str).unwrap_or_else(OffsetDateTime::now_utc);
            let updated_at = parse_datetime(&updated_str).unwrap_or_else(OffsetDateTime::now_utc);

            lists.push(List {
                id: ListId::from_uuid(uuid),
                name,
                description,
                icon,
                color,
                position,
                created_at,
                updated_at,
            });
        }

        Ok(lists)
    }
}
