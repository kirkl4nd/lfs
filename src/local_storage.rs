use async_trait::async_trait;
use bytes::Bytes;
use tokio::fs::File;
use std::fs;
use std::path::PathBuf;
use std::pin::Pin;
use std::io;
use tokio::io::AsyncWriteExt;
use tokio_stream::{Stream, StreamExt};

use crate::storage::{DeleteFileResult, ReadFileResult, Storage, WriteFileResult};

pub struct LocalStorage {
    storage_path: PathBuf,
}

impl LocalStorage {
    pub fn new(storage_path: PathBuf) -> Self {
        fs::create_dir_all(&storage_path).expect("Failed to create storage directory");
        LocalStorage { storage_path }
    }

    fn file_path(&self, uuid: &str) -> PathBuf {
        self.storage_path.join(uuid)
    }
}

#[async_trait]
impl Storage for LocalStorage {
    async fn write_file(
        &self,
        uuid: &str,
        mut data: Pin<Box<dyn Stream<Item = Result<Bytes, io::Error>> + Send>>,
    ) -> WriteFileResult {
        let file_path = self.file_path(uuid);
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

    async fn read_file(
        &self,
        uuid: &str,
    ) -> ReadFileResult {
        let file_path = self.file_path(uuid);

        if !file_path.exists() {
            return ReadFileResult::NotFound;
        }

        match File::open(&file_path).await {
            Ok(file) => {
                let stream = tokio_util::io::ReaderStream::new(file);
                ReadFileResult::Success(Box::pin(stream))
            }
            Err(e) => ReadFileResult::Failure(e),
        }
    }

    async fn delete_file(&self, uuid: &str) -> DeleteFileResult {
        let file_path = self.file_path(uuid);
        
        if !file_path.exists() {
            return DeleteFileResult::NotFound;
        }

        match tokio::fs::remove_file(file_path).await {
            Ok(_) => DeleteFileResult::Success,
            Err(e) => DeleteFileResult::Failure(e),
        }
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
        let file_path = storage.file_path(TEST_UUID);

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

        // Test read_file
        let read_stream = match storage.read_file(TEST_UUID).await {
            ReadFileResult::Success(stream) => stream,
            _ => panic!("Failed to read file"),
        };
        let mut read_stream = read_stream;
        let mut read_data = Vec::new();
        while let Some(chunk) = read_stream.next().await {
            read_data.extend_from_slice(&chunk.unwrap());
        }
        assert_eq!(read_data, data, "Read data should match written data");

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



