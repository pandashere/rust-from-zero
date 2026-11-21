//! 主练习：实现文件统计命令 `rstat`。
//!
//! 命令形式：
//!
//! ```text
//! rstat [--bytes] [--words] [--lines] [--non-empty] <path>
//! ```
//!
//! 请只替换四个函数体里的 `todo!()`，不要修改类型定义、函数签名，也不要
//! 改动本文件底部的测试。每个函数上方的注释就是它的完整行为约定。
//!
//! 完成后的验收命令：
//!
//! ```bash
//! cargo test -p l18_files_cli exercise
//! ```

use std::ffi::OsString;
use std::fmt;
use std::io;
use std::path::PathBuf;

/// 参数错误的固定提示行，会附在每一条用法错误后面。
const USAGE: &str = "usage: rstat [--bytes] [--words] [--lines] [--non-empty] <path>";

/// 一次运行需要的全部配置：统计哪个文件、请求哪些指标、如何统计行。
///
/// `lines` 有三种状态：`None` 表示完全不统计行；`Some(LineMode::All)`
/// 统计全部行；`Some(LineMode::NonEmpty)` 只统计去掉两端空白后非空的行。
#[derive(Debug, PartialEq, Eq)]
pub struct Config {
    pub path: PathBuf,
    pub bytes: bool,
    pub words: bool,
    pub lines: Option<LineMode>,
}

/// 行统计的两种模式。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineMode {
    /// 每一行都计数，包括空行和只有空白的行。
    All,
    /// 只统计去掉两端空白后仍有内容的行。
    NonEmpty,
}

/// 统计结果。未请求的指标保留 `None`，请求了的指标是 `Some(数值)`。
#[derive(Debug, PartialEq, Eq)]
pub struct Stats {
    pub bytes: Option<u64>,
    pub words: Option<u64>,
    pub lines: Option<u64>,
}

/// 运行 `rstat` 时可能出现的失败原因。
///
/// 参数错误、读取错误、内容编码错误和输出错误分开表示，调用者可以据此
/// 决定退出码，也可以取出原始错误继续检查。
#[derive(Debug)]
pub enum AppError {
    /// 用户给出的参数不合法。字符串是具体原因，不包含固定的 usage 行；
    /// 固定的 usage 行由 `Display` 统一附加。
    Usage(String),
    /// 无法读取目标文件。保留用户给出的路径和操作系统返回的原始错误。
    Read { path: PathBuf, source: io::Error },
    /// 文件字节不是合法的 UTF-8 文本，无法按“词”或“行”解释。
    InvalidUtf8 {
        path: PathBuf,
        source: std::str::Utf8Error,
    },
    /// 结果写入标准输出时失败，例如下游管道提前关闭。
    Output(io::Error),
}

impl fmt::Display for AppError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Usage(message) => write!(formatter, "{message}\n{USAGE}"),
            AppError::Read { path, source } => {
                write!(formatter, "failed to read {}: {source}", path.display())
            }
            AppError::InvalidUtf8 { path, source } => {
                write!(formatter, "{} is not valid UTF-8: {source}", path.display())
            }
            AppError::Output(source) => write!(formatter, "failed to write stdout: {source}"),
        }
    }
}

impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AppError::Usage(_) => None,
            AppError::Read { source, .. } | AppError::Output(source) => Some(source),
            AppError::InvalidUtf8 { source, .. } => Some(source),
        }
    }
}

