use serde::{Deserialize, Serialize};

/// 滑块容差回落值 (旧记录无 tolerance 字段时使用)
fn default_slider_tolerance() -> u32 {
    5
}

/// 验证码答案存储协议
///
/// 所有类型的验证码答案统一序列化为 JSON 字符串存储。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AnswerData {
    /// 图像/音频类答案 (字符串匹配，大小写不敏感)
    #[serde(rename = "text")]
    Text { answer: String },

    /// 滑块类答案 (x 坐标匹配，含容差)
    #[serde(rename = "slider")]
    Slider {
        x: u32,
        /// 验证容差 (px)，生成时按图像宽度比例计算；
        /// 旧记录无此字段时回落 5
        #[serde(default = "default_slider_tolerance")]
        tolerance: u32,
    },

    /// 点选类答案 (坐标区域匹配)
    #[serde(rename = "click")]
    Click { targets: Vec<ClickTarget> },
}

/// 点选目标区域
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClickTarget {
    pub char: String,
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

impl AnswerData {
    /// 从 JSON 字符串反序列化
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// 序列化为 JSON 字符串
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }

    /// 创建文本类答案
    pub fn text(answer: impl Into<String>) -> Self {
        Self::Text {
            answer: answer.into(),
        }
    }

    /// 创建滑块类答案 (默认容差 5px)
    pub fn slider(x: u32) -> Self {
        Self::Slider {
            x,
            tolerance: default_slider_tolerance(),
        }
    }

    /// 创建滑块类答案 (自定义容差，生成时按宽度比例计算)
    pub fn slider_with_tolerance(x: u32, tolerance: u32) -> Self {
        Self::Slider {
            x,
            tolerance: tolerance.max(1),
        }
    }

    /// 创建点选类答案
    pub fn click(targets: Vec<ClickTarget>) -> Self {
        Self::Click { targets }
    }
}
