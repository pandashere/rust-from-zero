//! 本课的 binary：只负责调用 library 并打印结果。
//!
//! 这里没有 `mod endpoint;`。binary 是独立的 crate，它像外部用户一样通过
//! package 名 `l07_modules_crates` 使用 library 的公共 API。若在这里再声明
//! 一次 `mod endpoint;`，会得到 binary 自己的另一套模块和另一份实现，
//! 而不是“引用 library 的模块”。
//!
//! 在完成 `todo!()` 之前，`cargo run -p l07_modules_crates` 会停在第一条输入
//! 的 `todo!()` 上 panic，这是脚手架在提醒你还有未实现的行为。

use l07_modules_crates::{parse_endpoint, parse_module_path};

fn main() {
    for input in ["  example.com:443 ", "localhost", "example.com:70000"] {
        match parse_endpoint(input) {
            Ok(endpoint) => {
                println!(
                    "endpoint {input:?} -> {}:{}",
                    endpoint.host(),
                    endpoint.port()
                );
            }
            Err(error) => println!("endpoint {input:?} -> 错误 {error:?}"),
        }
    }

    for input in ["crate::endpoint::parser", "crate::Parser", "crate::"] {
        match parse_module_path(input) {
            Ok(path) => println!("path {input:?} -> {path:?}"),
            Err(error) => println!("path {input:?} -> 错误 {error:?}"),
        }
    }
}
