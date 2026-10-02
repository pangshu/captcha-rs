use crate::error::{CaptchaError, Result};
use crate::image::item::ImageItem;
use crate::store::AnswerData;
use crate::util::random_id;
use image::{Rgba, RgbaImage};
use rand::Rng;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// 滑块配置
#[derive(Debug, Clone)]
pub struct SliderConfig {
    pub width: u32,
    pub height: u32,
    pub piece_size: u32,
    pub bg_image_paths: Vec<std::path::PathBuf>,
}

impl Default for SliderConfig {
    fn default() -> Self {
        Self {
            width: crate::config::DEFAULT_SLIDER_WIDTH,
            height: crate::config::DEFAULT_SLIDER_HEIGHT,
            piece_size: crate::config::DEFAULT_SLIDER_PIECE_SIZE,
            bg_image_paths: Vec::new(),
        }
    }
}

/// 生成滑块验证码
pub fn generate_slider(config: &SliderConfig) -> Result<(String, String, String, u32, u32)> {
    // (id, bg_base64, piece_base64, y_position, x_answer)
    if config.bg_image_paths.is_empty() {
        return Err(CaptchaError::ConfigError(
            "no background images provided for slider captcha".to_string(),
        ));
    }

    // 尺寸校验：防止 u32 下溢与坐标回绕产生不可解验证码
    if config.width < config.piece_size + 10 || config.height < config.piece_size {
        return Err(CaptchaError::ConfigError(format!(
            "slider size too small: {}x{} with piece {}, width must be >= piece_size + 10 and height >= piece_size",
            config.width, config.height, config.piece_size
        )));
    }

    let mut rng = rand::thread_rng();
    let bg_path = &config.bg_image_paths[rng.gen_range(0..config.bg_image_paths.len())];

    // 加载背景图
    let bg_img = image::open(Path::new(bg_path))
        .map_err(|e| CaptchaError::ConfigError(format!("failed to load bg image: {}", e)))?;
    let bg_img = bg_img.to_rgba8();

    // 缩放到目标尺寸
    let bg = image::imageops::resize(
        &bg_img,
        config.width,
        config.height,
        image::imageops::FilterType::Lanczos3,
    );

    // 随机位置: x 在右侧 60% 区域, y 居中区域
    let x_min = config.width * 40 / 100;
    let x_max = config.width - config.piece_size - 10;
    let x_answer = if x_max > x_min {
        rng.gen_range(x_min..=x_max)
    } else {
        x_min
    };
    let y_center = config.height / 2;
    let y_position = y_center.saturating_sub(config.piece_size / 2);

    // 创建带缺口的背景图
    let mut bg_with_hole = bg.clone();
    // 在缺口位置画半透明灰色
    for dy in 0..config.piece_size {
        for dx in 0..config.piece_size {
            let px = x_answer + dx;
            let py = y_position + dy;
            if px < config.width && py < config.height {
                let p = bg_with_hole.get_pixel(px, py);
                let darkened = Rgba([
                    (p[0] as u32 * 6 / 10) as u8,
                    (p[1] as u32 * 6 / 10) as u8,
                    (p[2] as u32 * 6 / 10) as u8,
                    255,
                ]);
                bg_with_hole.put_pixel(px, py, darkened);
            }
        }
    }

    // 创建拼图块图片 (从原图裁剪)
    let mut piece = RgbaImage::new(config.piece_size, config.piece_size);
    for dy in 0..config.piece_size {
        for dx in 0..config.piece_size {
            let px = x_answer + dx;
            let py = y_position + dy;
            if px < config.width && py < config.height {
                piece.put_pixel(dx, dy, bg.get_pixel(px, py).clone());
            }
        }
    }

    // 编码为 base64
    let bg_item = ImageItem::new(bg_with_hole);
    let bg_base64 = bg_item.encode_base64()?;
    let piece_item = ImageItem::new(piece);
    let piece_base64 = piece_item.encode_base64()?;

    let id = random_id();
    Ok((id, bg_base64, piece_base64, y_position, x_answer))
}

/// 生成滑块验证码并存储答案
pub fn generate_slider_with_store(
    config: &SliderConfig,
    store: &dyn crate::store::Store,
) -> Result<crate::interactive::item::SliderResult> {
    let (id, bg_base64, piece_base64, y_position, x_answer) = generate_slider(config)?;
    // 容差随答案存储：按宽度 2% 计算，下限 5px，
    // 避免大尺寸图下固定 5px 过严、小图下过松
    let tolerance = (config.width * 2 / 100).max(5);
    let answer_data = AnswerData::slider_with_tolerance(x_answer, tolerance);
    store.set(&id, &answer_data.to_json());
    Ok(crate::interactive::item::SliderResult {
        id,
        bg_image: bg_base64,
        piece_image: piece_base64,
        y_position,
    })
}

/// 滑块验证码 Builder
pub struct SliderBuilder {
    store: Arc<dyn crate::store::Store>,
    config: SliderConfig,
}

impl SliderBuilder {
    pub fn new(store: Arc<dyn crate::store::Store>) -> Self {
        Self {
            store,
            config: SliderConfig::default(),
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
    pub fn piece_size(mut self, s: u32) -> Self {
        self.config.piece_size = s;
        self
    }
    pub fn bg_image_paths(mut self, paths: Vec<PathBuf>) -> Self {
        self.config.bg_image_paths = paths;
        self
    }

    pub fn generate(self) -> Result<crate::interactive::item::SliderResult> {
        generate_slider_with_store(&self.config, self.store.as_ref())
    }
}
