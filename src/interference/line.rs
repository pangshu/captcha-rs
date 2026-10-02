use super::api::blend_at;
use super::{ColorPolicy, Interference, RenderRegion};
use image::{Rgba, RgbaImage};
use rand::Rng;

/// 通用线条绘制
fn draw_line(image: &mut RgbaImage, points: &[(i32, i32)], color: Rgba<u8>) {
    let (width, height) = image.dimensions();
    for window in points.windows(2) {
        let (x0, y0) = window[0];
        let (x1, y1) = window[1];
        let dx = (x1 - x0).abs();
        let dy = (y1 - y0).abs();
        let steps = dx.max(dy);
        if steps == 0 {
            continue;
        }
        for i in 0..=steps {
            let t = i as f32 / steps as f32;
            let x = x0 as f32 + (x1 - x0) as f32 * t;
            let y = y0 as f32 + (y1 - y0) as f32 * t;
            let px = x as u32;
            let py = y as u32;
            if px < width && py < height {
                blend_at(image, px, py, color);
            }
        }
    }
}

fn random_point(region: &RenderRegion) -> (i32, i32) {
    // 空区域防御：退化尺寸下返回原点，由 draw_line 的边界检查拦截
    if region.width == 0 || region.height == 0 {
        return (0, 0);
    }
    let mut rng = rand::thread_rng();
    (
        rng.gen_range(0..region.width as i32),
        rng.gen_range(0..region.height as i32),
    )
}

/// 空心线 (两条平行线)
pub struct LineHollow {
    pub count: u32,
    pub color: ColorPolicy,
}

impl Interference for LineHollow {
    fn apply(&self, image: &mut RgbaImage, region: &RenderRegion) {
        for _ in 0..self.count {
            let color = self.color.generate(region);
            let (x0, y0) = random_point(region);
            let (x1, y1) = random_point(region);
            let offset = 3;
            draw_line(image, &[(x0, y0), (x1, y1)], color);
            draw_line(image, &[(x0, y0 + offset), (x1, y1 + offset)], color);
        }
    }
}

/// 细线
pub struct LineSlime {
    pub count: u32,
    pub color: ColorPolicy,
}

impl Interference for LineSlime {
    fn apply(&self, image: &mut RgbaImage, region: &RenderRegion) {
        for _ in 0..self.count {
            let color = self.color.generate(region);
            let (x0, y0) = random_point(region);
            let (x1, y1) = random_point(region);
            draw_line(image, &[(x0, y0), (x1, y1)], color);
        }
    }
}

/// 正弦曲线
pub struct LineSine {
    pub count: u32,
    pub color: ColorPolicy,
    pub amplitude: f32,
    pub frequency: f32,
}

impl Default for LineSine {
    fn default() -> Self {
        Self {
            count: 1,
            color: ColorPolicy::Dark,
            amplitude: 5.0,
            frequency: 0.1,
        }
    }
}

impl Interference for LineSine {
    fn apply(&self, image: &mut RgbaImage, region: &RenderRegion) {
        if region.width == 0 || region.height == 0 {
            return;
        }
        let mut rng = rand::thread_rng();
        // 振幅与频率直接使用用户配置的字段 (参数化可调)
        let amp = self.amplitude;
        let freq = self.frequency;
        for _ in 0..self.count {
            let color = self.color.generate(region);
            let y_base = rng.gen_range(0..region.height as i32);
            let points: Vec<(i32, i32)> = (0..region.width as i32)
                .map(|x| {
                    let y = y_base as f32 + amp * (x as f32 * freq).sin();
                    (x, y as i32)
                })
                .collect();
            draw_line(image, &points, color);
        }
    }
}

/// 贝塞尔曲线
pub struct LineBezier {
    pub count: u32,
    pub color: ColorPolicy,
}

