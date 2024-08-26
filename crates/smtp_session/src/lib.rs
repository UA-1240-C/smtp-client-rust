use async_stream::AsyncStream;
use error_handler::Error;

use SmtpCommand::*;
pub enum SmtpCommand {
    Ehlo,
    StartTls,
    AuthPlain,
    MailFrom,
    RcptTo,
    Data,
    Quit,
}

impl SmtpCommand {
    fn to_str(cmd: Self) -> String {
        match cmd {
            Self::Ehlo => "EHLO".to_string(),
            Self::StartTls => "STARTTLS".to_string(),
            Self::AuthPlain => "AUTH PLAIN".to_string(),
            Self::MailFrom => "MAIL FROM".to_string(),
            Self::RcptTo => "RCPT TO".to_string(),
            Self::Data => "DATA".to_string(),
            Self::Quit => "QUIT".to_string(),
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

    pub async fn send_ehlo_cmd(&mut self) -> Result<usize, Error> {
        let command = SmtpCommand::to_str(Ehlo) + " host \r\n";
        self.m_stream.write(command.as_bytes()).await
    }

    pub async fn encrypt_connection(&mut self) -> Result<bool, Error> {
        self.m_stream.try_upgrade_to_tls().await?;
        Ok(true)
    }

    pub async fn send_starttls_cmd(&mut self) -> Result<usize, Error> {
        self.send_cmd(StartTls).await
    }

    pub async fn send_data_cmd(&mut self) -> Result<usize, Error> {
        self.send_cmd(Data).await
    }

    pub async fn send_quit_cmd(&mut self) -> Result<usize, Error> {
        self.send_cmd(Quit).await
    }

    pub async fn read_response(&mut self) -> Result<String, Error> {
        self.m_stream.read().await
    }

    async fn send_cmd(&mut self, cmd: SmtpCommand) -> Result<usize, Error> {
        let command = SmtpCommand::to_str(cmd) + "\r\n";
        self.m_stream.write(command.as_bytes()).await
    }
}