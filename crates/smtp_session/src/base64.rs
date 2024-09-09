use base64::{engine::general_purpose::STANDARD, Engine as _};

pub fn encode(data: &str) -> String {
    STANDARD.encode(data.as_bytes())
}