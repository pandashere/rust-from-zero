//! 课后巩固：用声明宏生成二维数字表（可以理解为若干行数字）。
//!
//! 这是主练习之后的独立任务：数据从“带名称的库存条目”换成了“按行排列的
//! 整数”，边界也从“空清单、默认数量”扩展到了“空行、长短不一的行、空表”。
//! 请只替换宏展开模板中的 `todo!()`（以及占位结构），不要修改宏的调用形式、
//! 测试和本说明。
//!
//! 完成后的验收命令：
//!
//! ```bash
//! cargo test -p l20_macros consolidation
//! ```

/// 用方括号按行书写数字，生成 `Vec<Vec<i64>>`。
///
/// 调用形式：
///
/// - `matrix![]`：空表，得到空的 `Vec<Vec<i64>>`；
/// - 一行或多行：每行写成 `[数字, 数字, ...]`，行与行之间用逗号分隔。
///
/// 具体行为：
///
/// - 每一行本身是一个 `Vec<i64>`，行的顺序与书写顺序一致；
/// - 各行长度可以不同；一行里可以一个数字也没有，得到空行；
/// - 每个数字都可以是任意 `i64` 表达式，并且只求值一次；
/// - 每一行内部的最后一项之后允许一个尾逗号；最后一行之后也允许一个
///   尾逗号；两者可以同时出现。
///
/// 这个宏同样要在别的 crate 中可以直接调用，展开模板不能依赖调用处的
/// 导入。
#[macro_export]
macro_rules! matrix {
    () => {{
        // 脚手架占位：返回类型已经写对，但一执行就会 panic。
        // 实现时请用真正的展开模板替换整个 `{{ ... }}`。
        fn placeholder() -> ::std::vec::Vec<::std::vec::Vec<i64>> {
            todo!()
        }
        placeholder()
    }};
    ($([$($value:expr),* $(,)?]),+ $(,)?) => {{
        fn placeholder() -> ::std::vec::Vec<::std::vec::Vec<i64>> {
            todo!()
        }
        placeholder()
    }};
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    #[test]
    fn matrix_builds_rows_in_order() {
        let grid = matrix![[1, 2, 3], [4, 5, 6]];
        assert_eq!(grid, vec![vec![1, 2, 3], vec![4, 5, 6]]);
    }

    #[test]
    fn matrix_accepts_ragged_rows() {
        let grid = matrix![[1], [2, 3, 4], [5, 6]];
        assert_eq!(grid, vec![vec![1], vec![2, 3, 4], vec![5, 6]]);
    }

    #[test]
    fn matrix_accepts_empty_rows() {
        let grid = matrix![[], [7], []];
        assert_eq!(grid, vec![vec![], vec![7], vec![]]);
    }

    #[test]
    fn matrix_accepts_empty_matrix() {
        let grid: Vec<Vec<i64>> = matrix![];
        assert!(grid.is_empty());
    }

    #[test]
    fn matrix_accepts_trailing_commas() {
        let grid = matrix![[1, 2,], [3,],];
        assert_eq!(grid, vec![vec![1, 2], vec![3]]);
    }

    #[test]
    fn matrix_keeps_negative_and_computed_values() {
        let grid = matrix![[-1, 2 - 5], [10 / 2]];
        assert_eq!(grid, vec![vec![-1, -3], vec![5]]);
    }

    #[test]
    fn matrix_evaluates_each_value_once() {
        let calls = Cell::new(0);
        let grid = matrix![
            [
                {
                    calls.set(calls.get() + 1);
                    1
                },
                {
                    calls.set(calls.get() + 1);
                    2
                }
            ],
            [{
                calls.set(calls.get() + 1);
                3
            }]
        ];
        assert_eq!(calls.get(), 3);
        assert_eq!(grid, vec![vec![1, 2], vec![3]]);
    }
}
