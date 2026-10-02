use super::Driver;
use crate::config::TXT_NUMBERS;
use crate::error::Result;
use crate::util::{random_chars, random_id};

/// 数字验证码驱动
pub struct DriverDigit {
    length: usize,
}

impl DriverDigit {
    pub fn new(length: usize) -> Self {
        Self { length }
    }
}

impl Driver for DriverDigit {
    fn generate(&self) -> Result<(String, String, String)> {
        let id = random_id();
        let content = random_chars(TXT_NUMBERS, self.length);
        let answer = content.clone();
        Ok((id, content, answer))
    }

    fn name(&self) -> &str {
        "DriverDigit"
    }
}
