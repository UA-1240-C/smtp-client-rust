use tokio::io::Result;
use smtp_session::{SmtpSession, SmtpMessageBuilder, SmtpMessage};

#[tokio::main]
async fn main() -> Result<()> {
    let mut client = SmtpSession::connect("smtp.gmail.com:587").await.unwrap();

    
    let message = SmtpMessageBuilder::new()
        .from("")
        .to("")
        .subject("")
        .body("");

    client.encrypt_connection().await.unwrap();
    client.authenticate("", "").await.unwrap();
    client.send_message(message.build().unwrap()).await.unwrap();

    Ok(())
}