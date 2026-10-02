use crate::config::DEFAULT_MAX_ATTEMPTS;
use crate::store::{AnswerData, Store};
use std::sync::Arc;

/// 验证码 ID 最大长度
const MAX_ID_LENGTH: usize = 128;

/// 点选验证码用户答案的最大坐标数量
const MAX_CLICK_POINTS: usize = 20;

/// 点选验证码用户答案 JSON 的最大字节数，防止 DoS
const MAX_CLICK_ANSWER_BYTES: usize = 4096;

/// 验证码统一引擎
///
/// 所有验证码类型通过此引擎生成和验证。
pub struct Captcha {
    store: Arc<dyn Store>,
}

impl Captcha {
    /// 创建引擎，使用默认 MemoryStore
    pub fn new() -> Self {
        Self {
            store: Arc::new(crate::store::MemoryStore::new()),
        }
    }

    /// 创建引擎，注入自定义 Store
    pub fn with_store(store: Arc<dyn Store>) -> Self {
        Self { store }
    }

    /// 获取内部 Store 引用
    pub fn store(&self) -> &Arc<dyn Store> {
        &self.store
    }

    // ---- 图像验证码 ----

    #[cfg(feature = "image")]
    pub fn image(&self, driver_type: crate::image::DriverType) -> crate::image::ImageBuilder {
        crate::image::ImageBuilder::new(self.store.clone(), driver_type)
    }

    // ---- 音频验证码 ----

    #[cfg(feature = "audio")]
    pub fn audio(&self) -> crate::audio::AudioBuilder {
        crate::audio::AudioBuilder::new(self.store.clone())
    }

    // ---- 交互验证码 ----

    #[cfg(feature = "interactive")]
    pub fn slider(&self) -> crate::interactive::SliderBuilder {
        crate::interactive::SliderBuilder::new(self.store.clone())
    }

    #[cfg(feature = "interactive")]
    pub fn click_text(&self) -> crate::interactive::ClickTextBuilder {
        crate::interactive::ClickTextBuilder::new(self.store.clone())
    }

    // ---- 统一验证 ----

    /// 验证答案
    ///
    /// - `id`: 验证码 ID
    /// - `answer`: 用户提交的答案
    ///   - 图像/音频: 纯文本字符串 (大小写不敏感)
    ///   - 滑块: x 坐标数值字符串 (如 "186")
    ///   - 点选: JSON 坐标序列 (如 `[{"x":50,"y":30},{"x":100,"y":60}]`)
    /// - `clear`: 验证失败后是否删除记录
    ///
    /// 验证成功时答案总是被销毁 (一次性语义，防重放)；
    /// `clear=true` 时失败也会原子取走答案，并发验证同一 ID
    /// 恰好只有一个调用者能成功。
    pub fn verify(&self, id: &str, answer: &str, clear: bool) -> bool {
        // 0. ID 输入校验
        if id.is_empty() || id.len() > MAX_ID_LENGTH {
            return false;
        }

        // 1. 获取答案：
        //    clear=true 时原子取走 (读取并删除)，保证并发下一次性语义；
        //    clear=false 时普通读取，记录保留供后续重试
        let stored = if clear {
            match self.store.take(id) {
                Some(json) => json,
                None => return false,
            }
        } else {
            match self.store.get(id) {
                Some(json) => json,
                None => return false,
            }
        };

        // 2. 递增尝试次数 (仅在此处递增，单次验证只计 1 次)
        let attempts = self.store.incr_attempts(id);
        if attempts > DEFAULT_MAX_ATTEMPTS {
            self.store.remove(id);
            return false;
        }

        // 3. 解析答案类型并验证
        let result = match AnswerData::from_json(&stored) {
            Ok(answer_data) => self.verify_answer(&answer_data, answer),
            Err(_) => false,
        };

        // 4. 清理：验证成功必须销毁答案 (一次性语义，防重放)；
        //    clear=true 时答案已被 take 原子删除，此处同步清理计数；
        //    clear=false 仅在失败时保留记录 (含计数) 供后续重试
        if result || clear {
            self.store.remove(id);
        }

        result
    }

