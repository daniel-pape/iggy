use crate::binary::sender::Sender;
use crate::tcp::sender;
use async_trait::async_trait;
use iggy::error::Error;
use tokio::net::TcpStream;
use tokio_native_tls::TlsStream;

#[derive(Debug)]
pub struct TcpTlsSender {
    pub(crate) stream: TlsStream<TcpStream>,
}

unsafe impl Send for TcpTlsSender {}
unsafe impl Sync for TcpTlsSender {}

#[async_trait]
impl Sender for TcpTlsSender {
    async fn read(&mut self, buffer: &mut [u8]) -> Result<usize, Error> {
        sender::read(&mut self.stream, buffer).await
    }

    async fn send_empty_ok_response(&mut self) -> Result<(), Error> {
        sender::send_empty_ok_response(&mut self.stream).await
    }

    async fn send_ok_response(&mut self, payload: &[u8]) -> Result<(), Error> {
        sender::send_ok_response(&mut self.stream, payload).await
    }

    async fn send_error_response(&mut self, error: Error) -> Result<(), Error> {
        sender::send_error_response(&mut self.stream, error).await
    }
}
