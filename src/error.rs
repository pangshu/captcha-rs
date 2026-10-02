use thiserror::Error;

/// 验证码库统一错误类型
#[derive(Error, Debug)]
pub enum CaptchaError {
    /// 图像绘制失败
    #[error("failed to draw captcha image: {0}")]
    DrawError(String),

    /// 图像编码失败
    #[cfg(feature = "image")]
    #[error("failed to encode image: {0}")]
    EncodeError(#[from] image::ImageError),

    /// 字体加载失败
    #[error("failed to load font '{name}': {reason}")]
    FontLoadError { name: String, reason: String },

    /// 配置错误
    #[error("config error: {0}")]
    ConfigError(String),

    /// JSON 序列化/反序列化失败
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// IO 错误
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, CaptchaError>;
