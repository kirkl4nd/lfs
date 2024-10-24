use sqlx::{Executor, SqlitePool};
use uuid::Uuid;
use std::error::Error;
use crate::{database::Database, entry::Entry};
use async_trait::async_trait;
use chrono::Utc;

pub struct SqliteDatabase {
    pool: SqlitePool,
}

impl SqliteDatabase {
    /// Constructor to create a new SqliteDatabase with an initialized connection pool
    /// Also ensures that the `entries` table is created if it does not exist.
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        // Initialize the connection pool
        let pool = SqlitePool::connect(database_url).await?;

        // Create the `entries` table if it does not exist
        pool.execute(
            r#"
            CREATE TABLE IF NOT EXISTS entries (
                uuid TEXT PRIMARY KEY,
                file_name TEXT NOT NULL,
                file_size INTEGER NOT NULL,
                source_ip TEXT NOT NULL,
                timestamp TEXT NOT NULL
            );
            "#
        )
        .await?;

        Ok(SqliteDatabase { pool })
    }
}

#[async_trait]
impl Database for SqliteDatabase {
    async fn list_uuids(&self) -> Result<Vec<Uuid>, Box<dyn Error>> {
        let rows = sqlx::query_unchecked!("SELECT uuid FROM entries")
            .fetch_all(&self.pool)
            .await?;

        let uuids = rows.into_iter()
            .map(|row| Uuid::parse_str(&row.uuid))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(uuids)
    }

    async fn get_entry(&self, uuid: Uuid) -> Result<Option<Entry>, Box<dyn Error>> {
        let row = sqlx::query_as_unchecked!(Entry, "SELECT * FROM entries WHERE uuid = ?", uuid.to_string())
            .fetch_optional(&self.pool)
            .await?;

        Ok(row)
    }

    async fn create_entry(&self, input: Entry) -> Result<Uuid, Box<dyn Error>> {
        let new_uuid = Uuid::new_v4();

        sqlx::query_unchecked!(
            "INSERT INTO entries (uuid, field1, field2) VALUES (?, ?, ?)",
            new_uuid.to_string(),
            input.field1,
            input.field2,
        )
        .execute(&self.pool)
        .await?;

        Ok(new_uuid)
    }

    async fn delete_entry(&self, uuid: Uuid) -> Result<bool, Box<dyn Error>> {
        let result = sqlx::query_unchecked!("DELETE FROM entries WHERE uuid = ?", uuid.to_string())
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }
}