use super::config::AudioConfig;
use super::item::{AudioItem, AudioResult};
use super::wav::generate_captcha_wav;
use crate::config::TXT_NUMBERS;
use crate::error::Result;
use crate::store::{AnswerData, Store};
use crate::util::{random_chars, random_id};
use std::sync::Arc;

/// 音频验证码 Builder
pub struct AudioBuilder {
    store: Arc<dyn Store>,
    config: AudioConfig,
}

impl AudioBuilder {
    pub fn new(store: Arc<dyn Store>) -> Self {
        Self {
            store,
            config: AudioConfig::default(),
        }
    }

    pub fn length(mut self, l: usize) -> Self {
        self.config.length = l;
        self
    }

    pub fn source(mut self, s: impl Into<String>) -> Self {
        self.config.source = Some(s.into());
        self
    }

    /// 生成音频验证码
    pub fn generate(self) -> Result<AudioResult> {
        let source = self.config.source.as_deref().unwrap_or(TXT_NUMBERS);

        let id = random_id();
        let content = random_chars(source, self.config.length);
        let answer = content.clone();

        let wav_data = generate_captcha_wav(&content)?;
        let item = AudioItem::new(wav_data);
        let data = item.encode_base64()?;

        let answer_data = AnswerData::text(answer);
        self.store.set(&id, &answer_data.to_json());

        Ok(AudioResult {
            id,
            data,
            mime_type: crate::config::MIME_TYPE_AUDIO.to_string(),
        })
    }
}
