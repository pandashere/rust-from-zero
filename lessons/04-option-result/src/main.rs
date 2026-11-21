//! 第四课的演示入口：用几组输入展示主练习函数的行为。
//!
//! 在完成 `exercise.rs` 中的 `todo!()` 之前，运行本程序会在第一组输入处
//! 中止（panic）——这是预期现象，先去做练习，再回来看完整输出。

use l04_option_result::exercise::parse_endpoint;

fn main() {
    let samples = [
        "localhost:8080",
        " example.com : 443 ",
        "",
        "localhost",
        ":8080",
        "localhost:abc",
        "localhost:0",
    ];

    for input in samples {
        match parse_endpoint(input) {
            Ok(endpoint) => println!(
                "{input:?} => 主机 {}，端口 {}",
                endpoint.host, endpoint.port
            ),
            Err(error) => println!("{input:?} => 解析失败：{error:?}"),
        }
    }
}
