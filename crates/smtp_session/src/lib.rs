use async_stream::AsyncStream;
use error_handler::Error;

use SmtpCommand::*;
pub enum SmtpCommand {
    Ehlo,
    StartTls,
}

impl SmtpCommand {
    fn get(cmd: Self) -> String {
        match cmd {
            Self::Ehlo => "EHLO".to_string(),
            Self::StartTls => "STARTTLS".to_string(),
        }
    }
}

pub struct SmtpSession {
    m_stream: AsyncStream,
}

impl SmtpSession {
    pub async fn connect(server: &str) -> Result<Self, Error> {
        let stream = AsyncStream::new(server).await?;
        Ok(
            Self {
                m_stream: stream,
            }
        )
    }

    pub async fn initialize_secure_conncetion(&mut self) -> Result<(), Error> {
        print!("{}", self.read_response().await?);

        self.send_ehlo().await?;
        print!("{}", self.read_response().await?);

        self.send_starttls().await?;
        print!("{}", self.read_response().await?);

        self.m_stream.try_upgrade_to_tls().await?;

        self.send_ehlo().await?;
        print!("{}", self.read_response().await?);

        Ok(())
    }

    async fn send_ehlo(&mut self) -> Result<usize, Error> {
        let command = SmtpCommand::get(Ehlo) + " host \r\n";
        self.m_stream.write(command.as_bytes()).await
    }

    async fn send_starttls(&mut self) -> Result<usize, Error> {
        let command = SmtpCommand::get(StartTls) + "\r\n";
        self.m_stream.write(command.as_bytes()).await
    }

    pub async fn read_response(&mut self) -> Result<String, Error> {
        self.m_stream.read().await
    }
}