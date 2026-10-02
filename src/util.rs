use rand::seq::SliceRandom;
use rand::Rng;

/// 生成随机验证码 ID
///
/// 格式: 8位十六进制时间戳 + 20位随机字符 = 28字符
/// 时间戳前缀大幅降低高并发下的碰撞概率
pub fn random_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;
    let ts_hex = format!("{:08x}", ts & 0xFFFFFFFF);

    let mut rng = rand::thread_rng();
    let random_part: String = (0..20)
        .map(|_| {
            let idx = rng.gen_range(0..62);
            if idx < 10 {
                (b'0' + idx) as char
            } else if idx < 36 {
                (b'a' + (idx - 10)) as char
            } else {
                (b'A' + (idx - 36)) as char
            }
        })
        .collect();
    format!("{}{}", ts_hex, random_part)
}

/// 从字符串中随机选取 n 个字符
pub fn random_chars(source: &str, count: usize) -> String {
    if source.is_empty() || count == 0 {
        return String::new();
    }
    let mut rng = rand::thread_rng();
    let chars: Vec<char> = source.chars().collect();
    (0..count)
        .map(|_| chars[rng.gen_range(0..chars.len())])
        .collect()
}

/// 从字符池洗牌后不放回地选取两组互不重复的字符
///
/// 返回 (第一组, 第二组)，组内与组间均无重复字符。
/// 点选验证码依赖此性质保证目标字与干扰字可区分。
/// 池中字符数不足时返回 None。
pub fn random_unique_char_groups(
    source: &str,
    first: usize,
    second: usize,
) -> Option<(String, String)> {
    let mut pool: Vec<char> = source.chars().collect();
    if first + second > pool.len() {
        return None;
    }
    let mut rng = rand::thread_rng();
    pool.shuffle(&mut rng);
    let g1: String = pool[..first].iter().collect();
    let g2: String = pool[first..first + second].iter().collect();
    Some((g1, g2))
}

/// 生成随机角度 (弧度, 用于字符倾斜)
pub fn random_angle(max_skew: f64) -> f64 {
    let mut rng = rand::thread_rng();
    (rng.gen::<f64>() - 0.5) * 2.0 * max_skew
}

/// 生成随机 f32 范围 [min, max]
pub fn random_range_f32(min: f32, max: f32) -> f32 {
    let mut rng = rand::thread_rng();
    rng.gen_range(min..=max)
}

/// 生成随机 u32 范围 [min, max]
pub fn random_range_u32(min: u32, max: u32) -> u32 {
    let mut rng = rand::thread_rng();
    rng.gen_range(min..=max)
}

// ---- 图像相关工具函数 (仅 image/interactive feature) ----

#[cfg(any(feature = "image", feature = "interactive"))]
pub use image_dependent::*;

#[cfg(any(feature = "image", feature = "interactive"))]
mod image_dependent {
    use image::Rgba;
    use rand::Rng;

    /// 生成随机颜色 (RGB)
    pub fn random_color() -> Rgba<u8> {
        let mut rng = rand::thread_rng();
        Rgba([
            rng.gen_range(0..=255),
            rng.gen_range(0..=255),
            rng.gen_range(0..=255),
            255,
        ])
    }

    /// 生成随机浅色背景
    pub fn random_light_color() -> Rgba<u8> {
        let mut rng = rand::thread_rng();
        Rgba([
            rng.gen_range(200..=255),
            rng.gen_range(200..=255),
            rng.gen_range(200..=255),
            255,
        ])
    }

    /// 生成随机深色 (用于文字)
    pub fn random_dark_color() -> Rgba<u8> {
        let mut rng = rand::thread_rng();
        Rgba([
            rng.gen_range(0..=100),
            rng.gen_range(0..=100),
            rng.gen_range(0..=100),
            255,
        ])
    }

    /// 从基准颜色生成近似颜色 (用于前景背景融合)
    pub fn color_near(base: Rgba<u8>, variance: u8) -> Rgba<u8> {
        let mut rng = rand::thread_rng();
        let clamp = |v: i32| -> u8 { v.clamp(0, 255) as u8 };
        Rgba([
            clamp(base[0] as i32 + rng.gen_range(-(variance as i32)..=variance as i32)),
            clamp(base[1] as i32 + rng.gen_range(-(variance as i32)..=variance as i32)),
            clamp(base[2] as i32 + rng.gen_range(-(variance as i32)..=variance as i32)),
            255,
        ])
    }

    /// alpha 混合两个颜色
    pub fn blend(base: Rgba<u8>, overlay: Rgba<u8>) -> Rgba<u8> {
        let alpha = overlay[3] as f32 / 255.0;
        let inv = 1.0 - alpha;
        Rgba([
            (base[0] as f32 * inv + overlay[0] as f32 * alpha) as u8,
            (base[1] as f32 * inv + overlay[1] as f32 * alpha) as u8,
            (base[2] as f32 * inv + overlay[2] as f32 * alpha) as u8,
            255,
        ])
    }
}
