//! 主练习：表单检查流水线。
//!
//! 一个表单在提交前要经过若干检查：不能为空、不能超过长度上限、必须
//! 全部是数字。不同检查产出的结果类型不同（字符数、数字值），但调用方
//! 希望用同一套方式运行它们，并把失败原因统一报告出来。
//!
//! 请只替换各个 `todo!()`，不要修改类型定义、方法签名和文件底部的测试。
//! 每段注释描述的是可观察行为，不限定实现写法。

use std::fmt::Debug;

/// 检查失败的原因。
#[derive(Debug, PartialEq, Eq)]
pub enum CheckError {
    /// 输入为空，或去掉首尾空白后为空。
    Empty,
    /// 字符数超过了检查项允许的上限。
    TooLong { limit: usize, actual: usize },
    /// 输入包含非 `0` 到 `9` 的字符。
    NotDigits,
    /// 输入全是数字，但数值超出了 `u64` 能表示的范围。
    TooLarge,
}

/// 一项表单检查的行为契约。
///
/// 一个类型实现这个 trait，就承诺提供“名称、检查、说明”三件事。
pub trait Check {
    /// 检查通过时产出的值类型。
    ///
    /// 不同检查项可以产出不同类型：长度检查产出 `usize`，数字检查产出
    /// `u64`。这个类型由每个实现自己指定，调用方通过 `C::Value` 引用它。
    type Value;

    /// 检查项的名称，用于生成报告。
    fn label(&self) -> &str;

    /// 检查 `input`，通过时返回值，失败时返回具体原因。
    fn check(&self, input: &str) -> Result<Self::Value, CheckError>;

    /// 报告中的说明文字。
    ///
    /// 默认行为是把名称放进一对方括号，例如名称为 `非空` 时得到
    /// `[非空]`。实现可以覆盖它，给出更适合自己的说明。
    fn describe(&self) -> String {
        format!("[{}]", self.label())
    }
}

/// 检查输入去掉首尾空白后是否还有内容。
#[derive(Debug)]
pub struct NonEmpty;

impl Check for NonEmpty {
    /// 通过时产出“去掉首尾空白后的字符数”。
    type Value = usize;

    fn label(&self) -> &str {
        "非空"
    }

    /// 行为约定：
    ///
    /// - 先去掉输入首尾的空白字符（空格、制表符、换行都算）；
    /// - 剩下为空时返回 `Err(CheckError::Empty)`；
    /// - 否则返回 `Ok(剩余字符数)`，按字符计数，不按字节计数。
    ///
    /// 例如 `"  rust  "` 通过并得到 `4`，`"你好"` 得到 `2`，
    /// `"   "` 返回 `Err(Empty)`。
    fn check(&self, input: &str) -> Result<Self::Value, CheckError> {
        let _ = &input;
        todo!()
    }
}

/// 检查输入的字符数是否不超过上限。
#[derive(Debug)]
pub struct MaxChars {
    /// 允许的最大字符数。`0` 表示只接受空输入。
    pub limit: usize,
}

impl Check for MaxChars {
    /// 通过时产出输入的实际字符数。
    type Value = usize;

    fn label(&self) -> &str {
        "长度上限"
    }

    /// 行为约定：
    ///
    /// - 按字符计数，不按字节计数；
    /// - 首尾空白也计入长度（本检查不负责去空白）；
    /// - 实际字符数小于等于 `limit` 时返回 `Ok(实际字符数)`；
    /// - 超过时返回 `Err(CheckError::TooLong { limit, actual })`，
    ///   其中 `actual` 是实际字符数。
    ///
    /// 例如 `limit` 为 `3` 时，`"abc"` 通过并得到 `3`，`"abcd"` 返回
    /// `Err(TooLong { limit: 3, actual: 4 })`。
    fn check(&self, input: &str) -> Result<Self::Value, CheckError> {
        let _ = &input;
        todo!()
    }
}

/// 检查输入是否完全由十进制数字组成，并把它解析成整数。
#[derive(Debug)]
pub struct Digits;

impl Check for Digits {
    /// 通过时产出解析后的无符号整数。
    type Value = u64;

    fn label(&self) -> &str {
        "纯数字"
    }

    /// 行为约定：
    ///
    /// - 空输入返回 `Err(CheckError::Empty)`；
    /// - 只要出现一个不是 `0` 到 `9` 的字符（包括空格、正负号、小数点、
    ///   全角数字），就返回 `Err(CheckError::NotDigits)`；
    /// - 全部是数字但数值超出 `u64` 范围时返回 `Err(CheckError::TooLarge)`；
    /// - 否则返回 `Ok(解析出的数值)`，允许前导零。
    ///
    /// 例如 `"00123"` 得到 `123`，`"12a"` 返回 `Err(NotDigits)`，
    /// `"18446744073709551616"`（`u64` 最大值加一）返回 `Err(TooLarge)`。
    fn check(&self, input: &str) -> Result<Self::Value, CheckError> {
        let _ = &input;
        todo!()
    }