impl Interference for LineBezier {
    fn apply(&self, image: &mut RgbaImage, region: &RenderRegion) {
        for _ in 0..self.count {
            let color = self.color.generate(region);
            let p0 = random_point(region);
            let p1 = random_point(region);
            let cp = random_point(region); // 控制点
            let steps = 60;
            let points: Vec<(i32, i32)> = (0..=steps)
                .map(|i| {
                    let t = i as f32 / steps as f32;
                    let x = (1.0 - t).powi(2) * p0.0 as f32
                        + 2.0 * (1.0 - t) * t * cp.0 as f32
                        + t.powi(2) * p1.0 as f32;
                    let y = (1.0 - t).powi(2) * p0.1 as f32
                        + 2.0 * (1.0 - t) * t * cp.1 as f32
                        + t.powi(2) * p1.1 as f32;
                    (x as i32, y as i32)
                })
                .collect();
            draw_line(image, &points, color);
        }
    }
}

/// 圆弧
pub struct LineArc {
    pub count: u32,
    pub color: ColorPolicy,
}

impl Interference for LineArc {
    fn apply(&self, image: &mut RgbaImage, region: &RenderRegion) {
        if region.width == 0 || region.height == 0 {
            return;
        }
        let mut rng = rand::thread_rng();
        for _ in 0..self.count {
            let color = self.color.generate(region);
            let cx = rng.gen_range(0..region.width as i32);
            let cy = rng.gen_range(0..region.height as i32);
            let radius = rng.gen_range(20..=80) as f32;
            let start = rng.gen::<f32>() * std::f32::consts::PI;
            let end = start + rng.gen_range(std::f32::consts::PI..=std::f32::consts::TAU);
            let steps = 50;
            let points: Vec<(i32, i32)> = (0..=steps)
                .map(|i| {
                    let t = start + (end - start) * (i as f32 / steps as f32);
                    let x = cx as f32 + radius * t.cos();
                    let y = cy as f32 + radius * t.sin();
                    (x as i32, y as i32)
                })
                .collect();
            draw_line(image, &points, color);
        }
    }
}

/// 多段折线
pub struct LinePolyline {
    pub count: u32,
    pub color: ColorPolicy,
    pub segments: u32,
}

impl Default for LinePolyline {
    fn default() -> Self {
        Self {
            count: 1,
            color: ColorPolicy::Dark,
            segments: 5,
        }
    }
}

impl Interference for LinePolyline {
    fn apply(&self, image: &mut RgbaImage, region: &RenderRegion) {
        for _ in 0..self.count {
            let color = self.color.generate(region);
            let points: Vec<(i32, i32)> =
                (0..self.segments).map(|_| random_point(region)).collect();
            draw_line(image, &points, color);
        }
    }
}

/// 螺旋线
pub struct LineSpiral {
    pub count: u32,
    pub color: ColorPolicy,
}

impl Interference for LineSpiral {
    fn apply(&self, image: &mut RgbaImage, region: &RenderRegion) {
        if region.width == 0 || region.height == 0 {
            return;
        }
        let mut rng = rand::thread_rng();
        for _ in 0..self.count {
            let color = self.color.generate(region);
            let cx = rng.gen_range(0..region.width as i32);
            let cy = rng.gen_range(0..region.height as i32);
            let max_r = rng.gen_range(20.0..=60.0) as f32;
            let steps = 100;
            let points: Vec<(i32, i32)> = (0..=steps)
                .map(|i| {
                    let t = i as f32 / steps as f32;
                    let angle = t * std::f32::consts::TAU * 2.0;
                    let r = max_r * t;
                    let x = cx as f32 + r * angle.cos();
                    let y = cy as f32 + r * angle.sin();
                    (x as i32, y as i32)
                })
                .collect();
            draw_line(image, &points, color);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sine_uses_configured_amplitude() {
        // 振幅字段被消费：amplitude = 0 时曲线应为水平线
        let region = RenderRegion::new(100, 50);
        let mut img = RgbaImage::from_pixel(100, 50, Rgba([220, 220, 220, 255]));
        LineSine {
            count: 1,
            color: ColorPolicy::Dark,
            amplitude: 0.0,
            frequency: 0.1,
        }
        .apply(&mut img, &region);
        let dark_rows: Vec<u32> = (0..50u32)
            .filter(|&y| (0..100u32).any(|x| img.get_pixel(x, y)[0] < 150))
            .collect();
        // 水平线只应占据一行 (抗锯齿最多相邻两行)
        assert!(
            dark_rows.len() <= 2,
            "flat sine should be horizontal: {:?}",
            dark_rows
        );
    }
}
