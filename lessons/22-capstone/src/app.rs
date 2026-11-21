//! 应用层：把解析好的命令连接到模型、存储、查询和输出。
//!
//! `main` 只负责取得操作系统参数和设置退出码；真正的命令执行放在这里，
//! 因此集成测试可以直接调用 [`run`] 并捕获输出，不需要启动子进程。

use std::fs;
use std::io::{self, Write};
use std::path::Path;

use crate::cli::{self, Cli, Command};
use crate::error::AppError;
use crate::model::Record;
use crate::output;
use crate::query;
use crate::stats;
use crate::store;

/// 程序入口：锁定标准输出和标准错误，返回进程退出码。
pub fn main_entry<I>(args: I) -> i32
where
    I: IntoIterator<Item = String>,
{
    let stdout = io::stdout();
    let stderr = io::stderr();
    let mut out = stdout.lock();
    let mut err = stderr.lock();
    run(args, &mut out, &mut err)
}

/// 解析参数并执行命令。
///
/// 退出码约定：
/// - `0`：成功；
/// - `2`：参数无法解析，或参数组合不合法；
/// - `1`：参数合法，但读取、验证、查询或写入失败。
///
/// 成功时只向 `out` 写数据，失败时只向 `err` 写一条 `error: ...`。参数
/// 错误会额外输出 [`cli::USAGE`]。
pub fn run<I, W, E>(args: I, out: &mut W, err: &mut E) -> i32
where
    I: IntoIterator<Item = String>,
    W: Write,
    E: Write,
{
    let cli = match cli::parse_args(args) {
        Ok(cli) => cli,
        Err(error) => {
            let _ = writeln!(err, "error: {error}");
            let _ = writeln!(err, "{}", cli::USAGE);
            return 2;
        }
    };

    match execute(cli, out) {
        Ok(()) => 0,
        Err(error) => {
            let _ = writeln!(err, "error: {error}");
            1
        }
    }
}

/// 执行一个已经解析好的命令。
pub fn execute<W: Write>(cli: Cli, out: &mut W) -> Result<(), AppError> {
    match cli.command {
        Command::Import { path, replace } => import(&cli.store, &path, replace, out),
        Command::List(query) => list(&cli.store, &query, out),
        Command::Stats { grouped } => show_stats(&cli.store, grouped, out),
        Command::Check => check(&cli.store, out),
    }
}

fn import<W: Write>(
    store_path: &Path,
    input_path: &Path,
    replace: bool,
    out: &mut W,
) -> Result<(), AppError> {
    let existing = store::load(store_path)?;
    let text = fs::read_to_string(input_path).map_err(|source| AppError::ReadImport {
        path: input_path.to_path_buf(),
        source,
    })?;

    let mut parsed: Vec<(usize, Record)> = Vec::new();
    for (index, line) in text.lines().enumerate() {
        let line_number = index + 1;
        if line.trim().is_empty() {
            continue;
        }
        let record = Record::parse_line(line).map_err(|source| AppError::ImportLine {
            path: input_path.to_path_buf(),
            line: line_number,
            source,
        })?;
        if let Some((first_line, _)) = parsed.iter().find(|(_, existing)| existing.id == record.id)
        {
            return Err(AppError::DuplicateId {
                id: record.id,
                first: format!("input line {first_line}"),
                second: format!("input line {line_number}"),
            });
        }
        parsed.push((line_number, record));
    }

    if !replace {
        for (line_number, record) in &parsed {
            if existing.iter().any(|stored| stored.id == record.id) {
                return Err(AppError::DuplicateId {
                    id: record.id,
                    first: "existing store".to_string(),
                    second: format!("input line {line_number}"),
                });
            }
        }
    }

    let imported_count = parsed.len();
    let combined = if replace {
        parsed
            .into_iter()
            .map(|(_, record)| record)
            .collect::<Vec<_>>()
    } else {
        let mut combined = existing;
        combined.extend(parsed.into_iter().map(|(_, record)| record));
        combined
    };
    let total = combined.len();
    store::save(store_path, &combined)?;

    if replace {
        writeln!(out, "imported {imported_count} records; store replaced").map_err(AppError::Output)
    } else {
        writeln!(
            out,
            "imported {imported_count} records; store has {total} records"
        )
        .map_err(AppError::Output)
    }
}

fn list<W: Write>(store_path: &Path, query: &query::Query, out: &mut W) -> Result<(), AppError> {
    let records = store::load(store_path)?;
    let selected = query::execute(&records, query)?;
    let table = output::render_records(&selected);
    out.write_all(table.as_bytes()).map_err(AppError::Output)
}

fn show_stats<W: Write>(store_path: &Path, grouped: bool, out: &mut W) -> Result<(), AppError> {
    let records = store::load(store_path)?;
    let text = if grouped {
        output::render_groups(&stats::group_by_category(&records))
    } else {
        output::render_summary(&stats::Summary::summarize(&records))
    };
    out.write_all(text.as_bytes()).map_err(AppError::Output)
}

fn check<W: Write>(store_path: &Path, out: &mut W) -> Result<(), AppError> {
    let records = store::load(store_path)?;
    writeln!(out, "ok: {} records", records.len()).map_err(AppError::Output)
}
