use error_handler::Error;

#[derive(Debug)]
pub struct SmtpMessage {
    pub from: String,
    pub to: Vec<String>,
    pub subject: String,
    pub body: String,
}

impl SmtpMessage {

    pub fn builder() -> SmtpMessageBuilder {
        SmtpMessageBuilder::default()
    }

    pub fn to_imf(&self) -> String {
        let mut imf_message = String::new();
        
        imf_message.push_str(&format!("From: {}\r\n", self.from));
        
        imf_message.push_str(&format!("To: {}\r\n", self.to.join(", ") ));

        imf_message.push_str(&format!("Subject: {}\r\n", self.subject));
        
        imf_message.push_str("\r\n");
        
        imf_message.push_str(&self.body);
        
        imf_message
    }
}



#[derive(Default)]
pub struct SmtpMessageBuilder {
    from: Option<String>,
    to: Vec<String>,
    subject: Option<String>,
    body: Option<String>,
}

impl SmtpMessageBuilder {
    pub fn default() -> Self {
        Default::default()
    }

    pub fn from(mut self, from: &str) -> Self {
        self.from = Some(from.to_string());
        self
    }

    pub fn to(mut self, to: &str) -> Self {
        self.to.push(to.to_string());
        self
    }

    pub fn subject(mut self, subject: &str) -> Self {
        self.subject = Some(subject.to_string());
        self
    }

    pub fn body(mut self, body: &str) -> Self {
        self.body = Some(body.to_string());
        self
    }

    pub fn build(self) -> Result<SmtpMessage, Error> {
        if self.from.is_none() {
            return Err(Error::MessageBuild("Missing 'from' field".to_string()));
        }

        if self.to.is_empty() {
            return Err(Error::MessageBuild("Missing 'to' field".to_string()));
        }

        if self.subject.is_none() {
            return Err(Error::MessageBuild("Missing 'subject' field".to_string()));
        }

        if self.body.is_none() {
            return Err(Error::MessageBuild("Missing 'body' field".to_string()));
        }

        Ok(SmtpMessage {
            from: self.from.unwrap(),
            to: self.to,
            subject: self.subject.unwrap(),
            body: self.body.unwrap(),
        })
    }
}




// tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smtp_to_imf() {
        let message = SmtpMessage::builder()
            .from("johndoe@gmail.com")
            .to("emilydoe@gmail.com")
            .subject("Hello")
            .body("Hello, Emily!")
            .build().unwrap();

        assert_eq!(message.to_imf(),
                concat!("From: johndoe@gmail.com\r\n",
                        "To: emilydoe@gmail.com\r\n",
                        "Subject: Hello\r\n",
                        "\r\n",
                        "Hello, Emily!"));
    }

    #[test]
    fn test_smtp_to_imf_multiple_recipients() {
        let message = SmtpMessage::builder()
            .from("johndoe@gmail.com")
            .to("emilydoe@gmail.com")
            .to("alicedoe@gmail.com")
            .subject("Good evening")
            .body("Good evening, Emily and Alice!")
            .build().unwrap();

        assert_eq!(message.to_imf(),
                concat!("From: johndoe@gmail.com\r\n",
                        "To: emilydoe@gmail.com, alicedoe@gmail.com\r\n",
                        "Subject: Good evening\r\n",
                        "\r\n",
                        "Good evening, Emily and Alice!"));
    }

    #[test]
    fn test_smtp_to_imf_missing_from() {
        let message = SmtpMessage::builder()
            .to("emilydoe@gmail.com")
            .subject("Hello")
            .body("Hello, Emily!");

        match message.build() {
            Err(e) => assert_eq!(e, Error::MessageBuild("Missing 'from' field".to_string())),
            _ => panic!("Expected an error"),
        }
    }
}