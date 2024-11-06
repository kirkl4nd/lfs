use async_trait::async_trait;
use tokio_stream::Stream;
use bytes::Bytes;
use std::pin::Pin;
use std::io;

pub enum WriteFileResult {
    Success,
    Failure(io::Error),
}

pub enum ReadFileResult {
    Success(Pin<Box<dyn Stream<Item = Result<Bytes, io::Error>> + Send>>),
    NotFound,
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

    async fn read_file(
        &self,
        uuid: &str,
    ) -> ReadFileResult;

    async fn delete_file(&self, uuid: &str) -> DeleteFileResult;
}
