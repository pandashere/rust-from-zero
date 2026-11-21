//! 主练习：多工作线程校验流水线。
//!
//! 一个程序收到若干块字节数据。每块数据有一个调用者指定的任务编号，需要
//! 独立计算校验值。数据块之间没有依赖，可以把它们分给固定数量的工作线程
//! 同时处理；主线程负责分发任务、收集结果，并按任务编号恢复稳定顺序。
//!
//! 请只替换 `checksum` 和 `run_pipeline_with` 函数体里的 `todo!()`，不要
//! 修改类型定义、函数签名，也不要改动本文件底部的测试。每个函数上方的
//! 注释就是它的完整行为约定。

// 实现时会用到 `std::thread` 和 `std::sync::mpsc`。脚手架故意不预先导入
// 它们：未使用的 `use` 会触发警告，完成实现时再按需添加即可。

/// 一个待处理的数据块。
///
/// `id` 是调用者给这块数据起的编号，用来在结果乱序到达时恢复稳定顺序；
/// `bytes` 是这块数据的原始字节，所有权随任务一起交给工作线程。
#[derive(Debug, PartialEq, Eq)]
pub struct Task {
    pub id: usize,
    pub bytes: Vec<u8>,
}

/// 一个数据块的处理结果。
///
/// `id` 原样来自对应 `Task` 的 `id`；`value` 是对 `Task::bytes` 计算出的
/// 校验值；`worker_id` 是实际处理该任务的工作线程编号，编号从 `0` 开始，
/// 按工作线程创建顺序分配。
#[derive(Debug, PartialEq, Eq)]
pub struct Checksum {
    pub id: usize,
    pub value: u64,
    pub worker_id: usize,
}

/// 流水线可能返回的错误。
///
/// 三种错误有明确的优先级：先检查配置错误，再检查工作线程 panic，最后
/// 检查结果数量。这样调用者总能先看到更根本的问题。
#[derive(Debug, PartialEq, Eq)]
pub enum PipelineError {
    /// `worker_count` 为 `0`，没有任何线程能处理任务。
    ZeroWorkers,
    /// 编号为 `worker_id` 的工作线程 panic 了；该线程的结果不完整。
    WorkerPanicked { worker_id: usize },
    /// 所有工作线程结束后，收到的结果数量与任务数量不同。
    ///
    /// `expected` 是输入任务数，`actual` 是实际收到的结果数。
    ResultCountMismatch { expected: usize, actual: usize },
}

/// 计算一块字节的校验值。
///
/// 行为约定：
///
/// - 从 `0_u64` 开始，按字节在切片中的先后顺序逐个处理；
/// - 每个字节先被看成 `0` 到 `255` 的整数，然后执行
///   `acc = acc × 31 + byte`；
/// - 乘法和加法按 `u64` 的回绕（wrapping）语义进行：超出 `u64` 能表示的
///   范围时从另一端绕回，而不是 panic；
/// - 空切片返回 `0`。
///
/// 参数是只读切片，函数不取得数据的所有权，也不会修改调用者的数据。
pub fn checksum(bytes: &[u8]) -> u64 {
    let _ = bytes;
    todo!()
}

/// 用默认的 [`checksum`] 计算方式运行流水线。
///
/// 这个函数已经写好：它把 `checksum` 包装成“每个任务都产生一个结果”的
/// 计算步骤，再交给 [`run_pipeline_with`]。你只需要完成
/// `run_pipeline_with`，本函数的行为就会随之正确。
pub fn run_pipeline(tasks: Vec<Task>, worker_count: usize) -> Result<Vec<Checksum>, PipelineError> {
    run_pipeline_with(tasks, worker_count, |bytes| Some(checksum(bytes)))
}

