use crate::error::{CaptchaError, Result};
use ab_glyph::{Font, FontArc, FontVec};
use rand::Rng;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// 默认字体全局缓存，进程生命周期内只加载一次
static DEFAULT_FONTS: OnceLock<FontLoader> = OnceLock::new();

/// 获取默认字体的全局缓存引用
///
/// 首次调用时从 fonts/ 目录加载字体文件，后续调用直接返回缓存。
/// 字体文件通常较大（NotoSansSC 17MB），避免每次生成验证码时重复 IO。
pub fn get_default_fonts() -> &'static FontLoader {
    DEFAULT_FONTS.get_or_init(FontLoader::default_fonts)
}

/// 从文件数据加载字体
fn load_font_from_data(data: Vec<u8>, name: &str) -> Result<FontArc> {
    let font_vec = FontVec::try_from_vec(data).map_err(|e| CaptchaError::FontLoadError {
        name: name.to_string(),
        reason: format!("failed to parse font: {}", e),
    })?;
    Ok(FontArc::from(font_vec))
}

/// 字体加载器
///
/// 运行时加载 TrueType/OpenType 字体文件 (.ttf/.ttc)，
/// 支持字形覆盖检查，为每个字符找到可渲染的字体。
#[derive(Debug, Default, Clone)]
pub struct FontLoader {
    /// 已加载的字体列表
    fonts: Vec<FontArc>,
    /// 字体来源路径 (用于错误信息)
    paths: Vec<PathBuf>,
}

impl FontLoader {
    /// 创建空的字体加载器
    pub fn new() -> Self {
        Self::default()
    }

    /// 从指定路径列表加载字体
    ///
    /// 自动跳过无法加载的字体文件并记录日志。
    pub fn from_paths(paths: &[&str]) -> Result<Self> {
        let mut loader = Self::new();
        for path_str in paths {
            let path = PathBuf::from(path_str);
            match std::fs::read(&path) {
                Ok(data) => match load_font_from_data(data, path_str) {
                    Ok(font) => {
                        log::debug!("loaded font: {}", path.display());
                        loader.fonts.push(font);
                        loader.paths.push(path);
                    }
                    Err(e) => {
                        log::warn!("failed to parse font '{}': {}", path.display(), e);
                    }
                },
                Err(e) => {
                    log::warn!("failed to read font '{}': {}", path.display(), e);
                }
            }
        }
        if loader.fonts.is_empty() {
            return Err(CaptchaError::FontLoadError {
                name: paths.join(", "),
                reason: "no fonts could be loaded from the provided paths".to_string(),
            });
        }
        Ok(loader)
    }

    /// 从指定目录扫描所有 .ttf/.ttc 文件
    pub fn from_dir(dir: &str) -> Result<Self> {
        let dir_path = Path::new(dir);
        if !dir_path.is_dir() {
            return Err(CaptchaError::FontLoadError {
                name: dir.to_string(),
                reason: "not a directory".to_string(),
            });
        }

        let mut font_paths: Vec<PathBuf> = std::fs::read_dir(dir_path)
            .map_err(|e| CaptchaError::FontLoadError {
                name: dir.to_string(),
                reason: format!("failed to read directory: {}", e),
            })?
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                if let Some(ext) = path.extension() {
                    if ext == "ttf" || ext == "ttc" {
                        return Some(path);
                    }
                }
                None
            })
            .collect();

        font_paths.sort();

        if font_paths.is_empty() {
            return Err(CaptchaError::FontLoadError {
                name: dir.to_string(),
                reason: "no .ttf/.ttc files found in directory".to_string(),
            });
        }

        let paths_str: Vec<&str> = font_paths
            .iter()
            .map(|p| p.to_str().unwrap_or(""))
            .collect();
        Self::from_paths(&paths_str)
    }

    /// 尝试加载系统默认字体
    ///
    /// 按平台搜索常见的系统字体路径。
    pub fn system() -> Result<Self> {
        let candidates: &[&str] = if cfg!(target_os = "macos") {
            &[
                "/System/Library/Fonts/Helvetica.ttc",
                "/System/Library/Fonts/Supplemental/Arial.ttf",
                "/Library/Fonts/Arial Unicode.ttf",
            ]
        } else if cfg!(target_os = "linux") {
            &[
                "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
                "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
                "/usr/share/fonts/wqy-microhei/wqy-microhei.ttc",
            ]
        } else if cfg!(target_os = "windows") {
            &[
                "C:\\Windows\\Fonts\\arial.ttf",
                "C:\\Windows\\Fonts\\msyh.ttc",
            ]
        } else {
            &[]
        };

        let mut loader = Self::new();
        for path in candidates {
            if let Ok(data) = std::fs::read(path) {
                if let Ok(font) = load_font_from_data(data, path) {
                    log::debug!("loaded system font: {}", path);
                    loader.fonts.push(font);
                    loader.paths.push(PathBuf::from(path));
                }
            }
        }

        if loader.fonts.is_empty() {
            Err(CaptchaError::FontLoadError {
                name: "system".to_string(),
                reason: "no system fonts could be loaded".to_string(),
            })
        } else {
            Ok(loader)
        }
    }

    /// 加载项目默认字体 (fonts/ 目录)
    ///
    /// 尝试从项目根目录的 fonts/ 子目录加载字体。
    /// 如果当前工作目录下没有 fonts/ 目录，返回空加载器。
    pub fn default_fonts() -> Self {
        let mut loader = Self::new();
        let default_paths: &[&str] = &["fonts/DejaVuSans.ttf", "fonts/NotoSansSC-Regular.ttf"];

        for path_str in default_paths {
            if let Ok(data) = std::fs::read(path_str) {
                if let Ok(font) = load_font_from_data(data, path_str) {
                    loader.fonts.push(font);
                    loader.paths.push(PathBuf::from(path_str));
                }
            }
        }

        loader
    }

    /// 添加一个已加载的字体
    pub fn add_font(&mut self, font: FontArc) {
        self.fonts.push(font);
    }

    /// 获取所有已加载的字体
    pub fn fonts(&self) -> &[FontArc] {
        &self.fonts
    }

    /// 获取字体数量
    pub fn len(&self) -> usize {
        self.fonts.len()
    }

    /// 是否没有加载任何字体
    pub fn is_empty(&self) -> bool {
        self.fonts.is_empty()
    }

    /// 随机获取一个字体
    pub fn random_font(&self) -> Option<&FontArc> {
        if self.fonts.is_empty() {
            return None;
        }
        let mut rng = rand::thread_rng();
        let idx = rng.gen_range(0..self.fonts.len());
        Some(&self.fonts[idx])
    }

    /// 获取能渲染指定字符的字体 (字形覆盖检查)
    ///
    /// 遍历所有字体，返回第一个包含该字符字形的字体。
    /// 如果没有字体能渲染该字符，返回 None。
    pub fn font_for_char(&self, c: char) -> Option<&FontArc> {
        for font in &self.fonts {
            let glyph_id = font.glyph_id(c);
            if glyph_id.0 != 0 {
                return Some(font);
            }
        }
        None
    }

    /// 获取能渲染指定字符的所有字体
    pub fn fonts_for_char(&self, c: char) -> Vec<&FontArc> {
        self.fonts
            .iter()
            .filter(|font| font.glyph_id(c).0 != 0)
            .collect()
    }
}