    /// 根据答案类型分发验证
    fn verify_answer(&self, stored: &AnswerData, user_answer: &str) -> bool {
        match stored {
            AnswerData::Text { answer } => {
                // 大小写不敏感比较
                answer.eq_ignore_ascii_case(user_answer)
            }
            AnswerData::Slider { x, tolerance } => {
                // 解析用户答案为数值，容差随答案生成时按宽度比例计算
                match user_answer.parse::<i32>() {
                    Ok(user_x) => (user_x - *x as i32).abs() <= *tolerance as i32,
                    Err(_) => false,
                }
            }
            AnswerData::Click { targets } => {
                // 解析用户答案为 JSON 坐标序列
                #[derive(serde::Deserialize)]
                struct ClickPoint {
                    x: u32,
                    y: u32,
                }

                // 限制用户答案长度，防止 DoS
                if user_answer.len() > MAX_CLICK_ANSWER_BYTES {
                    return false;
                }

                match serde_json::from_str::<Vec<ClickPoint>>(user_answer) {
                    Ok(points) => {
                        // 限制坐标数量，防止超大数组
                        if points.len() > MAX_CLICK_POINTS {
                            return false;
                        }
                        if points.len() != targets.len() {
                            return false;
                        }
                        // 逐一校验是否落在目标区域内 (按目标顺序匹配)
                        for (point, target) in points.iter().zip(targets.iter()) {
                            if point.x < target.x
                                || point.x > target.x + target.w
                                || point.y < target.y
                                || point.y > target.y + target.h
                            {
                                return false;
                            }
                        }
                        true
                    }
                    Err(_) => false,
                }
            }
        }
    }
}

impl Default for Captcha {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::{ClickTarget, MemoryStore};

    /// 构造使用独立 MemoryStore 的 Captcha
    fn new_captcha() -> Captcha {
        Captcha::with_store(Arc::new(MemoryStore::new()))
    }

    #[test]
    fn test_verify_text_case_insensitive() {
        let captcha = new_captcha();
        captcha
            .store()
            .set("t1", &AnswerData::text("AbCd").to_json());
        assert!(captcha.verify("t1", "abcd", true));
    }

    #[test]
    fn test_verify_invalid_id_rejected() {
        let captcha = new_captcha();
        // 空 ID 与超长 ID 直接拒绝
        assert!(!captcha.verify("", "x", true));
        let long_id = "a".repeat(MAX_ID_LENGTH + 1);
        assert!(!captcha.verify(&long_id, "x", true));
    }

    #[test]
    fn test_single_failure_incr_attempts_once() {
        // P0 双重计数回归防护：验证失败一次后计数应为 1 而非 2
        let captcha = new_captcha();
        captcha
            .store()
            .set("t2", &AnswerData::text("ABCD").to_json());
        assert!(!captcha.verify("t2", "WRONG", false));
        assert_eq!(captcha.store().get_attempts("t2"), 1);
    }

    #[test]
    fn test_attempt_limit_locks_and_removes() {
        // 5 次失败后锁定；第 6 次即使答案正确也拒绝，且记录被移除
        let captcha = new_captcha();
        captcha
            .store()
            .set("t3", &AnswerData::text("ABCD").to_json());
        for _ in 0..DEFAULT_MAX_ATTEMPTS {
            assert!(!captcha.verify("t3", "WRONG", false));
        }
        assert!(!captcha.verify("t3", "ABCD", false));
        assert!(captcha.store().get("t3").is_none());
    }

    #[test]
    fn test_verify_clear_one_shot_semantics() {
        // clear=true 验证成功后答案即删除，重放失败
        let captcha = new_captcha();
        captcha
            .store()
            .set("t4", &AnswerData::text("ABCD").to_json());
        assert!(captcha.verify("t4", "abcd", true));
        assert!(captcha.store().get("t4").is_none());
        assert!(!captcha.verify("t4", "abcd", true));
    }

    #[test]
    fn test_verify_concurrent_clear_only_one_success() {
        // P1 一次性语义并发防护：并发验证同一 clear=true 的 ID 恰好一个成功
        let captcha = Arc::new(new_captcha());
        captcha
            .store()
            .set("t5", &AnswerData::text("ABCD").to_json());
        let mut handles = vec![];
        for _ in 0..10 {
            let c = captcha.clone();
            handles.push(std::thread::spawn(move || c.verify("t5", "ABCD", true)));
        }
        let successes: u32 = handles.into_iter().map(|h| h.join().unwrap() as u32).sum();
        assert_eq!(successes, 1);
    }

    #[test]
    fn test_verify_slider_tolerance() {
        let captcha = new_captcha();
        captcha
            .store()
            .set("t6", &AnswerData::slider(100).to_json());
        // ±5px 边界内通过 (成功即销毁，每次重新置入)
        assert!(captcha.verify("t6", "95", false));
        captcha
            .store()
            .set("t6", &AnswerData::slider(100).to_json());
        assert!(captcha.verify("t6", "105", false));
        // 超出边界拒绝
        captcha
            .store()
            .set("t6", &AnswerData::slider(100).to_json());
        assert!(!captcha.verify("t6", "94", false));
        assert!(!captcha.verify("t6", "106", false));
        // 非数值拒绝
        assert!(!captcha.verify("t6", "abc", false));
    }

