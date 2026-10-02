use super::{Interference, RenderRegion};
use image::{Rgba, RgbaImage};
use rand::Rng;

/// RGB 通道偏移
pub struct ColorChannelShift {
    pub max_shift: i32,
}

impl Default for ColorChannelShift {
    fn default() -> Self {
        Self { max_shift: 2 }
    }
}

impl Interference for ColorChannelShift {
    fn apply(&self, image: &mut RgbaImage, _region: &RenderRegion) {
        // 参数边界收敛：max_shift <= 0 视为最小偏移 1，避免空随机区间
        let max_shift = self.max_shift.max(1);
        let mut rng = rand::thread_rng();
        let (w, h) = image.dimensions();
        if w == 0 || h == 0 {
            return;
        }
        let r_shift = rng.gen_range(1..=max_shift);
        let b_shift = rng.gen_range(1..=max_shift);
        // 复制原图
        let original = image.clone();
        // 对每个像素，R 通道左移，B 通道右移
        for y in 0..h {
            for x in 0..w {
                let rx = (x as i32 - r_shift).max(0) as u32;
                let bx = (x as i32 + b_shift).min(w as i32 - 1) as u32;
                let orig_pixel = original.get_pixel(x, y);
                let r_pixel = original.get_pixel(rx, y);
                let b_pixel = original.get_pixel(bx, y);
                image.put_pixel(x, y, Rgba([r_pixel[0], orig_pixel[1], b_pixel[2], 255]));
            }
        }
    }
}

/// 渐变色文字
pub struct ColorGradientText {
    pub colors: Vec<Rgba<u8>>,
}

impl Default for ColorGradientText {
    fn default() -> Self {
        Self {
            colors: vec![
                Rgba([200, 50, 50, 255]),
                Rgba([50, 200, 50, 255]),
                Rgba([50, 50, 200, 255]),
            ],
        }
    }
}

impl Interference for ColorGradientText {
    fn apply(&self, image: &mut RgbaImage, region: &RenderRegion) {
        let (w, h) = image.dimensions();
        if region.char_bounds.is_empty() || self.colors.is_empty() {
            return;
        }
        let mut rng = rand::thread_rng();
        for bounds in &region.char_bounds {
            let x0 = bounds.x;
            let y0 = bounds.y;
            let x1 = (bounds.x + bounds.w).min(w);
            let y1 = (bounds.y + bounds.h).min(h);
            if x1 <= x0 || y1 <= y0 {
                continue;
            }
            let c0 = self.colors[rng.gen_range(0..self.colors.len())];
            let c1 = self.colors[rng.gen_range(0..self.colors.len())];
            let width = (x1 - x0).max(1);
            for y in y0..y1 {
                for x in x0..x1 {
                    let t = (x - x0) as f32 / width as f32;
                    let r = (c0[0] as f32 * (1.0 - t) + c1[0] as f32 * t) as u8;
                    let g = (c0[1] as f32 * (1.0 - t) + c1[1] as f32 * t) as u8;
                    let b = (c0[2] as f32 * (1.0 - t) + c1[2] as f32 * t) as u8;
                    let p = image.get_pixel(x, y);
                    if p[0] < 150 && p[1] < 150 && p[2] < 150 {
                        image.put_pixel(x, y, Rgba([r, g, b, 255]));
                    }
                }
            }
        }
    }
}

/// 局部色彩反转
pub struct ColorInversion {
    pub region_size: u32,
    pub probability: f32,
}

impl Default for ColorInversion {
    fn default() -> Self {
        Self {
            region_size: 20,
            probability: 0.3,
        }
    }
}

impl Interference for ColorInversion {
    fn apply(&self, image: &mut RgbaImage, region: &RenderRegion) {
        let mut rng = rand::thread_rng();
        let (w, h) = image.dimensions();
        if w == 0 || h == 0 {
            return;
        }
        // 参数边界收敛：区域尺寸钳制到 [1, min(w,h)]，
        // 防止 region_size=0 除零或 >= 图像尺寸时空随机区间
        let region_size = self.region_size.clamp(1, w.min(h));
        let count = (region.width * region.height / (region_size * region_size * 4)).max(1);
        for _ in 0..count {
            if rng.gen::<f32>() > self.probability {
                continue;
            }
            let w_span = w.saturating_sub(region_size);
            let h_span = h.saturating_sub(region_size);
            if w_span == 0 || h_span == 0 {
                continue;
            }
            let x0 = rng.gen_range(0..w_span);
            let y0 = rng.gen_range(0..h_span);
            for dy in 0..region_size {
                for dx in 0..region_size {
                    let px = x0 + dx;
                    let py = y0 + dy;
                    if px < w && py < h {
                        let p = image.get_pixel(px, py);
                        image.put_pixel(px, py, Rgba([255 - p[0], 255 - p[1], 255 - p[2], 255]));
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_image() -> RgbaImage {
        RgbaImage::from_pixel(40, 30, Rgba([220, 220, 220, 255]))
    }

    #[test]
    fn boundary_params_do_not_panic() {
        let region = RenderRegion::new(40, 30);
        // max_shift = 0 (空随机区间)
        ColorChannelShift { max_shift: 0 }.apply(&mut test_image(), &region);
        // region_size = 0 (除零) 与 region_size 大于图像尺寸 (空随机区间)
        ColorInversion {
            region_size: 0,
            probability: 1.0,
        }
        .apply(&mut test_image(), &region);
        ColorInversion {
            region_size: 1000,
            probability: 1.0,
        }
        .apply(&mut test_image(), &region);
    }
}
