use crate::config::MIME_TYPE_IMAGE;
use crate::error::{CaptchaError, Result};
use base64::Engine;
use image::{ImageEncoder, RgbaImage};

/// 图像验证码绘制结果
pub struct ImageItem {
    pub image: RgbaImage,
}

impl ImageItem {
    pub fn new(image: RgbaImage) -> Self {
        Self { image }
    }

    /// 编码为 PNG 字节数据
    pub fn encode_png(&self) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        let encoder = image::codecs::png::PngEncoder::new(&mut buf);
        encoder
            .write_image(
                self.image.as_raw(),
                self.image.width(),
                self.image.height(),
                image::ExtendedColorType::Rgba8,
            )
            .map_err(CaptchaError::from)?;
        Ok(buf)
    }

    /// 编码为 base64 data URI
    pub fn encode_base64(&self) -> Result<String> {
        let png_data = self.encode_png()?;
        let b64 = base64::engine::general_purpose::STANDARD.encode(&png_data);
        Ok(format!("{},{}", MIME_TYPE_IMAGE, b64))
    }
}

/// 图像验证码生成结果
#[derive(Debug, Clone)]
pub struct ImageResult {
    pub id: String,
    pub data: String,
    pub mime_type: String,
}
