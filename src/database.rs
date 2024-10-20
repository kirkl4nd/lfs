use std::error::Error;

use uuid::Uuid;

use crate::entry::{Entry, EntryInput};

pub trait Database {
    fn list_uuids(&self) -> Result<Vec<Uuid>, Box<dyn Error>>;
    fn get_entry(&self, uuid: Uuid) -> Result<Option<Option<Entry>>, Box<dyn Error>>;
    fn create_entry(&self, input: Entry) -> Result<Uuid, Box<dyn Error>>;
    fn delete_entry(&self, uuid: Uuid) -> Result<bool, Box<dyn Error>>;
}
