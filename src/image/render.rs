use crate::config::Background;
use crate::error::{CaptchaError, Result};
use crate::font::FontLoader;
use crate::interference::{Rect, RenderRegion};
use ab_glyph::{Font, FontArc, PxScale, ScaleFont};
use image::{Rgba, RgbaImage};
use rand::Rng;
use std::path::Path;

/// 渲染结果
pub struct RenderResult {
    pub image: RgbaImage,
    pub region: RenderRegion,
}

/// 绘制背景
pub fn draw_background(img: &mut RgbaImage, bg: &Background) {
    let (w, h) = img.dimensions();
    match bg {
        Background::Solid => {
            let color = crate::util::random_light_color();
            // 使用 from_pixel 批量填充，避免逐像素 put_pixel
            *img = RgbaImage::from_pixel(w, h, color);
        }
        Background::Gradient { colors } => {
            let mut rng = rand::thread_rng();
            let c0 = if colors.is_empty() {
                Rgba([
                    rng.gen_range(200..=255),
                    rng.gen_range(200..=255),
                    rng.gen_range(200..=255),
                    255,
                ])
            } else {
                Rgba([colors[0][0], colors[0][1], colors[0][2], 255])
            };
            let c1 = if colors.len() < 2 {
                Rgba([
                    rng.gen_range(200..=255),
                    rng.gen_range(200..=255),
                    rng.gen_range(200..=255),
                    255,
                ])
            } else {
                Rgba([colors[1][0], colors[1][1], colors[1][2], 255])
            };
            for y in 0..h {
                let t = y as f32 / h.max(1) as f32;
                let r = (c0[0] as f32 * (1.0 - t) + c1[0] as f32 * t) as u8;
                let g = (c0[1] as f32 * (1.0 - t) + c1[1] as f32 * t) as u8;
                let b = (c0[2] as f32 * (1.0 - t) + c1[2] as f32 * t) as u8;
                for x in 0..w {
                    img.put_pixel(x, y, Rgba([r, g, b, 255]));
                }
            }
        }
        Background::Pattern(paths) => {
            if paths.is_empty() {
                *img = RgbaImage::from_pixel(w, h, crate::util::random_light_color());
            } else {
                let mut rng = rand::thread_rng();
                let path = &paths[rng.gen_range(0..paths.len())];
                match image::open(Path::new(path)) {
                    Ok(loaded) => {
                        // resize 已产出完整尺寸图像，直接 move 替换，避免逐像素拷贝
                        *img = image::imageops::resize(
                            &loaded.to_rgba8(),
                            w,
                            h,
                            image::imageops::FilterType::Lanczos3,
                        );
                    }
                    Err(_) => {
                        *img = RgbaImage::from_pixel(w, h, crate::util::random_light_color());
                    }
                }
            }
        }
    }
}

