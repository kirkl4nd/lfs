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
