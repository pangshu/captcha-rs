use super::Driver;
use crate::config::TXT_ALPHABET;
use crate::error::Result;
use crate::util::{random_chars, random_id};

/// 字母数字验证码驱动
pub struct DriverString {
    length: usize,
    source: String,
}

impl DriverString {
    pub fn new(length: usize, source: Option<&str>) -> Self {
        Self {
            length,
            source: source
                .map(|s| s.to_string())
                .unwrap_or_else(|| TXT_ALPHABET.to_string()),
        }
    }
}

impl Driver for DriverString {
    fn generate(&self) -> Result<(String, String, String)> {
        let id = random_id();
        let content = random_chars(&self.source, self.length);
        let answer = content.clone();
        Ok((id, content, answer))
    }

    fn name(&self) -> &str {
        "DriverString"
    }
}
