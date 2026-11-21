//! 主练习第一部分：不借助标准库的 `split_at_mut`，用裸指针把一段可变
//! 切片拆成前后两段。
//!
//! 只替换函数体里的 `todo!()`；不要修改函数签名，也不要改动测试。
//! 完整的行为规格与实现约束见 README「主练习」。

/// 把 `values` 在下标 `mid` 处拆成前后两段可变借用。
///
/// 行为约定：
///
/// - `mid <= values.len()` 时返回 `Some((前段, 后段))`：前段包含下标
///   `0..mid` 的元素，后段包含下标 `mid..len` 的元素；两段都保持原有
///   顺序，并且可以分别修改；
/// - `mid == 0` 时前段为空，`mid == values.len()` 时后段为空；
/// - `mid > values.len()` 时返回 `None`，不 panic，也不移动或修改元素；
/// - 空切片只在 `mid == 0` 时可拆，返回两段空切片；
/// - 返回的两段借用与输入 `values` 的借用范围相同：调用者使用它们期间
///   不能再通过别的路径访问 `values`。
///
/// 实现约束（README 有完整说明）：使用裸指针和
/// `std::slice::from_raw_parts_mut` 完成拆分，把危险操作收在一个尽可能
/// 小的 `unsafe` 块里，并写出说明前置条件为何成立的 SAFETY 注释。
pub fn split_at_mut_checked<T>(values: &mut [T], mid: usize) -> Option<(&mut [T], &mut [T])> {
    let _ = &values;
    let _ = mid;
    todo!()
}
