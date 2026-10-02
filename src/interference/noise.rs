use super::api::blend_at;
use super::{ColorPolicy, Interference, RenderRegion};
use image::RgbaImage;
use rand::Rng;

/// 单像素噪点
pub struct NoisePixel {
    pub count: u32,
}

impl Default for NoisePixel {
    fn default() -> Self {
        Self { count: 60 }
    }
}

impl Interference for NoisePixel {
    fn apply(&self, image: &mut RgbaImage, region: &RenderRegion) {
        if region.width == 0 || region.height == 0 {
            return;
        }
        let mut rng = rand::thread_rng();
        let policy = ColorPolicy::Random;
        for _ in 0..self.count {
            let color = policy.generate(region);
            let x = rng.gen_range(0..region.width);
            let y = rng.gen_range(0..region.height);
            blend_at(image, x, y, color);
        }
    }
}

/// 短线段噪点
pub struct NoiseShortLine {
    pub count: u32,
    pub max_length: u32,
}

impl Default for NoiseShortLine {
    fn default() -> Self {
        Self {
            count: 20,
            max_length: 8,
        }
    }
}

impl Interference for NoiseShortLine {
    fn apply(&self, image: &mut RgbaImage, _region: &RenderRegion) {
        let mut rng = rand::thread_rng();
        let (w, h) = image.dimensions();
        if w == 0 || h == 0 {
            return;
        }
        // 参数边界收敛：max_length < 2 时使用最小长度 2，避免空随机区间
        let max_length = self.max_length.max(2);
        for _ in 0..self.count {
            let color = crate::util::random_color();
            let x0 = rng.gen_range(0..w);
            let y0 = rng.gen_range(0..h);
            let len = rng.gen_range(2..=max_length);
            let angle = rng.gen::<f32>() * std::f32::consts::TAU;
            let dx = (angle.cos() * len as f32) as i32;
            let dy = (angle.sin() * len as f32) as i32;
            let steps = len;
            for i in 0..=steps {
                let t = i as f32 / steps as f32;
                let px = x0 as f32 + dx as f32 * t;
                let py = y0 as f32 + dy as f32 * t;
                if px >= 0.0 && py >= 0.0 {
                    let pxu = px as u32;
                    let pyu = py as u32;
                    if pxu < w && pyu < h {
                        blend_at(image, pxu, pyu, color);
                    }
                }
            }
        }
    }
}

/// 椭圆斑点
pub struct NoiseBlob {
    pub count: u32,
    pub max_size: u32,
}

impl Default for NoiseBlob {
    fn default() -> Self {
        Self {
            count: 15,
            max_size: 6,
        }
    }
}

impl Interference for NoiseBlob {
    fn apply(&self, image: &mut RgbaImage, _region: &RenderRegion) {
        let mut rng = rand::thread_rng();
        let (w, h) = image.dimensions();
        if w == 0 || h == 0 {
            return;
        }
        // 参数边界收敛：上限至少为 2，避免 max_size < 2 或图宽/高过小时空随机区间
        let max_rw = self.max_size.min(w / 2).max(2);
        let max_rh = self.max_size.min(h / 2).max(2);
        for _ in 0..self.count {
            let color = crate::util::random_color();
            let cx = rng.gen_range(0..w);
            let cy = rng.gen_range(0..h);
            let rw = rng.gen_range(2..=max_rw);
            let rh = rng.gen_range(2..=max_rh);
            // 斑点级生成一次 alpha，保证斑点内部均匀
            let alpha = rng.gen_range(30..=80);
            let mut c = color;
            c[3] = alpha;
            // 椭圆判定：像素到中心的归一化距离 <= 1
            let half_w = rw as f32 / 2.0;
            let half_h = rh as f32 / 2.0;
            for dy in 0..rh {
                for dx in 0..rw {
                    let ex = (dx as f32 - half_w + 0.5) / half_w;
                    let ey = (dy as f32 - half_h + 0.5) / half_h;
                    if ex * ex + ey * ey > 1.0 {
                        continue;
                    }
                    let px = cx + dx;
                    let py = cy + dy;
                    if px < w && py < h {
                        blend_at(image, px, py, c);
                    }
                }
            }
        }
    }
}

/// 簇状噪点
pub struct NoiseCluster {
    pub count: u32,
    pub spread: u32,
}

impl Default for NoiseCluster {
    fn default() -> Self {
        Self {
            count: 5,
            spread: 10,
        }
    }
}

impl Interference for NoiseCluster {
    fn apply(&self, image: &mut RgbaImage, _region: &RenderRegion) {
        let mut rng = rand::thread_rng();
        let (w, h) = image.dimensions();
        if w == 0 || h == 0 {
            return;
        }
        for _ in 0..self.count {
            let color = crate::util::random_color();
            let cx = rng.gen_range(0..w);
            let cy = rng.gen_range(0..h);
            let n = rng.gen_range(5..=15);
            for _ in 0..n {
                let dx = rng.gen_range(-(self.spread as i32)..=self.spread as i32);
                let dy = rng.gen_range(-(self.spread as i32)..=self.spread as i32);
                let px = cx as i32 + dx;
                let py = cy as i32 + dy;
                if px >= 0 && py >= 0 {
                    let pxu = px as u32;
                    let pyu = py as u32;
                    if pxu < w && pyu < h {
                        blend_at(image, pxu, pyu, color);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    #[test]
    fn boundary_params_do_not_panic() {
        let region = RenderRegion::new(4, 4);
        let mut img = RgbaImage::from_pixel(4, 4, Rgba([220, 220, 220, 255]));
        // max_size = 1 (< 2) 且图宽 4 (w/2 = 2)
        NoiseBlob {
            count: 3,
            max_size: 1,
        }
        .apply(&mut img, &region);
        // max_length = 0
        NoiseShortLine {
            count: 3,
            max_length: 0,
        }
        .apply(&mut img, &region);
    }
}
