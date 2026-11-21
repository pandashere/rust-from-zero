//! 主练习：并发构建倒排索引。
//!
//! 倒排索引（inverted index，把“单词”映射到“包含它的文档编号”的表）是
//! 搜索引擎最基础的数据结构之一。本练习把一批文档按输入位置轮流分给
//! 固定数量的工作线程（worker，执行任务的线程），每个线程先在本地完成
//! 分词和去重，再把本地结果合并进一个所有线程共享的索引。
//!
//! 请只替换函数体里的 `todo!()`，不要修改类型定义、函数签名，也不要
//! 改动本文件底部的测试。每个函数上方的注释就是它的完整行为约定。
//!
//! 完成后的验收命令：
//!
//! ```bash
//! cargo test -p l17_shared_concurrency
//! ```

use std::collections::HashMap;
use std::sync::atomic::AtomicUsize;
use std::sync::{Arc, Mutex};
use std::thread;

/// 一份待索引的文档。
///
/// `id` 是文档编号，会出现在索引结果里；`text` 是文档正文。编号允许
/// 重复，重复编号在最终结果中只保留一次。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    pub id: usize,
    pub text: String,
}

/// 索引：每个规范化单词映射到包含它的文档编号列表。
pub type Index = HashMap<String, Vec<usize>>;

/// 可被多个线程共同拥有的索引。
pub type SharedIndex = Arc<Mutex<Index>>;

/// 构建索引时可能出现的错误。
///
/// 实现时可以直接构造这些变体。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IndexError {
    /// 请求的工作线程数为 0，无法分配任何文档。
    ZeroWorkers,
    /// `worker_id` 号线程在合并结果前 panic（当前线程异常终止）。
    WorkerPanicked { worker_id: usize },
    /// 共享索引的互斥锁已经被之前的 panic 标记为“中毒”。
    IndexPoisoned,
    /// 已处理文档数与输入文档数不一致。
    ProcessedCountMismatch { expected: usize, actual: usize },
}

/// 把一段文本切成规范化单词。
///
/// 这个函数已经写好，不需要修改。规则是：
///
/// - 连续的 Unicode 字母或数字字符组成一个单词；下划线、空白、标点和
///   其他符号都是分隔符；
/// - 每个字符用 `char::to_lowercase` 转成小写，一个字符可能变成多个
///   小写字符；
/// - 不做额外的 Unicode 规范化或整词大小写折叠；
/// - 没有任何单词的文本返回空列表。
pub fn normalized_words(text: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current = String::new();

    for character in text.chars() {
        if character.is_alphanumeric() {
            for lowercase in character.to_lowercase() {
                current.push(lowercase);
            }
        } else if !current.is_empty() {
            words.push(std::mem::take(&mut current));
        }
    }

    if !current.is_empty() {
        words.push(current);
    }

    words
}

/// 顺序处理一个线程分到的文档，构建该线程自己的局部索引。
///
/// 行为约定：
///
/// - 本函数不创建线程，只处理传入的这一批文档；
/// - 按 `documents` 的先后顺序处理，每份文档的正文交给
///   [`normalized_words`] 分词；
/// - 同一个单词在同一份文档中出现多次时，该文档编号只追加一次；
/// - 同一个单词出现在不同文档中时，每个文档编号各追加一次；
/// - 没有单词的文档不产生索引项，但仍然算“已处理”；
/// - 每完整处理一份文档，`processed` 加 1，使用
///   `Ordering::Relaxed`；这个计数只记录进度，不保护索引内容。
pub fn index_documents(documents: Vec<Document>, processed: &AtomicUsize) -> Index {
    let _ = &processed;
    let _ = &documents;
    todo!()
}

