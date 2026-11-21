//! 应用层错误：把“用户输入”“文件系统”“存储格式”三类失败带到同一处。
//!
//! 这一层不负责打印，也不调用 `process::exit`；它只保存足够的上下文，
//! 让 `app` 模块决定输出到标准错误和退出码。

use std::error::Error;
use std::fmt;
use std::io;
use std::path::PathBuf;

use crate::cli::UsageError;
use crate::model::ParseRecordError;
use crate::query::QueryError;
use crate::store::StoreError;

/// 一次命令执行可能遇到的失败。
#[derive(Debug)]
pub enum AppError {
    /// 命令行参数本身不合法。
    Usage(UsageError),
    /// 导入文件无法读取。
    ReadImport { path: PathBuf, source: io::Error },
    /// 导入文件的某一行无法变成记录。
    ImportLine {
        path: PathBuf,
        line: usize,
        source: ParseRecordError,
    },
    /// 同一个 id 出现了两次。`first` 与 `second` 是人类可读的位置说明。
    DuplicateId {
        id: u64,
        first: String,
        second: String,
    },
    /// 查询条件在语义上不成立。
    Query(QueryError),
    /// 存储层失败。
    Store(StoreError),
    /// 向标准输出或调用方提供的 writer 写入失败。
    Output(io::Error),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usage(error) => write!(f, "{error}"),
            Self::ReadImport { path, source } => {
                write!(f, "cannot read import file {}: {source}", path.display())
            }
            Self::ImportLine { path, line, source } => {
                write!(f, "{}: line {line}: {source}", path.display())
            }
            Self::DuplicateId { id, first, second } => {
                write!(f, "duplicate id {id} ({first} and {second})")
            }
            Self::Query(error) => write!(f, "{error}"),
            Self::Store(error) => write!(f, "{error}"),
            Self::Output(error) => write!(f, "cannot write output: {error}"),
        }
    }
}

impl Error for AppError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Usage(error) => Some(error),
            Self::ReadImport { source, .. } | Self::Output(source) => Some(source),
            Self::ImportLine { source, .. } => Some(source),
            Self::Query(error) => Some(error),
            Self::Store(error) => Some(error),
            Self::DuplicateId { .. } => None,
        }
    }
}

impl From<UsageError> for AppError {
    fn from(error: UsageError) -> Self {
        Self::Usage(error)
    }
}

impl From<QueryError> for AppError {
    fn from(error: QueryError) -> Self {
        Self::Query(error)
    }
}

impl From<StoreError> for AppError {
    fn from(error: StoreError) -> Self {
        Self::Store(error)
    }
}
