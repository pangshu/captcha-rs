use super::region::RenderRegion;
use image::{Rgba, RgbaImage};

/// 干扰器 trait
///
/// 所有干扰手段实现此接口，按链式顺序依次执行。
pub trait Interference: Send + Sync {
    /// 对图像应用干扰
    fn apply(&self, image: &mut RgbaImage, region: &RenderRegion);
}

/// 对 Vec<Box<dyn Interference>> 的批量执行
pub fn apply_all(
    interferences: &[Box<dyn Interference>],
    image: &mut RgbaImage,
    region: &RenderRegion,
) {
    // 空图防御：退化尺寸下所有随机坐标区间为空，直接跳过
    if image.width() == 0 || image.height() == 0 {
        return;
    }
    for item in interferences {
        item.apply(image, region);
    }
}

/// 对图像指定像素做 alpha 混合 (越界坐标自动忽略)
///
/// 替代 image 0.25 已废弃的 `GenericImage::blend_pixel`，
/// 干扰器模块统一经此函数写入像素，未来 image 升级只需改这一处。
pub(crate) fn blend_at(img: &mut RgbaImage, x: u32, y: u32, color: Rgba<u8>) {
    if x >= img.width() || y >= img.height() {
        return;
    }
    let alpha = color[3] as f32 / 255.0;
    let inv = 1.0 - alpha;
    let pixel = img.get_pixel_mut(x, y);
    pixel[0] = (pixel[0] as f32 * inv + color[0] as f32 * alpha) as u8;
    pixel[1] = (pixel[1] as f32 * inv + color[1] as f32 * alpha) as u8;
    pixel[2] = (pixel[2] as f32 * inv + color[2] as f32 * alpha) as u8;
    pixel[3] = 255;
}
