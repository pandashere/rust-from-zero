//! 课后巩固：用 `Vec` 保存不定数量子节点的目录树。
//!
//! 每个条目要么是文件，要么是目录。目录可以有任意多个子条目，所以递归
//! 字段不是 `Box<Entry>`，而是 `Vec<Entry>`：`Vec` 的控制结构大小固定，
//! 子条目存放在堆上的独立缓冲区里。
//!
//! 请只替换方法体里的 `todo!()`，不要修改类型定义、方法签名或文件底部的
//! 测试。

/// 一个目录条目的具体内容。
#[derive(Debug)]
pub enum EntryKind {
    /// 文件，`size` 是它占用的字节数。
    File { size: u64 },
    /// 目录，`children` 按加入顺序保存直接子条目。
    Directory { children: Vec<Entry> },
}

/// 目录树中的一个条目：一个名字加上“文件或目录”的内容。
#[derive(Debug)]
pub struct Entry {
    name: String,
    kind: EntryKind,
}

impl Entry {
    /// 创建一个文件条目。名字可以是 `String`，也可以是字符串字面量。
    pub fn file(name: impl Into<String>, size: u64) -> Self {
        Self {
            name: name.into(),
            kind: EntryKind::File { size },
        }
    }

    /// 创建一个目录条目，`children` 的顺序就是查找时使用的顺序。
    pub fn directory(name: impl Into<String>, children: Vec<Entry>) -> Self {
        Self {
            name: name.into(),
            kind: EntryKind::Directory { children },
        }
    }

    /// 返回条目的名字。
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 返回条目的内容类别。
    pub fn kind(&self) -> &EntryKind {
        &self.kind
    }

    /// 返回整棵子树中所有文件的大小总和。
    ///
    /// 行为约定：
    ///
    /// - 文件条目返回自己的 `size`；
    /// - 目录条目返回所有子条目结果的总和，顺序不影响结果；
    /// - 空目录返回 0；
    /// - 大小为 0 的文件仍然参与求和，只是不增加总和。
    ///
    /// 本方法只借用树。
    pub fn total_size(&self) -> u64 {
        todo!()
    }

    /// 返回整棵子树中的文件个数。
    ///
    /// 行为约定：
    ///
    /// - 文件条目返回 1；
    /// - 目录条目返回所有子条目结果的总和；
    /// - 空目录返回 0；
    /// - 目录本身不计入文件个数，无论它是否为空。
    ///
    /// 本方法只借用树。
    pub fn file_count(&self) -> usize {
        todo!()
    }

    /// 返回整棵子树的层级数。
    ///
    /// 行为约定：
    ///
    /// - 文件条目和空目录的深度都是 1；
    /// - 非空目录的深度是 `1 + 所有子条目中最大的深度`；
    /// - 深度按“从当前条目到最深后代经过的条目数”计算。
    ///
    /// 本方法只借用树。
    pub fn max_depth(&self) -> usize {
        todo!()
    }

