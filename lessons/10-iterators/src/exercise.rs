//! 主练习：把一段日志文本转换成结构化数据。
//!
//! 日志的每一行形如 `"LEVEL: message"`，例如：
//!
//! ```text
//! INFO: service started
//! WARN: disk usage above 80%
//! ERROR: connection refused
//! ```
//!
//! 请只替换本文件中函数体里的 `todo!()`，不要修改类型定义、函数签名，也
//! 不要改动文件底部的测试。每个函数上方的注释就是它的完整行为约定。
//!
//! 完成后的验收命令见本课 README；最直接的一条是：
//!
//! ```bash
//! cargo test -p l10_iterators
//! ```

use std::collections::HashMap;

/// 日志级别。
///
/// 只识别三种级别；级别文本必须与变体一一对应，比较时区分大小写。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Level {
    /// `"INFO"`：普通信息。
    Info,
    /// `"WARN"`：需要留意的警告。
    Warn,
    /// `"ERROR"`：已经发生的错误。
    Error,
}

/// 解析成功的一行日志。
///
/// - `line`：这一行在原始输入中的行号，从 1 开始，由调用者传入；
/// - `level`：解析出的级别；
/// - `message`：去掉两端空白之后的日志正文，不包含 `LEVEL:` 前缀和分隔冒号。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogEntry {
    pub line: usize,
    pub level: Level,
    pub message: String,
}

/// 解析某一行失败时携带的信息。
///
/// - `line`：失败的行号，从 1 开始，原样保留调用者传入的值；
/// - `text`：这一行的原始内容，原样保存（包括两端空白），方便调用者向用户
///   展示是哪一行出了问题。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseLogError {
    pub line: usize,
    pub text: String,
}

/// 解析一行日志。
///
/// 行为约定（按此顺序判断）：
///
/// 1. 在 `content` 中找**第一个** `:`，把它之前的部分当级别文本、之后的部分
///    当日志正文。找不到 `:` 时返回 `Err`，`text` 是 `content` 的完整副本。
/// 2. 级别文本必须与 `"INFO"`、`"WARN"`、`"ERROR"` 之一**完全相同**；不去
///    两端空白，区分大小写。其他内容返回 `Err`，`text` 仍是完整原文。
/// 3. 日志正文去掉两端空白（空格、制表符、换行等）后不能为空；为空时返回
///    `Err`，`text` 仍是完整原文。
/// 4. 全部通过时返回 `Ok(LogEntry)`：`line` 原样保留，`message` 是去掉两端
///    空白后的正文，并成为拥有所有权的 `String`。
///
/// `line` 只是随结果携带的编号，本函数不检查它是否从 1 开始。
pub fn parse_line(line: usize, content: &str) -> Result<LogEntry, ParseLogError> {
    // 完成实现前避免“参数未使用”的提示，写好函数体后可删除。
    let _ = line;
    let _ = content;
    todo!()
}

/// 把多行日志全部解析成 `LogEntry`。
///
/// 行为约定：
///
/// - 第 1 行对应 `line = 1`，第 2 行对应 `line = 2`，依此类推；
/// - 所有行都能解析时，返回 `Ok`，其中 `LogEntry` 的顺序与输入顺序一致；
/// - 只要有一行无法解析，就返回 `Err`，并且返回的是**最靠前**的那一行的
///   错误；后续行是否合法不影响结果；
/// - 输入为空时返回 `Ok(空列表)`。
pub fn parse_all(lines: &[&str]) -> Result<Vec<LogEntry>, ParseLogError> {
    // 完成实现前避免“参数未使用”的提示，写好函数体后可删除。
    let _ = lines;
    todo!()
}

/// 同时收集所有成功解析的日志和所有失败的行。
///
/// 行为约定：
///
/// - 返回值的第一个 `Vec` 按输入顺序保存所有成功解析的 `LogEntry`；
/// - 第二个 `Vec` 按输入顺序保存所有失败行对应的 `ParseLogError`；
/// - 两边的行号都按原始输入从 1 开始计算；
/// - 空输入返回两个空列表；全部成功时第二个列表为空，全部失败时第一个
///   列表为空。
///
/// 与 [`parse_all`] 的差别：本函数不会在第一个错误处停下，而是把成功项和
/// 错误项都保留下来。
pub fn partition_logs(lines: &[&str]) -> (Vec<LogEntry>, Vec<ParseLogError>) {
    // 完成实现前避免“参数未使用”的提示，写好函数体后可删除。
    let _ = lines;
    todo!()
}

