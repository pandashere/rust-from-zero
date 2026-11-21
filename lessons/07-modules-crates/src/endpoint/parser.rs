//! `endpoint` 领域的解析器：公共入口 `parse_endpoint` 和私有辅助函数
//! `parse_port`。

use super::{Endpoint, ParseEndpointError};

/// 解析 `host:port` 形式的端点文本。
///
/// 主机名与端口两端的多余空白会被忽略。
///
/// # Examples
///
/// ```
/// use l07_modules_crates::parse_endpoint;
///
/// let endpoint = parse_endpoint("  example.com:443  ")?;
/// assert_eq!(endpoint.host(), "example.com");
/// assert_eq!(endpoint.port(), 443);
/// # Ok::<(), l07_modules_crates::ParseEndpointError>(())
/// ```
///
/// `Endpoint` 的字段是私有的，所以它的值只能由解析器创建：
///
/// ```compile_fail
/// use l07_modules_crates::Endpoint;
///
/// let _endpoint = Endpoint {
///     host: String::from("example.com"),
///     port: 443,
/// };
/// ```
pub fn parse_endpoint(input: &str) -> Result<Endpoint, ParseEndpointError> {
    let (host, port_text) = input
        .split_once(':')
        .ok_or(ParseEndpointError::MissingSeparator)?;
    let host = host.trim();
    if host.is_empty() {
        return Err(ParseEndpointError::EmptyHost);
    }

    let port = parse_port(port_text)?;
    Ok(Endpoint::new(host.to_owned(), port))
}

/// 把端口文本解析为 `u16`。
///
/// 行为约定：
///
/// - 先去掉 `input` 两端的空白，再解析；
/// - 能解析为 `u16`（0 到 65535，含两端）时返回 `Ok(数值)`；
/// - 其余情况（空文本、非数字、负数、超过 65535 等）返回
///   `Err(ParseEndpointError::InvalidPort(...))`，错误中携带去掉两端
///   空白后的端口文本的 owned 拷贝（`String`）。
///
/// 注意：`" 443 "` 是合法输入；`"4 43"` 中间有空格，不是合法数字。
fn parse_port(input: &str) -> Result<u16, ParseEndpointError> {
    // 完成实现前避免“参数未使用”的警告，写好函数体后可删除。
    let _ = &input;
    todo!()
}

#[cfg(test)]
mod tests {
    use super::{ParseEndpointError, parse_endpoint, parse_port};

    #[test]
    fn private_parse_port_accepts_boundary_values() {
        assert_eq!(parse_port("0"), Ok(0));
        assert_eq!(parse_port("65535"), Ok(65535));
    }

    #[test]
    fn private_parse_port_ignores_surrounding_whitespace() {
        assert_eq!(parse_port("  443\n"), Ok(443));
        assert_eq!(parse_port("\t 65535 "), Ok(65535));
    }

    #[test]
    fn private_parse_port_preserves_the_trimmed_offending_text() {
        assert_eq!(
            parse_port("many"),
            Err(ParseEndpointError::InvalidPort(String::from("many")))
        );
        assert_eq!(
            parse_port("65536"),
            Err(ParseEndpointError::InvalidPort(String::from("65536")))
        );
        assert_eq!(
            parse_port(" -1 "),
            Err(ParseEndpointError::InvalidPort(String::from("-1")))
        );
        assert_eq!(
            parse_port("4 43"),
            Err(ParseEndpointError::InvalidPort(String::from("4 43")))
        );
        assert_eq!(
            parse_port(""),
            Err(ParseEndpointError::InvalidPort(String::new()))
        );
    }

    #[test]
    fn parse_endpoint_accepts_normal_input_and_unicode_hosts() {
        let endpoint = parse_endpoint("example.com:443").unwrap();
        assert_eq!(endpoint.host(), "example.com");
        assert_eq!(endpoint.port(), 443);

        let unicode = parse_endpoint("例子:8080").unwrap();
        assert_eq!(unicode.host(), "例子");
        assert_eq!(unicode.port(), 8080);
    }

    #[test]
    fn parse_endpoint_ignores_surrounding_whitespace_on_both_parts() {
        let endpoint = parse_endpoint("  example.com: 443  ").unwrap();
        assert_eq!(endpoint.host(), "example.com");
        assert_eq!(endpoint.port(), 443);
    }

    #[test]
    fn parse_endpoint_rejects_missing_separator_and_empty_host() {
        assert_eq!(
            parse_endpoint("localhost"),
            Err(ParseEndpointError::MissingSeparator)
        );
        assert_eq!(parse_endpoint(":8080"), Err(ParseEndpointError::EmptyHost));
        assert_eq!(
            parse_endpoint("  :8080"),
            Err(ParseEndpointError::EmptyHost)
        );
    }

    #[test]
    fn parse_endpoint_treats_a_second_separator_as_invalid_port() {
        assert_eq!(
            parse_endpoint("localhost:80:90"),
            Err(ParseEndpointError::InvalidPort(String::from("80:90")))
        );
    }
}