    #[test]
    fn test_verify_slider_custom_tolerance() {
        // 容差随答案存储：自定义宽容差生效
        let captcha = new_captcha();
        captcha
            .store()
            .set("t9", &AnswerData::slider_with_tolerance(100, 12).to_json());
        assert!(captcha.verify("t9", "88", false));
        captcha
            .store()
            .set("t9", &AnswerData::slider_with_tolerance(100, 12).to_json());
        assert!(captcha.verify("t9", "112", false));
        captcha
            .store()
            .set("t9", &AnswerData::slider_with_tolerance(100, 12).to_json());
        assert!(!captcha.verify("t9", "87", false));
    }

    #[test]
    fn test_slider_old_json_without_tolerance_falls_back() {
        // 旧记录无 tolerance 字段时 serde default 回落 5px
        let captcha = new_captcha();
        captcha.store().set("t10", r#"{"type":"slider","x":100}"#);
        assert!(captcha.verify("t10", "105", false));
        captcha.store().set("t10", r#"{"type":"slider","x":100}"#);
        assert!(!captcha.verify("t10", "106", false));
    }

    #[test]
    fn test_success_always_consumes_answer() {
        // P2-1：验证成功时答案无条件销毁，即使 clear=false 也不可重放
        let captcha = new_captcha();
        captcha
            .store()
            .set("t11", &AnswerData::text("ABCD").to_json());
        assert!(captcha.verify("t11", "abcd", false));
        // 成功后重放同答案 → 记录已删除，拒绝
        assert!(!captcha.verify("t11", "abcd", false));
        assert!(captcha.store().get("t11").is_none());
    }

    #[test]
    fn test_verify_click_order_and_bounds() {
        let captcha = new_captcha();
        let targets = vec![
            ClickTarget {
                char: "一".into(),
                x: 10,
                y: 20,
                w: 30,
                h: 30,
            },
            ClickTarget {
                char: "二".into(),
                x: 100,
                y: 60,
                w: 30,
                h: 30,
            },
        ];
        captcha
            .store()
            .set("t7", &AnswerData::click(targets.clone()).to_json());

        // 命中盒内边界通过 (含边界值；成功即销毁，后续断言需重新置入)
        assert!(captcha.verify("t7", r#"[{"x":10,"y":20},{"x":130,"y":90}]"#, false));
        captcha
            .store()
            .set("t7", &AnswerData::click(targets.clone()).to_json());
        // 顺序颠倒: 第一个点落在第二个目标盒内 -> 拒绝
        assert!(!captcha.verify("t7", r#"[{"x":110,"y":70},{"x":20,"y":30}]"#, false));
        // 越界一点即拒绝
        assert!(!captcha.verify("t7", r#"[{"x":41,"y":30},{"x":110,"y":70}]"#, false));
        // 点数不匹配拒绝
        assert!(!captcha.verify("t7", r#"[{"x":20,"y":30}]"#, false));
    }

    #[test]
    fn test_verify_click_dos_guards() {
        let captcha = new_captcha();
        let targets: Vec<ClickTarget> = (0..4)
            .map(|i| ClickTarget {
                char: "一".into(),
                x: i * 10,
                y: 0,
                w: 30,
                h: 30,
            })
            .collect();
        captcha
            .store()
            .set("t8", &AnswerData::click(targets).to_json());

        // 超过 20 个坐标点 -> 拒绝
        let many: String = (0..21)
            .map(|i| format!(r#"{{"x":{},"y":0}}"#, i))
            .collect::<Vec<_>>()
            .join(",");
        assert!(!captcha.verify("t8", &format!("[{}]", many), false));

        // 超过 4KB 的 JSON -> 拒绝
        let big = format!(r#"[{{"x":0,"y":0,"pad":"{}"}}]"#, "a".repeat(5000));
        assert!(big.len() > MAX_CLICK_ANSWER_BYTES);
        assert!(!captcha.verify("t8", &big, false));
    }

    #[test]
    fn test_verify_nonexistent_id() {
        let captcha = new_captcha();
        assert!(!captcha.verify("no-such-id", "any", true));
        assert_eq!(captcha.store().get_attempts("no-such-id"), 0);
    }
}
