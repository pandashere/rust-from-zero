//! 主练习：通过 crate root 的公共 API 使用 `endpoint` 领域。
//!
//! 每个 `tests/*.rs` 文件都会被 Cargo 编译成独立的 crate，因此这里的
//! `use` 只能看见 library 的公共名字。它无法访问私有的 `endpoint` 模块，
//! 也无法调用 `Endpoint::new`——这正是要验证的公共契约。

use l07_modules_crates::{Endpoint, ParseEndpointError, parse_endpoint};

#[test]
fn public_api_parses_an_endpoint_through_crate_root_reexports() {
    let endpoint: Endpoint = parse_endpoint("example.com:443").unwrap();

    assert_eq!(endpoint.host(), "example.com");
    assert_eq!(endpoint.port(), 443);
}

#[test]
fn public_api_accepts_port_boundaries_unicode_hosts_and_surrounding_space() {
    let lowest = parse_endpoint("localhost:0").unwrap();
    assert_eq!(lowest.port(), 0);
    assert_eq!(lowest.host(), "localhost");

    let highest = parse_endpoint(" 例子:65535 ").unwrap();
    assert_eq!(highest.host(), "例子");
    assert_eq!(highest.port(), 65535);
}

#[test]
fn public_api_reports_structured_errors() {
    assert_eq!(
        parse_endpoint("example.com"),
        Err(ParseEndpointError::MissingSeparator)
    );
    assert_eq!(parse_endpoint(":443"), Err(ParseEndpointError::EmptyHost));
    assert_eq!(
        parse_endpoint("example.com:not-a-port"),
        Err(ParseEndpointError::InvalidPort(String::from("not-a-port")))
    );
}

#[test]
fn public_api_keeps_the_offending_port_text_in_the_error() {
    assert_eq!(
        parse_endpoint("example.com:443:extra"),
        Err(ParseEndpointError::InvalidPort(String::from("443:extra")))
    );
    assert_eq!(
        parse_endpoint("example.com: 65536 "),
        Err(ParseEndpointError::InvalidPort(String::from("65536")))
    );
}
