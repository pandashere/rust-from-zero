//! 第二十一课的演示入口：安全代码能直接做到什么，以及 C 字符串在内存里
//! 的样子。
//!
//! 练习本身用测试验收，完成 `todo!()` 之前运行测试会失败，这是预期现象：
//!
//! ```bash
//! cargo test -p l21_unsafe_ffi
//! ```

use std::ffi::CString;

fn main() {
    let mut value = 41_i32;
    let pointer = &raw mut value;
    // SAFETY: pointer 来自仍然存活的局部变量 value，类型和对齐都正确。
    unsafe {
        *pointer += 1;
    }
    println!("通过裸指针修改后，value = {value}");

    let mut data = [1, 2, 3, 4, 5];
    {
        let (left, right) = data.split_at_mut(2);
        left[0] = 10;
        right[0] = 30;
    }
    println!("标准库拆分并修改后: {data:?}");

    let name = CString::new("ferris").expect("文本中没有 NUL 字节");
    println!(
        "C 字符串的字节（末尾 0 是 NUL）: {:?}",
        name.as_bytes_with_nul()
    );

    println!("你的实现用 `cargo test -p l21_unsafe_ffi` 验收。");
}
