//! 第十七课的演示入口：并发构建一份小型倒排索引。
//!
//! 在完成 `exercise.rs` 中的 `todo!()` 之前，运行本程序会在第一次调用
//! `build_index` 时中止（panic）——这是预期现象。先完成主练习，再回来
//! 观察完整输出。
//!
//! ```bash
//! cargo run -p l17_shared_concurrency
//! ```

use l17_shared_concurrency::exercise::{Document, build_index};

fn main() {
    let documents = vec![
        Document {
            id: 8,
            text: String::from("Rust rust, RUST!"),
        },
        Document {
            id: 3,
            text: String::from("Rust 并发"),
        },
        Document {
            id: 5,
            text: String::from("并发基础"),
        },
    ];

    match build_index(documents, 2) {
        Ok(index) => {
            let mut entries: Vec<_> = index.iter().collect();
            entries.sort_by(|left, right| left.0.cmp(right.0));
            for (word, ids) in entries {
                println!("{word}: {ids:?}");
            }
        }
        Err(error) => println!("构建索引失败: {error:?}"),
    }
}
