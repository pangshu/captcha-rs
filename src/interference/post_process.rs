use super::{Interference, RenderRegion};
use image::{Rgba, RgbaImage};
use rand::Rng;

/// 高斯模糊 (盒式近似)
pub struct PostBlur {
    pub sigma: f32,
}

impl Default for PostBlur {
    fn default() -> Self {
        Self { sigma: 0.8 }
    }
}

/// 一维盒式模糊的单次水平/垂直遍历
///
/// 对每个像素取 [pos-radius, pos+radius] 区间的均值 (边缘 clamp)，
/// 两次遍历 (先水平后垂直) 近似二维盒式模糊，
/// 复杂度 O(2·w·h·r)，远低于逐像素二维采样的 O(w·h·r²)。
fn box_blur_pass(src: &RgbaImage, dst: &mut RgbaImage, radius: i32, horizontal: bool) {
    let (w, h) = src.dimensions();
    let (outer, inner) = if horizontal { (h, w) } else { (w, h) };
    for o in 0..outer {
        for i in 0..inner {
            let (mut r_sum, mut g_sum, mut b_sum, mut count) = (0u32, 0u32, 0u32, 0u32);
            for d in -radius..=radius {
                let s = (i as i32 + d).clamp(0, inner as i32 - 1);
                let p = if horizontal {
                    src.get_pixel(s as u32, o)
                } else {
                    src.get_pixel(o, s as u32)
                };
                r_sum += p[0] as u32;
                g_sum += p[1] as u32;
                b_sum += p[2] as u32;
                count += 1;
            }
            let pixel = Rgba([
                (r_sum / count) as u8,
                (g_sum / count) as u8,
                (b_sum / count) as u8,
                255,
            ]);
            if horizontal {
                dst.put_pixel(i, o, pixel);
            } else {
                dst.put_pixel(o, i, pixel);
            }
        }
    }
}

impl Interference for PostBlur {
    fn apply(&self, image: &mut RgbaImage, _region: &RenderRegion) {
        let (w, h) = image.dimensions();
        let radius = (self.sigma * 3.0).ceil() as i32;
        if radius < 1 || w == 0 || h == 0 {
            return;
        }
        // 可分离卷积：水平一遍 + 垂直一遍，仅需一个中间缓冲
        let mut horizontal = RgbaImage::new(w, h);
        box_blur_pass(image, &mut horizontal, radius, true);
        box_blur_pass(&horizontal, image, radius, false);
    }
}

/// 色彩量化
pub struct PostQuantize {
    pub bits: u8,
}

impl Default for PostQuantize {
    fn default() -> Self {
        Self { bits: 4 }
    }
}

impl Interference for PostQuantize {
    fn apply(&self, image: &mut RgbaImage, _region: &RenderRegion) {
        // 参数边界收敛：钳制到 [1, 8]，防止 bits >= 9 时 step 为 0 (除零)
        // 或 debug 构建下移位溢出
        let bits = self.bits.clamp(1, 8);
        let levels = (1u32 << bits) - 1;
        let step = (255 / levels).max(1);
        for pixel in image.pixels_mut() {
            pixel[0] = (pixel[0] as u32 / step * step).min(255) as u8;
            pixel[1] = (pixel[1] as u32 / step * step).min(255) as u8;
            pixel[2] = (pixel[2] as u32 / step * step).min(255) as u8;
        }
    }
}

/// JPEG 伪影
pub struct PostJpegArtifacts {
    pub strength: u8,
}

impl Default for PostJpegArtifacts {
    fn default() -> Self {
        Self { strength: 5 }
    }
}

impl Interference for PostJpegArtifacts {
    fn apply(&self, image: &mut RgbaImage, _region: &RenderRegion) {
        // 模拟 JPEG 8x8 块效应
        let (w, h) = image.dimensions();
        if w == 0 || h == 0 {
            return;
        }
        let block = 8u32;
        let noise_level = self.strength as u32 * 5;
        let mut rng = rand::thread_rng();
        for by in (0..h).step_by(block as usize) {
            for bx in (0..w).step_by(block as usize) {
                // 每块添加随机噪声偏移
                let n_r = rng.gen_range(0..=noise_level) as i32 - (noise_level as i32 / 2);
                let n_g = rng.gen_range(0..=noise_level) as i32 - (noise_level as i32 / 2);
                let n_b = rng.gen_range(0..=noise_level) as i32 - (noise_level as i32 / 2);
                for dy in 0..block {
                    for dx in 0..block {
                        let x = bx + dx;
                        let y = by + dy;
                        if x < w && y < h {
                            let p = image.get_pixel(x, y);
                            let r = (p[0] as i32 + n_r).clamp(0, 255) as u8;
                            let g = (p[1] as i32 + n_g).clamp(0, 255) as u8;
                            let b = (p[2] as i32 + n_b).clamp(0, 255) as u8;
                            image.put_pixel(x, y, Rgba([r, g, b, 255]));
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boundary_params_do_not_panic() {
        let region = RenderRegion::new(40, 30);
        // bits = 9 (除零) / bits = 0 (移位溢出) / bits = 255
        for bits in [0u8, 9, 255] {
            let mut img = RgbaImage::from_pixel(40, 30, Rgba([200, 100, 50, 255]));
            PostQuantize { bits }.apply(&mut img, &region);
        }
        // sigma = 0 (radius < 1 直接返回)
        PostBlur { sigma: 0.0 }.apply(&mut test_image(), &region);
    }

    fn test_image() -> RgbaImage {
        RgbaImage::from_pixel(40, 30, Rgba([220, 220, 220, 255]))
    }
}
