//! 第三课可运行入口：一小段 move 与借用的演示，以及练习位置指引。
//! 这个演示不依赖练习代码，脚手架状态下也能直接 `cargo run -p l03_ownership_borrowing`。

fn text_len(text: &str) -> usize {
    text.len()
}

fn main() {
    // move：所有权从 first 转移到 second，first 从此失效
    let first = String::from("rust");
    let second = first;
    println!("move 之后 second 拥有文本：{second}");

    // 借用：text_len 只借走 text 看一眼，text 之后仍可用、仍可修改
    let mut text = String::from("hello");
    let length = text_len(&text);
    println!("借用读到长度 {length}，原值仍可用：{text}");
    text.push('!');
    println!("借用结束后还能修改：{text}");

    println!();
    println!("主练习在 src/lib.rs（Note），课后巩固在 src/text.rs。");
    println!("完成 todo!() 后运行：cargo test -p l03_ownership_borrowing");
}
