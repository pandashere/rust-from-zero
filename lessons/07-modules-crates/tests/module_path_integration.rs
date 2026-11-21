//! 课后巩固：通过 crate root 的公共 API 使用 `module_path` 领域。
//!
//! 与上一个集成测试一样，本文件是独立 crate，只能看到 `lib.rs` 用
//! `pub use` 暴露的三个名字。

use l07_modules_crates::{ModulePath, ParseModulePathError, parse_module_path};

#[test]
fn public_api_parses_a_rooted_module_path() {
    let path: ModulePath = parse_module_path("crate::endpoint::parser").unwrap();

    assert_eq!(path.segments().len(), 3);
    assert_eq!(path.segments()[1], "endpoint");
    assert!(path.starts_at_crate_root());
}

#[test]
fn public_api_accepts_a_single_segment_and_trims_the_whole_input() {
    let path = parse_module_path("  main  ").unwrap();

    assert_eq!(path.segments(), ["main"]);
    assert!(!path.starts_at_crate_root());
}

#[test]
fn public_api_reports_empty_path_and_empty_segment_with_position() {
    assert_eq!(
        parse_module_path(" \n "),
        Err(ParseModulePathError::EmptyPath)
    );
    assert_eq!(
        parse_module_path("crate::"),
        Err(ParseModulePathError::EmptySegment { index: 1 })
    );
}

#[test]
fn public_api_reports_invalid_segment_with_position_and_text() {
    assert_eq!(
        parse_module_path("crate::Parser"),
        Err(ParseModulePathError::InvalidSegment {
            index: 1,
            text: String::from("Parser"),
        })
    );
    assert_eq!(
        parse_module_path("crate::endpoint::模块"),
        Err(ParseModulePathError::InvalidSegment {
            index: 2,
            text: String::from("模块"),
        })
    );
}
