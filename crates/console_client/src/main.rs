use std::error::Error;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_native_tls::{native_tls::TlsConnector as NativeTlsConnector, TlsConnector};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Connect to Google's SMTP
    let address = "smtp.gmail.com:587";
    let mut stream = TcpStream::connect(address).await?;

    // Read the first response
    let mut response = read_response_until_crlf(&mut stream).await?;
    println!("Initial Response: {}", response);

    // Send EHLO
    let ehlo_cmd = "EHLO 127\r\n";
    stream.write(ehlo_cmd.as_bytes()).await?;

    // Read response after EHLO
    response = read_response_until_crlf(&mut stream).await?;
    println!("{}", response);

    // Send STARTTLS
    let starttls_cmd = "STARTTLS\r\n";
    stream.write(starttls_cmd.as_bytes()).await?;

    // Read response after STARTTLS
    response = read_response_until_crlf(&mut stream).await?;
    println!("{}", response);


    // Upgrade to TLS
    /* Disable peer verification or add a root certificate */
    let mut tls_builder = NativeTlsConnector::builder();
    let native_tls_connector = tls_builder
        .danger_accept_invalid_certs(true)
        .build()?;

    // Connect to the server using TLS
    let tls_connector = TlsConnector::from(native_tls_connector);
    let mut tls_stream = tls_connector.connect("smtp.gmail.com", stream).await?;
    
    // Send EHLO again
    tls_stream.write(ehlo_cmd.as_bytes()).await?;

    // Read response after tls EHLO
    response = read_response_until_crlf(&mut tls_stream).await?;
    print!("{}", response);

    Ok(())
}

// A function to read the response from the server until a CRLF (endl) is found
async fn read_response_until_crlf<S: AsyncReadExt + Unpin>(stream: &mut S) -> Result<String, Box<dyn Error>> {
    let mut response = String::new();
    let mut buffer: [u8; 1024] = [0u8; 1024];
    
    loop {
        let bytes_read = stream.read(&mut buffer).await?;
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