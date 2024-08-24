
use std::pin::Pin;
use std::task::{Context, Poll};
use error_handler::Error;

use tokio::io::{AsyncReadExt, AsyncWriteExt, AsyncWrite, AsyncRead, ReadBuf};
use tokio_native_tls::{native_tls::TlsConnector as NativeTlsConnector, TlsConnector, TlsStream};
use tokio::net::TcpStream;

pub enum StreamIo<T: AsyncRead + AsyncWrite + Unpin> {
    Plain(T),
    Encrypted(TlsStream<T>),
}

impl<T: AsyncRead + AsyncWrite + Unpin> AsyncRead for StreamIo<T> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        match *self {
            Self::Plain(ref mut stream) => Pin::new(stream).poll_read(cx, buf),
            Self::Encrypted(ref mut stream) => Pin::new(stream).poll_read(cx, buf),
        }
    }
}

impl <T: AsyncRead + AsyncWrite + Unpin> AsyncWrite for StreamIo<T> {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, std::io::Error>> {
        match *self {
            Self::Plain(ref mut stream) => Pin::new(stream).poll_write(cx, buf),
            Self::Encrypted(ref mut stream) => Pin::new(stream).poll_write(cx, buf),
        }
    }

    fn poll_flush(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>
    ) -> Poll<Result<(), std::io::Error>> {
        match *self {
            Self::Plain(ref mut stream) => Pin::new(stream).poll_flush(cx),
            Self::Encrypted(ref mut stream) => Pin::new(stream).poll_flush(cx),
        }
    }

    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>
    ) -> Poll<Result<(), std::io::Error>> {
        match *self {
            Self::Plain(ref mut stream) => Pin::new(stream).poll_shutdown(cx),
            Self::Encrypted(ref mut stream) => Pin::new(stream).poll_shutdown(cx),   
        }

    }
}

pub struct AsyncStream {
    m_stream: Option<StreamIo<TcpStream>>,
    m_is_encrypted: bool,
    m_server: String,
}


impl AsyncStream {
    pub async fn new(server: &str) -> Result<Self, Error> {
        let stream = TcpStream::connect(server).await?;
        Ok(
            Self {
                m_stream: Some(StreamIo::Plain(stream)),
                m_is_encrypted: false,
                m_server: server.to_string(),
            }
        )
    }

    pub async fn try_upgrade_to_tls(&mut self) -> Result<(), Error> {
        let native_tls_connector = NativeTlsConnector::builder()
            .danger_accept_invalid_certs(true)
            .build()?;

        let tls_connector = TlsConnector::from(native_tls_connector);

        if let Some(StreamIo::Plain(stream)) = self.m_stream.take() {
            let tls_stream = tls_connector.connect(&self.m_server, stream).await?;
            self.m_stream = Some(StreamIo::Encrypted(tls_stream));
            Ok(())
        } else {
            Err(Error::TlsUpgrade("Connection is already encrypted".to_string()))
        }
    }

    pub fn is_encrypted(&self) -> bool {
        self.m_is_encrypted
    }

    pub async fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        self.m_stream.as_mut().unwrap().write(buf).await.map_err(Error::from)
    }

    pub async fn read(&mut self) -> Result<String, Error> {
        self.read_response_until_crlf().await
    }

    async fn read_response_until_crlf(&mut self) -> Result<String, Error> {
        let mut response = String::new();
        let mut buffer: [u8; 1024] = [0u8; 1024];
        
        loop {
            let bytes_read = self.m_stream.as_mut().unwrap().read(&mut buffer).await?;
            if bytes_read == 0 {
                break;
            }
            
            let chunk = &buffer[..bytes_read];
            response.push_str(&String::from_utf8_lossy(chunk));

            if response.ends_with("\r\n") {
                break;
            }
        }
        Ok(response)
    }
}