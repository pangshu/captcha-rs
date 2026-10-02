use std::path::PathBuf;

// ---- MIME 类型 ----

/// 验证码图像 MIME 类型
pub const MIME_TYPE_IMAGE: &str = "data:image/png;base64";

/// 验证码音频 MIME 类型
pub const MIME_TYPE_AUDIO: &str = "data:audio/wav;base64";

// ---- 字符集 ----

/// 数字字符集
pub const TXT_NUMBERS: &str = "0123456789";

/// 英文字母字符集 (大小写)
pub const TXT_ALPHABET: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

/// 简单字符集 (去除了易混淆字符)
pub const TXT_SIMPLE_CHARS: &str = "2345678abcdefhkmnprstuvwxyzABCDEFGHJKLMNPRSTUVWXYZ";

/// 中文字符集 (常用汉字)
pub const TXT_CHINESE_CHARS: &str = "的一是在不了有和人这中大为上个国我以要他时来用们生到作地于出就分对成会可主发年动同工也能下过子说产种面而方后多定行学法所民得经十三之进着等部度家电力里如水化高自二理起小物现实加量都两体制机当使点从业本去把性好应开它合还因由其些然前外天政四日那社义事平形相全表间样与关各重新线内数正心反你明看原又么利比或但质气第向道命此变条只没结解问意建月公无系军很情者最立代想已通并提直题党程展五果料象员革位入常文总次品式活设及管特件长求老头基资边流路级少图山统接知较将组见计别她手角期根论运农指几九区强放决西被干做必战先回则任取据处队南给色光门即保治北造百规热领七海口东导器压志世金增争济阶油思术极交受联什认六共权收证改清己美再采转更单风切打白教速花带安场身车例真务具万每目至达走积示议声报斗完类八离华名确才科张信马节话米整空元况今集温传土许步群广石记需段研界拉林律叫且究观越织装影算低持音众书布复容儿须际商非验连断深难近矿千周委素技备半办青省列习响约支般史感劳便团往酸历市克何除消构府称太准精值号率族维划选标写存候毛亲快效斯院查江型眼王按格养易置派层片始却专状育厂京识适属圆包火住调满县局照参红细引听该铁价严";

/// 数学运算符
pub const MATH_OPERATORS: &[&str] = &["+", "-", "×"];

// ---- 图像默认值 ----

/// 图像默认宽度
pub const DEFAULT_IMG_WIDTH: u32 = 240;

/// 图像默认高度
pub const DEFAULT_IMG_HEIGHT: u32 = 80;

/// 默认验证码长度
pub const DEFAULT_CAPTCHA_LENGTH: usize = 4;

/// 默认字符重叠比例
pub const DEFAULT_CHAR_OVERLAP: f32 = 0.1;

/// 默认字符间距抖动比例
pub const DEFAULT_CHAR_SPACING_JITTER: f32 = 0.2;

/// 默认字体大小抖动比例
pub const DEFAULT_FONT_SIZE_JITTER: f32 = 0.15;

// ---- 存储默认值 ----

/// 存储默认过期时间 (秒)
pub const DEFAULT_EXPIRATION_SECS: u64 = 600;

/// 存储默认最大条目数 (moka 容量上限，超出后近 LRU 驱逐)
pub const DEFAULT_MAX_ENTRIES: usize = 10240;

/// 最大验证尝试次数
pub const DEFAULT_MAX_ATTEMPTS: u32 = 5;

// ---- 滑块默认值 ----

/// 滑块验证码默认宽度
pub const DEFAULT_SLIDER_WIDTH: u32 = 300;

/// 滑块验证码默认高度
pub const DEFAULT_SLIDER_HEIGHT: u32 = 150;

/// 滑块拼图块默认大小
pub const DEFAULT_SLIDER_PIECE_SIZE: u32 = 44;

// ---- 点选默认值 ----

/// 点选验证码默认宽度
pub const DEFAULT_CLICK_WIDTH: u32 = 300;

/// 点选验证码默认高度
pub const DEFAULT_CLICK_HEIGHT: u32 = 150;

/// 点选目标字默认数量
pub const DEFAULT_CLICK_COUNT: usize = 4;

// ---- 难度级别 ----

/// 难度级别
///
/// 难度级别是预置的干扰器组合 (见 `interference::preset_interferences`)，
/// 同时决定字符倾斜幅度与噪点数量；用户也可通过 interferences 字段完全自定义。
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Difficulty {
    /// 简单：噪点30 + 细线1条 + 倾斜0.3
    Easy,
    /// 中等：噪点60 + 细线2条 + 正弦线1条 + 倾斜0.5
    Medium,
    /// 困难：噪点100 + 空心线2条 + 正弦线1条 + 贝塞尔线1条 + 波浪扭曲 + RGB通道偏移 + 倾斜0.8
    Hard,
}

impl Default for Difficulty {
    fn default() -> Self {
        Self::Medium
    }
}

impl Difficulty {
    /// 根据难度级别返回默认噪点数量
    pub fn default_noise_count(self) -> u32 {
        match self {
            Self::Easy => 30,
            Self::Medium => 60,
            Self::Hard => 100,
        }
    }

    /// 根据难度级别返回默认最大倾斜
    pub fn default_max_skew(self) -> f64 {
        match self {
            Self::Easy => 0.3,
            Self::Medium => 0.5,
            Self::Hard => 0.8,
        }
    }
}

// ---- 背景类型 ----

/// 背景类型
#[derive(Debug, Clone, Default)]
pub enum Background {
    /// 随机浅色纯色 (默认)
    #[default]
    Solid,
    /// 渐变背景 (指定颜色列表，空则随机)
    Gradient { colors: Vec<[u8; 3]> },
    /// 从图片路径列表随机选取背景图
    Pattern(Vec<PathBuf>),
}