/// 用指定的计算步骤运行流水线。
///
/// `work` 是一个函数指针（只记录一段代码的位置，不捕获周围变量）。每个
/// 工作线程对拿到的 `Task::bytes` 调用一次 `work`：
///
/// - 返回 `Some(value)`：产生一条 `Checksum`，其中 `id` 来自任务，
///   `value` 是 `work` 的返回值，`worker_id` 是实际处理者；
/// - 返回 `None`：这个任务不产生结果，但工作线程继续处理后面的任务。
///
/// 行为约定：
///
/// - `worker_count` 为 `0` 时立即返回 [`PipelineError::ZeroWorkers`]，不
///   启动线程，也不处理任何任务；
/// - 任务按它在输入 `Vec` 中的位置（从 `0` 开始）轮流分配给工作线程：
///   位置 `0` 给 worker `0`，位置 `1` 给 worker `1`，依此类推，到末尾后
///   回到 worker `0`；
/// - 输入为空时返回 `Ok` 和一个空 `Vec`，即使 `worker_count` 大于 `0`；
/// - 返回的结果按 `Task::id` 升序排列，不依赖线程调度或消息到达顺序；
///   输入中的任务编号应唯一，重复编号的行为不在本练习的保证范围内；
/// - 任一工作线程 panic 时，等所有已创建的工作线程都结束后返回
///   [`PipelineError::WorkerPanicked`]；如果有多个线程 panic，报告编号
///   最小的那个；
/// - 没有线程 panic，但实际结果数少于任务数（例如 `work` 对某些任务返回
///   `None`，或某个任务没能送达工作线程）时，返回
///   [`PipelineError::ResultCountMismatch`]，并带上期望值和实际值；
/// - 错误路径也不能留下仍在运行的工作线程：返回前所有已创建的工作线程
///   都必须结束；
/// - 计算步骤 panic 属于 [`PipelineError::WorkerPanicked`]，与 `work`
///   返回 `None` 造成的数量不足是两种不同的错误。
///
/// 参数 `tasks` 和其中每个 `Task::bytes` 的所有权都会交给工作线程；函数
/// 返回后调用者不能再使用这些任务。
pub fn run_pipeline_with(
    tasks: Vec<Task>,
    worker_count: usize,
    work: fn(&[u8]) -> Option<u64>,
) -> Result<Vec<Checksum>, PipelineError> {
    let _ = (&tasks, worker_count, work);
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn task(id: usize, bytes: &[u8]) -> Task {
        Task {
            id,
            bytes: bytes.to_vec(),
        }
    }

    fn keep_every_result(bytes: &[u8]) -> Option<u64> {
        Some(checksum(bytes))
    }

    // ---- 概念测试：不调用待实现函数，初始状态下就应通过 ----

    #[test]
    fn scoped_thread_can_borrow_a_local_slice_without_copying_it() {
        let values = vec![1, 2, 3];

        let total = std::thread::scope(|scope| {
            let handle = scope.spawn(|| values.iter().sum::<i32>());
            handle.join().unwrap()
        });

        assert_eq!(total, 6);
        assert_eq!(values, vec![1, 2, 3]);
    }

    #[test]
    fn a_failed_send_returns_the_unsent_owned_value() {
        let (sender, receiver) = std::sync::mpsc::channel::<String>();
        drop(receiver);

        let error = sender.send(String::from("recover me")).unwrap_err();
        assert_eq!(error.0, "recover me");
    }

    // ---- 校验算法：正常、边界与回绕 ----

    #[test]
    fn checksum_of_empty_block_is_zero() {
        assert_eq!(checksum(&[]), 0);
    }

    #[test]
    fn checksum_of_small_blocks_matches_the_defined_formula() {
        assert_eq!(checksum(&[1, 2, 3]), 1_026);
        assert_eq!(checksum(&[255]), 255);
        assert_eq!(checksum(b"learn rust"), 2_944_394_269_393_984);
    }

    #[test]
    fn checksum_wraps_at_the_u64_boundary_instead_of_panicking() {
        assert_eq!(checksum(&[255; 13]), 4_634_958_717_477_794_239);
    }

    // ---- 流水线：正常、边界与错误 ----

    #[test]
    fn zero_workers_are_rejected_before_any_task_is_processed() {
        assert_eq!(run_pipeline(Vec::new(), 0), Err(PipelineError::ZeroWorkers));
        assert_eq!(
            run_pipeline(vec![task(9, b"abc")], 0),
            Err(PipelineError::ZeroWorkers)
        );
    }

    #[test]
    fn empty_task_list_returns_an_empty_result_even_with_many_workers() {
        assert_eq!(run_pipeline(Vec::new(), 3), Ok(Vec::new()));
    }

    #[test]
    fn one_worker_processes_every_task_and_results_are_sorted_by_id() {
        let tasks = vec![task(20, &[1, 2, 3]), task(10, &[]), task(30, &[255])];

        assert_eq!(
            run_pipeline(tasks, 1),
            Ok(vec![
                Checksum {
                    id: 10,
                    value: 0,
                    worker_id: 0,
                },
                Checksum {
                    id: 20,
                    value: 1_026,
                    worker_id: 0,
                },
                Checksum {
                    id: 30,
                    value: 255,
                    worker_id: 0,
                },
            ])
        );
    }

    #[test]
    fn round_robin_uses_input_positions_when_workers_exceed_tasks() {
        let tasks = vec![task(30, &[3]), task(10, &[1]), task(20, &[2])];

        assert_eq!(
            run_pipeline(tasks, 5),
            Ok(vec![
                Checksum {
                    id: 10,
                    value: 1,
                    worker_id: 1,
                },
                Checksum {
                    id: 20,
                    value: 2,
                    worker_id: 2,
                },
                Checksum {
                    id: 30,
                    value: 3,
                    worker_id: 0,
                },
            ])
        );
    }

    #[test]
    fn varied_blocks_keep_ids_checksums_and_worker_ownership() {
        let tasks = vec![
            task(100, &[255; 13]),
            task(5, &[1, 2, 3]),
            task(50, b"learn rust"),
            task(20, &[0]),
        ];

        assert_eq!(
            run_pipeline(tasks, 3),
            Ok(vec![
                Checksum {
                    id: 5,
                    value: 1_026,
                    worker_id: 1,
                },
                Checksum {
                    id: 20,
                    value: 0,
                    worker_id: 0,
                },
                Checksum {
                    id: 50,
                    value: 2_944_394_269_393_984,
                    worker_id: 2,
                },
                Checksum {
                    id: 100,
                    value: 4_634_958_717_477_794_239,
                    worker_id: 0,
                },
            ])
        );
    }

    #[test]
    fn worker_panic_is_reported_with_the_smallest_panicking_worker_id() {
        fn panics_on_every_block(_bytes: &[u8]) -> Option<u64> {
            panic!("worker failed on purpose");
        }

        // 只有位置 0 的任务会交给 worker 0，worker 1 没有任务，所以这个
        // 场景每次都会稳定地报告 worker 0。
        assert_eq!(
            run_pipeline_with(vec![task(7, b"x")], 2, panics_on_every_block),
            Err(PipelineError::WorkerPanicked { worker_id: 0 })
        );
    }

    #[test]
    fn a_panic_in_one_worker_is_reported_with_that_worker_id_instead_of_partial_results() {
        fn panics_on_bad_block(bytes: &[u8]) -> Option<u64> {
            if bytes == b"bad".as_slice() {
                panic!("bad block");
            }
            Some(checksum(bytes))
        }

        let tasks = vec![task(10, b"ok"), task(20, b"bad"), task(30, b"ok")];

        assert_eq!(
            run_pipeline_with(tasks, 2, panics_on_bad_block),
            Err(PipelineError::WorkerPanicked { worker_id: 1 })
        );
    }

    #[test]
    fn panic_while_dispatching_still_joins_every_worker_before_returning() {
        fn panics_on_every_block(_bytes: &[u8]) -> Option<u64> {
            panic!("worker failed on purpose");
        }

        let tasks = (0..8).map(|id| task(id, b"x")).collect();

        assert_eq!(
            run_pipeline_with(tasks, 3, panics_on_every_block),
            Err(PipelineError::WorkerPanicked { worker_id: 0 })
        );
    }

    #[test]
    fn missing_results_are_reported_as_a_count_mismatch_after_all_workers_finish() {
        fn produces_no_result(_bytes: &[u8]) -> Option<u64> {
            None
        }

        let tasks = vec![task(1, b"a"), task(2, b"b"), task(3, b"c")];

        assert_eq!(
            run_pipeline_with(tasks, 2, produces_no_result),
            Err(PipelineError::ResultCountMismatch {
                expected: 3,
                actual: 0,
            })
        );
    }

    #[test]
    fn custom_work_function_can_transform_each_block_without_changing_the_worker_order() {
        fn length_as_u64(bytes: &[u8]) -> Option<u64> {
            Some(bytes.len() as u64)
        }

        let tasks = vec![task(4, b"abcd"), task(1, b"a"), task(9, b"")];

        assert_eq!(
            run_pipeline_with(tasks, 2, length_as_u64),
            Ok(vec![
                Checksum {
                    id: 1,
                    value: 1,
                    worker_id: 1,
                },
                Checksum {
                    id: 4,
                    value: 4,
                    worker_id: 0,
                },
                Checksum {
                    id: 9,
                    value: 0,
                    worker_id: 0,
                },
            ])
        );
    }

    #[test]
    fn default_pipeline_uses_the_same_round_robin_assignment_as_custom_work() {
        let default_tasks = vec![task(2, b"b"), task(1, b"a")];
        let custom_tasks = vec![task(2, b"b"), task(1, b"a")];

        assert_eq!(
            run_pipeline(default_tasks, 2),
            run_pipeline_with(custom_tasks, 2, keep_every_result)
        );
    }
}
