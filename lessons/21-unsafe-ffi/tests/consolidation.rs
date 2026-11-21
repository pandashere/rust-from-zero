//! 课后巩固的行为测试：C 字符串转拥有型 Rust 文本。

use std::ffi::CString;
use std::ptr;

use l21_unsafe_ffi::{NameError, read_c_name};

#[test]
fn c_string_reads_ascii_name() {
    let source = CString::new("ferris").expect("没有内部 NUL");

    // SAFETY: source 在调用期间存活，内容以 NUL 结尾且没有并发修改。
    let name = unsafe { read_c_name(source.as_ptr()) }.expect("内容是合法 UTF-8");

    assert_eq!(name, "ferris");
}

#[test]
fn c_string_empty_content_becomes_empty_string() {
    let source = CString::new("").expect("空文本是合法的 C 字符串");

    // SAFETY: 同上；空 C 字符串的第一个字节就是 NUL。
    let name = unsafe { read_c_name(source.as_ptr()) }.expect("空文本是合法 UTF-8");

    assert_eq!(name, "");
}

#[test]
fn c_string_reads_utf8_text() {
    let source = CString::new("海豚").expect("没有内部 NUL");

    // SAFETY: source 在调用期间存活，内容以 NUL 结尾且没有并发修改。
    let name = unsafe { read_c_name(source.as_ptr()) }.expect("内容是合法 UTF-8");

    assert_eq!(name, "海豚");
}

#[test]
fn c_string_null_pointer_is_reported() {
    // SAFETY: 空指针是函数契约明确允许并会处理的输入。
    let result = unsafe { read_c_name(ptr::null()) };

    assert_eq!(result, Err(NameError::NullPointer));
}

#[test]
fn c_string_invalid_utf8_is_reported() {
    let source = CString::new(vec![b'f', 0x80, b'o']).expect("没有内部 NUL");

    // SAFETY: source 在调用期间存活，内容以 NUL 结尾且没有并发修改。
    let result = unsafe { read_c_name(source.as_ptr()) };

    assert_eq!(result, Err(NameError::NotUtf8));
}

#[test]
fn c_string_result_owns_text_after_source_is_dropped() {
    let name = {
        let source = CString::new("temporary").expect("没有内部 NUL");
        // SAFETY: source 在本块内保持存活，返回值是拥有型 String。
        unsafe { read_c_name(source.as_ptr()) }.expect("内容是合法 UTF-8")
    };

    // source 已经销毁；如果返回值借用了它，这里会编译失败或读到悬垂内存。
    assert_eq!(name, "temporary");
}
