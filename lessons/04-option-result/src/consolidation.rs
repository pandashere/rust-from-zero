//! 课后巩固：解析 `"90s"`、`"5m"`、`"2h"` 这类时长文本，算出总秒数。
//!
//! 这是主练习之后的独立任务：数据不同（时长而不是网络地址），边界也
//! 不同（“没有单位”“单位不认识”“乘法溢出”）。请只替换两个函数体里的
//! `todo!()`，不要修改类型定义、函数签名，也不要改动本文件底部的测试。
//! 每个函数上方的注释就是它的完整行为约定。
//!
//! 完成后的验收命令：
//!
//! ```bash
//! cargo test -p l04_option_result consolidation
//! ```

/// 解析时长文本时可能出现的失败原因。
#[derive(Debug, PartialEq, Eq)]
pub enum DurationError {
    /// 整个输入去掉两端空白后没有任何字符。
    Empty,
    /// 文本开头没有数字部分，例如 `"s"`、`"m"`。
    MissingNumber,
    /// 数字部分写不出来或超出 `u64` 能表示的范围；附带这段数字文本本身。
    InvalidNumber(String),
    /// 只有数字、没有任何单位，例如 `"90"`。
    MissingUnit,
    /// 单位不是支持的 `"s"`、`"m"`、`"h"`；附带这段单位文本本身。
    UnknownUnit(String),
    /// 数字和单位都合法，但换算成秒之后超出 `u64` 能表示的范围。
    Overflow,
}

/// 返回一个时间单位相当于多少秒。
///
/// - `"s"` 表示秒，相当于 1 秒；
/// - `"m"` 表示分钟，相当于 60 秒；
/// - `"h"` 表示小时，相当于 3600 秒；
/// - 其他任何单位（包括空文本）都不被支持，用“没有值”表示。
///
/// 返回值是 `Option<u64>`：支持的单位返回 `Some(秒数)`，不支持的单位
/// 返回 `None`。
pub fn unit_seconds(unit: &str) -> Option<u64> {
    // 完成实现前避免“参数未使用”的提示，写好函数体后可删除。
    let _ = unit;
    todo!()
}

/// 把时长文本解析成总秒数。
///
/// 输入由“数字部分”和“单位部分”组成：数字部分是开头的连续十进制
/// 数字（0-9），单位部分是数字之后剩下的内容。行为约定（按此顺序
/// 判断，命中即返回）：
///
/// 1. 整个 `input` 去掉两端空白后为空：`Err(DurationError::Empty)`。
/// 2. 开头没有数字：`Err(DurationError::MissingNumber)`。
/// 3. 数字部分解析不成 `u64`（超出表示范围）：
///    `Err(DurationError::InvalidNumber(数字文本))`，附带数字部分原文。
/// 4. 数字之后没有任何内容：`Err(DurationError::MissingUnit)`。
/// 5. 单位部分去掉两端空白后不是支持的单位：
///    `Err(DurationError::UnknownUnit(单位文本))`，附带去掉空白后的单位。
/// 6. 数值换算成秒时超出 `u64` 范围：`Err(DurationError::Overflow)`。
/// 7. 全部通过时返回 `Ok(总秒数)`；`"0s"` 是合法输入，结果为 `0`。
///
/// 约束：不允许用 `unwrap`、`expect` 或 `panic!` 处理输入；函数体中至少
/// 使用一次 `?` 来传播失败。
pub fn parse_duration(input: &str) -> Result<u64, DurationError> {
    // 完成实现前避免“参数未使用”的提示，写好函数体后可删除。
    let _ = input;
    todo!()
}

#[cfg(test)]
mod tests {
    use super::{DurationError, parse_duration, unit_seconds};

    #[test]
    fn unit_seconds_recognizes_supported_units() {
        assert_eq!(unit_seconds("s"), Some(1));
        assert_eq!(unit_seconds("m"), Some(60));
        assert_eq!(unit_seconds("h"), Some(3600));
    }

    #[test]
    fn unit_seconds_returns_none_for_unsupported_units() {
        assert_eq!(unit_seconds("d"), None);
        assert_eq!(unit_seconds("S"), None);
        assert_eq!(unit_seconds(""), None);
    }

    #[test]
    fn parse_duration_converts_each_unit_to_seconds() {
        assert_eq!(parse_duration("90s"), Ok(90));
        assert_eq!(parse_duration("5m"), Ok(300));
        assert_eq!(parse_duration(" 2h "), Ok(7200));
        assert_eq!(parse_duration("0s"), Ok(0));
    }

    #[test]
    fn parse_duration_rejects_empty_input() {
        assert_eq!(parse_duration(""), Err(DurationError::Empty));
        assert_eq!(parse_duration("   "), Err(DurationError::Empty));
    }

    #[test]
    fn parse_duration_requires_a_leading_number() {
        assert_eq!(parse_duration("s"), Err(DurationError::MissingNumber));
        assert_eq!(parse_duration("m"), Err(DurationError::MissingNumber));
    }

    #[test]
    fn parse_duration_requires_a_unit_after_the_number() {
        assert_eq!(parse_duration("90"), Err(DurationError::MissingUnit));
    }

    #[test]
    fn parse_duration_reports_unknown_unit_with_its_text() {
        assert_eq!(
            parse_duration("10d"),
            Err(DurationError::UnknownUnit(String::from("d")))
        );
        assert_eq!(
            parse_duration("10 d"),
            Err(DurationError::UnknownUnit(String::from("d")))
        );
    }

    #[test]
    fn parse_duration_reports_number_beyond_u64() {
        let huge = "99999999999999999999999999";
        assert_eq!(
            parse_duration(&format!("{huge}s")),
            Err(DurationError::InvalidNumber(String::from(huge)))
        );
    }

    #[test]
    fn parse_duration_detects_overflow_when_scaling_to_seconds() {
        // u64 能表示的最大值本身可以解析，但乘上 3600 秒就放不下了。
        assert_eq!(
            parse_duration("18446744073709551615h"),
            Err(DurationError::Overflow)
        );
    }
}
