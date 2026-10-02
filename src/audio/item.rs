use crate::config::MIME_TYPE_AUDIO;
use crate::error::Result;
use base64::Engine;

/// 音频验证码绘制结果
pub struct AudioItem {
    pub wav_data: Vec<u8>,
}

impl AudioItem {
    pub fn new(wav_data: Vec<u8>) -> Self {
        Self { wav_data }
    }

    /// 编码为 base64 data URI
    pub fn encode_base64(&self) -> Result<String> {
        let b64 = base64::engine::general_purpose::STANDARD.encode(&self.wav_data);
        Ok(format!("{},{}", MIME_TYPE_AUDIO, b64))
    }
}

/// 音频验证码生成结果
#[derive(Debug, Clone)]
pub struct AudioResult {
    pub id: String,
    pub data: String,
    pub mime_type: String,
}
