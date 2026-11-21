//! 第六课的演示入口：用几篇文档展示主练习 `TagIndex` 的行为。
//!
//! 在完成 `exercise.rs` 中的 `todo!()` 之前，运行本程序会在第一次调用
//! 索引方法时中止（panic）——这是预期现象，先去做练习，再回来看完整
//! 输出。
//!
//! `HashMap` 和 `HashSet` 的遍历顺序不稳定，所以这里在打印标签集合前
//! 先收集并排序；主练习的 `documents_with` 本身已经保证排序。

use l06_hash_collections::exercise::TagIndex;

fn main() {
    let mut index = TagIndex::new();

    index
        .add_document(String::from("rust-guide"), &["Rust", " ownership "])
        .expect("文档名非空，应当成功");
    index
        .add_document(String::from("cli-notes"), &["RUST", "cli"])
        .expect("文档名非空，应当成功");
    index
        .add_document(String::from("web-notes"), &["web", "CLI"])
        .expect("文档名非空，应当成功");

    println!("rust 被几篇文档使用：{}", index.count("rust"));

    // 标签集合直接打印时顺序不稳定，先排序再展示。
    if let Some(tag_set) = index.tags_for("cli-notes") {
        let mut tags: Vec<&str> = Vec::new();
        for tag in tag_set {
            tags.push(tag.as_str());
        }
        tags.sort_unstable();
        println!("cli-notes 的标签：{tags:?}");
    }

    println!("带 rust 标签的文档：{:?}", index.documents_with("rust"));
    println!("最常用的标签：{:?}", index.most_common());

    // 用新的标签集合替换整篇文档，旧标签的计数会相应撤销。
    index
        .add_document(String::from("cli-notes"), &["cli"])
        .expect("文档名非空，应当成功");
    println!("替换后 rust 的计数：{}", index.count("rust"));

    let removed = index.remove_document("web-notes");
    println!(
        "删除 web-notes：{removed}，cli 的计数：{}",
        index.count("cli")
    );
}
