//! 证明 `async fn` 调用后不会立即执行函数体。
//!
//! 运行：
//!
//! ```text
//! cargo run -p l19_async --example future_laziness
//! ```

use l19_async::runtime::block_on;

async fn double(value: i32) -> i32 {
    println!("函数体开始运行");
    value * 2
}

fn main() {
    let future = double(21);
    println!("future 已创建，函数体还没有运行");

    let answer = block_on(future);
    println!("block_on 之后得到 {answer}");
}
