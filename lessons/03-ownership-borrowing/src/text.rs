//! 课后巩固脚手架：借用与 slice 小工具。
//!
//! 这三个函数都不取得输入的所有权：字符串参数是 `&str`，数字序列参数是 `&[i32]`。
//! 规格见 README「课后巩固」一节，卡住时按级查看 HINTS.md。

/// 返回 `text` 的第一行：从开头到第一个换行符 `'\n'` 之间的内容（不含换行符本身）。
/// - `text` 不含换行符时，返回整个 `text`；
/// - `text` 为空，或第一个字符就是换行符时，返回空字符串。
///
/// 返回的是 `text` 内部的一段借用，不复制字符，所以不能在函数里新建字符串再返回它的引用。
pub fn first_line(text: &str) -> &str {
    let _ = &text;
    todo!()
}

/// 返回 `values` 中所有元素相加的和。空 slice 的和为 0。
/// `values` 只是借用，函数不能修改也不能占有它。
pub fn total(values: &[i32]) -> i32 {
    let _ = &values;
    todo!()
}

/// 返回 `text` 开头的一段借用，长度尽量接近但不超过 `max_bytes` 个字节：
/// - `max_bytes` 大于等于 `text` 的字节长度时，返回整个 `text`；
/// - 否则，若第 `max_bytes` 个字节恰好是某个字符的起点，返回前 `max_bytes` 个字节；
/// - 若第 `max_bytes` 个字节落在某个 UTF-8 字符中间，则向前回退到最近的一个完整字符边界，
///   返回比 `max_bytes` 短的一段（例如 `"你好"` 共 6 字节，`max_bytes` 为 4 或 5 时返回 `"你"`）；
/// - `max_bytes` 为 0 时返回空字符串。
///
/// 返回的是 `text` 内部的一段借用，不复制字符，且不允许 panic。
pub fn byte_prefix(text: &str, max_bytes: usize) -> &str {
    let _ = (&text, &max_bytes);
    todo!()
}
