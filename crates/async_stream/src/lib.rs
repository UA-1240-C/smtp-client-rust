use std::pin::Pin;
use std::task::{Context, Poll};
use error_handler::Error;

use tokio::io::{AsyncReadExt, AsyncWriteExt, AsyncWrite, AsyncRead, ReadBuf};
use tokio_native_tls::{native_tls::TlsConnector as NativeTlsConnector, TlsConnector, TlsStream};

use tokio::net::{TcpStream, lookup_host};
use std::net::{SocketAddr, Ipv4Addr};

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

use NodeType::*;

#[derive(Clone, Copy)]
pub enum NodeType {
    Host,
    Peer,
}

#[derive(Clone, Copy)]
pub struct NodeInfo {
    m_node_type: NodeType,
    m_ipv4: Ipv4Addr,
    m_port: u16,
}

impl NodeInfo {
    pub async fn new(node_type: NodeType, host: &str) -> Result<Self, Error> {
        let result_addrs = lookup_host(host).await?;
        for addr in result_addrs {
            if let SocketAddr::V4(v4) = addr {
                return Ok(
                    Self {
                        m_node_type: node_type,
                        m_ipv4: *v4.ip(),
                        m_port: v4.port(),
                    }
                )
            }
        }
        Err(Error::AsyncStream("Invalid address".to_string() + " " + host))
    }

    pub fn get_ipv4(&self) -> Ipv4Addr {
        self.m_ipv4
    }

    pub fn get_port(&self) -> u16 {
        self.m_port
    }

    pub fn get_node_type(&self) -> NodeType {
        self.m_node_type
    }

    pub fn get_connection_string(&self) -> String {
        format!("{}:{}", self.m_ipv4, self.m_port)
    }
}

#[derive(Clone, Copy)]
struct StreamInfo {
    m_is_encrypted: bool,
    m_host: NodeInfo,
    m_peer: NodeInfo,
}

impl StreamInfo {
    pub fn is_encrypted(&self) -> bool {
        self.m_is_encrypted
    }

    pub fn get_host(&mut self) -> &mut NodeInfo {
        &mut self.m_host
    }

    pub fn get_peer(&self) -> &NodeInfo {
        &self.m_peer
    }
}

pub struct AsyncStream {
    m_stream: Option<StreamIo<TcpStream>>,
    m_stream_info: Option<StreamInfo>,
    m_buffsize: u16,
}

impl AsyncStream {
    pub async fn new(server: &str) -> Result<Self, Error> {
        let host = NodeInfo::new(Host, server).await?;
        let peer = NodeInfo::new(Peer, "api.ipify.org:80").await?;

        let stream = TcpStream::connect(server).await?;
        Ok(
            Self {
                m_stream: Some(StreamIo::Plain(stream)),
                m_buffsize: 1024,
                m_stream_info: Some(
                    StreamInfo {
                        m_is_encrypted: false,
                        m_host: host,
                        m_peer: peer,
                    }
                ), 
            }
        )
    }

    pub async fn try_upgrade_to_tls(&mut self) -> Result<(), Error> {
        if !self.is_open() {
            return Err(Error::ClosedConnection("Encrypt connection".to_string()));
        }

        let native_tls_connector = NativeTlsConnector::builder()
            .danger_accept_invalid_certs(true)
            .build()?;

        let tls_connector = TlsConnector::from(native_tls_connector);

        if let Some(StreamIo::Plain(stream)) = self.m_stream.take() {
            if self.m_stream_info.is_some() {
                let host = *self.m_stream_info.unwrap().get_host();
                let tls_stream = tls_connector.connect(&host.get_connection_string(), stream).await?;
                self.m_stream = Some(StreamIo::Encrypted(tls_stream));
                self.m_stream_info.unwrap().m_is_encrypted = true;
                Ok(())
            } else {
                Err(Error::TlsUpgrade("Encrypt connection. Connection is already encrypted".to_string()))
            }
        } else {
            Err(Error::TlsUpgrade("Encrypt connection. Connection is already encrypted".to_string()))
        }
    }

    pub fn get_host_info(&self) -> Result<NodeInfo, Error> {
        if self.m_stream_info.is_some() {
            Ok(*self.m_stream_info.unwrap().get_host())
        } else {
            Err(Error::ClosedConnection("Get host info".to_string()))
        }
    }

    pub fn get_peer_info(&self) -> Result<NodeInfo, Error> {
        if self.m_stream_info.is_some() {
            Ok(*self.m_stream_info.unwrap().get_peer())
        } else {
            Err(Error::ClosedConnection("Get peer info".to_string()))
        }
    }

    pub fn is_open(&self) -> bool {
        self.m_stream.is_some()
    }

    pub fn close(&mut self) {
        self.m_stream.take();
        self.m_stream_info.take();
    }

    pub fn is_encrypted(&self) -> Result<bool, Error> {
        if self.m_stream_info.is_some() {
            Ok(self.m_stream_info.unwrap().is_encrypted())
        } else {
            Err(Error::ClosedConnection("Check encryption status".to_string()))
        }
    }

    pub async fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        if self.is_open() {
            self.m_stream.as_mut().unwrap().write(buf).await.map_err(Error::from)
        } else {
            Err(Error::ClosedConnection("Write".to_string()))
        }
    }

    pub async fn read(&mut self) -> Result<String, Error> {
        if self.is_open() {
            self.read_response_until_crlf().await
        } else {
            Err(Error::ClosedConnection("Read".to_string()))
        }
    }

    async fn read_response_until_crlf(&mut self) -> Result<String, Error> {
        let mut response = String::new();
        let mut buffer: Vec<u8> = vec![0; self.m_buffsize as usize];
        
        loop {
            if !self.is_open() {
                break;
            }

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
