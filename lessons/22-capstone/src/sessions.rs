//! 课后巩固：另一份数据——学习或工作时段记录。
//!
//! 主练习的记录有五个字段、允许负值；这里换成三个字段，数值是“分钟”，
//! 边界也不同：每条时段必须大于 0 且不超过一天（1440 分钟）。同一项目在
//! 同一天只允许出现一次，避免重复计数。
//!
//! 输入格式：
//!
//! ```text
//! 2026-11-03|rust-from-zero|90
//! 2026-11-03|datakit|45
//! ```
//!
use std::error::Error;
use std::fmt;

/// 一行时段记录必须包含的字段数。
pub const SESSION_FIELD_COUNT: usize = 3;
/// 一天有 1440 分钟，单条记录的分钟数不能超过它。
pub const MINUTES_PER_DAY: u32 = 1440;

/// 一条通过检查的时段记录。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Session {
    /// 日期文本，例如 `2026-11-03`。
    pub date: String,
    /// 去掉空白并转成小写的项目名。
    pub project: String,
    /// 时长，范围是 1 到 1440（包含两端）。
    pub minutes: u32,
}

/// 单条时段不符合规则时的错误。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionError {
    /// 用 `|` 切分后的字段数量不对。
    FieldCount { expected: usize, found: usize },
    /// 日期去掉空白后为空。
    EmptyDate,
    /// 项目名去掉空白后为空。
    EmptyProject,
    /// 分钟数不是合法的 `u32`。
    InvalidMinutes { text: String },
    /// 分钟数是 0 或超过 1440。
    MinutesOutOfRange { minutes: u32 },
    /// 同一天同一项目重复出现。`first_line` 是第一次出现的行号。
    DuplicateProjectDay {
        date: String,
        project: String,
        first_line: usize,
    },
}

impl fmt::Display for SessionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldCount { expected, found } => {
                write!(
                    f,
                    "expected {expected} fields separated by '|', found {found}"
                )
            }
            Self::EmptyDate => write!(f, "date is empty after trimming"),
            Self::EmptyProject => write!(f, "project is empty after trimming"),
            Self::InvalidMinutes { text } => {
                write!(
                    f,
                    "invalid minutes {text:?}: expected a non-negative integer"
                )
            }
            Self::MinutesOutOfRange { minutes } => write!(
                f,
                "minutes ({minutes}) must be between 1 and {MINUTES_PER_DAY}"
            ),
            Self::DuplicateProjectDay {
                date,
                project,
                first_line,
            } => write!(
                f,
                "duplicate project {project:?} on {date} (first seen on line {first_line})"
            ),
        }
    }
}

impl Error for SessionError {}

/// 带行号的时段错误。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionLineError {
    /// 从 1 开始的行号。
    pub line: usize,
    /// 该行的具体问题。
    pub error: SessionError,
}

impl fmt::Display for SessionLineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}: {}", self.line, self.error)
    }
}

impl Error for SessionLineError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.error)
    }
}

/// 解析整份时段文本。
///
/// 规则：
/// - 每一行必须是 `date|project|minutes`，字段数不是 3 时返回
///   [`SessionError::FieldCount`]；
/// - 只含空白字符的行被忽略，但它仍然占用行号；
/// - `date` 去掉两端空白，保持大小写与内部字符，空字符串返回
///   [`SessionError::EmptyDate`]；
/// - `project` 去掉两端空白后转小写，空字符串返回
///   [`SessionError::EmptyProject`]；
/// - `minutes` 必须能解析成 `u32`，否则返回
///   [`SessionError::InvalidMinutes`]；解析成功后还必须在 1 到 1440
///   （包含两端）之间，否则返回 [`SessionError::MinutesOutOfRange`]；
/// - `(date, project)` 这一对不能重复；第二次出现时返回
///   [`SessionError::DuplicateProjectDay`]，`first_line` 是第一次出现的行号。
///
/// 任一错误都带上它所在的行号，返回类型是 [`SessionLineError`]。
pub fn parse_sessions(text: &str) -> Result<Vec<Session>, SessionLineError> {
    let _ = text;
    todo!()
}

/// 按项目汇总分钟数，结果按项目名从小到大排列。
///
/// 返回的分钟数是 `u64`：单条记录最多 1440，但总量可以超过 `u32`。
/// 空输入得到空 `Vec`。同一项目在同一天不会重复，因此直接相加即可。
pub fn total_minutes_by_project(sessions: &[Session]) -> Vec<(String, u64)> {
    let _ = sessions;
    todo!()
}

/// 找总分钟数最多的一天，返回日期和总分钟数。
///
/// 总数相同时选择字典序更小的日期（本课的日期使用 `YYYY-MM-DD`，所以这也
/// 就是更早的日期）。空输入返回 `None`。返回的日期借用输入切片中的文本。
pub fn busiest_day(sessions: &[Session]) -> Option<(&str, u64)> {
    let _ = sessions;
    todo!()
}
