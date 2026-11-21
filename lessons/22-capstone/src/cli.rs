//! 命令行解析：把一段字符串参数变成 [`Cli`] 值。
//!
//! 这一层只做“形状”检查：命令是否存在、选项有没有值、数字能否解析、
//! 是否重复。查询语义（例如 `min > max`）也在解析结束时检查，但真正的
//! 筛选由 [`crate::query`] 完成。

use std::error::Error;
use std::fmt;
use std::path::PathBuf;

use crate::query::{Query, SortKey};

/// 没有显式给出 `--store` 时使用的文件名，相对于当前工作目录。
pub const DEFAULT_STORE: &str = "datakit.db";

/// 一行帮助文本，参数错误时打印到标准错误。
pub const USAGE: &str = "usage: datakit [--store PATH] <import|list|stats|check> [options]";

/// 一次完整调用的配置。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cli {
    /// store 文件路径。
    pub store: PathBuf,
    /// 要执行的子命令。
    pub command: Command,
}

/// 支持的四个子命令。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// 从文本文件导入记录。
    Import {
        /// 输入文件路径。
        path: PathBuf,
        /// 是否用输入完整替换现有 store。
        replace: bool,
    },
    /// 筛选并列出记录。
    List(Query),
    /// 输出统计；`grouped` 为真时按 category 分组。
    Stats {
        /// 是否按 category 分组。
        grouped: bool,
    },
    /// 检查 store 是否完整可读。
    Check,
}

/// 参数不合法时返回的错误。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UsageError {
    /// 没有给出子命令。
    MissingCommand,
    /// 子命令不在支持列表中。
    UnknownCommand(String),
    /// 选项不在当前位置允许的集合中。
    UnknownOption(String),
    /// 选项后面没有值。
    MissingOptionValue(String),
    /// 选项的值无法解析或不在允许范围内。
    InvalidOptionValue { option: String, value: String },
    /// 同一个选项出现两次。
    DuplicateOption(String),
    /// 命令缺少必需的位置参数。
    MissingArgument(&'static str),
    /// 命令多出了位置参数。
    UnexpectedArgument(String),
    /// `--min` 大于 `--max`。
    InvalidRange { min: i64, max: i64 },
}

impl fmt::Display for UsageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingCommand => write!(f, "missing command"),
            Self::UnknownCommand(command) => write!(f, "unknown command {command:?}"),
            Self::UnknownOption(option) => write!(f, "unknown option {option:?}"),
            Self::MissingOptionValue(option) => write!(f, "option {option:?} needs a value"),
            Self::InvalidOptionValue { option, value } => {
                write!(f, "invalid value {value:?} for option {option:?}")
            }
            Self::DuplicateOption(option) => write!(f, "option {option:?} was given twice"),
            Self::MissingArgument(argument) => write!(f, "missing argument: {argument}"),
            Self::UnexpectedArgument(argument) => {
                write!(f, "unexpected argument {argument:?}")
            }
            Self::InvalidRange { min, max } => {
                write!(f, "min ({min}) must not be greater than max ({max})")
            }
        }
    }
}

impl Error for UsageError {}

/// 解析参数。
///
/// 语法：
/// - 全局选项 `--store PATH` 只能出现在子命令之前，默认 `datakit.db`；
/// - `import <file> [--replace]`；
/// - `list [--category C] [--tag T] [--min N] [--max N] [--contains TEXT]
///   [--sort id|value|category] [--desc] [--limit N]`；
/// - `stats [--group category]`；
/// - `check`。
///
/// 数字选项使用 Rust 的 `parse`：`--min`/`--max` 是 `i64`，`--limit` 是
/// `usize`。同一个选项出现两次会被拒绝，避免“最后一次悄悄生效”。所有
/// 选项都必须带值，只有 `--desc` 和 `--replace` 是开关。
///
/// 解析结束后会检查 `--min` 与 `--max` 的组合；`min > max` 返回
/// [`UsageError::InvalidRange`]。
pub fn parse_args<I, S>(args: I) -> Result<Cli, UsageError>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let mut args = args.into_iter().map(Into::into).peekable();
    let mut store = PathBuf::from(DEFAULT_STORE);
    let mut store_seen = false;

    while let Some(next) = args.peek() {
        if next.as_str() != "--store" {
            break;
        }
        if store_seen {
            return Err(UsageError::DuplicateOption("--store".to_string()));
        }
        store_seen = true;
        args.next();
        let value = args
            .next()
            .ok_or_else(|| UsageError::MissingOptionValue("--store".to_string()))?;
        store = PathBuf::from(value);
    }

    let name = args.next().ok_or(UsageError::MissingCommand)?;
    let command = match name.as_str() {
        "import" => parse_import(args.collect())?,
        "list" => Command::List(parse_list(args.collect())?),
        "stats" => parse_stats(args.collect())?,
        "check" => {
            reject_extra(args.collect())?;
            Command::Check
        }
        other => return Err(UsageError::UnknownCommand(other.to_string())),
    };

    let cli = Cli { store, command };
    validate(&cli)?;
    Ok(cli)
}

