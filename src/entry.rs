use chrono::{DateTime, Utc};
use uuid::Uuid;

/// A single entry in the database
pub struct Entry {
    pub uuid: Uuid,
    pub file_name: String,
    pub file_size: u64,
    pub source_ip: String,
    pub timestamp: DateTime<Utc>,
}


/// Input for a new entry that will be sent to the server
/// Other fields will be filled in by the server
pub struct EntryInput {
    pub file_name: String,
    pub file_size: u64,
}
