use crate::entry::Entry;
use async_trait::async_trait;
use std::error::Error;
use uuid::Uuid;

/// Traits used to define platform-independent database operations.
///
/// Return types are set up to handle data-level and database-level errors separately.
/// For example, looking up a nonexistent entry by UUID returns Ok(None),
///     whereas if there were a database error, the error variant would be returned.
#[async_trait]
pub trait Database: Send + Sync {
    async fn list_uuids(&self) -> Result<Vec<Uuid>, Box<dyn Error>>;
    async fn get_entry(&self, uuid: Uuid) -> Result<Option<Entry>, Box<dyn Error>>;
    async fn insert_entry(&self, input: Entry) -> Result<Uuid, Box<dyn Error>>;
    async fn delete_entry(&self, uuid: Uuid) -> Result<bool, Box<dyn Error>>;
}
