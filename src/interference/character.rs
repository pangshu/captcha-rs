use super::api::blend_at;
use super::{Interference, RenderRegion};
use image::{Rgba, RgbaImage};
use rand::Rng;

/// 文字描边
pub struct CharOutline {
    pub color: Rgba<u8>,
}

impl Default for CharOutline {
    fn default() -> Self {
        Self {
            color: Rgba([80, 80, 80, 120]),
        }
    }
}

impl Interference for CharOutline {
    fn apply(&self, image: &mut RgbaImage, region: &RenderRegion) {
        let (w, h) = image.dimensions();
        // 对每个字符边界框周围画细线
        for bounds in &region.char_bounds {
            let x0 = bounds.x;
            let y0 = bounds.y;
            let x1 = (bounds.x + bounds.w).min(w);
            let y1 = (bounds.y + bounds.h).min(h);
            // 画边框
            for x in x0..x1 {
                if y0 < h {
                    blend_at(image, x, y0, self.color);
                }
                if y1 > 0 && y1 < h {
                    blend_at(image, x, y1 - 1, self.color);
                }
            }
            for y in y0..y1 {
                if x0 < w {
                    blend_at(image, x0, y, self.color);
                }
                if x1 > 0 && x1 < w {
                    blend_at(image, x1 - 1, y, self.color);
                }
            }
        }
    }
}

/// 文字阴影
pub struct CharShadow {
    pub offset: (i32, i32),
    pub alpha: u8,
}

impl Default for CharShadow {
    fn default() -> Self {
        Self {
            offset: (2, 2),
            alpha: 80,
        }
    }
}

impl Interference for CharShadow {
    fn apply(&self, image: &mut RgbaImage, region: &RenderRegion) {
        let (w, h) = image.dimensions();
        // 对每个字符区域，将深色像素偏移复制
        for bounds in &region.char_bounds {
            let x0 = bounds.x;
            let y0 = bounds.y;
            let x1 = (bounds.x + bounds.w).min(w);
            let y1 = (bounds.y + bounds.h).min(h);
            // 收集深色像素
            let mut dark_pixels: Vec<(u32, u32)> = Vec::new();
            for y in y0..y1 {
                for x in x0..x1 {
                    let p = image.get_pixel(x, y);
                    if p[0] < 150 && p[1] < 150 && p[2] < 150 {
                        dark_pixels.push((x, y));
                    }
                }
            }
            // 偏移绘制阴影
            let shadow_color = Rgba([0, 0, 0, self.alpha]);
            for (x, y) in dark_pixels {
                let sx = x as i32 + self.offset.0;
                let sy = y as i32 + self.offset.1;
                if sx >= 0 && sy >= 0 {
                    let sxu = sx as u32;
                    let syu = sy as u32;
                    if sxu < w && syu < h {
                        blend_at(image, sxu, syu, shadow_color);
                    }
                }
            }
        }
    }
}

/// 半透明随机色块叠加
///
/// 注意：绘制的是随机色块而非真实字符（干扰器拿不到字体上下文），
/// 需要真实字符级干扰请使用渲染管线的字符参数或文字级干扰器。
pub struct ColorBlockOverlay {
    /// 每个色块的透明度
    pub alpha: u8,
}

impl Default for ColorBlockOverlay {
    fn default() -> Self {
        Self { alpha: 40 }
    }
}

impl Interference for ColorBlockOverlay {
    fn apply(&self, image: &mut RgbaImage, _region: &RenderRegion) {
        let mut rng = rand::thread_rng();
        let (w, h) = image.dimensions();
        if w == 0 || h == 0 {
            return;
        }
        let overlay_count = rng.gen_range(2..=5);
        for _ in 0..overlay_count {
            let x = rng.gen_range(0..w);
            let y = rng.gen_range(0..h);
            let block_size = rng.gen_range(8..=16);
            let color = Rgba([
                rng.gen_range(0..=255),
                rng.gen_range(0..=255),
                rng.gen_range(0..=255),
                self.alpha,
            ]);
            for dy in 0..block_size {
                for dx in 0..block_size {
                    let px = x + dx;
                    let py = y + dy;
                    if px < w && py < h {
                        blend_at(image, px, py, color);
                    }
                }
            }
        }
    }
}

/// 笔画断裂
pub struct CharStrokeBreak {
    pub probability: f32,
    pub gap: u32,
}

impl Default for CharStrokeBreak {
    fn default() -> Self {
        Self {
            probability: 0.1,
            gap: 2,
        }
    }
}

impl Interference for CharStrokeBreak {
    fn apply(&self, image: &mut RgbaImage, region: &RenderRegion) {
        let mut rng = rand::thread_rng();
        let (w, h) = image.dimensions();
        for bounds in &region.char_bounds {
            if rng.gen::<f32>() > self.probability {
                continue;
            }
            let x0 = bounds.x;
            let y0 = bounds.y;
            let x1 = (bounds.x + bounds.w).min(w);
            let y1 = (bounds.y + bounds.h).min(h);
            // 边界防护：退化矩形 (高度为 0 或完全出界) 时跳过
            if y1 <= y0 || x1 <= x0 {
                continue;
            }
            // 随机选择一行进行清除
            let break_y = rng.gen_range(y0..y1);
            // 将断裂行向白色提亮以模拟擦除，
            // 相比随机浅色硬覆盖，在渐变/图片背景下更自然
            let lighten = |c: u8| ((c as u32 * 2 + 255) / 3).min(255) as u8;
            for x in x0..x1 {
                for g in 0..self.gap {
                    let py = break_y + g;
                    if py < h {
                        let p = image.get_pixel(x, py);
                        image.put_pixel(
                            x,
                            py,
                            Rgba([lighten(p[0]), lighten(p[1]), lighten(p[2]), 255]),
                        );
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interference::Rect;

    fn test_image() -> RgbaImage {
        RgbaImage::from_pixel(40, 30, Rgba([220, 220, 220, 255]))
    }

    #[test]
    fn boundary_params_do_not_panic() {
        let region = RenderRegion::new(40, 30);
        // 空字符集 (原 CharOverlay 空串 panic 路径已随重设计消除)
        ColorBlockOverlay { alpha: 40 }.apply(&mut test_image(), &region);
        // 空目标区域 + 退化矩形 (高度为 0)
        let mut degenerate = RenderRegion::new(40, 30);
        degenerate.char_bounds = vec![Rect {
            x: 0,
            y: 10,
            w: 10,
            h: 0,
        }];
        CharStrokeBreak {
            probability: 1.0,
            gap: 2,
        }
        .apply(&mut test_image(), &degenerate);
        // 完全出界的边界框
        let mut out_of_bounds = RenderRegion::new(40, 30);
        out_of_bounds.char_bounds = vec![Rect {
            x: 0,
            y: 50,
            w: 10,
            h: 10,
        }];
        CharStrokeBreak {
            probability: 1.0,
            gap: 2,
        }
        .apply(&mut test_image(), &out_of_bounds);
    }
}
