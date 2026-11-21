//! 课后巩固（`text` 模块）的行为测试。只使用公开 API，描述可观察行为。

use l03_ownership_borrowing::text::{byte_prefix, first_line, total};

#[test]
fn first_line_returns_content_before_first_newline() {
    assert_eq!(first_line("hello\nworld"), "hello");
}

#[test]
fn first_line_keeps_only_the_first_of_many_lines() {
    assert_eq!(first_line("a\nb\nc"), "a");
}

#[test]
fn first_line_without_newline_returns_the_whole_text() {
    assert_eq!(first_line("no newline here"), "no newline here");
}

#[test]
fn first_line_of_empty_text_is_empty() {
    assert_eq!(first_line(""), "");
}

#[test]
fn first_line_starting_with_newline_is_empty() {
    assert_eq!(first_line("\nsecond"), "");
}

#[test]
fn total_sums_positive_and_negative_values() {
    assert_eq!(total(&[3, -1, 10, -4]), 8);
}

#[test]
fn total_of_empty_slice_is_zero() {
    assert_eq!(total(&[]), 0);
}

#[test]
fn total_of_single_element_is_that_element() {
    assert_eq!(total(&[42]), 42);
}

#[test]
fn byte_prefix_within_ascii_text_returns_that_many_bytes() {
    assert_eq!(byte_prefix("ownership", 4), "owne");
}

#[test]
fn byte_prefix_covering_or_exceeding_text_returns_the_whole_text() {
    assert_eq!(byte_prefix("rust", 4), "rust");
    assert_eq!(byte_prefix("rust", 100), "rust");
}

#[test]
fn byte_prefix_mid_character_backs_off_to_char_boundary() {
    // "你好" 共 6 个字节，每个汉字占 3 个字节
    assert_eq!(byte_prefix("你好", 4), "你");
    assert_eq!(byte_prefix("你好", 5), "你");
    assert_eq!(byte_prefix("你好", 6), "你好");
}

#[test]
fn byte_prefix_zero_returns_empty() {
    assert_eq!(byte_prefix("hello", 0), "");
}
