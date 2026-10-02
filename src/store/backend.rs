/// 存储后端 trait
///
/// 所有存储实现需实现此接口。默认实现为 [MemoryStore](memory::MemoryStore)。
pub trait Store: Send + Sync {
    /// 存储验证码答案 (JSON 序列化字符串)
    fn set(&self, id: &str, answer: &str);

    /// 获取存储的答案 (不删除)
    fn get(&self, id: &str) -> Option<String>;

    /// 原子取走答案 (读取并删除，返回旧值)
    ///
    /// 用于一次性验证：并发调用同一 ID 时恰好只有一个调用者能取到值，
    /// 保证"验证通过即作废"语义在并发下成立。
    fn take(&self, id: &str) -> Option<String>;

    /// 删除指定 ID 的记录 (无副作用：不递增尝试次数)
    fn remove(&self, id: &str);

    /// 递增尝试次数，返回当前值
    fn incr_attempts(&self, id: &str) -> u32;

    /// 获取当前尝试次数
    fn get_attempts(&self, id: &str) -> u32;
}
