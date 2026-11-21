//! 主练习的行为测试。
//!
//! 测试只使用公开 API，并且完全确定：没有真实网络、没有墙钟睡眠。
//! 在 `src/exercise.rs` 还是脚手架时，这些测试会因为 `todo!()` 失败；
//! 实现完成后它们应全部通过。

use l19_async::exercise::{
    BatchConfig, BatchError, FakeService, Job, JobError, JobOutcome, RetryPolicy, Step, TimedOut,
    run_batch, with_timeout,
};
use l19_async::runtime::{block_on, yield_now};

fn job(id: usize) -> Job {
    Job::new(id, format!("user{id}@example.test"), format!("消息 {id}"))
}

fn policy(max_attempts: usize, patience: usize) -> RetryPolicy {
    RetryPolicy {
        max_attempts,
        patience,
    }
}

fn config(limit: usize, max_attempts: usize, patience: usize) -> BatchConfig {
    BatchConfig {
        limit,
        policy: policy(max_attempts, patience),
    }
}

async fn after_yields<T>(yields: usize, value: T) -> T {
    for _ in 0..yields {
        yield_now().await;
    }
    value
}

fn success_text(id: usize) -> String {
    format!("已通知 user{id}@example.test：消息 {id}")
}

#[test]
fn with_timeout_returns_output_within_patience() {
    assert_eq!(
        block_on(with_timeout(after_yields(2, "完成"), 2)),
        Ok("完成")
    );
}

#[test]
fn with_timeout_allows_zero_pending_polls_for_a_ready_future() {
    assert_eq!(block_on(with_timeout(after_yields(0, 7), 0)), Ok(7));
}

#[test]
fn with_timeout_reports_a_future_that_needs_one_more_poll() {
    assert_eq!(block_on(with_timeout(after_yields(1, 7), 0)), Err(TimedOut));
}

#[test]
fn empty_batch_returns_empty_report() {
    let service = FakeService::new(vec![]);
    let report = block_on(run_batch(&service, vec![], config(2, 3, 10))).expect("有效配置应成功");

    assert!(report.outcomes.is_empty());
    assert_eq!(service.peak_active(), 0);
}

#[test]
fn zero_limit_is_rejected_even_for_empty_input() {
    let service = FakeService::new(vec![]);
    let result = block_on(run_batch(&service, vec![], config(0, 3, 10)));

    assert_eq!(result, Err(BatchError::ZeroLimit));
}

#[test]
fn zero_attempts_is_rejected() {
    let service = FakeService::new(vec![]);
    let result = block_on(run_batch(&service, vec![job(1)], config(2, 0, 10)));

    assert_eq!(result, Err(BatchError::ZeroAttempts));
}

#[test]
fn zero_limit_is_reported_before_zero_attempts() {
    let service = FakeService::new(vec![]);
    let result = block_on(run_batch(&service, vec![], config(0, 0, 10)));

    assert_eq!(result, Err(BatchError::ZeroLimit));
}

#[test]
fn single_success_reports_output_and_one_attempt() {
    let service = FakeService::new(vec![(1, vec![Step::success(0)])]);
    let report =
        block_on(run_batch(&service, vec![job(1)], config(4, 3, 5))).expect("有效配置应成功");

    assert_eq!(
        report.outcomes,
        vec![JobOutcome::Success {
            id: 1,
            output: success_text(1),
            attempts: 1,
        }]
    );
    assert_eq!(service.calls_for(1), 1);
}

#[test]
fn temporary_failure_then_success_uses_two_attempts() {
    let service = FakeService::new(vec![(
        1,
        vec![Step::temporary("第一次网络抖动", 0), Step::success(0)],
    )]);
    let report =
        block_on(run_batch(&service, vec![job(1)], config(4, 3, 5))).expect("有效配置应成功");

    assert_eq!(
        report.outcomes,
        vec![JobOutcome::Success {
            id: 1,
            output: success_text(1),
            attempts: 2,
        }]
    );
    assert_eq!(service.calls_for(1), 2);
}

