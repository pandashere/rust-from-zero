//! 第十九课的可运行入口：观察 future 的惰性与两个任务的交替推进。
//!
//! 这个演示只使用已经写好的 `runtime` 模块，不调用练习里的 `todo!()`，
//! 因此脚手架状态下也能直接运行：
//!
//! ```text
//! cargo run -p l19_async
//! ```

use l19_async::runtime::{block_on, join_all, yield_now};

async fn task(name: &str, steps: usize) {
    for step in 1..=steps {
        println!("{name}：第 {step} 步");
        yield_now().await;
    }
}

fn main() {
    let future = task("未启动", 3);
    println!("刚创建 future 时，任务体一行都还没运行。");
    drop(future);
    println!("丢弃 future 后，任务体仍然一行都没运行。");
    println!();

    println!("把两个任务交给 join_all，它们在同一个线程上交替推进：");
    block_on(join_all(vec![task("A", 3), task("B", 2)]));
    println!();
    println!("交替出现说明一个任务让出执行权时，另一个任务获得了推进机会。");
    println!();
    println!("主练习在 src/exercise.rs，课后巩固在 src/consolidation.rs。");
    println!("完成后运行：cargo test -p l19_async");
}
