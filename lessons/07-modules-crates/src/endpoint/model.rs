//! `endpoint` 领域的数据模型：类型公开，字段私有，构造器只开放给父模块。

/// 一个已经通过解析验证的网络端点，由主机名和端口号组成。
///
/// 字段保持私有，因此 crate 外的调用者无法用 struct 字面量跳过验证直接
/// 拼出一个 `Endpoint`；只能通过 [`parse_endpoint`](crate::parse_endpoint)
/// 获得它，再用 `host()`、`port()` 读取内容。
///
/// 不变量：`host` 非空，`port` 是 `u16`（0 到 65535，含两端）。
#[derive(Debug, PartialEq, Eq)]
pub struct Endpoint {
    host: String,
    port: u16,
}

impl Endpoint {
    /// 构造一个端点。只开放给父模块 `endpoint` 的范围使用：兄弟模块
    /// `parser` 在完成验证后可以调用它，crate 外的调用者看不到它。
    pub(super) fn new(host: String, port: u16) -> Self {
        Self { host, port }
    }

    /// 返回该端点主机名的只读借用（`&str`），内容就是解析时保存的主机名
    /// 文本。本方法只借出已有数据，不复制文本、不分配新的 `String`。
    pub fn host(&self) -> &str {
        todo!()
    }

    /// 返回该端点的端口号数值（`u16`），与解析时保存的端口一致。
    ///
    /// `u16` 是“无符号 16 位整数”，只能表示 0 到 65535 的整数；这个类型
    /// 是 Copy 的，读取它会复制数值本身。
    pub fn port(&self) -> u16 {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::Endpoint;

    #[test]
    fn getters_return_the_host_and_port_given_to_the_constructor() {
        let endpoint = Endpoint::new(String::from("localhost"), 8080);

        assert_eq!(endpoint.host(), "localhost");
        assert_eq!(endpoint.port(), 8080);
    }

    #[test]
    fn getters_cover_port_boundaries() {
        let lowest = Endpoint::new(String::from("example.com"), 0);
        let highest = Endpoint::new(String::from("example.com"), 65535);

        assert_eq!(lowest.port(), 0);
        assert_eq!(highest.port(), 65535);
        assert_eq!(highest.host(), "example.com");
    }
}