/// alpha 混合像素
fn blend_pixel(img: &mut RgbaImage, x: u32, y: u32, color: Rgba<u8>) {
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

/// 绘制单个斜体字符，返回字符边界框
fn draw_italic_char(
    img: &mut RgbaImage,
    font: &FontArc,
    ch: char,
    x: u32,
    y: u32,
    font_size: f32,
    color: Rgba<u8>,
    skew: f64,
) -> Option<Rect> {
    let scale = PxScale::from(font_size);
    let scaled_font = font.as_scaled(scale);
    let glyph_id = scaled_font.glyph_id(ch);
    if glyph_id.0 == 0 {
        return None;
    }
    let glyph = glyph_id.with_scale_and_position(scale, ab_glyph::point(x as f32, y as f32));
    let outlined = font.outline_glyph(glyph)?;
    let bounds = outlined.px_bounds();
    let skew_offset = (skew * font_size as f64) as i32;
    let min_x = bounds.min.x as i32;
    let min_y = bounds.min.y as i32;
    let height = bounds.height() as i32;

    // 追踪倾斜后的实际绘制范围，使返回的边界框与字形落点一致，
    // 供基于 char_bounds 的干扰器 (描边/阴影/渐变/断裂) 精确命中
    let (mut real_min_x, mut real_max_x) = (i32::MAX, i32::MIN);
    let (mut real_min_y, mut real_max_y) = (i32::MAX, i32::MIN);

    outlined.draw(|px, py, v| {
        let img_x = px as i32 + min_x;
        let img_y = py as i32 + min_y;
        let adjusted_x = img_x + (img_y - min_y) * skew_offset / height.max(1);
        if adjusted_x >= 0
            && adjusted_x < img.width() as i32
            && img_y >= 0
            && img_y < img.height() as i32
        {
            let alpha = (v * 255.0) as u8;
            if alpha > 0 {
                real_min_x = real_min_x.min(adjusted_x);
                real_max_x = real_max_x.max(adjusted_x);
                real_min_y = real_min_y.min(img_y);
                real_max_y = real_max_y.max(img_y);
                blend_pixel(
                    img,
                    adjusted_x as u32,
                    img_y as u32,
                    Rgba([color[0], color[1], color[2], alpha]),
                );
            }
        }
    });

    // 优先返回倾斜后的实际绘制包围盒；无可见像素时退回原始字形包围盒
    if real_min_x <= real_max_x {
        Some(Rect {
            x: real_min_x as u32,
            y: real_min_y as u32,
            w: (real_max_x - real_min_x + 1) as u32,
            h: (real_max_y - real_min_y + 1) as u32,
        })
    } else {
        Some(Rect {
            x: bounds.min.x as u32,
            y: bounds.min.y as u32,
            w: bounds.width() as u32,
            h: bounds.height() as u32,
        })
    }
}

/// 在图像上绘制文本，返回渲染区域信息
pub fn draw_text(
    img: &mut RgbaImage,
    text: &str,
    fonts: &FontLoader,
    max_skew: f64,
    char_overlap: f32,
    char_spacing_jitter: f32,
) -> Result<RenderRegion> {
    if fonts.is_empty() {
        return Err(CaptchaError::DrawError("no fonts available".into()));
    }

    let mut rng = rand::thread_rng();
    let (img_width, img_height) = img.dimensions();
    let char_count = text.chars().count();
    if char_count == 0 {
        return Ok(RenderRegion::new(img_width, img_height));
    }

    let padding = 10;
    let available_width = (img_width as i32 - padding * 2).max(1) as u32;
    let base_char_width = available_width / char_count as u32;
    // 基准高度 0.55 (项目规范)，受可用宽度约束；下限 8px 防止退化尺寸下零尺寸字形
    let font_size = (img_height as f32 * 0.55)
        .min(base_char_width as f32 * 0.9)
        .max(8.0);

    let mut char_bounds = Vec::new();
    let mut text_colors = Vec::new();
    let overlap_px = (font_size * char_overlap) as i32;

    for (i, ch) in text.chars().enumerate() {
        // 空格是排版占位字符，无字形轮廓属正常，跳过绘制不报错
        if ch == ' ' {
            continue;
        }
        // 其余任一字符无可用字形即报错：
        // 静默跳过会生成"可见字符数少于答案长度"的不可解验证码
        let font = fonts.font_for_char(ch).ok_or_else(|| {
            CaptchaError::DrawError(format!("no font contains a glyph for character '{}'", ch))
        })?;
        let color = crate::util::random_dark_color();
        let skew = crate::util::random_angle(max_skew);
        let jitter = (rng.gen::<f32>() - 0.5) * 2.0 * char_spacing_jitter * base_char_width as f32;
        let x = padding as f32 + i as f32 * base_char_width as f32 + base_char_width as f32 / 4.0
            - overlap_px as f32 * i as f32
            + jitter;
        let y = img_height as f32 / 2.0 + font_size * 0.3;
        let rect = draw_italic_char(img, font, ch, x as u32, y as u32, font_size, color, skew)
            .ok_or_else(|| {
                CaptchaError::DrawError(format!("failed to render character '{}'", ch))
            })?;
        char_bounds.push(rect);
        text_colors.push(color);
    }

    Ok(RenderRegion {
        width: img_width,
        height: img_height,
        char_bounds,
        text_colors,
    })
}

/// 完整渲染管线：背景 + 文字 + 干扰器链
pub fn render_image(
    config: &crate::image::config::ImageConfig,
    content: &str,
    fonts: &FontLoader,
) -> Result<RenderResult> {
    let mut img = RgbaImage::new(config.width, config.height);

    // 1. 绘制背景
    draw_background(&mut img, &config.background);

    // 2. 绘制文字
    let region = draw_text(
        &mut img,
        content,
        fonts,
        config.max_skew,
        config.char_overlap,
        config.char_spacing_jitter,
    )?;

    // 3. 执行干扰器链
    crate::interference::apply_all(&config.interferences, &mut img, &region);

    Ok(RenderResult { image: img, region })
}
