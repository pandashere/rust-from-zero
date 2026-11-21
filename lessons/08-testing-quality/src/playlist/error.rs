//! 往歌单里添加曲目时可能失败的原因。

/// 添加曲目失败的原因。
///
/// 每一种失败都单独占一个变体（variant，枚举中的一个可选值），调用者
/// 可以据此决定怎么处理。`DuplicateTitle` 和 `Full` 还携带了调用者需要
/// 的上下文：重复的标题、已满歌单的容量。
///
/// 本课只比较结构化的变体和字段，不比较给人看的错误文本；为用户展示
/// 消息的 `Display` 实现留到第十二课学习 trait 之后再补。
#[derive(Debug, PartialEq, Eq)]
pub enum PlaylistError {
    /// 标题去掉两端空白后没有任何字符。
    EmptyTitle,
    /// 曲目时长为 0 秒。
    ZeroLength,
    /// 歌单里已经有同名曲目；携带的是去掉两端空白后的标题。
    DuplicateTitle(String),
    /// 歌单已满，无法再追加；携带的是歌单容量。
    Full { capacity: usize },
}
