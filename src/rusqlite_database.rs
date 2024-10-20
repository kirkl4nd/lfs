use rusqlite::{Connection, Result as SqliteResult};
use uuid::Uuid;
use chrono::Utc;
use crate::entry::{Entry, EntryInput};
use crate::database::Database;
use std::error::Error;

pub struct RusqliteDatabase {
    conn: Connection,
}

impl RusqliteDatabase {
    pub fn new(path: &str) -> SqliteResult<Self> {
        let conn = Connection::open(path)?;
        let db = RusqliteDatabase { conn };
        db.init()?;
        Ok(db)
    }

    fn init(&self) -> SqliteResult<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS entries (
                uuid TEXT PRIMARY KEY,
                filename TEXT NOT NULL,
                size INTEGER NOT NULL,
                srcip TEXT NOT NULL,
                timestamp TEXT NOT NULL
            )",
            [],
        )?;
        Ok(())
    }
}

impl Database for RusqliteDatabase {

    fn list_uuids(&self) -> Result<Vec<Uuid>, Box<dyn Error>> {
        let mut stmt = self.conn.prepare("SELECT uuid FROM entries")?;
        let uuid_iter = stmt.query_map([], |row| {
            let uuid_str: String = row.get(0)?;
            Ok(Uuid::parse_str(&uuid_str).unwrap())
        })?;

        let uuids: Result<Vec<_>, _> = uuid_iter.collect();
        Ok(uuids?)
    }

    fn get_entry(&self, uuid: Uuid) -> Result<Option<Option<Entry>>, Box<dyn Error>> {
        let mut stmt = self.conn.prepare("SELECT uuid, filename, size, srcip, timestamp FROM entries WHERE uuid = ?")?;
        let mut rows = stmt.query([uuid.to_string()])?;

        match rows.next()? {
            Some(row) => {
                let entry = Entry {
                    uuid: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
                    file_name: row.get(1)?,
                    file_size: row.get(2)?,
                    source_ip: row.get(3)?,
                    timestamp: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?).unwrap().with_timezone(&Utc),
                };
                Ok(Some(Some(entry)))
            },
            None => Ok(Some(None)),
        }
    }

    fn create_entry(&self, input: Entry) -> Result<Uuid, Box<dyn Error>> {
        let uuid = Uuid::new_v4();
        let timestamp = Utc::now();
        
        self.conn.execute(
            "INSERT INTO entries (uuid, filename, size, srcip, timestamp) VALUES (?, ?, ?, ?, ?)",
            rusqlite::params![
                uuid.to_string(),
                input.file_name,
                input.file_size,
                input.source_ip,
                timestamp.to_rfc3339()
            ],
        )?;

        Ok(uuid)
    }

    fn delete_entry(&self, uuid: Uuid) -> Result<bool, Box<dyn Error>> {
        let result = self.conn.execute(
            "DELETE FROM entries WHERE uuid = ?",
            rusqlite::params![uuid.to_string()],
        )?;

        Ok(result > 0)
    }
}
