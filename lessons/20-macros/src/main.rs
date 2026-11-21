//! 第二十课的演示入口：用主练习的宏生成清单，再用课后巩固的宏生成数字表。
//!
//! 在完成 `exercise.rs` 和 `consolidation.rs` 中的宏之前，运行本程序会在
//! 第一次调用宏时中止（panic）——这是预期现象。先去做练习，再回来看完整
//! 输出。

use l20_macros::exercise::{StockError, StockItem, take_stock};
use l20_macros::{matrix, stock};

fn print_stock(label: &str, items: &[StockItem]) {
    println!("{label}：");
    for item in items {
        match take_stock(item, 2) {
            Ok(remaining) => println!(
                "  {:<8} 库存 {:>2}，取走 2 件后剩 {remaining}",
                item.name, item.quantity
            ),
            Err(StockError::NotEnough {
                available,
                requested,
            }) => println!(
                "  {:<8} 库存 {available}，想取 {requested} 件，数量不足",
                item.name
            ),
        }
    }
}

fn main() {
    let explicit = stock!["pen" => 12, "paper" => 3, "eraser" => 0];
    print_stock("写明数量的清单", &explicit);

    let defaults = stock!["ink", "ruler"];
    print_stock("使用默认数量的清单", &defaults);

    let empty: Vec<StockItem> = stock![];
    println!("空清单长度：{}", empty.len());

    let grid = matrix![[1, 2, 3], [4, 5], []];
    println!("数字表：{grid:?}");
}
