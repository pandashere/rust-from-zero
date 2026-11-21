//! 课后巩固：用作用域线程并行统计一组整数。
//!
//! 主练习让工作线程拥有任务数据，并通过 channel 把结果送回主线程。这个
//! 任务换一种数据（有符号整数）和另一种借用方式：输入切片由调用者拥有，
//! 工作线程只在函数返回前借用它，因此不需要复制整组数据，也不需要
//! channel。它练习 `thread::scope` 的适用场景。
//!
//! 请只替换 `parallel_stats` 函数体里的 `todo!()`，不要修改类型定义、
//! 函数签名，也不要改动本文件底部的测试。
//!
//! 完成后的验收命令：
//!
//! ```bash
//! cargo test -p l16_threads_channels consolidation
//! ```

/// 一组整数的统计结果。
///
/// `count` 是参与统计的整数个数；`min` 和 `max` 分别是其中最小和最大的
/// 值；`sum` 是全部整数的总和。
///
/// `sum` 使用 `i128` 而不是 `i64`：即使每个整数都在 `i64` 范围内，很多个
/// 整数相加也可能超出 `i64` 能表示的范围。`i128` 有更大的表示范围，这里
/// 用它避免把“统计总和”变成一次可能溢出的运算。
#[derive(Debug, PartialEq, Eq)]
pub struct Stats {
    pub count: usize,
    pub min: i64,
    pub max: i64,
    pub sum: i128,
}

/// 把一组整数分给若干工作线程，并行统计个数、最小值、最大值和总和。
///
/// 行为约定：
///
/// - `values` 为空时返回 `None`：没有任何整数，最小值、最大值都不存在；
/// - 否则返回 `Some(Stats)`，其中 `count` 等于 `values.len()`，每个位置的
///   整数都恰好参与一次统计，重复出现的整数按出现次数分别计数；
/// - `workers` 为 `0` 时按 `1` 个工作线程处理，不 panic；
/// - `workers` 大于整数个数时也可以正常工作，多出来的工作线程不应改变
///   结果；
/// - 参数是只读切片：函数只借用数据，不复制整个 `Vec`，调用后原数据仍可
///   使用；
/// - 工作线程 panic 时，让 panic 传播给调用者，而不是把它伪装成一个正常
///   的统计结果。
///
/// 本任务要求使用作用域线程（`thread::scope`）借用 `values` 的分段，不要
/// 为每个工作线程复制一份完整输入。
pub fn parallel_stats(values: &[i64], workers: usize) -> Option<Stats> {
    let _ = (&values, workers);
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_input_has_no_statistics() {
        assert_eq!(parallel_stats(&[], 3), None);
        assert_eq!(parallel_stats(&[], 0), None);
    }

    #[test]
    fn a_single_value_is_its_own_minimum_maximum_and_sum() {
        assert_eq!(
            parallel_stats(&[-5], 4),
            Some(Stats {
                count: 1,
                min: -5,
                max: -5,
                sum: -5,
            })
        );
    }

    #[test]
    fn several_values_are_counted_once_each_across_two_workers() {
        assert_eq!(
            parallel_stats(&[3, -1, 7, 7, 0], 2),
            Some(Stats {
                count: 5,
                min: -1,
                max: 7,
                sum: 16,
            })
        );
    }

    #[test]
    fn zero_workers_is_treated_as_one_worker() {
        let values = [4, -2, 9];

        assert_eq!(parallel_stats(&values, 0), parallel_stats(&values, 1));
        assert_eq!(
            parallel_stats(&values, 0),
            Some(Stats {
                count: 3,
                min: -2,
                max: 9,
                sum: 11,
            })
        );
    }

    #[test]
    fn more_workers_than_values_still_counts_every_value_once() {
        assert_eq!(
            parallel_stats(&[10, -10, 0], 100),
            Some(Stats {
                count: 3,
                min: -10,
                max: 10,
                sum: 0,
            })
        );
    }

    #[test]
    fn duplicate_values_are_counted_once_per_position() {
        assert_eq!(
            parallel_stats(&[4, 4, 4], 2),
            Some(Stats {
                count: 3,
                min: 4,
                max: 4,
                sum: 12,
            })
        );
    }

    #[test]
    fn sum_uses_i128_so_large_i64_values_do_not_overflow() {
        let values = [i64::MAX, i64::MIN, i64::MIN];
        let expected_sum = i128::from(i64::MAX) + i128::from(i64::MIN) * 2;

        assert_eq!(
            parallel_stats(&values, 2),
            Some(Stats {
                count: 3,
                min: i64::MIN,
                max: i64::MAX,
                sum: expected_sum,
            })
        );
    }

    #[test]
    fn all_negative_values_keep_the_correct_signs() {
        assert_eq!(
            parallel_stats(&[-9, -3, -12, -3], 3),
            Some(Stats {
                count: 4,
                min: -12,
                max: -3,
                sum: -27,
            })
        );
    }
}
