/// 滑块验证码生成结果
#[derive(Debug, Clone)]
pub struct SliderResult {
    pub id: String,
    pub bg_image: String,
    pub piece_image: String,
    pub y_position: u32,
}

/// 文字点选验证码生成结果
#[derive(Debug, Clone)]
pub struct ClickTextResult {
    pub id: String,
    pub image: String,
    pub target_chars: String,
}
