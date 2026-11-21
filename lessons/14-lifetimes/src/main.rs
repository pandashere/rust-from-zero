//! 第十四课的演示入口：从一段配置文本建立零复制的键值目录。
//!
//! 在完成 `src/exercise.rs` 中的 `todo!()` 之前，运行本程序会在
//! `Catalog::parse` 处中止（panic）——这是预期现象。先完成主练习，再回来
//! 看完整输出。

use l14_lifetimes::exercise::{Catalog, parse_record};

fn main() {
    // 整个配置只分配这一个 String；目录中的键和值都借用它。
    let source = "# application settings
language = rust

invalid line
 = no key
editor = helix
language = oxidized
empty =
token = a=b
";

    let catalog = Catalog::parse(source);

    println!("records: {:?}", catalog.records());
    println!("language: {:?}", catalog.get("language"));
    println!("missing: {:?}", catalog.get("missing"));
    println!("longest value: {:?}", catalog.longest_value());

    if let Some(record) = parse_record("token = a=b") {
        println!("{} -> {}", record.key, record.value);
    }
}
