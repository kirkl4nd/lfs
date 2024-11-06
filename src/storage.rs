use async_trait::async_trait;
use tokio_stream::Stream;
use bytes::Bytes;
use std::pin::Pin;
use std::io;
use std::path::PathBuf;

pub enum WriteFileResult {
    Success,
    Failure(io::Error),
}

pub enum DeleteFileResult {
    Success,
    NotFound,
    Failure(io::Error),
}

#[async_trait]
pub trait Storage: Send + Sync {
    async fn write_file(
        &self,
        uuid: &str,
        data: Pin<Box<dyn Stream<Item = Result<Bytes, io::Error>> + Send>>,
    ) -> WriteFileResult;

    fn get_file_path(&self, uuid: &str) -> PathBuf;
    
    async fn delete_file(&self, uuid: &str) -> DeleteFileResult;
}
