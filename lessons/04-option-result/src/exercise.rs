//! 主练习：解析 `host:port` 形式的端点文本。
//!
//! 一个“端点”（网络地址）由主机名和端口号组成，例如 `"localhost:8080"`。
//! 请只替换两个函数体里的 `todo!()`，不要修改类型定义、函数签名，也不要
//! 改动本文件底部的测试。每个函数上方的注释就是它的完整行为约定。

/// 解析成功时得到的端点：主机名加上端口号。
///
/// `host` 是去掉了两端空白之后的主机名文本，`port` 是 1 到 65535 之间的
/// 端口号。
#[derive(Debug, PartialEq, Eq)]
pub struct Endpoint {
    pub host: String,
    pub port: u16,
}

/// 解析端点文本时可能出现的失败原因。
///
/// 调用者可以据此区分“输入为空”“缺少分隔符”“主机名为空”“端口不是
/// 合法数字”和“端口为 0”这五类问题；`InvalidPort` 会带上无法解析的
/// 端口文本，方便调用者把原始输入展示给用户。
#[derive(Debug, PartialEq, Eq)]
pub enum ParseEndpointError {
    /// 整个输入去掉两端空白后没有任何字符。
    EmptyInput,
    /// 输入中找不到分隔主机名和端口的 `:`。
    MissingSeparator,
    /// 主机名部分单独去掉两端空白后没有任何字符。
    EmptyHost,
    /// 端口部分单独去掉两端空白后不是合法的 `u16` 数字；附带这段文本本身。
    InvalidPort(String),
    /// 端口数字合法但为 `0`；`0` 不是可用的端口号。
    ZeroPort,
}

/// 把一段端口文本解析成端口号。
///
/// 行为约定：
///
/// - 先去掉 `text` 两端的空白字符（空格、制表符、换行等），再判断内容；
/// - 去掉空白后的文本能解析成 `u16` 数字时：数字为 `0` 返回
///   `Err(ParseEndpointError::ZeroPort)`，否则返回 `Ok(端口号)`；
/// - 无法解析成 `u16` 数字时（包括空文本、含非数字字符、超出 0 到 65535
///   的范围），返回 `Err(ParseEndpointError::InvalidPort(文本))`，其中
///   “文本”是去掉两端空白之后的内容，保持原样、不做其他加工。
pub fn parse_port(text: &str) -> Result<u16, ParseEndpointError> {
    // 完成实现前避免“参数未使用”的提示，写好函数体后可删除。
    let _ = text;
    todo!()
}

/// 把 `host:port` 形式的文本解析成 `Endpoint`。
///
/// 行为约定（按此顺序判断，命中即返回）：
///
/// 1. 整个 `input` 去掉两端空白后为空：`Err(ParseEndpointError::EmptyInput)`。
/// 2. 找不到第一个 `:`：`Err(ParseEndpointError::MissingSeparator)`。
/// 3. 第一个 `:` 之前的主机名部分单独去掉两端空白后为空：
///    `Err(ParseEndpointError::EmptyHost)`。
/// 4. 第一个 `:` 之后的全部内容交给端口规则处理（即本文件的端口解析
///    函数）：端口失败时原样带出该错误。因此 `"host:8080:extra"` 的端口
///    文本是 `"8080:extra"`，属于无法解析的端口。
/// 5. 全部通过时返回 `Ok(Endpoint)`：`host` 是去掉两端空白后的主机名，
///    `port` 是解析出的端口号。
///
/// 约束：不允许用 `unwrap`、`expect` 或 `panic!` 处理输入；函数体中至少
/// 使用一次 `?` 来传播失败。
pub fn parse_endpoint(input: &str) -> Result<Endpoint, ParseEndpointError> {
    // 完成实现前避免“参数未使用”的提示，写好函数体后可删除。
    let _ = input;
    todo!()
}

#[cfg(test)]
mod tests {
    use super::{Endpoint, ParseEndpointError, parse_endpoint, parse_port};

    #[test]
    fn parse_port_accepts_valid_port_numbers() {
        assert_eq!(parse_port("8080"), Ok(8080));
        assert_eq!(parse_port(" 443 "), Ok(443));
        assert_eq!(parse_port("1"), Ok(1));
        assert_eq!(parse_port("65535"), Ok(65535));
    }

    #[test]
    fn parse_port_rejects_text_that_is_not_a_u16() {
        assert_eq!(
            parse_port("abc"),
            Err(ParseEndpointError::InvalidPort(String::from("abc")))
        );
        assert_eq!(
            parse_port(""),
            Err(ParseEndpointError::InvalidPort(String::from("")))
        );
        assert_eq!(
            parse_port("70000"),
            Err(ParseEndpointError::InvalidPort(String::from("70000")))
        );
        assert_eq!(
            parse_port("-1"),
            Err(ParseEndpointError::InvalidPort(String::from("-1")))
        );
    }

    #[test]
    fn parse_port_rejects_zero() {
        assert_eq!(parse_port("0"), Err(ParseEndpointError::ZeroPort));
        assert_eq!(parse_port(" 0 "), Err(ParseEndpointError::ZeroPort));
    }

    #[test]
    fn parse_endpoint_parses_host_and_port() {
        assert_eq!(
            parse_endpoint("localhost:8080"),
            Ok(Endpoint {
                host: String::from("localhost"),
                port: 8080,
            })
        );
        assert_eq!(
            parse_endpoint(" example.com : 443 "),
            Ok(Endpoint {
                host: String::from("example.com"),
                port: 443,
            })
        );
    }

    #[test]
    fn parse_endpoint_rejects_empty_input() {
        assert_eq!(parse_endpoint(""), Err(ParseEndpointError::EmptyInput));
        assert_eq!(parse_endpoint("   "), Err(ParseEndpointError::EmptyInput));
    }

    #[test]
    fn parse_endpoint_requires_a_separator() {
        assert_eq!(
            parse_endpoint("localhost"),
            Err(ParseEndpointError::MissingSeparator)
        );
    }

    #[test]
    fn parse_endpoint_requires_a_non_empty_host() {
        assert_eq!(parse_endpoint(":8080"), Err(ParseEndpointError::EmptyHost));
        assert_eq!(
            parse_endpoint("  :8080"),
            Err(ParseEndpointError::EmptyHost)
        );
    }

    #[test]
    fn parse_endpoint_reports_invalid_port_with_its_text() {
        for invalid in ["", "abc", "70000", "-1", "8080:extra"] {
            assert_eq!(
                parse_endpoint(&format!("localhost:{invalid}")),
                Err(ParseEndpointError::InvalidPort(String::from(invalid)))
            );
        }
    }

    #[test]
    fn parse_endpoint_rejects_zero_port() {
        assert_eq!(
            parse_endpoint("localhost:0"),
            Err(ParseEndpointError::ZeroPort)
        );
    }
}
