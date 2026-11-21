//! 主练习：整数区间分析器。
//!
//! 三个函数都以闭区间 `[start, end]`（两端的值都算在内）为输入。
//! 请只替换函数体里的 `todo!()`，不要修改函数签名，也不要改动本文件
//! 底部的测试。每个函数上方的注释就是它的完整行为约定。

/// 把 `value` 限制在闭区间 `[minimum, maximum]` 内。
///
/// - `value` 低于 `minimum` 时，返回 `minimum`；
/// - `value` 高于 `maximum` 时，返回 `maximum`；
/// - 其余情况（包括恰好等于某一端）返回 `value` 本身。
///
/// 可以假设调用方保证 `minimum <= maximum`。
pub fn clamp(value: i32, minimum: i32, maximum: i32) -> i32 {
    // 下面这行只是在你完成实现前，避免编译器提示“参数还没被使用”；
    // 写好函数体后可以把它删掉。
    let _ = (value, minimum, maximum);
    todo!()
}

/// 计算闭区间 `[start, end]` 内所有整数依次相加的和。
///
/// - 区间为空（`start > end`）时返回 `0`；
/// - 区间内每个整数都恰好参与一次加法，包括负数和零；
/// - 只有一个整数的区间返回这个整数本身。
///
/// 可以假设输入足够小，累加结果不会超出 `i32` 的范围。
pub fn sum_inclusive(start: i32, end: i32) -> i32 {
    // 完成实现前避免“参数未使用”的提示，写好函数体后可删除。
    let _ = (start, end);
    todo!()
}

/// 统计闭区间 `[start, end]` 内有多少个整数能被 `divisor` 整除。
///
/// “能被整除”指这个整数除以 `divisor` 的余数为 `0`；`0` 能被任何
/// 非零除数整除。返回满足条件的整数个数，个数不会是负数。
///
/// 两种输入没有可统计的对象，直接返回 `0`：
///
/// - 区间为空（`start > end`）；
/// - `divisor == 0`（此时对任何值做除法或取余都没有意义，必须避免）。
pub fn count_multiples(start: i32, end: i32, divisor: i32) -> u32 {
    // 完成实现前避免“参数未使用”的提示，写好函数体后可删除。
    let _ = (start, end, divisor);
    todo!()
}

#[cfg(test)]
mod tests {
    use super::{clamp, count_multiples, sum_inclusive};

    #[test]
    fn clamp_returns_minimum_when_value_is_below_range() {
        assert_eq!(clamp(-4, 0, 10), 0);
        assert_eq!(clamp(-100, 1, 5), 1);
    }

    #[test]
    fn clamp_returns_maximum_when_value_is_above_range() {
        assert_eq!(clamp(14, 0, 10), 10);
        assert_eq!(clamp(1000, -3, -1), -1);
    }

    #[test]
    fn clamp_keeps_value_inside_range_and_at_bounds() {
        assert_eq!(clamp(7, 0, 10), 7);
        assert_eq!(clamp(0, 0, 10), 0);
        assert_eq!(clamp(10, 0, 10), 10);
        assert_eq!(clamp(-2, -2, -2), -2);
    }

    #[test]
    fn sum_inclusive_adds_every_integer_in_range() {
        assert_eq!(sum_inclusive(1, 4), 10);
        assert_eq!(sum_inclusive(-2, 2), 0);
        assert_eq!(sum_inclusive(-5, -3), -12);
        assert_eq!(sum_inclusive(7, 7), 7);
    }

    #[test]
    fn sum_inclusive_returns_zero_for_empty_range() {
        assert_eq!(sum_inclusive(5, 3), 0);
        assert_eq!(sum_inclusive(0, -1), 0);
    }

    #[test]
    fn count_multiples_counts_values_divisible_by_divisor() {
        assert_eq!(count_multiples(1, 10, 2), 5);
        assert_eq!(count_multiples(-3, 3, 3), 3);
        assert_eq!(count_multiples(7, 7, 7), 1);
        assert_eq!(count_multiples(8, 8, 7), 0);
    }

    #[test]
    fn count_multiples_returns_zero_for_empty_range_or_zero_divisor() {
        assert_eq!(count_multiples(5, 3, 2), 0);
        assert_eq!(count_multiples(1, 10, 0), 0);
        assert_eq!(count_multiples(0, 0, 0), 0);
    }
}
