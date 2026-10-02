use crate::config::{Background, Difficulty};
use crate::interference::Interference;
use std::path::PathBuf;

/// 图像验证码配置
pub struct ImageConfig {
    pub width: u32,
    pub height: u32,
    pub length: usize,
    pub max_skew: f64,
    pub char_overlap: f32,
    pub char_spacing_jitter: f32,
    pub font_paths: Vec<PathBuf>,
    pub source: Option<String>,
    pub difficulty: Difficulty,
    pub background: Background,
    pub interferences: Vec<Box<dyn Interference>>,
}

impl Default for ImageConfig {
    fn default() -> Self {
        Self {
            width: crate::config::DEFAULT_IMG_WIDTH,
            height: crate::config::DEFAULT_IMG_HEIGHT,
            length: crate::config::DEFAULT_CAPTCHA_LENGTH,
            max_skew: Difficulty::default().default_max_skew(),
            char_overlap: crate::config::DEFAULT_CHAR_OVERLAP,
            char_spacing_jitter: crate::config::DEFAULT_CHAR_SPACING_JITTER,
            font_paths: Vec::new(),
            source: None,
            difficulty: Difficulty::default(),
            background: Background::default(),
            interferences: Vec::new(),
        }
    }
}

impl ImageConfig {
    /// 创建 Builder
    pub fn builder() -> ImageConfigBuilder {
        ImageConfigBuilder::default()
    }
}

/// 图像验证码配置 Builder
#[derive(Default)]
pub struct ImageConfigBuilder {
    config: ImageConfig,
}

impl ImageConfigBuilder {
    pub fn width(mut self, w: u32) -> Self {
        self.config.width = w;
        self
    }
    pub fn height(mut self, h: u32) -> Self {
        self.config.height = h;
        self
    }
    pub fn length(mut self, l: usize) -> Self {
        self.config.length = l;
        self
    }
    pub fn max_skew(mut self, s: f64) -> Self {
        self.config.max_skew = s;
        self
    }
    pub fn char_overlap(mut self, o: f32) -> Self {
        self.config.char_overlap = o;
        self
    }
    pub fn char_spacing_jitter(mut self, j: f32) -> Self {
        self.config.char_spacing_jitter = j;
        self
    }
    pub fn font_paths(mut self, paths: Vec<PathBuf>) -> Self {
        self.config.font_paths = paths;
        self
    }
    pub fn source(mut self, s: impl Into<String>) -> Self {
        self.config.source = Some(s.into());
        self
    }
    pub fn difficulty(mut self, d: Difficulty) -> Self {
        self.config.difficulty = d;
        self.config.max_skew = d.default_max_skew();
        self
    }
    pub fn background(mut self, b: Background) -> Self {
        self.config.background = b;
        self
    }
    pub fn interferences(mut self, i: Vec<Box<dyn Interference>>) -> Self {
        self.config.interferences = i;
        self
    }
    pub fn build(self) -> ImageConfig {
        self.config
    }
}
