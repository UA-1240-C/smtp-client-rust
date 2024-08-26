use tokio::io::Result;
use smtp_session::SmtpSession;

#[tokio::main]
async fn main() -> Result<()> {
    let mut client = SmtpSession::connect("smtp.gmail.com:587").await.unwrap();
    
    client.send_ehlo_cmd().await.unwrap();
    print!("{}", client.read_response().await.unwrap());

    client.send_starttls_cmd().await.unwrap();
    print!("{}", client.read_response().await.unwrap());

    client.encrypt_connection().await.unwrap();

    client.send_ehlo_cmd().await.unwrap();
    print!("{}", client.read_response().await.unwrap());
    
    Ok(())
}