/// 取出所有 `ERROR` 级别日志的正文，保持原有顺序。
///
/// 返回值借用 `entries` 里的消息，不复制文本；调用结束后 `entries` 仍然
/// 完整可用。没有 `ERROR` 级别条目（包括输入为空）时返回空列表。
pub fn error_messages(entries: &[LogEntry]) -> Vec<&str> {
    // 完成实现前避免“参数未使用”的提示，写好函数体后可删除。
    let _ = entries;
    todo!()
}

/// 统计每种级别各出现多少次。
///
/// 行为约定：
///
/// - 键是输入中出现过的级别，值是该级别的条目数量；
/// - 没有出现过的级别不出现在结果中；
/// - 空输入得到空映射。
///
/// 计数结果与输入顺序无关；同一个键只会出现一次。
pub fn summarize(entries: &[LogEntry]) -> HashMap<Level, usize> {
    // 完成实现前避免“参数未使用”的提示，写好函数体后可删除。
    let _ = entries;
    todo!()
}

/// 返回第一条 `ERROR` 级别的日志。
///
/// 只借用 `entries` 中的元素，不复制；没有 `ERROR` 级别条目（包括输入为
/// 空）时返回 `None`。
pub fn first_error(entries: &[LogEntry]) -> Option<&LogEntry> {
    // 完成实现前避免“参数未使用”的提示，写好函数体后可删除。
    let _ = entries;
    todo!()
}

#[cfg(test)]
mod tests {
    use super::{
        Level, LogEntry, ParseLogError, error_messages, first_error, parse_all, parse_line,
        partition_logs, summarize,
    };

    fn entry(line: usize, level: Level, message: &str) -> LogEntry {
        LogEntry {
            line,
            level,
            message: String::from(message),
        }
    }

    fn error(line: usize, text: &str) -> ParseLogError {
        ParseLogError {
            line,
            text: String::from(text),
        }
    }

    // ---- parse_line：正常输入 ----

    #[test]
    fn parse_line_parses_all_three_levels() {
        assert_eq!(
            parse_line(1, "INFO: service started"),
            Ok(entry(1, Level::Info, "service started"))
        );
        assert_eq!(
            parse_line(2, "WARN: disk usage above 80%"),
            Ok(entry(2, Level::Warn, "disk usage above 80%"))
        );
        assert_eq!(
            parse_line(3, "ERROR: connection refused"),
            Ok(entry(3, Level::Error, "connection refused"))
        );
    }

    #[test]
    fn parse_line_splits_at_the_first_colon_only() {
        assert_eq!(
            parse_line(4, "WARN: retry at 12:30:45"),
            Ok(entry(4, Level::Warn, "retry at 12:30:45"))
        );
    }

    #[test]
    fn parse_line_trims_the_message() {
        assert_eq!(
            parse_line(5, "ERROR:   disk full  "),
            Ok(entry(5, Level::Error, "disk full"))
        );
    }

    // ---- parse_line：错误输入 ----

    #[test]
    fn parse_line_rejects_a_line_without_a_colon() {
        assert_eq!(parse_line(6, "just noise"), Err(error(6, "just noise")));
    }

    #[test]
    fn parse_line_rejects_an_unknown_or_differently_cased_level() {
        for content in ["DEBUG: chatty", "info: lower case", " INFO: leading space"] {
            assert_eq!(parse_line(7, content), Err(error(7, content)));
        }
    }

    #[test]
    fn parse_line_rejects_an_empty_message() {
        for content in ["INFO:", "ERROR:   ", "WARN: \t "] {
            assert_eq!(parse_line(8, content), Err(error(8, content)));
        }
    }

    #[test]
    fn parse_line_keeps_the_original_text_in_errors() {
        assert_eq!(
            parse_line(9, "  DEBUG: padded  "),
            Err(error(9, "  DEBUG: padded  "))
        );
    }

    // ---- parse_all：正常、边界与错误 ----

    #[test]
    fn parse_all_numbers_lines_from_one() {
        let lines = ["INFO: started", "WARN: low disk", "ERROR: stopped"];

        assert_eq!(
            parse_all(&lines),
            Ok(vec![
                entry(1, Level::Info, "started"),
                entry(2, Level::Warn, "low disk"),
                entry(3, Level::Error, "stopped"),
            ])
        );
    }

    #[test]
    fn parse_all_returns_ok_for_empty_input() {
        assert_eq!(parse_all(&[]), Ok(Vec::new()));
    }

    #[test]
    fn parse_all_reports_the_first_invalid_line() {
        let lines = ["not a log line", "INFO: never reached"];

        assert_eq!(parse_all(&lines), Err(error(1, "not a log line")));
    }

