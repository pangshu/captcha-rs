use super::api::blend_at;
use super::{Interference, RenderRegion};
use image::{Rgba, RgbaImage};
use rand::Rng;

/// 网格线纹理
pub struct TextureGrid {
    pub spacing: u32,
    pub color: Rgba<u8>,
}

impl Default for TextureGrid {
    fn default() -> Self {
        Self {
            spacing: 10,
            color: Rgba([180, 180, 180, 40]),
        }
    }
}

impl Interference for TextureGrid {
    fn apply(&self, image: &mut RgbaImage, _region: &RenderRegion) {
        let (w, h) = image.dimensions();
        if w == 0 || h == 0 {
            return;
        }
        // 参数边界收敛：spacing 为 0 时 step_by(0) 会 panic，钳制到最小 1
        let spacing = self.spacing.max(1) as usize;
        // 水平线
        for y in (0..h).step_by(spacing) {
            for x in 0..w {
                blend_at(image, x, y, self.color);
            }
        }
        // 垂直线
        for x in (0..w).step_by(spacing) {
            for y in 0..h {
                blend_at(image, x, y, self.color);
            }
        }
    }
}

/// 点阵纹理
pub struct TextureDotMatrix {
    pub spacing: u32,
    pub color: Rgba<u8>,
}

impl Default for TextureDotMatrix {
    fn default() -> Self {
        Self {
            spacing: 8,
            color: Rgba([160, 160, 160, 50]),
        }
    }
}

impl Interference for TextureDotMatrix {
    fn apply(&self, image: &mut RgbaImage, _region: &RenderRegion) {
        let (w, h) = image.dimensions();
        if w == 0 || h == 0 {
            return;
        }
        // 参数边界收敛：spacing 为 0 时 step_by(0) 会 panic，钳制到最小 1
        let spacing = self.spacing.max(1) as usize;
        for y in (0..h).step_by(spacing) {
            for x in (0..w).step_by(spacing) {
                blend_at(image, x, y, self.color);
            }
        }
    }
}

/// 柏林噪声纹理 (简化实现)
pub struct TexturePerlinNoise {
    pub scale: f32,
    pub strength: f32,
}

impl Default for TexturePerlinNoise {
    fn default() -> Self {
        Self {
            scale: 0.5,
            strength: 20.0,
        }
    }
}

impl Interference for TexturePerlinNoise {
    fn apply(&self, image: &mut RgbaImage, _region: &RenderRegion) {
        let mut rng = rand::thread_rng();
        let (w, h) = image.dimensions();
        if w == 0 || h == 0 {
            return;
        }
        // 参数边界收敛：scale <= 0 时退化为最大网格，避免除零/无穷
        let scale = if self.scale > 0.0 { self.scale } else { 0.5 };
        // 简化的伪随机噪声纹理
        let grid_size = (20.0 / scale).max(4.0) as u32;
        let mut grid: Vec<Vec<f32>> = Vec::new();
        let gw = (w / grid_size + 2) as usize;
        let gh = (h / grid_size + 2) as usize;
        for _ in 0..gw {
            let row: Vec<f32> = (0..gh).map(|_| rng.gen::<f32>()).collect();
            grid.push(row);
        }
        for y in 0..h {
            for x in 0..w {
                let gx = x / grid_size;
                let gy = y / grid_size;
                let fx = (x % grid_size) as f32 / grid_size as f32;
                let fy = (y % grid_size) as f32 / grid_size as f32;
                // 双线性插值
                let v00 = grid[gx as usize][gy as usize];
                let v10 = grid[(gx + 1) as usize][gy as usize];
                let v01 = grid[gx as usize][(gy + 1) as usize];
                let v11 = grid[(gx + 1) as usize][(gy + 1) as usize];
                let v = v00 * (1.0 - fx) * (1.0 - fy)
                    + v10 * fx * (1.0 - fy)
                    + v01 * (1.0 - fx) * fy
                    + v11 * fx * fy;
                let noise = (v * self.strength) as i32;
                let p = image.get_pixel(x, y);
                let r = (p[0] as i32 + noise).clamp(0, 255) as u8;
                let g = (p[1] as i32 + noise).clamp(0, 255) as u8;
                let b = (p[2] as i32 + noise).clamp(0, 255) as u8;
                image.put_pixel(x, y, Rgba([r, g, b, 255]));
            }
        }
    }
}

/// 扫描线纹理
pub struct TextureScanline {
    pub spacing: u32,
    pub alpha: u8,
}

impl Default for TextureScanline {
    fn default() -> Self {
        Self {
            spacing: 3,
            alpha: 30,
        }
    }
}

impl Interference for TextureScanline {
    fn apply(&self, image: &mut RgbaImage, _region: &RenderRegion) {
        let (w, h) = image.dimensions();
        if w == 0 || h == 0 {
            return;
        }
        // 参数边界收敛：spacing 为 0 时 step_by(0) 会 panic，钳制到最小 1
        let spacing = self.spacing.max(1) as usize;
        let color = Rgba([0, 0, 0, self.alpha]);
        for y in (0..h).step_by(spacing) {
            for x in 0..w {
                blend_at(image, x, y, color);
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
        // spacing = 0 (step_by(0) 会 panic)
        TextureGrid {
            spacing: 0,
            color: Rgba([180, 180, 180, 40]),
        }
        .apply(&mut test_image(), &region);
        TextureScanline {
            spacing: 0,
            alpha: 30,
        }
        .apply(&mut test_image(), &region);
        // scale = 0 (除零/无穷)
        TexturePerlinNoise {
            scale: 0.0,
            strength: 20.0,
        }
        .apply(&mut test_image(), &region);
    }

    fn test_image() -> RgbaImage {
        RgbaImage::from_pixel(40, 30, Rgba([220, 220, 220, 255]))
    }
}
