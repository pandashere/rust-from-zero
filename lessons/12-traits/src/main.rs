//! 第十二课的演示入口：把三种表单检查放进同一条流水线。
//!
//! 在完成 `exercise.rs` 中的 `todo!()` 之前，运行本程序会在第一次调用
//! 检查方法时中止（panic）——这是预期现象，先去做练习，再回来看输出。

use l12_traits::exercise::{Digits, ErasedCheck, MaxChars, NonEmpty, run_all, run_one};

fn main() {
    let input = "42";

    println!("单项检查: {:?}", run_one(&NonEmpty, input));

    let checks: Vec<Box<dyn ErasedCheck>> = vec![
        Box::new(NonEmpty),
        Box::new(MaxChars { limit: 1 }),
        Box::new(Digits),
    ];

    for (index, result) in run_all(&checks, input).iter().enumerate() {
        println!("第 {} 项: {:?}", index + 1, result);
    }
}