/// 把一份局部索引合并进共享索引。
///
/// 行为约定：
///
/// - 合并期间必须独占访问共享索引，其他线程不能同时修改它；
/// - 对局部索引里的每个单词，把它的所有文档编号追加到共享索引的对应
///   列表末尾；共享索引中原本没有的单词要新建列表；
/// - 本函数不负责排序或去重，最终整理由 [`finalize`] 完成；
/// - 如果互斥锁已经被 panic 标记为中毒，返回
///   `Err(IndexError::IndexPoisoned)`，不能静默取出内部数据继续使用；
/// - `local` 的所有权被消耗，调用后不能继续使用。
pub fn merge_into(shared: &SharedIndex, local: Index) -> Result<(), IndexError> {
    let _ = &shared;
    let _ = &local;
    todo!()
}

/// 从共享索引取出一份快照，并整理成最终结果。
///
/// 行为约定：
///
/// - 读取共享索引时如果发现互斥锁已中毒，返回
///   `Err(IndexError::IndexPoisoned)`；
/// - 每个单词的文档编号列表按从小到大排序；
/// - 排序后去掉重复的文档编号，例如 `[3, 1, 3]` 变成 `[1, 3]`；
/// - 返回的 `Index` 与共享索引不再共享数据，调用者可以独立使用；
/// - 返回结果中键的顺序不重要，调用者需要顺序时应自行排序。
pub fn finalize(shared: &SharedIndex) -> Result<Index, IndexError> {
    let _ = shared;
    todo!()
}

/// 等待所有工作线程结束，然后检查错误、进度和最终索引。
///
/// 这是 [`build_index`] 的内部步骤，单独拆出来是为了让错误处理可以被
/// 直接测试。`handles` 中第 `i` 个线程的编号就是 `i`。
///
/// 行为约定：
///
/// - 必须等待 `handles` 中的每一个线程结束，即使前面的线程已经 panic；
/// - 外层 `join()` 返回 `Err` 表示线程本身 panic；记录编号最小的那个，
///   最后返回 `WorkerPanicked { worker_id }`；
/// - 线程正常结束但返回 `Err` 表示工作线程发现了业务错误；在所有线程
///   都结束之后，返回第一个这样的错误；
/// - 错误优先级从高到低是：`WorkerPanicked`、线程返回的业务错误（目前
///   只可能是 `IndexPoisoned`）、`ProcessedCountMismatch`；
/// - 没有错误时，把 `processed` 的当前值与 `expected` 比较；不相等就
///   返回 `ProcessedCountMismatch { expected, actual }`，不返回部分索引；
/// - 检查通过后调用 [`finalize`] 返回整理好的索引。
// 在 `build_index` 完成之前，这个内部函数还没有调用者；完成实现后可以删掉本行。
#[allow(dead_code)]
fn join_workers(
    handles: Vec<thread::JoinHandle<Result<(), IndexError>>>,
    expected: usize,
    processed: &AtomicUsize,
    shared: &SharedIndex,
) -> Result<Index, IndexError> {
    let _ = &handles;
    let _ = &expected;
    let _ = &processed;
    let _ = &shared;
    todo!()
}

