//! 第一课的可执行入口：用一组真实数据演示主练习三个函数的效果。
//!
//! 在 `src/exercise.rs` 中的函数还是 `todo!()` 时，运行本程序会在
//! 执行到未实现的函数时崩溃退出（panic）。这是正常的：先让
//! `cargo test -p l01_basics` 全部通过，再运行本程序查看演示输出。

use l01_basics::exercise::{clamp, count_multiples, sum_inclusive};

fn main() {
    let start = -3;
    let end = 12;

    let sum = sum_inclusive(start, end);
    let multiples_of_three = count_multiples(start, end, 3);
    let limited_sum = clamp(sum, 0, 50);

    println!("range: {start}..={end}");
    println!("sum: {sum}");
    println!("multiples of 3: {multiples_of_three}");
    println!("sum clamped to 0..=50: {limited_sum}");
}
