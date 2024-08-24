use tokio::io::Result;
use smtp_session::SmtpSession;

#[tokio::main]
async fn main() -> Result<()> {
    let mut client = SmtpSession::connect("smtp.gmail.com:587").await.unwrap();
    client.initialize_secure_conncetion().await.unwrap();
    
    Ok(())
}