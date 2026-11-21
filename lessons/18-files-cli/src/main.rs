//! `rstat` 的命令行边界：连接进程参数、标准输出、错误输出和退出码。
//!
//! 真正的解析、统计和格式化逻辑在库的 `exercise` 模块中。这个文件已经
//! 写好，不需要修改；它演示了 `main` 为什么应当保持得很薄。
//!
//! 在完成 `exercise.rs` 中的 `todo!()` 之前，运行本程序会在参数解析处
//! 中止（panic）——这是预期现象，先去做主练习，再回来观察真实输出。

use l18_files_cli::exercise::{AppError, parse_args, run};
use std::env;
use std::io::{self, Write};
use std::process::ExitCode;

fn main() -> ExitCode {
    let config = match parse_args(env::args_os()) {
        Ok(config) => config,
        Err(error) => {
            eprintln!("error: {error}");
            return ExitCode::from(2);
        }
    };

    let result = run(&config).and_then(|output| {
        let mut stdout = io::stdout().lock();
        stdout
            .write_all(output.as_bytes())
            .and_then(|()| stdout.flush())
            .map_err(AppError::Output)
    });

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}