#[test]
fn retries_stop_after_max_attempts() {
    let service = FakeService::new(vec![(
        1,
        vec![
            Step::temporary("第一次失败", 0),
            Step::temporary("第二次失败", 0),
            Step::success(0),
        ],
    )]);
    let report =
        block_on(run_batch(&service, vec![job(1)], config(4, 2, 5))).expect("有效配置应成功");

    assert_eq!(
        report.outcomes,
        vec![JobOutcome::Failed {
            id: 1,
            error: JobError::TemporaryExhausted {
                last_reason: String::from("第二次失败"),
            },
            attempts: 2,
        }]
    );
    assert_eq!(service.calls_for(1), 2);
}

#[test]
fn max_attempts_one_means_a_temporary_failure_is_final() {
    let service = FakeService::new(vec![(
        1,
        vec![Step::temporary("只允许一次", 0), Step::success(0)],
    )]);
    let report =
        block_on(run_batch(&service, vec![job(1)], config(4, 1, 5))).expect("有效配置应成功");

    assert_eq!(
        report.outcomes,
        vec![JobOutcome::Failed {
            id: 1,
            error: JobError::TemporaryExhausted {
                last_reason: String::from("只允许一次"),
            },
            attempts: 1,
        }]
    );
    assert_eq!(service.calls_for(1), 1);
}

#[test]
fn permanent_failure_is_not_retried() {
    let service = FakeService::new(vec![(
        1,
        vec![Step::permanent("收件人不存在", 0), Step::success(0)],
    )]);
    let report =
        block_on(run_batch(&service, vec![job(1)], config(4, 5, 5))).expect("有效配置应成功");

    assert_eq!(
        report.outcomes,
        vec![JobOutcome::Failed {
            id: 1,
            error: JobError::Permanent {
                reason: String::from("收件人不存在"),
            },
            attempts: 1,
        }]
    );
    assert_eq!(service.calls_for(1), 1);
}

#[test]
fn timeout_is_distinct_and_not_retried() {
    let service = FakeService::new(vec![(1, vec![Step::success(5)])]);
    let report =
        block_on(run_batch(&service, vec![job(1)], config(4, 3, 2))).expect("有效配置应成功");

    assert_eq!(
        report.outcomes,
        vec![JobOutcome::Failed {
            id: 1,
            error: JobError::TimedOut,
            attempts: 1,
        }]
    );
    assert_eq!(service.calls_for(1), 1);
    assert_eq!(service.active_now(), 0);
}

#[test]
fn timeout_allows_exactly_patience_pending_polls() {
    let service = FakeService::new(vec![(1, vec![Step::success(2)])]);
    let report =
        block_on(run_batch(&service, vec![job(1)], config(4, 3, 2))).expect("有效配置应成功");

    assert_eq!(
        report.outcomes,
        vec![JobOutcome::Success {
            id: 1,
            output: success_text(1),
            attempts: 1,
        }]
    );
}

#[test]
fn timeout_triggers_after_patience_pending_polls() {
    let service = FakeService::new(vec![(1, vec![Step::success(3)])]);
    let report =
        block_on(run_batch(&service, vec![job(1)], config(4, 3, 2))).expect("有效配置应成功");

    assert_eq!(
        report.outcomes,
        vec![JobOutcome::Failed {
            id: 1,
            error: JobError::TimedOut,
            attempts: 1,
        }]
    );
    assert_eq!(service.active_now(), 0);
}

#[test]
fn panicked_attempt_becomes_a_structured_failure() {
    let service = FakeService::new(vec![(1, vec![Step::Panic])]);
    let report =
        block_on(run_batch(&service, vec![job(1)], config(4, 3, 5))).expect("有效配置应成功");

    assert_eq!(
        report.outcomes,
        vec![JobOutcome::Failed {
            id: 1,
            error: JobError::Panicked,
            attempts: 1,
        }]
    );
    assert_eq!(service.calls_for(1), 1);
    assert_eq!(service.active_now(), 0);
}