    /// 覆盖默认说明：数字检查需要额外提醒允许的字符范围。
    ///
    /// 固定返回 `纯数字（只接受 0-9）`。
    fn describe(&self) -> String {
        todo!()
    }
}

/// 把检查结果转换成报告文本的能力。
///
/// 这个 trait 定义在本 crate 里，所以可以为标准库的 `usize`、`u64`
/// 实现它；反过来，给标准库类型实现标准库 trait 会被孤儿规则拒绝。
pub trait ValueText {
    /// 返回适合放进报告的文本。
    fn value_text(&self) -> String;
}

impl ValueText for usize {
    /// 返回这个数的十进制文本，例如 `123` 返回 `"123"`。
    fn value_text(&self) -> String {
        let _ = &self;
        todo!()
    }
}

impl ValueText for u64 {
    /// 返回这个数的十进制文本，例如 `42` 返回 `"42"`。
    fn value_text(&self) -> String {
        let _ = &self;
        todo!()
    }
}

/// 擦除了关联类型的检查契约。
///
/// `Check` 有一个关联类型 `Value`，不同实现的 `Value` 不同，所以
/// `Box<dyn Check>` 无法同时表示三种检查。这个 trait 把结果统一成
/// `String`，从而可以放进同一个 `Vec<Box<dyn ErasedCheck>>`。
pub trait ErasedCheck: Debug {
    /// 检查项名称。
    fn label(&self) -> &str;

    /// 报告中的说明文字。
    fn describe(&self) -> String;

    /// 检查 `input`，通过时把结果转成文本，失败时保留具体原因。
    fn result_text(&self, input: &str) -> Result<String, CheckError>;
}

impl<T: Check + Debug> ErasedCheck for T
where
    T::Value: ValueText,
{
    fn label(&self) -> &str {
        Check::label(self)
    }

    fn describe(&self) -> String {
        Check::describe(self)
    }

    fn result_text(&self, input: &str) -> Result<String, CheckError> {
        Ok(Check::check(self, input)?.value_text())
    }
}

/// 一次成功检查的结构化结果。
#[derive(Debug, PartialEq, Eq)]
pub struct Outcome<V> {
    /// 检查项名称的副本。
    pub label: String,
    /// 检查产出的值。
    pub value: V,
    /// 检查项的说明文字。
    pub description: String,
}

/// 运行一项检查，把结果和检查项信息装进 `Outcome`。
///
/// 行为约定：
///
/// - 检查失败时原样返回该错误，不构造 `Outcome`；
/// - 检查通过时，`label` 是检查项名称的副本，`value` 是检查产出的
///   值，`description` 是该检查项的说明文字。
pub fn run_one<C: Check>(check: &C, input: &str) -> Result<Outcome<C::Value>, CheckError> {
    let _ = (&check, &input);
    todo!()
}

/// 依次运行一组检查，返回与输入顺序一一对应的结果。
///
/// 每一项要么是转换后的结果文本，要么是该项检查的具体错误；某一项失败
/// 不会影响后面的检查继续运行。空切片返回空 Vec。
pub fn run_all(checks: &[Box<dyn ErasedCheck>], input: &str) -> Vec<Result<String, CheckError>> {
    let _ = (&checks, &input);
    todo!()
}

#[cfg(test)]
mod tests {
    use super::{
        Check, CheckError, Digits, ErasedCheck, MaxChars, NonEmpty, Outcome, ValueText, run_all,
        run_one,
    };

    #[test]
    fn associated_value_types_are_distinct() {
        let _: fn(&NonEmpty, &str) -> Result<usize, CheckError> = Check::check;
        let _: fn(&MaxChars, &str) -> Result<usize, CheckError> = Check::check;
        let _: fn(&Digits, &str) -> Result<u64, CheckError> = Check::check;
    }

    #[test]
    fn non_empty_counts_characters_after_trimming() {
        assert_eq!(Check::check(&NonEmpty, "  rust  "), Ok(4));
        assert_eq!(Check::check(&NonEmpty, "a b"), Ok(3));
    }

    #[test]
    fn non_empty_counts_characters_not_bytes() {
        assert_eq!(Check::check(&NonEmpty, "你好"), Ok(2));
        assert_eq!(Check::check(&NonEmpty, "a你"), Ok(2));
    }

