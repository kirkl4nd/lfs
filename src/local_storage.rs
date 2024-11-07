use async_trait::async_trait;
use bytes::Bytes;
use tokio::fs::File;
use std::{error::Error, fs};
use std::path::PathBuf;
use std::pin::Pin;
use std::io;
use tokio::io::AsyncWriteExt;
use tokio_stream::{Stream, StreamExt};

use crate::storage::{DeleteFileResult, Storage, WriteFileResult};

pub struct LocalStorage {
    storage_path: PathBuf,
}

impl LocalStorage {
    pub fn new(storage_path: PathBuf) -> Self {
        fs::create_dir_all(&storage_path).expect("Failed to create storage directory");
        LocalStorage { storage_path }
    }
}

#[async_trait]
impl Storage for LocalStorage {
    async fn write_file(
        &self,
        uuid: &str,
        mut data: Pin<Box<dyn Stream<Item = Result<Bytes, io::Error>> + Send>>,
    ) -> WriteFileResult {
        let file_path = self.get_file_path(uuid);
        let file = match File::create(&file_path).await {
            Ok(f) => f,
            Err(e) => return WriteFileResult::Failure(e),
        };
        
        let mut file = file;
        while let Some(chunk) = data.next().await {
            match chunk {
                Ok(chunk) => {
                    if let Err(e) = file.write_all(&chunk).await {
                        return WriteFileResult::Failure(e);
                    }
                }
                Err(e) => return WriteFileResult::Failure(e),
            }
        }
        
        WriteFileResult::Success
    }

    fn get_file_path(&self, uuid: &str) -> PathBuf {
        self.storage_path.join(uuid)
    }

    async fn delete_file(&self, uuid: &str) -> DeleteFileResult {
        let file_path = self.get_file_path(uuid);
        
        if !file_path.exists() {
            return DeleteFileResult::NotFound;
        }

        match tokio::fs::remove_file(file_path).await {
            Ok(_) => DeleteFileResult::Success,
            Err(e) => DeleteFileResult::Failure(e),
        }
    }

    async fn list_files(&self) -> Result<Vec<String>, Box<dyn Error>> {
        let mut files = Vec::new();
        let entries = fs::read_dir(&self.storage_path)?;

        for entry in entries {
            if let Ok(entry) = entry {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_file() {
                        if let Some(file_name) = entry.file_name().to_str() {
                            files.push(file_name.to_string());
                        }
                    }
                }
            }
        }

        Ok(files)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use futures_util::StreamExt;

    const TEST_UUID: &str = "550e8400-e29b-41d4-a716-446655440000";
    const TEST_DIR: &str = "./test-storage";

    #[tokio::test]
    async fn test_local_storage_operations() {
        let storage = LocalStorage::new(PathBuf::from(TEST_DIR));
        let file_path = storage.get_file_path(TEST_UUID);

        let data = vec![0u8; 1024];
        let chunks = vec![Ok(Bytes::from(data.clone()))];
        let stream = tokio_stream::iter(chunks);
        let boxed_stream = Box::pin(stream);

        // Test write_file
        match storage.write_file(TEST_UUID, boxed_stream).await {
            WriteFileResult::Success => (),
            _ => panic!("Failed to write file"),
        }
        assert!(file_path.exists(), "File should exist after writing");

        // Test delete_file
        match storage.delete_file(TEST_UUID).await {
            DeleteFileResult::Success => (),
            _ => panic!("Failed to delete file"),
        }
        assert!(!file_path.exists(), "File should not exist after deletion");

        // Clean up test directory
        fs::remove_dir_all(TEST_DIR).unwrap();
    }
}



