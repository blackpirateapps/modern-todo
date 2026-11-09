use application::{ApplicationError, TagRepository};
use domain::{Tag, TagId, TagName};
use rusqlite::params;
use uuid::Uuid;
use crate::database::Database;

#[derive(Clone)]
pub struct SqliteTagRepository {
    db: Database,
}

impl SqliteTagRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

impl TagRepository for SqliteTagRepository {
    fn create(&self, tag: &Tag) -> Result<(), ApplicationError> {
        let conn = self.db.lock();
        conn.execute(
            r#"
            INSERT INTO tags (id, name, color)
            VALUES (?1, ?2, ?3)
            ON CONFLICT(id) DO UPDATE SET name = excluded.name, color = excluded.color
            "#,
            params![tag.id.0.to_string(), tag.name.as_str(), tag.color],
        )
        .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        Ok(())
    }

    fn get_by_id(&self, id: &TagId) -> Result<Option<Tag>, ApplicationError> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare("SELECT id, name, color FROM tags WHERE id = ?1")
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        let mut rows = stmt
            .query(params![id.0.to_string()])
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        if let Some(row) = rows.next().map_err(|e| ApplicationError::Repository(e.to_string()))? {
            let id_str: String = row.get(0).map_err(|e| ApplicationError::Repository(e.to_string()))?;
            let name_str: String = row.get(1).map_err(|e| ApplicationError::Repository(e.to_string()))?;
            let color: Option<String> = row.get(2).map_err(|e| ApplicationError::Repository(e.to_string()))?;

            let uuid = Uuid::parse_str(&id_str).map_err(|e| ApplicationError::Repository(e.to_string()))?;
            let name = TagName::new(name_str)?;
            Ok(Some(Tag {
                id: TagId::from_uuid(uuid),
                name,
                color,
            }))
        } else {
            Ok(None)
        }
    }

    fn get_by_name(&self, name: &str) -> Result<Option<Tag>, ApplicationError> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare("SELECT id, name, color FROM tags WHERE name = ?1")
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        let mut rows = stmt
            .query(params![name])
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        if let Some(row) = rows.next().map_err(|e| ApplicationError::Repository(e.to_string()))? {
            let id_str: String = row.get(0).map_err(|e| ApplicationError::Repository(e.to_string()))?;
            let name_str: String = row.get(1).map_err(|e| ApplicationError::Repository(e.to_string()))?;
            let color: Option<String> = row.get(2).map_err(|e| ApplicationError::Repository(e.to_string()))?;

            let uuid = Uuid::parse_str(&id_str).map_err(|e| ApplicationError::Repository(e.to_string()))?;
            let tag_name = TagName::new(name_str)?;
            Ok(Some(Tag {
                id: TagId::from_uuid(uuid),
                name: tag_name,
                color,
            }))
        } else {
            Ok(None)
        }
    }

    fn get_all(&self) -> Result<Vec<Tag>, ApplicationError> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare("SELECT id, name, color FROM tags ORDER BY name ASC")
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        let rows = stmt
            .query_map([], |row| {
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

    fn delete(&self, id: &TagId) -> Result<(), ApplicationError> {
        let conn = self.db.lock();
        conn.execute("DELETE FROM tags WHERE id = ?1", params![id.0.to_string()])
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        Ok(())
    }
}
