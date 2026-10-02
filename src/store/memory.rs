use crate::config::{DEFAULT_EXPIRATION_SECS, DEFAULT_MAX_ENTRIES};
use crate::store::Store;
use moka::sync::Cache;
use std::time::Duration;

/// 内存存储实现
///
/// 基于 moka 高性能并发缓存：
/// - 无锁并发读 (答案与计数缓存)
/// - 自动过期剔除 (答案与尝试计数共享 TTL 治理，无泄漏)
/// - 容量保护 (防止无界内存增长)
/// - 原子 take 防 TOCTOU 竞态
/// - 尝试次数限制防暴力破解 (entry API 原子递增)
#[derive(Debug)]
pub struct MemoryStore {
    /// 答案缓存
    answers: Cache<String, String>,
    /// 尝试次数 (与答案缓存共享 TTL/容量治理)
    attempts: Cache<String, u32>,
}

impl MemoryStore {
    /// 创建新的内存存储
    ///
    /// 使用默认配置：容量 10240，过期 600 秒
    pub fn new() -> Self {
        Self::with_config(DEFAULT_MAX_ENTRIES, DEFAULT_EXPIRATION_SECS)
    }

    /// 自定义配置创建
    pub fn with_config(max_entries: usize, expiration_secs: u64) -> Self {
        let ttl = Duration::from_secs(expiration_secs);
        let answers = Cache::builder()
            .max_capacity(max_entries as u64)
            .time_to_live(ttl)
            .build();
        // 尝试计数与答案共享 TTL 与容量上限，条目随时间自动清理
        let attempts = Cache::builder()
            .max_capacity(max_entries as u64)
            .time_to_live(ttl)
            .build();
        Self { answers, attempts }
    }

    /// 获取当前存储的条目数
    pub fn len(&self) -> usize {
        self.answers.entry_count() as usize
    }

    /// 检查存储是否为空
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Default for MemoryStore {
    fn default() -> Self {
        Self::new()
    }
}

impl Store for MemoryStore {
    fn set(&self, id: &str, answer: &str) {
        self.answers.insert(id.to_string(), answer.to_string());
        // 重置尝试次数 (重新生成验证码不继承旧计数)
        self.attempts.invalidate(id);
    }

    fn get(&self, id: &str) -> Option<String> {
        self.answers.get(id)
    }

    fn take(&self, id: &str) -> Option<String> {
        // moka 的 remove 原子返回旧值，天然防 TOCTOU 竞态：
        // 并发调用同一 ID 时只有一个调用者能取到值
        self.answers.remove(id)
    }

    fn remove(&self, id: &str) {
        self.answers.remove(id);
        self.attempts.invalidate(id);
    }

    fn incr_attempts(&self, id: &str) -> u32 {
        // entry API 的 key 级锁保证 read-modify-write 原子性，无 TOCTOU 竞态
        self.attempts
            .entry(id.to_string())
            .and_upsert_with(|maybe_entry| {
                maybe_entry.map(|entry| entry.into_value() + 1).unwrap_or(1)
            })
            .into_value()
    }

    fn get_attempts(&self, id: &str) -> u32 {
        self.attempts.get(id).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::DEFAULT_MAX_ATTEMPTS;
    use crate::store::AnswerData;

    #[test]
    fn test_set_get() {
        let store = MemoryStore::new();
        let answer = AnswerData::text("ABCD");
        store.set("test1", &answer.to_json());
        assert!(store.get("test1").is_some());
    }

    #[test]
    fn test_take_removes_entry() {
        let store = MemoryStore::new();
        store.set("test1", "data");
        // take 原子取走：返回旧值且删除
        assert_eq!(store.take("test1").as_deref(), Some("data"));
        assert!(store.get("test1").is_none());
        // 再次 take 返回 None (一次性语义)
        assert!(store.take("test1").is_none());
    }

    #[test]
    fn test_take_concurrent_only_one_wins() {
        use std::sync::Arc;
        use std::thread;

        let store = Arc::new(MemoryStore::new());
        store.set("test1", "data");
        let mut handles = vec![];
        for _ in 0..10 {
            let s = store.clone();
            handles.push(thread::spawn(move || s.take("test1").is_some()));
        }
        let winners: u32 = handles.into_iter().map(|h| h.join().unwrap() as u32).sum();
        // 并发 take 恰好只有一个成功
        assert_eq!(winners, 1);
    }

    #[test]
    fn test_remove_no_side_effects() {
        let store = MemoryStore::new();
        store.set("test1", "data");
        store.incr_attempts("test1");
        // remove 是彻底清理：答案与尝试计数一并删除，不产生递增等副作用
        store.remove("test1");
        assert_eq!(store.get_attempts("test1"), 0);
        assert!(store.get("test1").is_none());
    }

    #[test]
    fn test_set_resets_attempts() {
        let store = MemoryStore::new();
        store.set("test1", "data");
        store.incr_attempts("test1");
        store.incr_attempts("test1");
        assert_eq!(store.get_attempts("test1"), 2);
        // 重新 set 应重置尝试次数
        store.set("test1", "new_data");
        assert_eq!(store.get_attempts("test1"), 0);
    }

    #[test]
    fn test_expiration() {
        let store = MemoryStore::with_config(100, 1);
        store.set("test1", "data");
        std::thread::sleep(Duration::from_secs(2));
        assert!(store.get("test1").is_none());
    }

    #[test]
    fn test_attempts_expiration() {
        // 尝试计数与答案共享 TTL，过期后自动清理 (无泄漏)
        let store = MemoryStore::with_config(100, 1);
        store.incr_attempts("test1");
        assert_eq!(store.get_attempts("test1"), 1);
        std::thread::sleep(Duration::from_secs(2));
        assert_eq!(store.get_attempts("test1"), 0);
    }

    #[test]
    fn test_concurrent_incr_attempts() {
        use std::sync::Arc;
        use std::thread;

        let store = Arc::new(MemoryStore::new());
        let mut handles = vec![];

        for _ in 0..10 {
            let s = store.clone();
            handles.push(thread::spawn(move || {
                s.incr_attempts("concurrent_test");
            }));
        }

        for h in handles {
            h.join().unwrap();
        }

        // 10 个线程各递增 1 次，总计应为 10
        assert_eq!(store.get_attempts("concurrent_test"), 10);
    }

    #[test]
    fn test_attempt_limit_semantics() {
        // 超过上限后计数继续递增 (由 Captcha::verify 层负责锁定并清理)
        let store = MemoryStore::new();
        store.set("test1", "data");
        for _ in 0..DEFAULT_MAX_ATTEMPTS {
            store.incr_attempts("test1");
        }
        assert_eq!(store.get_attempts("test1"), DEFAULT_MAX_ATTEMPTS);
        assert!(store.incr_attempts("test1") > DEFAULT_MAX_ATTEMPTS);
    }
}
