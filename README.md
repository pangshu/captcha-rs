# captcha-rs

Rust 验证码统一引擎：图像 / 音频 / 滑块 / 文字点选四类验证码共用一套生成与验证 API。

## 目录结构

```text
captcha-rs/
├── Cargo.toml
├── fonts/                        # 默认字体目录 (见 fonts/README.md)
└── src/
    ├── lib.rs                    # 模块声明与 feature 门控
    ├── captcha.rs                # 验证码统一引擎 (生成入口 + 统一验证)
    ├── config.rs                 # 全局常量、Difficulty、Background
    ├── error.rs                  # CaptchaError 统一错误
    ├── font.rs                   # FontLoader 字体加载与字形覆盖检查
    ├── util.rs                   # 随机 ID / 随机字符 / 颜色工具
    ├── store/                    # 答案存储
    │   ├── backend.rs            #   Store trait (set/get/take/remove/attempts)
    │   ├── answer.rs             #   AnswerData 答案协议 (text/slider/click)
    │   └── memory.rs             #   MemoryStore (moka, TTL + 容量 + 原子 take)
    ├── audio/                    # 音频验证码 (feature = "audio")
    │   ├── builder.rs            #   AudioBuilder
    │   ├── config.rs             #   AudioConfig
    │   ├── sounds.rs             #   字符 → 频率映射
    │   ├── wav.rs                #   正弦波合成 + WAV 封装 (16bit/8kHz/mono)
    │   └── item.rs               #   AudioItem / AudioResult
    ├── image/                    # 图像验证码 (feature = "image")
    │   ├── driver.rs             #   DriverType + Driver trait + 工厂
    │   ├── digit.rs / string.rs / chinese.rs / language.rs / math.rs / idiom.rs
    │   ├── config.rs             #   ImageConfig + Builder
    │   ├── render.rs             #   背景 / 斜体字符 / 文本 / 完整渲染管线
    │   ├── builder.rs            #   ImageBuilder
    │   └── item.rs               #   ImageItem (PNG/base64) / ImageResult
    ├── interactive/              # 交互验证码 (feature = "interactive", 依赖 image)
    │   ├── slider.rs             #   滑块拼图 (生成缺口背景 + 拼图块)
    │   ├── click_text.rs         #   文字点选 (目标字 + 干扰字 + 命中区域)
    │   └── item.rs               #   SliderResult / ClickTextResult
    └── interference/             # 干扰器接口库 (image/interactive 共用)
        ├── api.rs                #   Interference trait + apply_all + blend_at
        ├── region.rs             #   Rect / RenderRegion
        ├── policy.rs             #   ColorPolicy + preset_interferences (按难度)
        ├── line.rs               #   细线/空心线/正弦/贝塞尔/圆弧/折线/螺旋
        ├── noise.rs              #   噪点/短线/椭圆斑/簇状噪点
        ├── character.rs          #   描边/阴影/色块叠加/笔画断裂
        ├── color.rs              #   RGB 通道偏移/渐变文字/局部反转
        ├── distortion.rs         #   波浪扭曲/弯曲变形
        ├── texture.rs            #   网格/点阵/柏林噪声/扫描线
        └── post_process.rs       #   盒式模糊/色彩量化/JPEG 伪影
```

## Feature

| feature | 说明 |
| --- | --- |
| `image` | 图像验证码（引入 `image` crate） |
| `audio` | 音频验证码（纯手写 WAV，无额外解码依赖） |
| `interactive` | 滑块 / 文字点选（隐式启用 `image`） |

默认全开：`default = ["image", "audio", "interactive"]`。

## 快速上手

```rust
use captcha_rs::{Captcha, config::Difficulty};

let captcha = Captcha::new();

// 图像验证码
let img = captcha
    .image(captcha_rs::image::DriverType::Digit)
    .width(240)
    .height(80)
    .difficulty(Difficulty::Medium)
    .generate()?;
// img.id / img.data (base64 PNG data URI) 下发给前端

// 音频验证码
let audio = captcha.audio().length(4).generate()?;

// 滑块验证码 (需提供背景图)
let slider = captcha
    .slider()
    .bg_image_paths(vec!["assets/bg1.png".into()])
    .generate()?;

// 文字点选验证码：用户按顺序点选 target_chars 中的字
let click = captcha.click_text().generate()?;

// 统一验证：成功即销毁 (一次性语义，防重放)
let ok = captcha.verify(&img.id, &user_input, true);
```

用户答案格式（`Captcha::verify` 的 `answer` 参数）：

- 图像 / 音频：纯文本，大小写不敏感
- 滑块：x 坐标数值字符串，如 `"186"`
- 点选：JSON 坐标序列，如 `[{"x":50,"y":30},{"x":100,"y":60}]`

## 安全语义

- 验证成功时答案无条件销毁；`clear=true` 时失败也会原子取走，
  并发验证同一 ID 恰好只有一个调用者能成功。
- 默认 5 次失败后锁定并清除记录（`DEFAULT_MAX_ATTEMPTS`）。
- 存储 TTL 600s、容量 10240 条；答案与尝试计数共享治理，无泄漏。
- 点选答案限制 ≤ 20 个坐标且 ≤ 4KB，防 DoS。