/// 用固定数量的工作线程并发构建倒排索引。
///
/// 行为约定：
///
/// - `worker_count` 为 0 时立即返回 `Err(IndexError::ZeroWorkers)`，
///   空输入也一样；
/// - 保存输入文档总数，然后把每份文档及其 `text` 按输入位置轮流分配给
///   各线程：第 0 份给 0 号，第 1 份给 1 号，依此类推，分完一轮再从
///   0 号继续；不得深拷贝文档正文；
/// - 输入为空但 `worker_count > 0` 时，仍然创建规定数量的线程，最终
///   返回空索引；
/// - 每个线程先调用 [`index_documents`] 构建局部索引，再调用
///   [`merge_into`] 合并；分词、逐文档去重和局部索引构建都必须发生在
///   共享索引锁之外；
/// - 所有线程都通过 `join_workers` 统一等待和汇总错误；
/// - 成功时每个文档编号列表升序且无重复，结果不依赖线程完成顺序；
/// - 输入文档的编号可以重复，最终列表会去重。
pub fn build_index(documents: Vec<Document>, worker_count: usize) -> Result<Index, IndexError> {
    let _ = &documents;
    let _ = &worker_count;
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::Ordering;

    fn document(id: usize, text: &str) -> Document {
        Document {
            id,
            text: text.to_string(),
        }
    }

    fn completed(result: Result<(), IndexError>) -> thread::JoinHandle<Result<(), IndexError>> {
        thread::spawn(move || result)
    }

    fn poison_index(shared: &SharedIndex) {
        let worker = Arc::clone(shared);
        let _ = thread::spawn(move || {
            let _guard = worker.lock().unwrap();
            panic!("poison the shared index on purpose");
        })
        .join();
    }

    #[test]
    fn normalized_words_follows_the_documented_rules() {
        assert_eq!(
            normalized_words("C++ Rust_2026 你好，世界！"),
            ["c", "rust", "2026", "你好", "世界"]
        );
        assert_eq!(normalized_words("Rust_rust's"), ["rust", "rust", "s"]);
        assert_eq!(normalized_words("İ"), ["i\u{307}"]);
        assert!(normalized_words("!!! ___").is_empty());
    }

    #[test]
    fn local_index_deduplicates_words_within_each_document() {
        let processed = AtomicUsize::new(0);
        let documents = vec![
            document(8, "Rust rust, RUST!"),
            document(3, "Rust 并发"),
            document(5, "并发基础"),
        ];

        let index = index_documents(documents, &processed);

        assert_eq!(processed.load(Ordering::Relaxed), 3);
        assert_eq!(index["rust"], [8, 3]);
        assert_eq!(index["并发"], [3]);
        assert_eq!(index["并发基础"], [5]);
    }

    #[test]
    fn documents_without_words_are_counted_but_not_indexed() {
        let processed = AtomicUsize::new(7);
        let documents = vec![document(1, "   "), document(2, "!!! ___")];

        let index = index_documents(documents, &processed);

        assert!(index.is_empty());
        assert_eq!(processed.load(Ordering::Relaxed), 9);
    }

    #[test]
    fn merging_appends_every_document_id() {
        let shared: SharedIndex = Arc::new(Mutex::new(Index::new()));
        let mut local = Index::new();
        local.insert("rust".to_string(), vec![2]);
        local.insert("safe".to_string(), vec![1]);

        merge_into(&shared, local).unwrap();

        let snapshot = shared.lock().unwrap();
        assert_eq!(snapshot["rust"], [2]);
        assert_eq!(snapshot["safe"], [1]);
    }

    #[test]
    fn merging_reports_a_poisoned_mutex() {
        let shared: SharedIndex = Arc::new(Mutex::new(Index::new()));
        poison_index(&shared);

        let result = merge_into(&shared, Index::new());

        assert_eq!(result, Err(IndexError::IndexPoisoned));
    }

    #[test]
    fn finalizing_sorts_and_deduplicates_ids() {
        let mut raw = Index::new();
        raw.insert("b".to_string(), vec![3, 1, 3, 1]);
        raw.insert("a".to_string(), vec![2]);
        let shared: SharedIndex = Arc::new(Mutex::new(raw));

        let index = finalize(&shared).unwrap();

        assert_eq!(index["a"], [2]);
        assert_eq!(index["b"], [1, 3]);
    }

    #[test]
    fn finalizing_reports_a_poisoned_mutex() {
        let shared: SharedIndex = Arc::new(Mutex::new(Index::new()));
        poison_index(&shared);

        assert_eq!(finalize(&shared), Err(IndexError::IndexPoisoned));
    }

    #[test]
    fn joining_reports_the_first_panicking_worker() {
        let handles: Vec<thread::JoinHandle<Result<(), IndexError>>> = vec![
            completed(Err(IndexError::IndexPoisoned)),
            thread::spawn(|| panic!("worker failed on purpose")),
            thread::spawn(|| panic!("another worker failed on purpose")),
        ];
        let shared: SharedIndex = Arc::new(Mutex::new(Index::new()));
        let processed = AtomicUsize::new(0);

        let result = join_workers(handles, 0, &processed, &shared);

        assert_eq!(result, Err(IndexError::WorkerPanicked { worker_id: 1 }));
    }

    #[test]
    fn joining_waits_for_every_worker_even_after_a_panic() {
        let finished = Arc::new(AtomicUsize::new(0));
        let worker_finished = Arc::clone(&finished);
        let handles: Vec<thread::JoinHandle<Result<(), IndexError>>> = vec![
            thread::spawn(|| panic!("worker failed on purpose")),
            thread::spawn(move || {
                worker_finished.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }),
        ];
        let shared: SharedIndex = Arc::new(Mutex::new(Index::new()));
        let processed = AtomicUsize::new(0);

        let _ = join_workers(handles, 0, &processed, &shared);

        assert_eq!(finished.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn joining_reports_a_business_error_before_a_count_mismatch() {
        let handles = vec![completed(Err(IndexError::IndexPoisoned))];
        let shared: SharedIndex = Arc::new(Mutex::new(Index::new()));
        let processed = AtomicUsize::new(0);

        let result = join_workers(handles, 5, &processed, &shared);

        assert_eq!(result, Err(IndexError::IndexPoisoned));
    }

    #[test]
    fn joining_reports_a_processed_count_mismatch() {
        let handles = vec![completed(Ok(())), completed(Ok(()))];
        let shared: SharedIndex = Arc::new(Mutex::new(Index::new()));
        let processed = AtomicUsize::new(1);

        let result = join_workers(handles, 3, &processed, &shared);

        assert_eq!(
            result,
            Err(IndexError::ProcessedCountMismatch {
                expected: 3,
                actual: 1,
            })
        );
    }

    #[test]
    fn joining_finalizes_a_successful_run() {
        let mut raw = Index::new();
        raw.insert("rust".to_string(), vec![4, 2, 4]);
        let shared: SharedIndex = Arc::new(Mutex::new(raw));
        let processed = AtomicUsize::new(1);

        let index = join_workers(vec![completed(Ok(()))], 1, &processed, &shared).unwrap();

        assert_eq!(index["rust"], [2, 4]);
    }

    #[test]
    fn build_index_matches_the_documented_example() {
        let documents = vec![
            document(8, "Rust rust, RUST!"),
            document(3, "Rust 并发"),
            document(5, "并发基础"),
        ];

        let index = build_index(documents, 2).unwrap();

        assert_eq!(index.len(), 3);
        assert_eq!(index["rust"], [3, 8]);
        assert_eq!(index["并发"], [3]);
        assert_eq!(index["并发基础"], [5]);
    }

    #[test]
    fn build_index_sorts_ids_when_workers_finish_in_any_order() {
        let documents = vec![document(9, "shared"), document(2, "shared")];

        let index = build_index(documents, 3).unwrap();

        assert_eq!(index["shared"], [2, 9]);
    }

    #[test]
    fn build_index_deduplicates_repeated_document_ids() {
        let documents = vec![document(5, "same"), document(5, "same")];

        let index = build_index(documents, 2).unwrap();

        assert_eq!(index["same"], [5]);
    }

    #[test]
    fn build_index_accepts_empty_input_with_workers() {
        assert_eq!(build_index(Vec::new(), 3), Ok(Index::new()));
    }

    #[test]
    fn build_index_rejects_zero_workers() {
        assert_eq!(build_index(Vec::new(), 0), Err(IndexError::ZeroWorkers));
        assert_eq!(
            build_index(vec![document(1, "rust")], 0),
            Err(IndexError::ZeroWorkers)
        );
    }

    #[test]
    fn build_index_ignores_documents_without_words() {
        let documents = vec![document(1, "  "), document(2, "Rust")];

        let index = build_index(documents, 4).unwrap();

        assert_eq!(index.len(), 1);
        assert_eq!(index["rust"], [2]);
    }

    #[test]
    fn repeated_builds_produce_the_same_index() {
        let documents = || {
            vec![
                document(1, "alpha beta"),
                document(2, "beta gamma"),
                document(3, "alpha"),
                document(4, "delta"),
                document(5, "gamma alpha"),
            ]
        };

        let first = build_index(documents(), 4).unwrap();
        for _ in 0..10 {
            assert_eq!(build_index(documents(), 4).unwrap(), first);
        }
    }
}
