use crate::config::DEFAULT_CAPTCHA_LENGTH;

/// 音频验证码配置
#[derive(Debug, Clone)]
pub struct AudioConfig {
    pub length: usize,
    pub source: Option<String>,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            length: DEFAULT_CAPTCHA_LENGTH,
            source: None,
        }
    }
}
