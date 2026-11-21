//! 第十六课的演示入口：把若干数据块交给两个工作线程计算校验值。
//!
//! 在完成 `exercise.rs` 中的 `todo!()` 之前，运行本程序会在第一次调用
//! `checksum` 时中止（panic）——这是预期现象，先去做练习，再回来看完整
//! 输出。

use l16_threads_channels::exercise::{PipelineError, Task, run_pipeline};

fn main() {
    let tasks = vec![
        Task {
            id: 8,
            bytes: vec![1, 2, 3],
        },
        Task {
            id: 3,
            bytes: Vec::new(),
        },
        Task {
            id: 5,
            bytes: vec![255],
        },
        Task {
            id: 12,
            bytes: b"learn rust".to_vec(),
        },
    ];

    match run_pipeline(tasks, 2) {
        Ok(results) => {
            println!("流水线结果（按任务 ID 排序）：");
            for result in results {
                println!(
                    "任务 {:>2}: 校验值 {:>20}，处理者 worker {}",
                    result.id, result.value, result.worker_id
                );
            }
        }
        Err(PipelineError::ZeroWorkers) => {
            println!("错误：工作线程数量不能为 0");
        }
        Err(PipelineError::WorkerPanicked { worker_id }) => {
            println!("错误：worker {worker_id} 在处理任务时 panic 了");
        }
        Err(PipelineError::ResultCountMismatch { expected, actual }) => {
            println!("错误：期望 {expected} 条结果，实际收到 {actual} 条");
        }
    }

    println!();
    println!("零工作线程的错误演示：");
    match run_pipeline(Vec::new(), 0) {
        Ok(results) => println!("不应成功，却得到了 {results:?}"),
        Err(error) => println!("得到预期的错误：{error:?}"),
    }
}
