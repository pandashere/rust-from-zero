//! `endpoint` 领域的错误类型。

/// 解析 `host:port` 端点文本时可能出现的失败原因。
#[derive(Debug, PartialEq, Eq)]
pub enum ParseEndpointError {
    /// 输入中找不到分隔 host 与 port 的 `:`。
    MissingSeparator,
    /// `:` 存在，但它前面的主机名去掉两端空白后没有任何字符。
    EmptyHost,
    /// 端口部分无法解析为 `u16`（0 到 65535，含两端）。携带去掉两端
    /// 空白后的原始端口文本，便于向用户报告究竟哪段内容非法。
    InvalidPort(String),
}
