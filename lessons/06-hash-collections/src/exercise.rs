//! 主练习：标签索引。
//!
//! 一个笔记程序里，每篇文档（document）有一组标签（tag），例如
//! `"rust-guide"` 这篇文档带着 `["rust", "ownership"]` 两个标签。程序
//! 需要随时回答两类问题：“某个标签一共被几篇文档使用？”和“某篇文档
//! 现在带着哪些标签？”。`TagIndex` 就是同时维护这两份数据的索引。
//!
//! 请只替换各个方法体里的 `todo!()`，不要修改类型定义和方法签名，也
//! 不要改动本文件底部的测试。每个方法上方的注释就是它的完整行为约定。

use std::collections::{HashMap, HashSet};

/// 操作标签索引时可能出现的失败原因。
#[derive(Debug, PartialEq, Eq)]
pub enum IndexError {
    /// 文档名去掉两端空白后没有任何字符。
    EmptyDocumentName,
}

/// 标签索引：同时维护“每个标签被几篇文档使用”和“每篇文档带着哪些
/// 标签”两份数据，并保证它们在任何公开方法返回后都互相一致。
///
/// - `counts`：key 是规范化后的标签，value 是使用它的文档篇数；计数
///   降到 0 的标签必须整个移除，不允许留下计数为 0 的条目。
/// - `documents`：key 是文档名，value 是这篇文档规范化后的标签集合
///   （同一篇文档内不重复）。
// 字段在方法实现（todo!()）完成后才会被读取，先对这个类型单独放行。
#[allow(dead_code)]
#[derive(Debug)]
pub struct TagIndex {
    counts: HashMap<String, u32>,
    documents: HashMap<String, HashSet<String>>,
}

impl TagIndex {
    /// 创建一个没有任何文档、没有任何标签计数的空索引。
    pub fn new() -> Self {
        todo!()
    }

    /// 把一篇文档连同它的标签加入索引。
    ///
    /// 行为约定：
    ///
    /// - `name` 去掉两端空白后为空：返回 `Err(IndexError::EmptyDocumentName)`，
    ///   索引保持调用前的状态，一个条目都不能变。
    /// - 文档名按去掉两端空白后的结果保存；`tags_for` 和 `remove_document`
    ///   查找文档名时也先去掉两端空白，所以 `" guide "` 与 `"guide"` 指
    ///   同一篇文档。
    /// - 每个标签先做规范化：去掉两端空白，再把 ASCII 大写字母（A-Z）
    ///   转成对应小写（其他字符原样保留）；规范化后为空的标签忽略。
    /// - 同一篇文档里，规范化后相同的标签只算一次：无论出现几遍，它对
    ///   `counts` 的贡献都是 1。
    /// - 如果索引里已有同名文档，这次是**完整替换**而不是合并：先撤销旧
    ///   标签对计数的全部贡献，再按新标签计数；旧文档的标签集合也被新
    ///   集合替换。
    /// - 成功时返回 `Ok(())`；此后 `count` 对每个新标签多算一篇，`counts`
    ///   中不留计数为 0 的条目。
    pub fn add_document(&mut self, name: String, tags: &[&str]) -> Result<(), IndexError> {
        // 完成实现前避免“参数未使用”的提示，写好方法体后可删除。
        let _ = &name;
        let _ = &tags;
        todo!()
    }

    /// 把一篇文档从索引中移除，并撤销它的每个标签对计数的贡献。
    ///
    /// 查找前先把 `name` 去掉两端空白，保存时也使用去掉空白后的名字，
    /// 因此 `" guide "` 与 `"guide"` 指同一篇文档。返回 `bool` 报告这次
    /// 调用是否真的移除了一篇文档：文档存在时移除并返回 `true`，不存在
    /// 时不做任何修改、返回 `false`。被撤销计数后计数降到 0 的标签必须
    /// 从 `counts` 中移除。
    pub fn remove_document(&mut self, name: &str) -> bool {
        // 完成实现前避免“参数未使用”的提示，写好方法体后可删除。
        let _ = &name;
        todo!()
    }

    /// 返回某个标签当前被几篇文档使用。
    ///
    /// `tag` 按原样查找，调用者负责传入已规范化的标签；索引里没有这个
    /// 标签时返回 `0`。本方法只做只读查找，不能修改索引。
    pub fn count(&self, tag: &str) -> u32 {
        // 完成实现前避免“参数未使用”的提示，写好方法体后可删除。
        let _ = &tag;
        todo!()
    }

