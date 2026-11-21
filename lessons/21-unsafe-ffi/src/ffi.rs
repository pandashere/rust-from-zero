//! 主练习第二部分：跨越 C ABI 的字节计数。
//!
//! 真实项目中，下面的 `rfz_count_byte` 会由 C 编译器编译进静态库（链接
//! 时复制进最终程序的库）或动态库（运行时才加载的库）。为了让本课不依
//! 赖 C 编译器或第三方构建工具也能直接编译运行，这里用一个
//! `#[unsafe(no_mangle)]` 的 Rust 函数扮演 C 实现。
//! 它导出的符号名、参数布局和调用约定与 C 版本相同，所以调用方经过的
//! 仍是一条真实的 ABI 边界。
//!
//! 学习者负责的是安全包装 [`count_byte`]，不是这个替身实现。

unsafe extern "C" {
    /// C 端符号：统计 `data[0..len]` 中等于 `needle` 的字节个数。
    ///
    /// 对应的 C 声明是：
    ///
    /// ```c
    /// size_t rfz_count_byte(const uint8_t *data, size_t len, uint8_t needle);
    /// ```
    ///
    /// 这里的 Rust 名字与导出的符号名不同，`link_name` 指定链接时查找的
    /// 符号；这是同文件内同时提供“替身实现”和“外部声明”时的写法。
    #[link_name = "rfz_count_byte"]
    // 学习者在 `count_byte` 里调用它；实现前这个声明暂时没人使用。
    #[allow(dead_code)]
    fn c_count_byte(data: *const u8, len: usize, needle: u8) -> usize;
}

/// 本课使用的 C 实现替身：统计 `data` 指向的 `len` 个字节中等于
/// `needle` 的个数。
///
/// # Safety
///
/// 调用者必须保证 `data` 指向至少 `len` 个已初始化的字节，并且这块内存
/// 在整个调用期间可读。`len` 为 `0` 时仍要求 `data` 非空且对齐；从
/// `&[u8]` 得到的 `as_ptr()` 满足这些条件。函数只读取，不保存指针。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rfz_count_byte(data: *const u8, len: usize, needle: u8) -> usize {
    let mut index = 0;
    let mut count = 0;
    while index < len {
        // SAFETY: 函数契约保证 data 在调用期间可读至少 len 个字节；
        // index < len，所以这次读取位于同一块有效内存内。
        let byte = unsafe { data.add(index).read() };
        if byte == needle {
            count += 1;
        }
        index += 1;
    }
    count
}

/// 统计 `data` 中等于 `needle` 的字节个数。
///
/// 行为约定：
///
/// - 按长度扫描整个切片，不把字节 `0` 当作结束标记，因此 NUL 是普通
///   数据；
/// - 空切片返回 `0`；
/// - `needle` 不出现时返回 `0`，所有字节都相同时返回 `data.len()`；
/// - 结果与 `data.iter().filter(|byte| **byte == needle).count()` 一致。
///
/// 这是安全函数：调用者只需要提供正常的 `&[u8]`，不需要满足任何指针、
/// 长度或初始化前置条件。
pub fn count_byte(data: &[u8], needle: u8) -> usize {
    let _ = &data;
    let _ = needle;
    todo!()
}