/// 把命令行参数解析成 `Config`。
///
/// 参数迭代器 `args` 的第一项是程序名，必须跳过；之后是任意顺序的开关和
/// 唯一的路径。行为约定：
///
/// - `--bytes`、`--words`、`--lines` 分别请求字节、词、行统计。重复出现
///   同一个开关不改变结果。
/// - `--non-empty` 是行统计的修饰符：它把行模式设为 `LineMode::NonEmpty`，
///   本身不算“请求了某个指标”。只写它、不写三个指标开关时，仍然默认
///   请求全部三项。
/// - 三个指标开关一个都没有出现时，`bytes`、`words` 为 `true`，`lines`
///   为 `Some(LineMode::All)`；只要出现了其中任意一个，就只请求出现过的
///   指标，未出现的指标保持关闭。`--non-empty` 单独出现也属于“没有出现
///   指标开关”。
/// - 以 `--` 开头但不属于上述四个选项的参数，返回
///   `AppError::Usage("unknown option: <参数>")`，参数保持原样。
/// - 完全没有路径时返回 `AppError::Usage("missing input path")`。
/// - 出现第二个路径时返回 `AppError::Usage("multiple input paths")`。
/// - 路径保存为 `PathBuf`，不要求路径本身是合法 UTF-8。
///
/// 返回值只描述“要做什么”，不读取文件、不打印任何内容。
pub fn parse_args<I, S>(args: I) -> Result<Config, AppError>
where
    I: IntoIterator<Item = S>,
    S: Into<OsString>,
{
    let _ = &args;
    todo!()
}

/// 根据文件原始字节和配置计算统计结果。
///
/// 行为约定：
///
/// - `bytes`：请求了字节统计时是 `data` 的原始长度（字节数，不是字符
///   数），否则是 `None`。
/// - `words`：请求了词统计时，按 Unicode 空白切分文本，连续空白只起一次
///   分隔作用，结果是一段段非空文本的数量；没有请求时是 `None`。
/// - `lines`：`config.lines` 为 `None` 时不统计，结果为 `None`。为
///   `Some(LineMode::All)` 时统计所有行；为
///   `Some(LineMode::NonEmpty)` 时只统计去掉两端空白后非空的行。
/// - 行尾约定：空输入是 0 行；末尾的换行不会额外产生一行；`\r\n` 视为
///   一个行尾，`\r` 不计入行内容。例如 `"a\n"` 是 1 行，`"a\n\n"` 是
///   2 行。
/// - 只有请求了 `words` 或 `lines` 时才需要把字节解释成文本。此时若
///   `data` 不是合法 UTF-8，返回 `AppError::InvalidUtf8`，`path` 取
///   `config.path`，`source` 是底层解码错误。只请求 `bytes` 时，任何字节
///   序列都可以统计成功。
pub fn stats_from_bytes(data: &[u8], config: &Config) -> Result<Stats, AppError> {
    let _ = (&data, &config);
    todo!()
}

/// 把统计结果格式化成最终要写出的文本。
///
/// 行为约定：
///
/// - 字段顺序固定为 `bytes`、`words`、`lines`，与命令行里开关的顺序无关。
/// - 每个值为 `Some` 的字段输出一行 `字段名: 数值\n`，字段名与数值之间
///   是一个空格加一个冒号再加一个空格；`None` 字段整行跳过。
/// - 三个字段都是 `None` 时返回空字符串。
/// - 本函数只生成字符串，不写标准输出、不打印、不产生错误。
pub fn format_stats(stats: &Stats) -> String {
    let _ = &stats;
    todo!()
}