    /// 返回某篇文档当前携带的规范化标签集合的只读视图。
    ///
    /// 查找前先把 `name` 去掉两端空白。文档存在时返回 `Some(集合的借用)`，
    /// 调用者只能读取，不能通过返回值增删标签；文档不存在时返回 `None`。
    pub fn tags_for(&self, name: &str) -> Option<&HashSet<String>> {
        // 完成实现前避免“参数未使用”的提示，写好方法体后可删除。
        let _ = &name;
        todo!()
    }

    /// 返回所有带着某个标签的文档名，按字典序（字符串的默认大小比较）
    /// 从小到大排列。
    ///
    /// 返回的每个名字都是借用索引内部数据的 `&str`，不复制文本；索引中
    /// 没有文档带这个标签时返回空 `Vec`。`tag` 按原样查找，调用者负责
    /// 传入已规范化的标签。
    pub fn documents_with(&self, tag: &str) -> Vec<&str> {
        // 完成实现前避免“参数未使用”的提示，写好方法体后可删除。
        let _ = &tag;
        todo!()
    }

    /// 返回当前使用文档数最多的标签和它的计数。
    ///
    /// - 索引中没有任何标签（包括所有计数都被撤销干净）时返回 `None`；
    /// - 多个标签计数并列最高时，返回字典序最小的那个；
    /// - 返回的标签名是借用索引内部数据的 `&str`。
    pub fn most_common(&self) -> Option<(&str, u32)> {
        todo!()
    }
}

/// 让 `TagIndex::default()` 与 `TagIndex::new()` 等价：都创建一个空索引。
/// 这是 Rust 的惯例——提供了 `new` 的类型通常也提供 `Default`。
impl Default for TagIndex {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{HashSet, IndexError, TagIndex};

    fn tags(values: &[&str]) -> HashSet<String> {
        values.iter().map(|value| (*value).to_owned()).collect()
    }

    #[test]
    fn new_index_is_empty() {
        let index = TagIndex::new();

        assert_eq!(index.count("rust"), 0);
        assert_eq!(index.tags_for("missing"), None);
        assert_eq!(index.documents_with("rust"), Vec::<&str>::new());
        assert_eq!(index.most_common(), None);
    }

    #[test]
    fn add_document_normalizes_deduplicates_and_ignores_empty_tags() {
        let mut index = TagIndex::new();

        assert_eq!(
            index.add_document(
                String::from("guide"),
                &[" Rust ", "rust", "RUST", "", "   ", "CLI"],
            ),
            Ok(())
        );

        assert_eq!(index.count("rust"), 1);
        assert_eq!(index.count("cli"), 1);
        assert_eq!(index.tags_for("guide"), Some(&tags(&["rust", "cli"])));
    }

    #[test]
    fn normalization_only_lowercases_ascii_letters() {
        let mut index = TagIndex::new();

        index
            .add_document(String::from("unicode"), &[" RÜST "])
            .unwrap();

        assert_eq!(index.tags_for("unicode"), Some(&tags(&["rÜst"])));
    }

    #[test]
    fn counts_each_tag_once_per_document() {
        let mut index = TagIndex::new();

        index
            .add_document(String::from("first"), &["rust", "cli"])
            .unwrap();
        index
            .add_document(String::from("second"), &["rust", "web"])
            .unwrap();

        assert_eq!(index.count("rust"), 2);
        assert_eq!(index.count("cli"), 1);
        assert_eq!(index.count("web"), 1);
    }

    #[test]
    fn replacing_a_document_withdraws_old_counts_before_adding_new_ones() {
        let mut index = TagIndex::new();
        index
            .add_document(String::from("guide"), &["rust", "cli"])
            .unwrap();

        index
            .add_document(String::from("guide"), &["cli", "web"])
            .unwrap();

        assert_eq!(index.count("rust"), 0);
        assert_eq!(index.count("cli"), 1);
        assert_eq!(index.count("web"), 1);
        assert_eq!(index.tags_for("guide"), Some(&tags(&["cli", "web"])));
    }

    #[test]
    fn replacing_one_document_keeps_other_documents_counts() {
        let mut index = TagIndex::new();
        index
            .add_document(String::from("first"), &["rust", "cli"])
            .unwrap();
        index
            .add_document(String::from("second"), &["rust"])
            .unwrap();

        // 只替换 first；second 仍然带着 rust，所以 rust 的计数只能减一。
        index.add_document(String::from("first"), &["web"]).unwrap();

        assert_eq!(index.count("rust"), 1);
        assert_eq!(index.count("cli"), 0);
        assert_eq!(index.count("web"), 1);
        assert_eq!(index.documents_with("rust"), vec!["second"]);
        assert_eq!(index.tags_for("first"), Some(&tags(&["web"])));
    }