    #[test]
    fn parse_all_reports_an_invalid_line_in_the_middle() {
        let lines = ["INFO: ok", "bad line", "ERROR: never reached"];

        assert_eq!(
            parse_all(&lines),
            Err(ParseLogError {
                line: 2,
                text: String::from("bad line"),
            })
        );
    }

    // ---- partition_logs ----

    #[test]
    fn partition_logs_keeps_both_sides_in_input_order() {
        let lines = [
            "INFO: started",
            "bad line one",
            "ERROR: failed",
            "bad line two",
            "WARN: careful",
        ];

        let (entries, errors) = partition_logs(&lines);

        assert_eq!(
            entries,
            vec![
                entry(1, Level::Info, "started"),
                entry(3, Level::Error, "failed"),
                entry(5, Level::Warn, "careful"),
            ]
        );
        assert_eq!(
            errors,
            vec![error(2, "bad line one"), error(4, "bad line two")]
        );
    }

    #[test]
    fn partition_logs_handles_empty_input() {
        assert_eq!(
            partition_logs(&[]),
            (Vec::<LogEntry>::new(), Vec::<ParseLogError>::new())
        );
    }

    #[test]
    fn partition_logs_returns_only_entries_when_every_line_is_valid() {
        let lines = ["INFO: started", "WARN: careful"];

        let (entries, errors) = partition_logs(&lines);

        assert_eq!(
            entries,
            vec![
                entry(1, Level::Info, "started"),
                entry(2, Level::Warn, "careful"),
            ]
        );
        assert!(errors.is_empty());
    }

    #[test]
    fn partition_logs_returns_only_errors_when_every_line_is_invalid() {
        let lines = ["bad one", "DEBUG: bad two"];

        let (entries, errors) = partition_logs(&lines);

        assert!(entries.is_empty());
        assert_eq!(
            errors,
            vec![error(1, "bad one"), error(2, "DEBUG: bad two")]
        );
    }

    // ---- error_messages ----

    #[test]
    fn error_messages_returns_only_error_messages_in_order() {
        let entries = vec![
            entry(1, Level::Info, "started"),
            entry(2, Level::Error, "disk full"),
            entry(3, Level::Warn, "careful"),
            entry(4, Level::Error, "connection refused"),
        ];

        assert_eq!(
            error_messages(&entries),
            ["disk full", "connection refused"]
        );
        // 返回的是借用：原数据仍然完整可用。
        assert_eq!(entries.len(), 4);
        assert_eq!(entries[3].message, "connection refused");
    }

    #[test]
    fn error_messages_is_empty_when_there_are_no_errors() {
        let entries = vec![
            entry(1, Level::Info, "started"),
            entry(2, Level::Warn, "careful"),
        ];

        assert!(error_messages(&entries).is_empty());
        assert!(error_messages(&[]).is_empty());
    }

    // ---- summarize ----

    #[test]
    fn summarize_counts_each_present_level() {
        let entries = vec![
            entry(1, Level::Info, "a"),
            entry(2, Level::Warn, "b"),
            entry(3, Level::Error, "c"),
            entry(4, Level::Error, "d"),
            entry(5, Level::Warn, "e"),
        ];

        let summary = summarize(&entries);

        assert_eq!(summary.get(&Level::Info), Some(&1));
        assert_eq!(summary.get(&Level::Warn), Some(&2));
        assert_eq!(summary.get(&Level::Error), Some(&2));
        assert_eq!(summary.len(), 3);
    }

    #[test]
    fn summarize_omits_levels_that_never_appear() {
        let entries = vec![entry(1, Level::Warn, "only warnings")];

        let summary = summarize(&entries);

        assert_eq!(summary.get(&Level::Warn), Some(&1));
        assert_eq!(summary.get(&Level::Info), None);
        assert_eq!(summary.get(&Level::Error), None);
        assert_eq!(summary.len(), 1);
    }

    #[test]
    fn summarize_returns_an_empty_map_for_empty_input() {
        assert!(summarize(&[]).is_empty());
    }

    // ---- first_error ----

    #[test]
    fn first_error_returns_the_first_error_entry() {
        let entries = vec![
            entry(1, Level::Info, "started"),
            entry(2, Level::Error, "disk full"),
            entry(3, Level::Error, "connection refused"),
        ];

        assert_eq!(first_error(&entries), Some(&entries[1]));
        // 返回的是借用：原数据仍然完整可用。
        assert_eq!(entries.len(), 3);
    }

    #[test]
    fn first_error_returns_none_when_no_entry_is_an_error() {
        let entries = vec![
            entry(1, Level::Info, "started"),
            entry(2, Level::Warn, "careful"),
        ];

        assert_eq!(first_error(&entries), None);
        assert_eq!(first_error(&[]), None);
    }
}
