//! 第十一课的演示入口：用几组输入展示泛型容器和定长窗口的行为。
//!
//! 在完成 `exercise.rs` 中的 `todo!()` 之前，运行本程序会在第一次调用
//! 待实现方法时中止（panic）。这是预期现象：先完成练习，再回来看完整
//! 输出。

use l11_generics::exercise::{FixedWindow, Pair};

fn main() {
    let scores = Pair::new(88, 92);
    println!("两个分数: {:?}", scores.as_tuple());
    println!("较大的分数: {}", scores.max_ref());
    println!("左边的副本: {:?}", scores.duplicated_left());

    let swapped = Pair::new(1, 2).swap();
    println!("交换后: {:?}", swapped);

    let mapped = Pair::new(2, 3).map(|value| value * 10);
    println!("统一乘 10: {:?}", mapped);

    let suffix = String::from("!");
    let mixed = Pair::new(String::from("a"), String::from("b"))
        .map_each(|value| value.to_uppercase(), |value| value + &suffix);
    println!("分别处理: {:?}", mixed);

    let labels = Pair::new(String::from("left"), String::from("right"));
    println!("连接后: {}", labels.joined(" / "));

    let window = FixedWindow::<i32, 3>::new([10, 20, 30]);
    println!(
        "窗口: 第一个 {:?}，最后一个 {:?}，切片 {:?}，位置 3 {:?}",
        window.first(),
        window.last(),
        window.as_slice(),
        window.get(3)
    );

    let empty = FixedWindow::<i32, 0>::new([]);
    println!(
        "空窗口: 第一个 {:?}，最后一个 {:?}",
        empty.first(),
        empty.last()
    );
}