    #[test]
    fn zero_count_entries_do_not_stay_in_the_index() {
        let mut index = TagIndex::new();
        index
            .add_document(String::from("guide"), &["rust"])
            .unwrap();

        // 替换后索引里不再有任何非零计数；如果 "rust" 带着计数 0 留下，
        // 它会成为 most_common 的候选，这个断言就会失败。
        index
            .add_document(String::from("guide"), &["", "   "])
            .unwrap();

        assert_eq!(index.count("rust"), 0);
        assert_eq!(index.tags_for("guide"), Some(&HashSet::new()));
        assert_eq!(index.most_common(), None);
    }

    #[test]
    fn remove_document_updates_counts_and_reports_whether_it_existed() {
        let mut index = TagIndex::new();
        index
            .add_document(String::from("first"), &["rust", "cli"])
            .unwrap();
        index
            .add_document(String::from("second"), &["rust"])
            .unwrap();

        assert!(index.remove_document("first"));
        assert_eq!(index.count("rust"), 1);
        assert_eq!(index.count("cli"), 0);
        assert_eq!(index.tags_for("first"), None);
        assert_eq!(index.most_common(), Some(("rust", 1)));

        assert!(!index.remove_document("first"));
    }

    #[test]
    fn documents_with_returns_sorted_borrowed_names() {
        let mut index = TagIndex::new();
        for name in ["zeta", "alpha", "middle"] {
            index.add_document(name.to_owned(), &["rust"]).unwrap();
        }
        index
            .add_document(String::from("python-only"), &["python"])
            .unwrap();

        assert_eq!(
            index.documents_with("rust"),
            vec!["alpha", "middle", "zeta"]
        );
    }

    #[test]
    fn most_common_uses_lexical_order_to_break_ties() {
        let mut index = TagIndex::new();
        index
            .add_document(String::from("one"), &["rust", "cli"])
            .unwrap();
        index
            .add_document(String::from("two"), &["web", "rust"])
            .unwrap();
        assert_eq!(index.most_common(), Some(("rust", 2)));

        index.remove_document("two");

        assert_eq!(index.most_common(), Some(("cli", 1)));
    }

    #[test]
    fn document_names_are_stored_without_surrounding_whitespace() {
        let mut index = TagIndex::new();

        index
            .add_document(String::from("  guide  "), &["rust"])
            .unwrap();

        // 名字按去掉两端空白后的结果保存，所以两种写法都按 "guide" 处理。
        assert_eq!(index.tags_for("guide"), Some(&tags(&["rust"])));
        assert_eq!(index.documents_with("rust"), vec!["guide"]);

        // 带空白的新写法命中同一篇文档：是替换，不是新增一篇。
        index
            .add_document(String::from(" guide"), &["cli"])
            .unwrap();
        assert_eq!(index.tags_for("guide"), Some(&tags(&["cli"])));
        assert_eq!(index.count("rust"), 0);
    }

    #[test]
    fn document_name_lookups_ignore_surrounding_whitespace() {
        let mut index = TagIndex::new();
        index
            .add_document(String::from("guide"), &["rust"])
            .unwrap();

        assert_eq!(index.tags_for("  guide  "), Some(&tags(&["rust"])));
        assert!(index.remove_document(" guide "));
        assert_eq!(index.tags_for("guide"), None);
        assert_eq!(index.count("rust"), 0);

        // 去掉空白后为空的名字不是任何一篇文档，删除应正常报告 false。
        assert!(!index.remove_document("   "));
    }

    #[test]
    fn empty_document_name_fails_without_changing_the_index() {
        let mut index = TagIndex::new();
        index
            .add_document(String::from("existing"), &["rust"])
            .unwrap();

        assert_eq!(
            index.add_document(String::from("   "), &["new", "tags"]),
            Err(IndexError::EmptyDocumentName)
        );

        // 失败之后索引必须和调用前完全一样：旧文档、旧计数原样保留，
        // 失败输入里的标签一个也不能进来。
        assert_eq!(index.count("rust"), 1);
        assert_eq!(index.count("new"), 0);
        assert_eq!(index.count("tags"), 0);
        assert_eq!(index.tags_for("existing"), Some(&tags(&["rust"])));
        assert_eq!(index.most_common(), Some(("rust", 1)));
    }
}
