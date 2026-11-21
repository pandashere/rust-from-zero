//! 课后巩固：把 C 字符串读取成拥有型 Rust 文本。
//!
//! C 字符串以数值为 0 的字节（NUL）结尾，本身不携带长度；Rust 的
//! `String` 记录长度，并且允许内容中出现 NUL。这个任务练习在两者之间
//! 建立一条有明确契约的边界。
//!
//! 只替换函数体里的 `todo!()`；不要修改函数签名和测试。验收命令：
//!
//! ```bash
//! cargo test -p l21_unsafe_ffi --test consolidation
//! ```

use std::ffi::c_char;

/// 读取 C 字符串时可能遇到的、可以恢复的问题。
#[derive(Debug, PartialEq, Eq)]
pub enum NameError {
    /// 传入的是空指针，没有任何字节可读。
    NullPointer,
    /// 字节序列不是合法的 UTF-8 文本。
    NotUtf8,
}

/// 把以 NUL 结尾的 C 字符串复制成拥有型 Rust 文本。
///
/// 行为约定：
///
/// - 空指针返回 `Err(NameError::NullPointer)`，不读取任何内存；
/// - 指针有效且内容为空 C 字符串（第一个字节就是 NUL）时返回
///   `Ok(String::new())`；
/// - 内容是合法 UTF-8 时返回 `Ok(文本)`，文本内容不包含结尾的 NUL；
/// - 内容不是合法 UTF-8 时返回 `Err(NameError::NotUtf8)`，不 panic；
/// - 返回的 `String` 拥有自己的内存：调用结束后即使原始 C 字符串被
///   释放，返回的文本仍然可用。
///
/// # Safety
///
/// `pointer` 必须要么是空指针，要么指向一段以 NUL 结尾、在调用期间可读
/// 且不与其他代码发生数据竞争（多个线程同时读写同一数据且没有同步）的
/// 字节序列。函数会自行处理空指针和非法 UTF-8，但无法检查指针是否
/// 悬垂，也无法检查内容是否真的以 NUL 结尾。
pub unsafe fn read_c_name(pointer: *const c_char) -> Result<String, NameError> {
    let _ = pointer;
    todo!()
}