    #[test]
    fn non_empty_rejects_empty_and_blank_input() {
        assert_eq!(Check::check(&NonEmpty, ""), Err(CheckError::Empty));
        assert_eq!(Check::check(&NonEmpty, " \t\n "), Err(CheckError::Empty));
    }

    #[test]
    fn max_chars_accepts_text_at_the_limit() {
        let check = MaxChars { limit: 4 };
        assert_eq!(Check::check(&check, "abcd"), Ok(4));
        assert_eq!(Check::check(&check, ""), Ok(0));
    }

    #[test]
    fn max_chars_rejects_text_over_the_limit() {
        let check = MaxChars { limit: 3 };
        assert_eq!(
            Check::check(&check, "abcd"),
            Err(CheckError::TooLong {
                limit: 3,
                actual: 4
            })
        );
    }

    #[test]
    fn zero_limit_accepts_only_empty_input() {
        let check = MaxChars { limit: 0 };
        assert_eq!(Check::check(&check, ""), Ok(0));
        assert_eq!(
            Check::check(&check, "a"),
            Err(CheckError::TooLong {
                limit: 0,
                actual: 1
            })
        );
    }

    #[test]
    fn max_chars_counts_characters_not_bytes() {
        let check = MaxChars { limit: 2 };
        assert_eq!(Check::check(&check, "你好"), Ok(2));
        assert_eq!(
            Check::check(&check, "你好呀"),
            Err(CheckError::TooLong {
                limit: 2,
                actual: 3
            })
        );
    }

    #[test]
    fn digits_parses_decimal_text_and_allows_leading_zeros() {
        assert_eq!(Check::check(&Digits, "00123"), Ok(123));
        assert_eq!(Check::check(&Digits, "0"), Ok(0));
    }

    #[test]
    fn digits_rejects_empty_input() {
        assert_eq!(Check::check(&Digits, ""), Err(CheckError::Empty));
    }

    #[test]
    fn digits_rejects_non_digit_characters() {
        assert_eq!(Check::check(&Digits, "12a"), Err(CheckError::NotDigits));
        assert_eq!(Check::check(&Digits, " 12"), Err(CheckError::NotDigits));
        assert_eq!(Check::check(&Digits, "-12"), Err(CheckError::NotDigits));
        assert_eq!(Check::check(&Digits, "１２"), Err(CheckError::NotDigits));
    }

    #[test]
    fn digits_rejects_numbers_larger_than_u64() {
        assert_eq!(
            Check::check(&Digits, "18446744073709551616"),
            Err(CheckError::TooLarge)
        );
    }

    #[test]
    fn value_text_formats_both_number_types() {
        assert_eq!(123usize.value_text(), "123");
        assert_eq!(42u64.value_text(), "42");
    }

    #[test]
    fn default_describe_wraps_the_label_in_brackets() {
        assert_eq!(Check::describe(&NonEmpty), "[非空]");
        assert_eq!(Check::describe(&MaxChars { limit: 3 }), "[长度上限]");
    }

    #[test]
    fn digits_overrides_describe() {
        assert_eq!(Check::describe(&Digits), "纯数字（只接受 0-9）");
    }

    #[test]
    fn run_one_returns_value_and_check_information() {
        assert_eq!(
            run_one(&NonEmpty, "  rust  "),
            Ok(Outcome {
                label: String::from("非空"),
                value: 4,
                description: String::from("[非空]"),
            })
        );
    }

    #[test]
    fn run_one_propagates_the_specific_error() {
        let result: Result<Outcome<usize>, CheckError> = run_one(&MaxChars { limit: 2 }, "abc");
        assert_eq!(
            result,
            Err(CheckError::TooLong {
                limit: 2,
                actual: 3
            })
        );
    }

    #[test]
    fn run_all_preserves_order_and_keeps_going_after_an_error() {
        let checks: Vec<Box<dyn ErasedCheck>> = vec![
            Box::new(NonEmpty),
            Box::new(MaxChars { limit: 3 }),
            Box::new(Digits),
        ];

        assert_eq!(
            run_all(&checks, "1234"),
            vec![
                Ok(String::from("4")),
                Err(CheckError::TooLong {
                    limit: 3,
                    actual: 4
                }),
                Ok(String::from("1234")),
            ]
        );
    }

    #[test]
    fn run_all_on_an_empty_queue_returns_an_empty_vec() {
        let checks: Vec<Box<dyn ErasedCheck>> = Vec::new();
        assert!(run_all(&checks, "anything").is_empty());
    }

    #[test]
    fn erased_checks_can_be_formatted_with_debug() {
        let check: Box<dyn ErasedCheck> = Box::new(MaxChars { limit: 3 });
        assert!(format!("{check:?}").contains("MaxChars"));
    }
}
