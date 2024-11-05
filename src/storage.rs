use async_trait::async_trait;
use tokio_stream::Stream;
use bytes::Bytes;
use std::pin::Pin;
use std::io;

#[async_trait]
pub trait Storage: Send + Sync {
    async fn write_file(
        &self,
        uuid: &str,
        data: Pin<Box<dyn Stream<Item = Result<Bytes, io::Error>> + Send>>,
    ) -> io::Result<()>;

    async fn read_file(
        &self,
        uuid: &str,
    ) -> io::Result<Pin<Box<dyn Stream<Item = Result<Bytes, io::Error>> + Send>>>;

    async fn delete_file(&self, uuid: &str) -> io::Result<()>;
}
