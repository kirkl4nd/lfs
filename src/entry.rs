use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A single entry in the database
#[derive(Serialize)]
pub struct Entry {
    pub uuid: Uuid,
    pub file_name: String,
    pub file_size: u64,
    pub source_ip: String,
    pub timestamp: DateTime<Utc>,
}