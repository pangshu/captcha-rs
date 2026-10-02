use super::region::RenderRegion;
use crate::config::Difficulty;
use image::Rgba;
use rand::Rng;

/// 颜色策略
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorPolicy {
    /// 随机颜色
    Random,
    /// 深色 (适合浅色背景)
    Dark,
    /// 从文字颜色中采样 (前景背景融合)
    TextSample,
    /// 接近背景色 (低对比度干扰)
    BackgroundNear,
}

impl ColorPolicy {
    /// 根据策略生成颜色
    pub fn generate(self, region: &RenderRegion) -> Rgba<u8> {
        let mut rng = rand::thread_rng();
        match self {
            ColorPolicy::Random => crate::util::random_color(),
            ColorPolicy::Dark => crate::util::random_dark_color(),
            ColorPolicy::TextSample => {
                if region.text_colors.is_empty() {
                    crate::util::random_dark_color()
                } else {
                    let idx = rng.gen_range(0..region.text_colors.len());
                    let base = region.text_colors[idx];
                    crate::util::color_near(base, 40)
                }
            }
            ColorPolicy::BackgroundNear => Rgba([
                rng.gen_range(180..=255),
                rng.gen_range(180..=255),
                rng.gen_range(180..=255),
                rng.gen_range(30..=80),
            ]),
        }
    }
}

/// 按难度级别返回预置干扰器组合
///
/// 与 `Difficulty` 文档注释中的组合定义一一对应，
/// 用户未显式指定 interferences 时由 Builder 自动装配。
pub fn preset_interferences(difficulty: Difficulty) -> Vec<Box<dyn super::Interference>> {
    use super::color::ColorChannelShift;
    use super::distortion::DistortionWave;
    use super::line::{LineBezier, LineHollow, LineSine, LineSlime};
    use super::noise::NoisePixel;

    let mut list: Vec<Box<dyn super::Interference>> = vec![Box::new(NoisePixel {
        count: difficulty.default_noise_count(),
    })];

    match difficulty {
        Difficulty::Easy => {
            list.push(Box::new(LineSlime {
                count: 1,
                color: ColorPolicy::Dark,
            }));
        }
        Difficulty::Medium => {
            list.push(Box::new(LineSlime {
                count: 2,
                color: ColorPolicy::Dark,
            }));
            list.push(Box::new(LineSine::default()));
        }
        Difficulty::Hard => {
            list.push(Box::new(LineHollow {
                count: 2,
                color: ColorPolicy::Dark,
            }));
            list.push(Box::new(LineSine::default()));
            list.push(Box::new(LineBezier {
                count: 1,
                color: ColorPolicy::Dark,
            }));
            list.push(Box::new(DistortionWave::default()));
            list.push(Box::new(ColorChannelShift::default()));
        }
    }
    list
}
