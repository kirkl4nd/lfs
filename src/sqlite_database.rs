use crate::{database::Database, entry::Entry};
use async_trait::async_trait;
use chrono::Utc;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;
use std::error::Error;
use uuid::Uuid;

pub struct SqliteDatabase {
    pool: Pool<SqliteConnectionManager>,
}

impl SqliteDatabase {
    /// Constructor to create a new SqliteDatabase with an initialized connection pool
    /// Also ensures that the `entries` table is created if it does not exist.
    pub async fn new(database_url: &str) -> Result<Self, Box<dyn Error>> {
        let manager = SqliteConnectionManager::file(database_url);
        let pool = Pool::new(manager)?;

        // Create the table if it doesn't exist
        let conn = pool.get()?;
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS entries (
                uuid TEXT PRIMARY KEY,
                file_name TEXT NOT NULL,
                file_size INTEGER NOT NULL,
                source_ip TEXT NOT NULL,
                timestamp TEXT NOT NULL
            );
            "#,
            [],
        )?;

        Ok(SqliteDatabase { pool })
    }
}

#[async_trait]
impl Database for SqliteDatabase {
    async fn list_uuids(&self) -> Result<Vec<Uuid>, Box<dyn Error>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("SELECT uuid FROM entries")?;
        let rows = stmt.query_map([], |row| {
            let uuid_str: String = row.get(0)?;
            Ok(Uuid::parse_str(&uuid_str).unwrap())
        })?;

        let mut uuids = Vec::new();
        for uuid in rows {
            uuids.push(uuid?);
        }
        Ok(uuids)
    }

    async fn get_entry(&self, uuid: Uuid) -> Result<Option<Entry>, Box<dyn Error>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT uuid, file_name, file_size, source_ip, timestamp FROM entries WHERE uuid = ?",
        )?;

        let entry = stmt.query_row(params![uuid.to_string()], |row| {
            Ok(Entry {
                uuid: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
                file_name: row.get(1)?,
                file_size: row.get(2)?,
                source_ip: row.get(3)?,
                timestamp: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                    .unwrap()
                    .with_timezone(&Utc),
            })
        });

        match entry {
            Ok(entry) => Ok(Some(entry)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(Box::new(e)),
        }
    }

    async fn create_entry(&self, input: Entry) -> Result<Uuid, Box<dyn Error>> {
        let conn = self.pool.get()?;
        let new_uuid = Uuid::new_v4();

        conn.execute(
            "INSERT INTO entries (uuid, file_name, file_size, source_ip, timestamp) VALUES (?, ?, ?, ?, ?)",
            params![
                new_uuid.to_string(),
                input.file_name,
                input.file_size,
                input.source_ip,
                input.timestamp.to_rfc3339()
            ],
        )?;

        Ok(new_uuid)
    }

    async fn delete_entry(&self, uuid: Uuid) -> Result<bool, Box<dyn Error>> {
        let conn = self.pool.get()?;
        let affected = conn.execute(
            "DELETE FROM entries WHERE uuid = ?",
            params![uuid.to_string()],
        )?;

        Ok(affected > 0)
    }
}
