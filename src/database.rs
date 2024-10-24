use async_trait::async_trait;
use uuid::Uuid;
use std::error::Error;
use crate::entry::Entry;

/// Traits used to define platform-independent database operations.
/// 
/// Return types are set up to handle data-level and database-level errors separately.
/// For example, looking up a nonexistent entry by UUID returns Ok(None),
///     whereas if there were a database error, the error variant would be returned.
#[async_trait]
pub trait Database {
    async fn list_uuids(&self) -> Result<Vec<Uuid>, Box<dyn Error>>;
    async fn get_entry(&self, uuid: Uuid) -> Result<Option<Entry>, Box<dyn Error>>;
    async fn create_entry(&self, input: Entry) -> Result<Uuid, Box<dyn Error>>;
    async fn delete_entry(&self, uuid: Uuid) -> Result<bool, Box<dyn Error>>;
}
