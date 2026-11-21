//! 存储层：把记录读成 `Vec<Record>`，或整份原子地写回文件。
//!
//! 存储文件与导入文件使用同一种单行格式（见 [`crate::model`]）。读取时
//! 重新执行领域验证，因为用户可能手工编辑文件，不能假设“本程序写出的
//! 文件一定正确”。

use std::error::Error;
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};

use crate::model::{ParseRecordError, Record};

/// 存储层失败。
#[derive(Debug)]
pub enum StoreError {
    /// 打开或读取文件失败。
    Read { path: PathBuf, source: io::Error },
    /// 创建、写入或替换文件失败。
    Write { path: PathBuf, source: io::Error },
    /// 某一行不是合法记录。`line` 从 1 开始，空行不算记录但仍占用行号。
    Line {
        path: PathBuf,
        line: usize,
        source: ParseRecordError,
    },
    /// 同一个 id 在文件中出现两次。
    DuplicateId {
        path: PathBuf,
        id: u64,
        first_line: usize,
        second_line: usize,
    },
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read { path, source } => {
                write!(f, "cannot read store {}: {source}", path.display())
            }
            Self::Write { path, source } => {
                write!(f, "cannot write store {}: {source}", path.display())
            }
            Self::Line { path, line, source } => {
                write!(f, "{}: line {line}: {source}", path.display())
            }
            Self::DuplicateId {
                path,
                id,
                first_line,
                second_line,
            } => write!(
                f,
                "{}: duplicate id {id} at lines {first_line} and {second_line}",
                path.display()
            ),
        }
    }
}

impl Error for StoreError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Read { source, .. } | Self::Write { source, .. } => Some(source),
            Self::Line { source, .. } => Some(source),
            Self::DuplicateId { .. } => None,
        }
    }
}

/// 读取 store。
///
/// 规则：
/// - 文件不存在时返回空 `Vec`，让“第一次导入”不需要特殊命令；
/// - 空行（只含空白字符）被忽略，但后续行号仍按文件中的真实行数计算；
/// - 每个非空行交给 [`Record::parse_line`]，失败时返回
///   [`StoreError::Line`]，并保留路径和从 1 开始的行号；
/// - 读完所有记录后检查 id 唯一；重复时返回
///   [`StoreError::DuplicateId`]，包含两条记录各自的行号。
///
/// 读取顺序就是文件中的顺序，本函数不排序。
pub fn load(path: &Path) -> Result<Vec<Record>, StoreError> {
    let _ = path;
    todo!()
}

/// 用 `records` 完整替换 `path` 的内容。
///
/// 写入必须避免“先清空原文件、写到一半失败”的损坏：
/// - 先在目标文件所在目录创建一个临时文件；
/// - 逐条写入 [`Record::to_line`] 的结果，每条记录后面跟一个换行；
/// - flush 并 `sync_all` 后再用 rename 替换目标；
/// - 任何一步失败都删除临时文件并返回 [`StoreError::Write`]，原目标保持
///   不变。
///
/// 目标不存在时创建新文件；`records` 为空时得到空文件。临时文件使用
/// [`temporary_path`]。本实现假设同一路径不会被多个进程同时写入。
pub fn save(path: &Path, records: &[Record]) -> Result<(), StoreError> {
    let _ = (path, records);
    todo!()
}

/// 生成与 `path` 同目录的临时文件路径，供 [`save`] 使用。
///
/// 同目录可以避免跨文件系统 rename；文件名带上进程 id，降低并行测试互相
/// 覆盖的概率。
pub fn temporary_path(path: &Path) -> PathBuf {
    let mut name = path
        .file_name()
        .map(|name| name.to_os_string())
        .unwrap_or_default();
    name.push(format!(".{}.tmp", std::process::id()));
    path.with_file_name(name)
}
