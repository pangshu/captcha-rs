use super::{Interference, RenderRegion};
use image::RgbaImage;

/// 正弦波浪扭曲
pub struct DistortionWave {
    pub amplitude: f32,
    pub frequency: f32,
}

impl Default for DistortionWave {
    fn default() -> Self {
        Self {
            amplitude: 2.0,
            frequency: 0.08,
        }
    }
}

impl Interference for DistortionWave {
    fn apply(&self, image: &mut RgbaImage, _region: &RenderRegion) {
        let (w, h) = image.dimensions();
        if w == 0 || h == 0 {
            return;
        }
        // 逐行处理：每次仅借用一行快照，避免整图 clone
        for y in 0..h {
            let row: Vec<_> = (0..w).map(|x| *image.get_pixel(x, y)).collect();
            let offset = (self.amplitude * (y as f32 * self.frequency).sin()) as i32;
            for x in 0..w {
                let src_x = (x as i32 + offset).clamp(0, w as i32 - 1) as u32;
                image.put_pixel(x, y, row[src_x as usize]);
            }
        }
    }
}

/// 弯曲变形
pub struct DistortionBend {
    pub strength: f32,
}

impl Default for DistortionBend {
    fn default() -> Self {
        Self { strength: 0.3 }
    }
}

impl Interference for DistortionBend {
    fn apply(&self, image: &mut RgbaImage, _region: &RenderRegion) {
        let (w, h) = image.dimensions();
        if w == 0 || h == 0 {
            return;
        }
        let center_y = h as f32 / 2.0;
        // 逐行处理：每次仅借用一行快照，避免整图 clone
        for y in 0..h {
            let dist_from_center = (y as f32 - center_y) / center_y.max(1.0);
            let offset =
                (self.strength * dist_from_center * dist_from_center * w as f32 * 0.1) as i32;
            let row: Vec<_> = (0..w).map(|x| *image.get_pixel(x, y)).collect();
            for x in 0..w {
                let src_x = (x as i32 + offset).clamp(0, w as i32 - 1) as u32;
                image.put_pixel(x, y, row[src_x as usize]);
            }
        }
    }
}