#[test]
fn results_are_sorted_by_id_regardless_of_completion_order() {
    let service = FakeService::new(vec![
        (3, vec![Step::success(2)]),
        (1, vec![Step::success(0)]),
        (2, vec![Step::success(1)]),
    ]);
    let report = block_on(run_batch(
        &service,
        vec![job(3), job(1), job(2)],
        config(10, 2, 10),
    ))
    .expect("有效配置应成功");

    assert_eq!(
        report.outcomes,
        vec![
            JobOutcome::Success {
                id: 1,
                output: success_text(1),
                attempts: 1,
            },
            JobOutcome::Success {
                id: 2,
                output: success_text(2),
                attempts: 1,
            },
            JobOutcome::Success {
                id: 3,
                output: success_text(3),
                attempts: 1,
            },
        ]
    );
}

#[test]
fn limit_one_runs_jobs_without_overlap() {
    let ids = [1, 2, 3, 4];
    let service = FakeService::new(ids.iter().map(|&id| (id, vec![Step::success(2)])).collect());
    let jobs = ids.iter().map(|&id| job(id)).collect();
    let report = block_on(run_batch(&service, jobs, config(1, 2, 10))).expect("有效配置应成功");

    assert_eq!(report.outcomes.len(), 4);
    assert_eq!(service.peak_active(), 1);
}

#[test]
fn limit_two_caps_concurrency() {
    let ids = [1, 2, 3, 4, 5];
    let service = FakeService::new(ids.iter().map(|&id| (id, vec![Step::success(2)])).collect());
    let jobs = ids.iter().map(|&id| job(id)).collect();
    let report = block_on(run_batch(&service, jobs, config(2, 2, 10))).expect("有效配置应成功");

    assert_eq!(report.outcomes.len(), 5);
    assert_eq!(service.peak_active(), 2);
}

#[test]
fn limit_above_job_count_runs_all_jobs_together() {
    let ids = [1, 2, 3];
    let service = FakeService::new(ids.iter().map(|&id| (id, vec![Step::success(2)])).collect());
    let jobs = ids.iter().map(|&id| job(id)).collect();
    let report = block_on(run_batch(&service, jobs, config(10, 2, 10))).expect("有效配置应成功");

    assert_eq!(report.outcomes.len(), 3);
    assert_eq!(service.peak_active(), 3);
}

#[test]
fn one_failure_keeps_the_other_outcomes() {
    let service = FakeService::new(vec![
        (1, vec![Step::success(4)]),
        (2, vec![Step::permanent("地址不存在", 0)]),
        (3, vec![Step::success(1)]),
    ]);
    let report = block_on(run_batch(
        &service,
        vec![job(1), job(2), job(3)],
        config(3, 3, 10),
    ))
    .expect("有效配置应成功");

    assert_eq!(
        report.outcomes,
        vec![
            JobOutcome::Success {
                id: 1,
                output: success_text(1),
                attempts: 1,
            },
            JobOutcome::Failed {
                id: 2,
                error: JobError::Permanent {
                    reason: String::from("地址不存在"),
                },
                attempts: 1,
            },
            JobOutcome::Success {
                id: 3,
                output: success_text(3),
                attempts: 1,
            },
        ]
    );
}

#[test]
fn unicode_job_text_is_preserved_in_output() {
    let service = FakeService::new(vec![(7, vec![Step::success(0)])]);
    let unicode_job = Job::new(7, "张三", "你好，世界 🌍");
    let report =
        block_on(run_batch(&service, vec![unicode_job], config(1, 1, 3))).expect("有效配置应成功");

    assert_eq!(
        report.outcomes,
        vec![JobOutcome::Success {
            id: 7,
            output: String::from("已通知 张三：你好，世界 🌍"),
            attempts: 1,
        }]
    );
}

#[test]
fn missing_script_step_is_a_permanent_failure() {
    let service = FakeService::new(vec![]);
    let report =
        block_on(run_batch(&service, vec![job(9)], config(1, 3, 3))).expect("有效配置应成功");

    match &report.outcomes[0] {
        JobOutcome::Failed {
            error: JobError::Permanent { reason },
            attempts,
            ..
        } => {
            assert_eq!(*attempts, 1);
            assert_eq!(reason, "作业 #9 没有第 1 次尝试的脚本");
        }
        other => panic!("期望永久失败，实际得到 {other:?}"),
    }
    assert_eq!(service.calls_for(9), 1);
}
