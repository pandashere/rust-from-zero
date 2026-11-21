//! 第十课的演示入口：用几组输入展示主练习函数的行为。
//!
//! 在完成 `exercise.rs` 中的 `todo!()` 之前，运行本程序会在第一组输入处
//! 中止（panic）——这是预期现象，先去做练习，再回来看完整输出。
//!
//! 运行命令：
//!
//! ```bash
//! cargo run -p l10_iterators
//! ```

use l10_iterators::exercise::{
    Level, error_messages, first_error, parse_all, partition_logs, summarize,
};

fn main() {
    let complete = [
        "INFO: service started",
        "WARN: disk usage above 80%",
        "ERROR: connection refused",
    ];

    match parse_all(&complete) {
        Ok(entries) => {
            println!("全部 {} 行都解析成功：", entries.len());
            for entry in &entries {
                println!(
                    "  第 {} 行 [{:?}] {}",
                    entry.line, entry.level, entry.message
                );
            }
        }
        Err(error) => println!("第 {} 行解析失败：{:?}", error.line, error.text),
    }

    let mixed = [
        "INFO: started",
        "this line has no separator",
        "ERROR: disk full",
        "DEBUG: unknown level",
        "WARN: disk almost full",
    ];

    let (entries, errors) = partition_logs(&mixed);
    println!(
        "\n分区结果：{} 条合法日志，{} 条坏行",
        entries.len(),
        errors.len()
    );
    for error in &errors {
        println!("  坏行 {}：{:?}", error.line, error.text);
    }

    println!("\nERROR 正文：{:?}", error_messages(&entries));
    println!(
        "第一条 ERROR：{:?}",
        first_error(&entries).map(|entry| entry.message.as_str())
    );

    let summary = summarize(&entries);
    let count = |level: Level| summary.get(&level).copied().unwrap_or(0);
    println!(
        "\n统计：INFO={}，WARN={}，ERROR={}",
        count(Level::Info),
        count(Level::Warn),
        count(Level::Error)
    );
}
