//! 课后巩固：给待发送的消息编号、贴标签，并做一次性汇总。
//!
//! 主练习处理的是“调用者把闭包传进函数”。这个巩固任务换一组数据和
//! 边界，练习反方向：由函数**创建**闭包并返回给调用者。你会分别返回
//! 一个带可变状态的编号器、一个拥有 `String` 前缀的贴标签器，以及一个
//! 只会执行一次的汇总器。
//!
//! 请只替换函数体里的 `todo!()`。不要修改函数签名，也不要改动本文件
//! 底部的测试。每个函数上方的注释就是它的完整行为约定。
//!
//! 完成后的验收命令：
//!
//! ```bash
//! cargo test -p l09_closures consolidation
//! ```

/// 返回一个“编号器”闭包：每调用一次，产出一个编号。
///
/// 行为约定：
///
/// - 第一次调用返回 `start`；
/// - 之后每次调用返回上一次产出的编号加 1；
/// - 编号到达 `u32` 能表示的最大值后保持不变（再调用仍返回这个最大值），
///   不会因为加一而溢出；
/// - 每次调用本函数都得到一个独立的编号器，多个编号器互不影响。
pub fn make_counter(start: u32) -> impl FnMut() -> u32 {
    // 完成实现前避免“参数未使用”的提示；把整个闭包表达式替换成实现。
    let _ = &start;
    || todo!()
}

/// 返回一个“贴标签器”闭包，它拥有传入的 `prefix`。
///
/// 行为约定：
///
/// - 对任意文本 `text`，返回 `"{prefix}: {text}"`：先是前缀，接着一个
///   冒号和一个空格，最后是原文；
/// - `text` 为空文本时仍保留前缀和分隔符。例如前缀是 `"note"` 时，
///   对空文本返回 `"note: "`；
/// - 闭包可以被反复调用：每次调用都使用同一份前缀，不会把前缀消耗掉。
pub fn make_labeler(prefix: String) -> impl Fn(&str) -> String {
    // 完成实现前避免“参数未使用”的提示；把整个闭包表达式替换成实现。
    let _ = &prefix;
    |_| todo!()
}

/// 返回一个“一次性汇总”闭包，它接管整批消息。
///
/// 行为约定：
///
/// - 调用一次，把所有消息按原来的先后顺序用 `" | "` 连接成一个 `String`；
/// - 消息内容原样保留，不在开头或结尾额外添加分隔符；
/// - 空消息也占一个位置，例如两条空消息连接后是 `" | "`；
/// - 一批消息为空时返回空 `String`；
/// - 返回的闭包只会被调用一次：它可以把捕获的整批消息消耗掉。
pub fn make_outbox_summary(messages: Vec<String>) -> impl FnOnce() -> String {
    // 完成实现前避免“参数未使用”的提示；把整个闭包表达式替换成实现。
    let _ = &messages;
    || todo!()
}

#[cfg(test)]
mod tests {
    use super::{make_counter, make_labeler, make_outbox_summary};

    #[test]
    fn counter_starts_at_the_given_number() {
        let mut counter = make_counter(7);

        assert_eq!(counter(), 7);
    }

    #[test]
    fn counter_advances_on_each_call() {
        let mut counter = make_counter(0);

        assert_eq!(counter(), 0);
        assert_eq!(counter(), 1);
        assert_eq!(counter(), 2);
    }

    #[test]
    fn counters_from_separate_calls_are_independent() {
        let mut first = make_counter(10);
        let mut second = make_counter(100);

        assert_eq!(first(), 10);
        assert_eq!(second(), 100);
        assert_eq!(first(), 11);
        assert_eq!(second(), 101);
    }

    #[test]
    fn counter_stops_at_the_largest_u32_value() {
        let mut at_max = make_counter(u32::MAX);
        assert_eq!(at_max(), u32::MAX);
        assert_eq!(at_max(), u32::MAX);

        let mut just_below = make_counter(u32::MAX - 1);
        assert_eq!(just_below(), u32::MAX - 1);
        assert_eq!(just_below(), u32::MAX);
        assert_eq!(just_below(), u32::MAX);
    }

    #[test]
    fn labeler_prefixes_each_message() {
        let label = make_labeler(String::from("note"));

        assert_eq!(label("hello"), "note: hello");
    }

    #[test]
    fn labeler_keeps_the_prefix_across_calls() {
        let label = make_labeler(String::from("log"));

        assert_eq!(label("start"), "log: start");
        assert_eq!(label("stop"), "log: stop");
    }

    #[test]
    fn labeler_handles_an_empty_message() {
        let label = make_labeler(String::from("note"));

        assert_eq!(label(""), "note: ");
    }

    #[test]
    fn summary_joins_messages_in_their_original_order() {
        let summary = make_outbox_summary(vec![
            String::from("alpha"),
            String::from("beta"),
            String::from("gamma"),
        ]);

        assert_eq!(summary(), "alpha | beta | gamma");
    }

    #[test]
    fn summary_of_an_empty_batch_is_empty() {
        let summary = make_outbox_summary(Vec::new());

        assert_eq!(summary(), "");
    }

    #[test]
    fn summary_keeps_empty_messages_as_positions() {
        let summary =
            make_outbox_summary(vec![String::new(), String::from("middle"), String::new()]);

        assert_eq!(summary(), " | middle | ");
    }

    #[test]
    fn summary_handles_a_single_message() {
        let summary = make_outbox_summary(vec![String::from("solo")]);

        assert_eq!(summary(), "solo");
    }
}
