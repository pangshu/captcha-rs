use super::Driver;
use crate::config::TXT_CHINESE_CHARS;
use crate::error::Result;
use crate::util::{random_chars, random_id};

/// 中文字符验证码驱动
pub struct DriverChinese {
    length: usize,
}

impl DriverChinese {
    pub fn new(length: usize) -> Self {
        Self { length }
    }
}

impl Driver for DriverChinese {
    fn generate(&self) -> Result<(String, String, String)> {
        let id = random_id();
        let content = random_chars(TXT_CHINESE_CHARS, self.length);
        let answer = content.clone();
        Ok((id, content, answer))
    }

    fn name(&self) -> &str {
        "DriverChinese"
    }
}