/// 读取 `config.path` 指向的文件并返回格式化好的统计文本。
///
/// 行为约定：
///
/// - 读取失败时返回 `AppError::Read`，`path` 与 `config.path` 相同，
///   `source` 是操作系统返回的原始错误。
/// - 读取成功后把原始字节交给统计函数，再把结果交给格式化函数。
/// - 本函数不写标准输出，也不打印错误；所有失败都通过返回值交给调用者。
pub fn run(config: &Config) -> Result<String, AppError> {
    let _ = &config;
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::TestDir;
    use std::ffi::OsString;
    use std::fs;
    use std::path::Path;

    fn default_config(path: &str) -> Config {
        Config {
            path: PathBuf::from(path),
            bytes: true,
            words: true,
            lines: Some(LineMode::All),
        }
    }

    // 下面两个测试只验证本课依赖的标准库行为，不经过待实现的函数，初始
    // 状态就应当通过。
    #[test]
    fn pathbuf_borrows_as_path() {
        let path = PathBuf::from("directory with spaces").join("输入 file.txt");
        let borrowed: &Path = &path;

        assert_eq!(borrowed.file_name().unwrap(), "输入 file.txt");
        assert_eq!(path, PathBuf::from("directory with spaces/输入 file.txt"));
    }

    #[test]
    fn error_kind_is_stable_classification() {
        let error = io::Error::new(io::ErrorKind::NotFound, "missing");
        assert_eq!(error.kind(), io::ErrorKind::NotFound);
    }

    #[test]
    fn parse_defaults_when_no_metric_flags() {
        assert_eq!(
            parse_args(["rstat", "input file.txt"]).unwrap(),
            default_config("input file.txt")
        );
    }

    #[test]
    fn parse_honors_flag_order_selection_and_repetition() {
        assert_eq!(
            parse_args([
                "rstat",
                "--lines",
                "--non-empty",
                "--bytes",
                "--bytes",
                "input.txt",
            ])
            .unwrap(),
            Config {
                path: PathBuf::from("input.txt"),
                bytes: true,
                words: false,
                lines: Some(LineMode::NonEmpty),
            }
        );
    }

    #[test]
    fn parse_non_empty_alone_keeps_default_metrics() {
        assert_eq!(
            parse_args(["rstat", "--non-empty", "input.txt"]).unwrap(),
            Config {
                lines: Some(LineMode::NonEmpty),
                ..default_config("input.txt")
            }
        );
    }

    #[test]
    fn parse_accepts_options_after_the_path() {
        assert_eq!(
            parse_args(["rstat", "input.txt", "--words"]).unwrap(),
            Config {
                path: PathBuf::from("input.txt"),
                bytes: false,
                words: true,
                lines: None,
            }
        );
    }

    #[test]
    fn parse_rejects_usage_errors() {
        let cases: &[(&[&str], &str)] = &[
            (&["rstat"], "missing input path"),
            (
                &["rstat", "--unknown", "input.txt"],
                "unknown option: --unknown",
            ),
            (
                &["rstat", "first.txt", "second.txt"],
                "multiple input paths",
            ),
        ];

        for &(args, expected_message) in cases {
            match parse_args(args.iter().copied()) {
                Err(AppError::Usage(message)) => assert_eq!(message, expected_message),
                other => panic!("expected usage error for {args:?}, got {other:?}"),
            }
        }
    }

    #[test]
    fn parse_non_empty_does_not_enable_lines_without_line_flag() {
        assert_eq!(
            parse_args(["rstat", "--non-empty", "--bytes", "input.txt"]).unwrap(),
            Config {
                path: PathBuf::from("input.txt"),
                bytes: true,
                words: false,
                lines: None,
            }
        );
    }

    #[cfg(unix)]
    #[test]
    fn parse_keeps_non_utf8_path() {
        use std::os::unix::ffi::OsStringExt;

        let raw = OsString::from_vec(b"data-\xff.bin".to_vec());
        let config = parse_args([OsString::from("rstat"), raw.clone()]).unwrap();

        assert_eq!(config.path, PathBuf::from(raw));
    }

    #[test]
    fn empty_input_produces_zero_counts() {
        assert_eq!(
            stats_from_bytes(b"", &default_config("empty.txt")).unwrap(),
            Stats {
                bytes: Some(0),
                words: Some(0),
                lines: Some(0),
            }
        );
    }

    #[test]
    fn unicode_whitespace_and_line_endings_are_counted() {
        let text = "Rust 语言\r\n\n \t\nlast line";
        assert_eq!(
            stats_from_bytes(text.as_bytes(), &default_config("unicode.txt")).unwrap(),
            Stats {
                bytes: Some(text.len() as u64),
                words: Some(4),
                lines: Some(4),
            }
        );
    }

    #[test]
    fn non_empty_mode_ignores_blank_lines() {
        let config = Config {
            bytes: false,
            words: false,
            lines: Some(LineMode::NonEmpty),
            ..default_config("unicode.txt")
        };

        assert_eq!(
            stats_from_bytes("Rust 语言\n\n \t\nlast line".as_bytes(), &config).unwrap(),
            Stats {
                bytes: None,
                words: None,
                lines: Some(2),
            }
        );
    }

    #[test]
    fn line_count_does_not_invent_a_line_after_final_newline() {
        let config = Config {
            bytes: false,
            words: false,
            lines: Some(LineMode::All),
            ..default_config("lines.txt")
        };
        let cases = [
            ("", 0_u64),
            ("a", 1),
            ("a\n", 1),
            ("a\n\n", 2),
            ("\n", 1),
            ("\n\n", 2),
            ("a\r\nb\r\n", 2),
        ];

        for (text, expected) in cases {
            let stats = stats_from_bytes(text.as_bytes(), &config).unwrap();
            assert_eq!(stats.lines, Some(expected), "input: {text:?}");
        }
    }

    #[test]
    fn bytes_only_accepts_invalid_utf8() {
        let config = Config {
            bytes: true,
            words: false,
            lines: None,
            ..default_config("binary.dat")
        };

        assert_eq!(
            stats_from_bytes(&[0xff, 0x00, 0xfe], &config).unwrap(),
            Stats {
                bytes: Some(3),
                words: None,
                lines: None,
            }
        );
    }

    #[test]
    fn text_metrics_reject_invalid_utf8_with_path() {
        match stats_from_bytes(&[0xff], &default_config("binary.dat")) {
            Err(AppError::InvalidUtf8 { path, source }) => {
                assert_eq!(path, PathBuf::from("binary.dat"));
                assert_eq!(source.valid_up_to(), 0);
            }
            other => panic!("expected InvalidUtf8, got {other:?}"),
        }
    }

    #[test]
    fn formatting_uses_fixed_field_order() {
        let all = Stats {
            bytes: Some(12),
            words: Some(2),
            lines: Some(1),
        };
        assert_eq!(format_stats(&all), "bytes: 12\nwords: 2\nlines: 1\n");

        let words_only = Stats {
            bytes: None,
            words: Some(2),
            lines: None,
        };
        assert_eq!(format_stats(&words_only), "words: 2\n");

        let empty = Stats {
            bytes: None,
            words: None,
            lines: None,
        };
        assert_eq!(format_stats(&empty), "");
    }

    #[test]
    fn run_reads_paths_with_spaces() {
        let directory = TestDir::new();
        let path = directory.join("input file.txt");
        fs::write(&path, "hello world\n").unwrap();
        let config = Config {
            path,
            ..default_config("unused")
        };

        assert_eq!(run(&config).unwrap(), "bytes: 12\nwords: 2\nlines: 1\n");
    }

    #[test]
    fn run_preserves_io_error_kind_and_path() {
        let directory = TestDir::new();
        let missing = directory.join("missing.txt");
        let config = Config {
            path: missing.clone(),
            ..default_config("unused")
        };

        match run(&config) {
            Err(AppError::Read { path, source }) => {
                assert_eq!(path, missing);
                assert_eq!(source.kind(), io::ErrorKind::NotFound);
            }
            other => panic!("expected Read error, got {other:?}"),
        }
    }

    #[test]
    fn run_can_report_binary_bytes() {
        let directory = TestDir::new();
        let path = directory.join("binary.dat");
        fs::write(&path, [0xff, 0x00, 0xfe]).unwrap();
        let config = Config {
            path,
            bytes: true,
            words: false,
            lines: None,
        };

        assert_eq!(run(&config).unwrap(), "bytes: 3\n");
    }
}
