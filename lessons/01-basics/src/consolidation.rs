//! 课后巩固：数字步数与数位之和。
//!
//! 这两个函数不使用区间数据，边界也与主练习不同。完成主练习并复盘
//! 之后再来实现它们。同样只替换 `todo!()`，不要改签名和测试。

/// 计算从 `start` 出发，按以下规则反复变换，到第一次得到 `1` 为止
/// 需要的变换次数：
///
/// - 当前数字是偶数时，把它除以 `2`；
/// - 当前数字是奇数时，把它乘以 `3` 再加 `1`。
///
/// 边界与非法输入：
///
/// - `start` 已经是 `1` 时一次都不用变换，返回 `0`；
/// - `start == 0` 无法按规则到达 `1`，直接返回 `0`，不做任何变换。
///
/// 可以假设测试输入在到达 `1` 之前不会超出 `u32` 的范围。
pub fn collatz_steps(start: u32) -> u32 {
    // 完成实现前避免“参数未使用”的提示，写好函数体后可删除。
    let _ = start;
    todo!()
}

/// 计算 `value` 的十进制表示中每一位数字的和。
///
/// 结果只看数字本身，忽略正负号：负数和它的相反数得到相同结果；
/// 数字 `0` 的数位之和是 `0`；只有一位的数字返回它自身（忽略符号）。
///
/// 可以假设 `value` 不是 `i32` 能表示的最小值，因此取绝对值是安全的。
pub fn sum_of_digits(value: i32) -> u32 {
    // 完成实现前避免“参数未使用”的提示，写好函数体后可删除。
    let _ = value;
    todo!()
}

#[cfg(test)]
mod tests {
    use super::{collatz_steps, sum_of_digits};

    #[test]
    fn collatz_steps_counts_transformations_until_one() {
        // 6 → 3 → 10 → 5 → 16 → 8 → 4 → 2 → 1，共 8 次。
        assert_eq!(collatz_steps(6), 8);
        // 3 → 10 → 5 → 16 → 8 → 4 → 2 → 1，共 7 次。
        assert_eq!(collatz_steps(3), 7);
        assert_eq!(collatz_steps(2), 1);
    }

    #[test]
    fn collatz_steps_returns_zero_for_one_and_for_zero() {
        assert_eq!(collatz_steps(1), 0);
        assert_eq!(collatz_steps(0), 0);
    }

    #[test]
    fn sum_of_digits_adds_decimal_digits() {
        assert_eq!(sum_of_digits(123), 6);
        assert_eq!(sum_of_digits(907), 16);
        assert_eq!(sum_of_digits(7), 7);
    }

    #[test]
    fn sum_of_digits_ignores_sign_and_handles_zero() {
        assert_eq!(sum_of_digits(0), 0);
        assert_eq!(sum_of_digits(-123), 6);
        assert_eq!(sum_of_digits(-7), 7);
    }
}
