//! 第十三课的演示入口：构造一棵表达式树，观察借用遍历和消费遍历。
//!
//! 完成 `src/exercise.rs` 里的 `todo!()` 之前，本程序会在第一次调用
//! `eval` 时中止（panic），这是预期现象；完成实现后再运行即可看到完整输出。

use l13_box_recursion::exercise::Expr;

fn main() {
    // 对应数学式：(1 + 2) * 4 - (-3)
    let expression = Expr::Subtract(
        Box::new(Expr::Multiply(
            Box::new(Expr::Add(
                Box::new(Expr::Literal(1)),
                Box::new(Expr::Literal(2)),
            )),
            Box::new(Expr::Literal(4)),
        )),
        Box::new(Expr::Neg(Box::new(Expr::Literal(3)))),
    );

    println!("表达式树: {expression:?}");
    println!("节点指针大小: {} 字节", std::mem::size_of::<Box<Expr>>());
    println!("整棵树大小: {} 字节", std::mem::size_of::<Expr>());

    match expression.eval() {
        Ok(value) => println!("求值: {value}"),
        Err(error) => println!("求值失败: {error:?}"),
    }
    println!("深度: {}", expression.depth());
    println!("字面量（借用遍历）: {:?}", expression.literals());
    println!("借用后仍可求值: {:?}", expression.eval());
    println!("字面量（消费遍历）: {:?}", expression.into_literals());

    // 一个能恢复的错误：除数为 0。
    let invalid = Expr::Divide(Box::new(Expr::Literal(1)), Box::new(Expr::Literal(0)));
    println!("除零结果: {:?}", invalid.eval());
}
