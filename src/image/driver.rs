use crate::error::Result;

/// 图像验证码驱动类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriverType {
    Digit,
    String,
    Math,
    Chinese,
    Language,
    Idiom,
}

/// 图像验证码 Driver trait
///
/// 每种验证码类型实现此接口，负责生成题目内容和正确答案。
pub trait Driver: Send + Sync {
    /// 生成 (id, 展示内容, 正确答案)
    fn generate(&self) -> Result<(String, String, String)>;

    /// 获取驱动名称 (用于错误信息)
    fn name(&self) -> &str;
}

/// 根据类型创建 Driver
pub fn create_driver(
    driver_type: DriverType,
    config: &super::config::ImageConfig,
) -> Box<dyn Driver> {
    match driver_type {
        DriverType::Digit => Box::new(super::digit::DriverDigit::new(config.length)),
        DriverType::String => Box::new(super::string::DriverString::new(
            config.length,
            config.source.as_deref(),
        )),
        DriverType::Math => Box::new(super::math::DriverMath::new()),
        DriverType::Chinese => Box::new(super::chinese::DriverChinese::new(config.length)),
        DriverType::Language => Box::new(super::language::DriverLanguage::new(
            config.length,
            config.source.as_deref(),
        )),
        DriverType::Idiom => Box::new(super::idiom::DriverIdiom::new()),
    }
}
