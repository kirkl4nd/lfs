use async_trait::async_trait;
use bytes::Bytes;
use tokio::fs::File;
use std::fs;
use std::path::PathBuf;
use std::pin::Pin;
use std::io;
use tokio::io::AsyncWriteExt;
use tokio_stream::{Stream, StreamExt};

use crate::storage::Storage;

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
    ) -> io::Result<()> {
        let file_path = self.file_path(uuid);
        let mut file = File::create(&file_path).await?;
        
        while let Some(chunk) = data.next().await {
            let chunk = chunk?;
            file.write_all(&chunk).await?;
        }
        
        Ok(())
    }

    async fn read_file(
        &self,
        uuid: &str,
    ) -> io::Result<Pin<Box<dyn Stream<Item = Result<Bytes, io::Error>> + Send>>> {
        let file_path = self.file_path(uuid);

        if !file_path.exists() {
            return Err(io::Error::new(io::ErrorKind::NotFound, "File not found"));
        }

        let file = File::open(&file_path).await?;
        let stream = tokio_util::io::ReaderStream::new(file);
        Ok(Box::pin(stream))
    }

    async fn delete_file(&self, uuid: &str) -> io::Result<()> {
        let file_path = self.file_path(uuid);
        tokio::fs::remove_file(file_path).await
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
        // Create storage instance
        let storage = LocalStorage::new(PathBuf::from(TEST_DIR));
        let file_path = storage.file_path(TEST_UUID);

        // Create test data (1KB of zeros)
        let data = vec![0u8; 1024];
        let chunks = vec![Ok(Bytes::from(data.clone()))];
        let stream = tokio_stream::iter(chunks);
        let boxed_stream = Box::pin(stream);

        // Test write_file
        storage.write_file(TEST_UUID, boxed_stream).await.unwrap();
        assert!(file_path.exists(), "File should exist after writing");

        // Test read_file
        let mut read_stream = storage.read_file(TEST_UUID).await.unwrap();
        let mut read_data = Vec::new();
        while let Some(chunk) = read_stream.next().await {
            read_data.extend_from_slice(&chunk.unwrap());
        }
        assert_eq!(read_data, data, "Read data should match written data");

        // Test delete_file
        storage.delete_file(TEST_UUID).await.unwrap();
        assert!(!file_path.exists(), "File should not exist after deletion");

        // Clean up test directory
        fs::remove_dir_all(TEST_DIR).unwrap();
    }
}



