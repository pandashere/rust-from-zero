//! 课后巩固：并发汇总传感器读数。
//!
//! 这是主练习之后的独立任务，数据与边界都不同：主练习处理文本和文档
//! 编号，这里处理带正负值的温度读数；主练习用 `Mutex` 汇总索引，这里
//! 用 `RwLock` 保存一张按传感器名字排序的统计表。
//!
//! 请只替换函数体里的 `todo!()`，不要修改类型定义、函数签名，也不要
//! 改动本文件底部的测试。
//!
//! 完成后的验收命令：
//!
//! ```bash
//! cargo test -p l17_shared_concurrency consolidation
//! ```

use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};
use std::thread;

/// 一条温度读数：哪个传感器，读数是多少摄氏度。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reading {
    pub sensor: String,
    pub celsius: i32,
}

/// 一个传感器的汇总结果。
///
/// `sum` 使用 `i64`，因为把许多 `i32` 相加可能超出 `i32` 的范围。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SensorSummary {
    pub count: usize,
    pub min: i32,
    pub max: i32,
    pub sum: i64,
}

/// 可被多个线程共同拥有的统计表。
pub type SharedSummary = Arc<RwLock<BTreeMap<String, SensorSummary>>>;

/// 汇总传感器读数时可能出现的错误。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SensorError {
    /// 请求的工作线程数为 0。
    ZeroWorkers,
    /// `worker_id` 号线程在合并前 panic。
    WorkerPanicked { worker_id: usize },
    /// 共享统计表的读写锁已经被 panic 标记为中毒。
    Poisoned,
}

/// 顺序汇总一个线程分到的读数，返回该线程自己的局部统计表。
///
/// 行为约定：
///
/// - 本函数不创建线程，只处理传入的这批读数；
/// - 每个传感器只出现一次，`count` 是该传感器的读数条数；
/// - `min` 和 `max` 是出现过的最低、最高温度；
/// - `sum` 是所有读数用 `i64` 相加的结果；
/// - 空输入返回空表；
/// - 同一个传感器在本批次出现多次时，要合并到同一条统计里。
pub fn summarize_batch(readings: Vec<Reading>) -> BTreeMap<String, SensorSummary> {
    let _ = &readings;
    todo!()
}

/// 把一个线程的局部统计表合并进共享统计表。
///
/// 行为约定：
///
/// - 合并必须获得写锁，同一时刻不允许其他线程读或写这张表；
/// - 已存在的传感器：`count` 和 `sum` 相加，`min` 取更小值，`max`
///   取更大值；
/// - 不存在的传感器：插入它的统计结果；
/// - 如果读写锁已中毒，返回 `Err(SensorError::Poisoned)`，不能静默
///   取出内部数据；
/// - `local` 的所有权被消耗。
pub fn merge_summaries(
    shared: &SharedSummary,
    local: BTreeMap<String, SensorSummary>,
) -> Result<(), SensorError> {
    let _ = &shared;
    let _ = &local;
    todo!()
}

/// 等待所有工作线程结束，然后读取最终统计表。
///
/// 这是 [`summarize_readings`] 的内部步骤，单独拆出来是为了让错误处理
/// 可以被直接测试。`handles` 中第 `i` 个线程的编号就是 `i`。
///
/// 行为约定：
///
/// - 必须等待每一个线程结束，即使前面的线程已经 panic；
/// - 外层 `join()` 返回 `Err` 表示线程 panic；记录编号最小的那个，最后
///   返回 `WorkerPanicked { worker_id }`；
/// - 线程正常结束但返回 `Err` 时，在所有线程都结束之后返回第一个这样
///   的错误；
/// - 错误优先级：`WorkerPanicked` 高于业务错误；
/// - 没有错误时，用读锁取得一份快照，按传感器名字从小到大返回
///   `Vec<(String, SensorSummary)>`；
/// - 读取时如果锁已中毒，返回 `Poisoned`。
// 在 `summarize_readings` 完成之前，这个内部函数还没有调用者；完成后可以删掉本行。
#[allow(dead_code)]
fn join_summaries(
    handles: Vec<thread::JoinHandle<Result<(), SensorError>>>,
    shared: &SharedSummary,
) -> Result<Vec<(String, SensorSummary)>, SensorError> {
    let _ = &handles;
    let _ = &shared;
    todo!()
}

