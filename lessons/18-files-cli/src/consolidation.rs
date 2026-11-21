//! 课后巩固：整理文本文件，把清理后的内容写到另一个路径。
//!
//! 这是主练习之后的独立任务：主练习只读文件并统计，这里要读入、转换、
//! 再写出。数据不同（文本内容而不是统计指标），边界也不同（空行、行尾
//! 空白、空文件、输出路径不可写）。
//!
//! 请只替换两个函数体里的 `todo!()`，不要修改类型定义、函数签名，也不要
//! 改动本文件底部的测试。每个函数上方的注释就是它的完整行为约定。
//!
//! 完成后的验收命令：
//!
//! ```bash
//! cargo test -p l18_files_cli consolidation
//! ```

use std::fmt;
use std::io;
use std::path::Path;

/// 整理结果：写出了多少行、丢掉了多少行。
#[derive(Debug, PartialEq, Eq)]
pub struct TidyReport {
    pub written_lines: usize,
    pub removed_lines: usize,
}

/// 整理文件时可能出现的失败原因。每个失败都保留相关路径和底层错误。
#[derive(Debug)]
pub enum TidyError {
    /// 无法读取输入文件。
    Read {
        path: std::path::PathBuf,
        source: io::Error,
    },
    /// 无法写出输出文件，包括父目录不存在、没有权限、目标是目录等情况。
    Write {
        path: std::path::PathBuf,
        source: io::Error,
    },
    /// 输入文件的字节不是合法 UTF-8 文本。
    InvalidUtf8 {
        path: std::path::PathBuf,
        source: std::str::Utf8Error,
    },
}

impl fmt::Display for TidyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TidyError::Read { path, source } => {
                write!(formatter, "failed to read {}: {source}", path.display())
            }
            TidyError::Write { path, source } => {
                write!(formatter, "failed to write {}: {source}", path.display())
            }
            TidyError::InvalidUtf8 { path, source } => {
                write!(formatter, "{} is not valid UTF-8: {source}", path.display())
            }
        }
    }
}

impl std::error::Error for TidyError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            TidyError::Read { source, .. } | TidyError::Write { source, .. } => Some(source),
            TidyError::InvalidUtf8 { source, .. } => Some(source),
        }
    }
}

/// 清理一段文本，返回要写入文件的新文本。
///
/// 行为约定：
///
/// - 按行处理，保持原来的先后顺序。
/// - 每行去掉末尾的空白字符（空格、制表符等），行首空白保留。
/// - 去掉末尾空白后为空的行直接丢弃，不产生空行。
/// - 每个保留下来的行后面都写一个 `\n`，因此结果要么是空字符串，要么以
///   `\n` 结尾。
/// - `\r\n` 视为一个行尾，输出统一使用 `\n`。
/// - 输入为空、或所有行都是空白时，返回空字符串。
pub fn normalize_text(text: &str) -> String {
    let _ = &text;
    todo!()
}

/// 读取 `input`，按 `normalize_text` 的规则清理，再写到 `output`。
///
/// 行为约定：
///
/// - 读取失败返回 `TidyError::Read`，其中 `path` 是 `input`，`source` 是
///   原始 `io::Error`。
/// - 输入字节不是合法 UTF-8 时返回 `TidyError::InvalidUtf8`，`path` 是
///   `input`，`source` 是底层解码错误。
/// - 写出失败返回 `TidyError::Write`，其中 `path` 是 `output`，`source`
///   是原始 `io::Error`。
/// - 成功时返回 `TidyReport`：`written_lines` 是写出的行数，
///   `removed_lines` 是被丢弃的行数。两者之和等于输入按行切分后的总行数。
/// - 输出文件会被完整覆盖；输入为空时写出的文件也是空的，报告为
///   `0` 行写出、`0` 行丢弃。
/// - 本函数不打印任何内容。
pub fn tidy_file(input: &Path, output: &Path) -> Result<TidyReport, TidyError> {
    let _ = (&input, &output);
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::TestDir;
    use std::fs;

    #[test]
    fn normalize_trims_line_ends_and_skips_blank_lines() {
        let text = "  first  \r\n\r\n\tsecond\t\n   \nthird";

        assert_eq!(normalize_text(text), "  first\n\tsecond\nthird\n");
    }

    #[test]
    fn normalize_empty_and_blank_only_input_to_empty_string() {
        assert_eq!(normalize_text(""), "");
        assert_eq!(normalize_text("\n \n\t\n"), "");
    }

    #[test]
    fn normalize_converts_crlf_and_keeps_leading_whitespace() {
        assert_eq!(normalize_text("a\r\nb\r\n"), "a\nb\n");
        assert_eq!(normalize_text("    indented\n"), "    indented\n");
    }

    #[test]
    fn tidy_file_writes_normalized_text_and_counts_lines() {
        let directory = TestDir::new();
        let input = directory.join("raw.txt");
        let output = directory.join("clean.txt");
        fs::write(&input, "  first  \r\n\r\n\tsecond\t\n   \nthird").unwrap();

        let report = tidy_file(&input, &output).unwrap();

        assert_eq!(
            report,
            TidyReport {
                written_lines: 3,
                removed_lines: 2,
            }
        );
        assert_eq!(
            fs::read_to_string(&output).unwrap(),
            "  first\n\tsecond\nthird\n"
        );
    }

    #[test]
    fn tidy_file_handles_empty_input() {
        let directory = TestDir::new();
        let input = directory.join("empty.txt");
        let output = directory.join("empty-clean.txt");
        fs::write(&input, "").unwrap();

        let report = tidy_file(&input, &output).unwrap();

        assert_eq!(
            report,
            TidyReport {
                written_lines: 0,
                removed_lines: 0,
            }
        );
        assert_eq!(fs::read_to_string(&output).unwrap(), "");
    }

    #[test]
    fn tidy_file_reports_missing_input_path() {
        let directory = TestDir::new();
        let missing = directory.join("missing.txt");
        let output = directory.join("out.txt");

        match tidy_file(&missing, &output) {
            Err(TidyError::Read { path, source }) => {
                assert_eq!(path, missing);
                assert_eq!(source.kind(), io::ErrorKind::NotFound);
            }
            other => panic!("expected Read error, got {other:?}"),
        }
    }

    #[test]
    fn tidy_file_reports_invalid_utf8_with_input_path() {
        let directory = TestDir::new();
        let input = directory.join("binary.dat");
        let output = directory.join("out.txt");
        fs::write(&input, [0xff, b'a', b'\n']).unwrap();

        match tidy_file(&input, &output) {
            Err(TidyError::InvalidUtf8 { path, source }) => {
                assert_eq!(path, input);
                assert_eq!(source.valid_up_to(), 0);
            }
            other => panic!("expected InvalidUtf8 error, got {other:?}"),
        }
    }

    #[test]
    fn tidy_file_reports_write_failure_with_output_path() {
        let directory = TestDir::new();
        let input = directory.join("raw.txt");
        let output_directory = directory.join("cannot-write-here");
        fs::write(&input, "line\n").unwrap();
        fs::create_dir(&output_directory).unwrap();

        match tidy_file(&input, &output_directory) {
            Err(TidyError::Write { path, .. }) => {
                assert_eq!(path, output_directory);
            }
            other => panic!("expected Write error, got {other:?}"),
        }
    }
}