    /// 查找名字等于 `target` 的条目，返回从当前条目到它的完整路径。
    ///
    /// 行为约定：
    ///
    /// - 返回值的第一项是当前条目的名字，最后一项是找到的条目名字；
    /// - 当前条目本身就匹配时，返回只含一个名字的路径；
    /// - 否则按 `children` 中保存的顺序做深度优先查找：先把每个直接子
    ///   条目自己与 `target` 比较，再进入它的子树；
    /// - 多个条目同名时返回最先遇到的那一个；
    /// - 文件、空目录、非空目录都可以被找到；
    /// - 找不到时返回 `None`，不返回空 `Vec`。
    ///
    /// 本方法只借用树，路径里的名字是复制出来的 `String`。
    pub fn find_path(&self, target: &str) -> Option<Vec<String>> {
        let _ = &target;
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::Entry;

    fn sample_tree() -> Entry {
        Entry::directory(
            "root",
            vec![
                Entry::directory(
                    "src",
                    vec![Entry::file("main.rs", 120), Entry::file("util.rs", 30)],
                ),
                Entry::directory("notes", vec![]),
                Entry::file("README.md", 200),
            ],
        )
    }

    fn expected_path(parts: &[&str]) -> Option<Vec<String>> {
        Some(parts.iter().map(|part| String::from(*part)).collect())
    }

    // ---- 统计：正常输入 ----

    #[test]
    fn a_file_reports_its_own_size_count_and_depth() {
        let file = Entry::file("main.rs", 120);
        assert_eq!(file.total_size(), 120);
        assert_eq!(file.file_count(), 1);
        assert_eq!(file.max_depth(), 1);
    }

    #[test]
    fn nested_entries_add_up_size_and_file_count() {
        let tree = sample_tree();
        assert_eq!(tree.total_size(), 350);
        assert_eq!(tree.file_count(), 3);
    }

    // ---- 统计：边界 ----

    #[test]
    fn an_empty_directory_has_no_files_but_depth_one() {
        let empty = Entry::directory("empty", vec![]);
        assert_eq!(empty.total_size(), 0);
        assert_eq!(empty.file_count(), 0);
        assert_eq!(empty.max_depth(), 1);
    }

    #[test]
    fn depth_comes_from_the_deepest_branch() {
        let tree = Entry::directory(
            "root",
            vec![
                Entry::file("shallow.txt", 1),
                Entry::directory(
                    "a",
                    vec![Entry::directory("b", vec![Entry::file("deep.txt", 2)])],
                ),
            ],
        );
        assert_eq!(tree.max_depth(), 4);
    }

    #[test]
    fn zero_sized_files_still_count() {
        let tree = Entry::directory(
            "root",
            vec![
                Entry::file("empty", 0),
                Entry::directory("logs", vec![Entry::file("empty.log", 0)]),
            ],
        );
        assert_eq!(tree.total_size(), 0);
        assert_eq!(tree.file_count(), 2);
        assert_eq!(tree.max_depth(), 3);
    }

    #[test]
    fn total_size_can_reach_u64_max_without_overflow() {
        let tree = Entry::directory(
            "root",
            vec![Entry::file("almost", u64::MAX - 1), Entry::file("rest", 1)],
        );
        assert_eq!(tree.total_size(), u64::MAX);
    }

    // ---- 查找：正常输入 ----

    #[test]
    fn finding_nested_file_returns_path_from_root() {
        let tree = sample_tree();
        assert_eq!(
            tree.find_path("util.rs"),
            expected_path(&["root", "src", "util.rs"])
        );
    }

    #[test]
    fn finding_the_root_returns_only_the_root() {
        let tree = sample_tree();
        assert_eq!(tree.find_path("root"), expected_path(&["root"]));
    }

    #[test]
    fn finding_an_empty_directory_returns_its_path() {
        let tree = sample_tree();
        assert_eq!(tree.find_path("notes"), expected_path(&["root", "notes"]));
    }

    #[test]
    fn unicode_names_are_preserved_in_paths() {
        let tree = Entry::directory("资料", vec![Entry::file("数据.txt", 5)]);
        assert_eq!(
            tree.find_path("数据.txt"),
            expected_path(&["资料", "数据.txt"])
        );
    }

    // ---- 查找：同名与缺失 ----

    #[test]
    fn finding_prefers_the_first_match_in_stored_order() {
        let tree = Entry::directory(
            "root",
            vec![
                Entry::directory("left", vec![Entry::file("same.rs", 1)]),
                Entry::directory("right", vec![Entry::file("same.rs", 2)]),
            ],
        );
        assert_eq!(
            tree.find_path("same.rs"),
            expected_path(&["root", "left", "same.rs"])
        );
    }

    #[test]
    fn finding_a_missing_name_returns_none() {
        let tree = sample_tree();
        assert_eq!(tree.find_path("missing.rs"), None);
    }
}