/// 用固定数量的工作线程汇总一批传感器读数。
///
/// 行为约定：
///
/// - `worker_count` 为 0 时立即返回 `Err(SensorError::ZeroWorkers)`，
///   空输入也一样；
/// - 按读数在输入 `Vec` 中的位置轮流分配：第 0 条给 0 号线程，第 1 条
///   给 1 号线程，依此类推；`Reading` 及其 `sensor` 字符串必须移动给
///   线程，不得深拷贝；
/// - 每个线程先调用 [`summarize_batch`] 构建局部统计，再调用
///   [`merge_summaries`] 合并；局部统计必须发生在写锁之外；
/// - 所有线程都通过 `join_summaries` 统一等待和汇总错误；
/// - 输入为空但 `worker_count > 0` 时，仍然创建规定数量的线程，返回
///   空列表；
/// - 返回的列表按传感器名字升序排列，不依赖线程完成顺序。
pub fn summarize_readings(
    readings: Vec<Reading>,
    worker_count: usize,
) -> Result<Vec<(String, SensorSummary)>, SensorError> {
    let _ = &readings;
    let _ = &worker_count;
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn reading(sensor: &str, celsius: i32) -> Reading {
        Reading {
            sensor: sensor.to_string(),
            celsius,
        }
    }

    fn completed(result: Result<(), SensorError>) -> thread::JoinHandle<Result<(), SensorError>> {
        thread::spawn(move || result)
    }

    fn poison_summary(shared: &SharedSummary) {
        let worker = Arc::clone(shared);
        let _ = thread::spawn(move || {
            let _guard = worker.write().unwrap();
            panic!("poison the shared summary on purpose");
        })
        .join();
    }

    #[test]
    fn batch_aggregates_each_sensor_once() {
        let readings = vec![
            reading("kitchen", 21),
            reading("harbor", 18),
            reading("kitchen", -3),
        ];

        let summary = summarize_batch(readings);

        assert_eq!(
            summary["kitchen"],
            SensorSummary {
                count: 2,
                min: -3,
                max: 21,
                sum: 18,
            }
        );
        assert_eq!(
            summary["harbor"],
            SensorSummary {
                count: 1,
                min: 18,
                max: 18,
                sum: 18,
            }
        );
    }

    #[test]
    fn batch_with_no_readings_is_empty() {
        assert!(summarize_batch(Vec::new()).is_empty());
    }

    #[test]
    fn merging_combines_counts_ranges_and_sums() {
        let shared: SharedSummary = Arc::new(RwLock::new(BTreeMap::new()));
        let mut local = BTreeMap::new();
        local.insert(
            "kitchen".to_string(),
            SensorSummary {
                count: 2,
                min: -3,
                max: 21,
                sum: 18,
            },
        );

        merge_summaries(&shared, local).unwrap();

        let mut second = BTreeMap::new();
        second.insert(
            "kitchen".to_string(),
            SensorSummary {
                count: 1,
                min: 0,
                max: 30,
                sum: 30,
            },
        );
        merge_summaries(&shared, second).unwrap();

        let snapshot = shared.read().unwrap();
        assert_eq!(
            snapshot["kitchen"],
            SensorSummary {
                count: 3,
                min: -3,
                max: 30,
                sum: 48,
            }
        );
    }

    #[test]
    fn merging_reports_a_poisoned_lock() {
        let shared: SharedSummary = Arc::new(RwLock::new(BTreeMap::new()));
        poison_summary(&shared);

        assert_eq!(
            merge_summaries(&shared, BTreeMap::new()),
            Err(SensorError::Poisoned)
        );
    }

    #[test]
    fn joining_reports_the_first_panicking_worker() {
        let handles: Vec<thread::JoinHandle<Result<(), SensorError>>> = vec![
            completed(Err(SensorError::Poisoned)),
            thread::spawn(|| panic!("worker failed on purpose")),
        ];
        let shared: SharedSummary = Arc::new(RwLock::new(BTreeMap::new()));

        let result = join_summaries(handles, &shared);

        assert_eq!(result, Err(SensorError::WorkerPanicked { worker_id: 1 }));
    }

    #[test]
    fn joining_waits_for_every_worker_even_after_a_panic() {
        let finished = Arc::new(AtomicUsize::new(0));
        let worker_finished = Arc::clone(&finished);
        let handles: Vec<thread::JoinHandle<Result<(), SensorError>>> = vec![
            thread::spawn(|| panic!("worker failed on purpose")),
            thread::spawn(move || {
                worker_finished.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }),
        ];
        let shared: SharedSummary = Arc::new(RwLock::new(BTreeMap::new()));

        let _ = join_summaries(handles, &shared);

        assert_eq!(finished.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn joining_reports_a_business_error() {
        let handles = vec![completed(Err(SensorError::Poisoned))];
        let shared: SharedSummary = Arc::new(RwLock::new(BTreeMap::new()));

        assert_eq!(join_summaries(handles, &shared), Err(SensorError::Poisoned));
    }

    #[test]
    fn reading_threads_aggregate_a_mixed_batch() {
        let readings = vec![
            reading("kitchen", 21),
            reading("harbor", 18),
            reading("kitchen", -3),
            reading("balcony", 30),
            reading("harbor", 22),
        ];

        let summary = summarize_readings(readings, 3).unwrap();

        assert_eq!(
            summary,
            vec![
                (
                    "balcony".to_string(),
                    SensorSummary {
                        count: 1,
                        min: 30,
                        max: 30,
                        sum: 30,
                    },
                ),
                (
                    "harbor".to_string(),
                    SensorSummary {
                        count: 2,
                        min: 18,
                        max: 22,
                        sum: 40,
                    },
                ),
                (
                    "kitchen".to_string(),
                    SensorSummary {
                        count: 2,
                        min: -3,
                        max: 21,
                        sum: 18,
                    },
                ),
            ]
        );
    }

    #[test]
    fn reading_threads_accept_empty_input_with_workers() {
        assert_eq!(summarize_readings(Vec::new(), 4), Ok(Vec::new()));
    }

    #[test]
    fn reading_threads_reject_zero_workers() {
        assert_eq!(
            summarize_readings(Vec::new(), 0),
            Err(SensorError::ZeroWorkers)
        );
        assert_eq!(
            summarize_readings(vec![reading("kitchen", 20)], 0),
            Err(SensorError::ZeroWorkers)
        );
    }

    #[test]
    fn worker_count_may_exceed_reading_count() {
        let summary = summarize_readings(vec![reading("attic", -8)], 8).unwrap();

        assert_eq!(
            summary,
            vec![(
                "attic".to_string(),
                SensorSummary {
                    count: 1,
                    min: -8,
                    max: -8,
                    sum: -8,
                },
            )]
        );
    }

    #[test]
    fn repeated_runs_produce_the_same_summary() {
        let readings = || {
            vec![
                reading("a", 1),
                reading("b", -2),
                reading("a", 3),
                reading("c", 5),
                reading("b", -7),
            ]
        };

        let first = summarize_readings(readings(), 3).unwrap();
        for _ in 0..10 {
            assert_eq!(summarize_readings(readings(), 3).unwrap(), first);
        }
    }
}
