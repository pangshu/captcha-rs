use super::config::ImageConfig;
use super::driver::{create_driver, DriverType};
use super::item::{ImageItem, ImageResult};
use super::render::render_image;
use crate::error::Result;
use crate::font::FontLoader;
use crate::store::Store;
use std::path::PathBuf;
use std::sync::Arc;

/// 图像验证码 Builder
pub struct ImageBuilder {
    store: Arc<dyn Store>,
    driver_type: DriverType,
    config: ImageConfig,
}

impl ImageBuilder {
    pub fn new(store: Arc<dyn Store>, driver_type: DriverType) -> Self {
        Self {
            store,
            driver_type,
            config: ImageConfig::default(),
        }
    }

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
    /// 字符间距抖动比例 (0.0~1.0，默认 0.2)
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
    pub fn difficulty(mut self, d: crate::config::Difficulty) -> Self {
        self.config.max_skew = d.default_max_skew();
        self.config.difficulty = d;
        self
    }
    pub fn background(mut self, b: crate::config::Background) -> Self {
        self.config.background = b;
        self
    }
    pub fn interferences(mut self, i: Vec<Box<dyn crate::interference::Interference>>) -> Self {
        self.config.interferences = i;
        self
    }

    /// 生成验证码
    pub fn generate(self) -> Result<ImageResult> {
        let mut config = self.config;

        // 0. 配置校验：拒绝退化尺寸
        if config.width < 20 || config.height < 20 {
            return Err(crate::error::CaptchaError::ConfigError(format!(
                "image size too small: {}x{}, minimum is 20x20",
                config.width, config.height
            )));
        }

        // 1. 创建 Driver
        let driver = create_driver(self.driver_type, &config);
        // 2. 生成 id, content, answer
        let (id, content, answer) = driver.generate()?;
        // 3. 加载字体
        let fonts = load_fonts(&config)?;
        // 4. 装配干扰器：用户未显式指定时按难度预置组合
        if config.interferences.is_empty() {
            config.interferences = crate::interference::preset_interferences(config.difficulty);
        }
        // 5. 渲染图像
        let render_result = render_image(&config, &content, &fonts)?;
        // 6. 编码为 base64
        let item = ImageItem::new(render_result.image);
        let data = item.encode_base64()?;
        // 7. 存储答案
        let answer_data = crate::store::AnswerData::text(answer);
        self.store.set(&id, &answer_data.to_json());
        // 8. 返回结果
        Ok(ImageResult {
            id,
            data,
            mime_type: crate::config::MIME_TYPE_IMAGE.to_string(),
        })
    }
}

/// 加载字体
///
/// 使用全局缓存避免重复 IO：默认字体只加载一次，
/// 自定义字体路径每次加载（用户通常不会高频切换路径）。
fn load_fonts(config: &ImageConfig) -> Result<FontLoader> {
    if config.font_paths.is_empty() {
        // 使用全局缓存的默认字体
        let loader = crate::font::get_default_fonts();
        if loader.is_empty() {
            return Err(crate::error::CaptchaError::FontLoadError {
                name: "default fonts".to_string(),
                reason: "no fonts found in fonts/ directory, please specify font_paths".to_string(),
            });
        }
        Ok(loader.clone())
    } else {
        let paths: Vec<&str> = config
            .font_paths
            .iter()
            .filter_map(|p| p.to_str())
            .collect();
        FontLoader::from_paths(&paths)
    }
}