fn validate(cli: &Cli) -> Result<(), UsageError> {
    if let Command::List(query) = &cli.command
        && let (Some(min), Some(max)) = (query.min, query.max)
        && min > max
    {
        return Err(UsageError::InvalidRange { min, max });
    }
    Ok(())
}

fn parse_import(args: Vec<String>) -> Result<Command, UsageError> {
    let mut replace = false;
    let mut path = None;

    for arg in args {
        match arg.as_str() {
            "--replace" => {
                if replace {
                    return Err(UsageError::DuplicateOption("--replace".to_string()));
                }
                replace = true;
            }
            _ if arg.starts_with('-') => return Err(UsageError::UnknownOption(arg)),
            _ => {
                if path.is_some() {
                    return Err(UsageError::UnexpectedArgument(arg));
                }
                path = Some(PathBuf::from(arg));
            }
        }
    }

    let path = path.ok_or(UsageError::MissingArgument("import <file>"))?;
    Ok(Command::Import { path, replace })
}

fn parse_list(args: Vec<String>) -> Result<Query, UsageError> {
    let mut query = Query::default();
    let mut seen: Vec<String> = Vec::new();
    let mut index = 0;

    while index < args.len() {
        let option = args[index].clone();
        match option.as_str() {
            "--desc" => {
                mark_once(&mut seen, &option)?;
                query.descending = true;
            }
            "--category" | "--tag" | "--contains" | "--min" | "--max" | "--sort" | "--limit" => {
                mark_once(&mut seen, &option)?;
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| UsageError::MissingOptionValue(option.clone()))?
                    .clone();
                apply_list_value(&mut query, &option, value)?;
                index += 1;
            }
            _ if option.starts_with('-') => return Err(UsageError::UnknownOption(option)),
            _ => return Err(UsageError::UnexpectedArgument(option)),
        }
        index += 1;
    }

    Ok(query)
}

fn apply_list_value(query: &mut Query, option: &str, value: String) -> Result<(), UsageError> {
    match option {
        "--category" => query.category = Some(value),
        "--tag" => query.tag = Some(value),
        "--contains" => query.contains = Some(value),
        "--min" => {
            query.min = Some(value.parse().map_err(|_| invalid_value(option, &value))?);
        }
        "--max" => {
            query.max = Some(value.parse().map_err(|_| invalid_value(option, &value))?);
        }
        "--limit" => {
            query.limit = Some(value.parse().map_err(|_| invalid_value(option, &value))?);
        }
        "--sort" => {
            query.sort = match value.as_str() {
                "id" => SortKey::Id,
                "value" => SortKey::Value,
                "category" => SortKey::Category,
                _ => return Err(invalid_value(option, &value)),
            };
        }
        _ => return Err(UsageError::UnknownOption(option.to_string())),
    }
    Ok(())
}

fn parse_stats(args: Vec<String>) -> Result<Command, UsageError> {
    let mut grouped = false;
    let mut index = 0;

    while index < args.len() {
        let option = args[index].clone();
        match option.as_str() {
            "--group" => {
                if grouped {
                    return Err(UsageError::DuplicateOption("--group".to_string()));
                }
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| UsageError::MissingOptionValue("--group".to_string()))?;
                if value != "category" {
                    return Err(invalid_value("--group", value));
                }
                grouped = true;
                index += 1;
            }
            _ if option.starts_with('-') => return Err(UsageError::UnknownOption(option)),
            _ => return Err(UsageError::UnexpectedArgument(option)),
        }
        index += 1;
    }

    Ok(Command::Stats { grouped })
}

fn reject_extra(args: Vec<String>) -> Result<(), UsageError> {
    if let Some(arg) = args.into_iter().next() {
        if arg.starts_with('-') {
            Err(UsageError::UnknownOption(arg))
        } else {
            Err(UsageError::UnexpectedArgument(arg))
        }
    } else {
        Ok(())
    }
}

fn mark_once(seen: &mut Vec<String>, option: &str) -> Result<(), UsageError> {
    if seen.iter().any(|existing| existing == option) {
        Err(UsageError::DuplicateOption(option.to_string()))
    } else {
        seen.push(option.to_string());
        Ok(())
    }
}

fn invalid_value(option: &str, value: &str) -> UsageError {
    UsageError::InvalidOptionValue {
        option: option.to_string(),
        value: value.to_string(),
    }
}
