use std::fmt;

use async_stream::AsyncStream;
use error_handler::Error;

mod base64;
mod message;
mod smtp_response;

pub use message::{SmtpMessage, SmtpMessageBuilder};

use smtp_response::{SmtpResponse, SmtpResponseBuilder, SmtpStatus};
use SmtpCommand::*;
pub enum SmtpCommand {
    Ehlo,
    StartTls,
    Register,
    AuthPlain,
    MailFrom,
    RcptTo,
    Data,
    Quit,
    Dot,
}

impl fmt::Display for SmtpCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ehlo => write!(f, "EHLO"),
            Self::StartTls => write!(f, "STARTTLS"),
            Self::Register => write!(f, "REGISTER"),
            Self::AuthPlain => write!(f, "AUTH PLAIN"),
            Self::MailFrom => write!(f, "MAIL FROM:"),
            Self::RcptTo => write!(f, "RCPT TO:"),
            Self::Data => write!(f, "DATA"),
            Self::Quit => write!(f, "QUIT"),
            Self::Dot => write!(f, "\r\n.\r\n"),
        }
    }
}

pub struct SmtpSession {
    m_stream: AsyncStream,
}

impl SmtpSession {
    pub async fn connect(server: &str) -> Result<Self, Error> {
        let stream = AsyncStream::new(server).await?;
        let mut smtp_session = Self { m_stream: stream };

        smtp_session.handle_response().await?.status_should_be(SmtpStatus::PositiveCompletion)?;
        smtp_session.send_ehlo_cmd().await?;

        Ok(smtp_session)
    }

    pub async fn encrypt_connection(&mut self) -> Result<bool, Error> {
        self.send_starttls_cmd().await?;
        self.m_stream.try_upgrade_to_tls().await?;
        Ok(true)
    }

    pub async fn register(&mut self, username: &str, password: &str) -> Result<usize, Error> {
        let encoded_register = base64::encode(format!("\0{}\0{}", username, password).as_str());
        self.send_register_cmd(encoded_register.as_str()).await
    }

    pub async fn authenticate(&mut self, username: &str, password: &str) -> Result<usize, Error> {
        let encoded_auth = base64::encode(format!("\0{}\0{}", username, password).as_str());
        self.send_auth_plain_cmd(encoded_auth.as_str()).await
    }

    pub async fn send_message(&mut self, message: SmtpMessage) -> Result<usize, Error> {

        self.send_mail_from_cmd(&message.from).await?;
        for to in message.to.iter() {
            self.send_rcpt_to_cmd(to).await?;
        }

        self.send_data_cmd().await?;
        self.send_message_imf(&message).await
    }

    
    async fn send_ehlo_cmd(&mut self) -> Result<usize, Error> {
        let request = self.send_cmd_with_arg(Ehlo, "localhost").await?;
        self.handle_response().await?.status_should_be(SmtpStatus::PositiveCompletion)?;

        Ok(request)
    }

    async fn send_starttls_cmd(&mut self) -> Result<usize, Error> {
        let request = self.send_cmd(StartTls).await?;
        self.handle_response().await?.status_should_be(SmtpStatus::PositiveCompletion)?;

        Ok(request)
    }

    async fn send_register_cmd(&mut self, encoded_auth: &str) -> Result<usize, Error> {
        let request = self.send_cmd_with_arg(Register, encoded_auth).await?;
        self.handle_response().await?.status_should_be(SmtpStatus::PositiveCompletion)?;

        Ok(request)
    }

    async fn send_auth_plain_cmd(&mut self, encoded_auth: &str) -> Result<usize, Error> {
        let request = self.send_cmd_with_arg(AuthPlain, encoded_auth).await?;
        self.handle_response().await?.status_should_be(SmtpStatus::PositiveCompletion)?;

        Ok(request)
    }

    async fn send_mail_from_cmd(&mut self, from: &str) -> Result<usize, Error> {
        let arg = format!("<{from}>");
        let request = self.send_cmd_with_arg(MailFrom, &arg).await?;
        self.handle_response().await?.status_should_be(SmtpStatus::PositiveCompletion)?;

        Ok(request)
    }

    async fn send_rcpt_to_cmd(&mut self, to: &str) -> Result<usize, Error> {
        let arg = format!("<{to}>");
        let request = self.send_cmd_with_arg(RcptTo, &arg).await?;
        self.handle_response().await?.status_should_be(SmtpStatus::PositiveCompletion)?;

        Ok(request)
    }

    async fn send_data_cmd(&mut self) -> Result<usize, Error> {
        let request = self.send_cmd(Data).await?;
        self.handle_response().await?.status_should_be(SmtpStatus::PositiveIntermediate)?;

        Ok(request)
    }

    pub async fn send_quit_cmd(&mut self) -> Result<usize, Error> {
        let request = self.send_cmd(Quit).await?;
        self.handle_response().await?.status_should_be(SmtpStatus::PositiveCompletion)?;

        Ok(request)
    }

    async fn send_message_imf(&mut self, message: &SmtpMessage) -> Result<usize, Error> {
        let message = format!("{}{}", message.to_imf(),  Dot);
        print!("{}", message);
        let request = self.m_stream.write(message.as_bytes()).await;
        request
    }


    async fn send_cmd(&mut self, cmd: SmtpCommand) -> Result<usize, Error> {
        let command = format!("{cmd}\r\n");
        print!("{}", command);
        self.m_stream.write(command.as_bytes()).await
    }

    async fn send_cmd_with_arg(&mut self, cmd: SmtpCommand, arg: &str) -> Result<usize, Error> {
        let command = format!("{cmd} {arg}\r\n");
        print!("{}", command);
        self.m_stream.write(command.as_bytes()).await
    }


    async fn handle_response(&mut self) -> Result<SmtpResponse, Error> {
        let smtp_response_builder = SmtpResponseBuilder::new();

        match smtp_response_builder.build(&self.m_stream.read().await?) {
            Ok(smtp_response) => {
                print!("{}", smtp_response.get_raw_response());
                Ok(smtp_response)
            },
            Err(e) => Err(e),
        }
    }
}
