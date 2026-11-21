//! 课后巩固：一组测量值的平均数与中位数。
//!
//! 这里的数据和主练习完全不同：主练习处理文本标题和整数秒数，这里处理
//! 浮点数（`f64`，带小数的数字）并涉及舍入误差与容差比较。

/// 计算一组测量值的算术平均数（平均值），也就是“总和除以个数”。
///
/// 返回值是 `Option<f64>` 而不是 `f64`：`values` 为空时没有“个数”可除，
/// 平均值不存在，此时返回 `None`。这个设计来自第四课——把“可能没有值”
/// 写进类型，调用者就不得不处理它。
///
/// 输入不包含 `NaN`（“不是一个数字”，浮点数里表示无效计算结果的特殊
/// 值）；本课的测试也不使用它。
///
/// # 例子
///
/// ```rust,no_run
/// use l08_testing_quality::statistics::mean;
///
/// assert_eq!(mean(&[]), None);
/// assert_eq!(mean(&[2.0, 4.0, 6.0]), Some(4.0));
/// ```
//
// 这个例子标记为 `no_run`：它必须能编译，但执行要等你完成本函数。
pub fn mean(values: &[f64]) -> Option<f64> {
    let _ = &values;
    todo!("实现 mean：空输入返回 None，否则返回总和除以个数")
}

/// 计算一组测量值的中位数。
///
/// 中位数是“排好序之后正中间的那个数”，它不受个别极端值影响，因此和
/// 平均数一起看能发现偏斜的数据。
///
/// 契约：
///
/// - `values` 为空时返回 `None`；
/// - 个数为奇数时，返回排序后正中间的那个值；
/// - 个数为偶数时，返回排序后中间两个值的平均数；
/// - 输入的顺序任意，函数自己排序；参数是只读借用，不能修改调用者的
///   数据，所以排序必须发生在一份副本上。
///
/// 输入不包含 `NaN`，理由同 [`mean`]。
///
/// # 例子
///
/// ```rust,no_run
/// use l08_testing_quality::statistics::median;
///
/// assert_eq!(median(&[9.0, 1.0, 5.0]), Some(5.0));
/// assert_eq!(median(&[4.0, 1.0, 3.0, 2.0]), Some(2.5));
/// ```
//
// 这个例子同样标记为 `no_run`，原因与 `mean` 相同。
pub fn median(values: &[f64]) -> Option<f64> {
    let _ = &values;
    todo!("实现 median：空输入返回 None，否则排序副本后取中间值或中间两个值的平均")
}

#[cfg(test)]
mod tests {
    use super::{mean, median};

    /// 浮点数的比较容差（tolerance，允许的误差范围）。
    ///
    /// 十进制小数在二进制里通常存不下精确值，`0.1 + 0.2` 并不等于
    /// `0.3`，所以涉及算术的浮点断言比较“差是否足够小”，而不是
    /// 要求两个数完全相等。
    const TOLERANCE: f64 = 1e-12;

    /// 断言一个 `Option<f64>` 结果：`None` 必须与 `None` 配对，`Some`
    /// 则按容差比较。
    fn assert_option_close(actual: Option<f64>, expected: Option<f64>, case: &str) {
        match (actual, expected) {
            (None, None) => {}
            (Some(actual), Some(expected)) => {
                assert!(
                    (actual - expected).abs() < TOLERANCE,
                    "case {case:?}：期望 {expected}，实际 {actual}"
                );
            }
            (actual, expected) => {
                panic!("case {case:?}：期望 {expected:?}，实际 {actual:?}")
            }
        }
    }

    // ---- mean ----

    #[test]
    fn mean_of_an_empty_slice_is_none() {
        assert_eq!(mean(&[]), None);
    }

    #[test]
    fn mean_of_a_single_value_is_that_value() {
        assert_eq!(mean(&[7.5]), Some(7.5));
    }

    #[test]
    fn mean_mixes_negative_and_positive_values() {
        // (-4 + 1 + 3) / 3 = 0
        assert_option_close(mean(&[-4.0, 1.0, 3.0]), Some(0.0), "正负混合");
    }

    #[test]
    fn mean_needs_a_tolerance_for_repeating_decimals() {
        // 三个 0.1 相加在二进制浮点里并不精确等于 0.3。
        assert_option_close(mean(&[0.1, 0.1, 0.1]), Some(0.1), "循环小数");
        assert_option_close(
            mean(&[0.1, 0.2]),
            Some(0.150_000_000_000_000_02),
            "0.1 与 0.2 的平均",
        );
    }

    #[test]
    fn mean_of_equal_values_is_that_value() {
        assert_eq!(mean(&[2.5, 2.5, 2.5, 2.5]), Some(2.5));
    }

    // ---- median ----

    #[test]
    fn median_of_an_empty_slice_is_none() {
        assert_eq!(median(&[]), None);
    }

    #[test]
    fn median_of_an_odd_count_is_the_middle_value_after_sorting() {
        assert_eq!(median(&[9.0, 1.0, 5.0]), Some(5.0));
    }

    #[test]
    fn median_of_an_even_count_averages_the_two_middle_values() {
        assert_eq!(median(&[4.0, 1.0, 3.0, 2.0]), Some(2.5));
    }

    #[test]
    fn median_of_a_single_value_is_that_value() {
        assert_eq!(median(&[42.0]), Some(42.0));
    }

    #[test]
    fn median_handles_duplicates_and_negatives() {
        assert_eq!(median(&[-1.0, -1.0, 2.0]), Some(-1.0));
        assert_eq!(median(&[5.0, 5.0, 5.0]), Some(5.0));
    }

    #[test]
    fn median_leaves_the_callers_data_untouched() {
        let values = [3.0, 1.0, 2.0];

        assert_eq!(median(&values), Some(2.0));
        assert_eq!(values, [3.0, 1.0, 2.0], "输入切片被修改了");
    }

    #[test]
    fn median_of_two_values_is_their_mean() {
        assert_eq!(median(&[8.0, 2.0]), Some(5.0));
    }

    // ---- 表格驱动：两个函数的多种输入 ----

    #[test]
    fn statistics_table_covers_normal_boundary_and_error_cases() {
        struct Case {
            name: &'static str,
            values: &'static [f64],
            expected_mean: Option<f64>,
            expected_median: Option<f64>,
        }

        let cases = [
            Case {
                name: "空切片",
                values: &[],
                expected_mean: None,
                expected_median: None,
            },
            Case {
                name: "单个值",
                values: &[3.25],
                expected_mean: Some(3.25),
                expected_median: Some(3.25),
            },
            Case {
                name: "升序偶数个",
                values: &[1.0, 2.0, 3.0, 4.0],
                expected_mean: Some(2.5),
                expected_median: Some(2.5),
            },
            Case {
                name: "乱序奇数个",
                values: &[100.0, -10.0, 0.0],
                expected_mean: Some(30.0),
                expected_median: Some(0.0),
            },
            Case {
                name: "全部为负",
                values: &[-5.0, -1.0, -3.0],
                expected_mean: Some(-3.0),
                expected_median: Some(-3.0),
            },
            Case {
                name: "重复值",
                values: &[2.0, 2.0, 2.0, 4.0],
                expected_mean: Some(2.5),
                expected_median: Some(2.0),
            },
        ];

        for case in cases {
            assert_option_close(mean(case.values), case.expected_mean, case.name);
            assert_option_close(median(case.values), case.expected_median, case.name);
        }
    }
}
