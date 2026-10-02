use super::Driver;
use crate::error::Result;
use crate::util::{random_chars, random_id};

/// 多语言验证码驱动
pub struct DriverLanguage {
    length: usize,
    source: String,
}

impl DriverLanguage {
    pub fn new(length: usize, source: Option<&str>) -> Self {
        Self {
            length,
            source: source
                .map(|s| s.to_string())
                .unwrap_or_else(|| "ABCDEFGHIJKLMNOPQRSTUVWXYZ".to_string()),
        }
    }

    /// 俄语预设
    pub fn russian(length: usize) -> Self {
        Self::new(length, Some("АБВГДЕЁЖЗИЙКЛМНОПРСТУФХЦЧШЩЪЫЬЭЮЯ"))
    }

    /// 韩语预设
    pub fn korean(length: usize) -> Self {
        Self::new(
            length,
            Some("ㄱㄴㄷㄹㅁㅂㅅㅇㅈㅊㅋㅌㅍㅎㅏㅑㅓㅕㅗㅛㅜㅠㅡㅣ"),
        )
    }

    /// 日语预设
    pub fn japanese(length: usize) -> Self {
        Self::new(
            length,
            Some("あいうえおかきくけこさしすせそたちつてとなにぬねのはひふへほまみむめもやゆよらりるれろわをん"),
        )
    }

    /// 阿拉伯语预设
    pub fn arabic(length: usize) -> Self {
        Self::new(length, Some("ابتثجحخدذرزسشصضطظعغفقكلمنهوي"))
    }
}

impl Driver for DriverLanguage {
    fn generate(&self) -> Result<(String, String, String)> {
        let id = random_id();
        let content = random_chars(&self.source, self.length);
        let answer = content.clone();
        Ok((id, content, answer))
    }

    fn name(&self) -> &str {
        "DriverLanguage"
    }
}
