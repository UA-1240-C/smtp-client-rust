use error_handler::Error;
use regex::Regex;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SmtpStatus {
    PositiveCompletion,
    PositiveIntermediate,
    TransientNegativeCompletion,
    PermanentNegativeCompletion,
    Unknown,
}

impl From<u16> for SmtpStatus {
    fn from(status: u16) -> Self {
        match status - (status % 100) {
            200 => SmtpStatus::PositiveCompletion,
            300 => SmtpStatus::PositiveIntermediate,
            400 => SmtpStatus::TransientNegativeCompletion,
            500 => SmtpStatus::PermanentNegativeCompletion,
            _ => SmtpStatus::Unknown,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct SmtpResponse {
    m_raw_response: String,
    m_status: SmtpStatus,
    m_text: String,
}

#[allow(dead_code)]
impl SmtpResponse {
    pub fn get_raw_response(&self) -> String {
        self.m_raw_response.clone()
    }

    pub fn get_status(&self) -> SmtpStatus {
        self.m_status.clone()
    }

    pub fn get_text(&self) -> String {
        self.m_text.clone()
    }

    pub fn status_should_be(&self, status: SmtpStatus) -> Result<(), Error> {
        if self.m_status == status {
            Ok(())
        } else {
            Err(Error::SmtpResponse("Unexpected status".to_string()))
        }
    }
}

pub struct SmtpResponseBuilder {
    m_regex: String,
}

impl SmtpResponseBuilder {
    pub fn new() -> Self {
        Self {
            m_regex: r"(\d{3})(?:[ -](\d\.\d\.\d))?[ -](.*)".to_string(),
        }
    }

    pub fn build(&self, raw_response: &str) -> Result<SmtpResponse, Error> {
        let status_code = self.parse_status_code(raw_response)?;
        let status = SmtpStatus::from(status_code);
        let text = self.parse_text(raw_response)?;

        Ok(SmtpResponse {
            m_raw_response: raw_response.to_string(),
            m_status: status,
            m_text: text,
        })
    }

    fn parse_status_code(&self, raw_response: &str) -> Result<u16, Error> {
        if self.is_valid_response(raw_response) {
            let re = Regex::new(r"(\d{3})").unwrap();
            let caps = re.captures(raw_response).unwrap();
            
            Ok(caps.get(1).unwrap().as_str().parse::<u16>().unwrap())
        } else {
            Err(Error::SmtpResponse("Invalid response".to_string()))
        }

    }

    fn parse_text(&self, raw_response: &str) -> Result<String, Error> {
        if self.is_valid_response(raw_response) {
            let re = Regex::new(&self.m_regex).unwrap();
            let caps = re.captures(raw_response).unwrap();
            Ok(caps.get(3).unwrap().as_str().to_string())
        } else {
            Err(Error::SmtpResponse("Invalid response".to_string()))
        }
    }

    fn is_valid_response(&self, raw_response: &str) -> bool {
        let re = Regex::new(&self.m_regex).unwrap();
        re.is_match(raw_response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_build() {
        let response = "250 OK";
        let builder = SmtpResponseBuilder::new();
        let smtp_response = builder.build(response).unwrap();
        assert_eq!(smtp_response.m_status, SmtpStatus::PositiveCompletion);
        assert_eq!(smtp_response.m_text, "OK");
    }

    #[test]
    fn test_parse_status_code() {
        let response = "250 OK";
        let builder = SmtpResponseBuilder::new();
        let status_code = builder.parse_status_code(response).unwrap();
        assert_eq!(status_code, 250);
    }

    #[test]
    fn test_parse_text() {
        let response = "250 OK";
        let builder = SmtpResponseBuilder::new();
        let text = builder.parse_text(response).unwrap();
        assert_eq!(text, "OK");
    }
    
    #[test]
    fn test_is_valid_response() {
        let response = "250 OK";
        let builder = SmtpResponseBuilder::new();
        assert_eq!(builder.is_valid_response(response), true);
    }

    #[test]
    fn test_new() {
        let response = "250 OK";
        let builder = SmtpResponseBuilder::new();
        let smtp_response = builder.build(response).unwrap();
        assert_eq!(smtp_response.m_status, SmtpStatus::PositiveCompletion);
    }

    #[test]
    fn test_from() {
        let status_code = 250;
        let smtp_status = SmtpStatus::from(status_code);
        assert_eq!(smtp_status, SmtpStatus::PositiveCompletion);
    }

    #[test]
    fn test_available_commends_response() {
        let response = "250-smtp.gmail.com at your service, [217.65.241.77]
                250-SIZE 35882577
                250-8BITMIME
                250-STARTTLS
                250-ENHANCEDSTATUSCODES
                250-PIPELINING
                250-CHUNKING
                130 SMTPUTF8";
        
        let builder = SmtpResponseBuilder::new();
        let smtp_response = builder.build(response).unwrap();
        assert_eq!(smtp_response.m_status, SmtpStatus::PositiveCompletion);
    }
}