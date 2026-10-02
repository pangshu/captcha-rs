use image::Rgba;

/// 矩形区域
#[derive(Debug, Clone, Copy, Default)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

/// 渲染区域信息，供干扰器决策
#[derive(Debug, Clone)]
pub struct RenderRegion {
    pub width: u32,
    pub height: u32,
    /// 各字符的边界框
    pub char_bounds: Vec<Rect>,
    /// 各字符的颜色
    pub text_colors: Vec<Rgba<u8>>,
}

impl RenderRegion {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            char_bounds: Vec::new(),
            text_colors: Vec::new(),
        }
    }
}
