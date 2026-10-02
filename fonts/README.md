# 默认字体目录

`crate::font::FontLoader::default_fonts()` 启动时会尝试从本目录加载以下字体文件：

| 文件名 | 用途 | 获取方式 |
| --- | --- | --- |
| `DejaVuSans.ttf` | 数字 / 拉丁字母渲染 | https://dejavu-fonts.github.io/ （DejaVu Sans 发行包内 `ttf/DejaVuSans.ttf`） |
| `NotoSansSC-Regular.ttf` | 中文 / 成语 / 点选目标字渲染 | https://fonts.google.com/noto/specimen/Noto+Sans+SC （下载 Static TTF） |

说明：

- 两个文件均为缺失时跳过（记录 warn 日志），全部缺失时 `ImageBuilder::generate()`
  会返回 `FontLoadError`，提示通过 `font_paths` 指定自定义字体。
- 中文字符（`TXT_CHINESE_CHARS`、成语、点选目标字）必须依赖含 CJK 字形的字体
  （如 NotoSansSC），仅 DejaVu 无法渲染汉字。
- 字体文件较大（NotoSansSC 约 17MB），默认字体经 `OnceLock` 全局缓存，
  进程生命周期内只加载一次。
