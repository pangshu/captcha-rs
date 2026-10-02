use super::item::ClickTextResult;
use crate::config::{Difficulty, TXT_CHINESE_CHARS};
use crate::error::{CaptchaError, Result};
use crate::font::FontLoader;
use crate::image::item::ImageItem;
use crate::image::render::render_image;
use crate::interference::Interference;
use crate::store::{AnswerData, ClickTarget, Store};
use crate::util::{random_id, random_unique_char_groups};
use std::path::PathBuf;
use std::sync::Arc;

/// 文字点选验证码配置
///
/// 展示 `count` 个目标字与 `distractor_count` 个干扰字，
/// 用户按顺序点选目标字完成验证。
pub struct ClickTextConfig {
    pub width: u32,
    pub height: u32,
    /// 目标字数量 (用户需按顺序点选的字数)
    pub count: usize,
    /// 干扰字数量
    pub distractor_count: usize,
    /// 候选字符池 (默认常用汉字)
    pub source: Option<String>,
    pub font_paths: Vec<PathBuf>,
    pub difficulty: Difficulty,
    pub interferences: Vec<Box<dyn Interference>>,
}

impl Default for ClickTextConfig {
    fn default() -> Self {
        Self {
            width: crate::config::DEFAULT_CLICK_WIDTH,
            height: crate::config::DEFAULT_CLICK_HEIGHT,
            count: crate::config::DEFAULT_CLICK_COUNT,
            distractor_count: crate::config::DEFAULT_CLICK_COUNT,
            source: None,
            font_paths: Vec::new(),
            difficulty: Difficulty::default(),
            interferences: Vec::new(),
        }
    }
}

/// 文字点选验证码 Builder
pub struct ClickTextBuilder {
    store: Arc<dyn Store>,
    config: ClickTextConfig,
}

impl ClickTextBuilder {
    pub fn new(store: Arc<dyn Store>) -> Self {
        Self {
            store,
            config: ClickTextConfig::default(),
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
    pub fn count(mut self, n: usize) -> Self {
        self.config.count = n;
        self
    }
    /// 干扰字数量 (默认与目标字数量一致)
    pub fn distractor_count(mut self, n: usize) -> Self {
        self.config.distractor_count = n;
        self
    }
    /// 候选字符池 (默认常用汉字)
    pub fn source(mut self, s: impl Into<String>) -> Self {
        self.config.source = Some(s.into());
        self
    }
    pub fn font_paths(mut self, paths: Vec<PathBuf>) -> Self {
        self.config.font_paths = paths;
        self
    }
    pub fn difficulty(mut self, d: Difficulty) -> Self {
        self.config.difficulty = d;
        self
    }
    pub fn interferences(mut self, i: Vec<Box<dyn Interference>>) -> Self {
        self.config.interferences = i;
        self
    }

    /// 生成文字点选验证码
    ///
    /// 目标字与干扰字从字符池不放回地随机选取 (组内组间无重复)，
    /// 渲染后依据字符包围盒生成点选目标区域并存储答案。
    pub fn generate(self) -> Result<ClickTextResult> {
        // 0. 配置校验：拒绝退化尺寸与零目标
        if self.config.width < 20 || self.config.height < 20 {
            return Err(CaptchaError::ConfigError(format!(
                "click image size too small: {}x{}, minimum is 20x20",
                self.config.width, self.config.height
            )));
        }
        let count = self.config.count;
        if count == 0 {
            return Err(CaptchaError::ConfigError(
                "click count must be greater than 0".to_string(),
            ));
        }

        // 1. 目标字与干扰字互不重复，保证可区分
        let source = self.config.source.as_deref().unwrap_or(TXT_CHINESE_CHARS);
        let (targets_str, distractors_str) = random_unique_char_groups(
            source,
            count,
            self.config.distractor_count,
        )
        .ok_or_else(|| {
            CaptchaError::ConfigError(
                "source charset too small for requested click count".to_string(),
            )
        })?;

        // 2. 目标字在前、干扰字在后，空格作排版间隔
        let content = format!("{} {}", targets_str, distractors_str);

        // 3. 加载字体 (与图像验证码一致：默认走全局缓存)
        let fonts = load_fonts(&self.config.font_paths)?;

        // 4. 渲染配置：点选字符不重叠 (重叠会让命中区域产生歧义)，
        //    干扰器未显式指定时按难度预置
        let interferences = if self.config.interferences.is_empty() {
            crate::interference::preset_interferences(self.config.difficulty)
        } else {
            self.config.interferences
        };
        let image_config = crate::image::config::ImageConfig {
            width: self.config.width,
            height: self.config.height,
            length: content.chars().filter(|c| !c.is_whitespace()).count(),
            max_skew: self.config.difficulty.default_max_skew(),
            char_overlap: 0.0,
            char_spacing_jitter: crate::config::DEFAULT_CHAR_SPACING_JITTER,
            font_paths: self.config.font_paths.clone(),
            source: None,
            difficulty: self.config.difficulty,
            background: crate::config::Background::default(),
            interferences,
        };

        // 5. 渲染并从字符包围盒推导点选目标区域
        //    (draw_text 按绘制顺序返回 char_bounds，目标字在前)
        let render_result = render_image(&image_config, &content, &fonts)?;
        let target_chars: Vec<char> = targets_str.chars().collect();
        let mut targets = Vec::with_capacity(count);
        for (i, bounds) in render_result.region.char_bounds.iter().take(count).enumerate() {
            targets.push(ClickTarget {
                char: target_chars[i].to_string(),
                x: bounds.x,
                y: bounds.y,
                w: bounds.w,
                h: bounds.h,
            });
        }
        if targets.len() < count {
            return Err(CaptchaError::DrawError(format!(
                "failed to render all {} target characters",
                count
            )));
        }

        // 6. 编码 + 存储答案 + 返回
        let item = ImageItem::new(render_result.image);
        let image = item.encode_base64()?;
        let id = random_id();
        let answer_data = AnswerData::click(targets);
        self.store.set(&id, &answer_data.to_json());

        Ok(ClickTextResult {
            id,
            image,
            target_chars: targets_str,
        })
    }
}

/// 加载字体 (与 image::builder::load_fonts 相同策略)
fn load_fonts(font_paths: &[PathBuf]) -> Result<FontLoader> {
    if font_paths.is_empty() {
        let loader = crate::font::get_default_fonts();
        if loader.is_empty() {
            return Err(CaptchaError::FontLoadError {
                name: "default fonts".to_string(),
                reason: "no fonts found in fonts/ directory, please specify font_paths".to_string(),
            });
        }
        Ok(loader.clone())
    } else {
        let paths: Vec<&str> = font_paths.iter().filter_map(|p| p.to_str()).collect();
        FontLoader::from_paths(&paths)
    }
}
