//! 课后巩固：借用的单词索引。
//!
//! 主练习处理“键 = 值”的配置行；这里的输入是一段普通文本，边界变成标点、
//! Unicode 字符和“一个单词也没有”的输入。`longest_word` 按字符个数比较，
//! 而不是按 UTF-8 字节数比较，和主练习的规则刻意不同。
//!
//! 本模块对“单词”的定义：把文本按空白字符（空格、换行、制表符等）切开，
//! 再删除每个片段两端属于下面集合的字符：
//!
//! ```text
//! . , ; : ! ? " ' ( ) [ ] ， 。 ； ： ！ ？ “ ” ‘ ’
//! ```
//!
//! 删除后为空的片段不算单词；单词内部出现的标点保留（例如 `don't` 里的
//! 撇号在中间，不会被删除）。
//!
//! 请只替换各个方法体里的 `todo!()`，不要修改类型定义、方法签名或文件
//! 底部的测试。每个方法上方的注释就是它的完整行为约定。
//!
//! 完成后验收：
//!
//! ```bash
//! cargo test -p l14_lifetimes consolidation
//! ```

/// 一段文本的只读索引；索引本身不拥有文本。
// 脚手架阶段还没有方法读取这个字段，先只对这个类型放行 dead_code。
#[allow(dead_code)]
#[derive(Debug)]
pub struct WordIndex<'a> {
    text: &'a str,
}

impl<'a> WordIndex<'a> {
    /// 为 `text` 建立一个索引。
    ///
    /// 索引只保存对 `text` 的借用，不复制文本，因此不能比 `text` 活得更久。
    pub fn new(text: &'a str) -> Self {
        let _ = &text;
        todo!()
    }

    /// 返回建立索引时的完整文本。
    ///
    /// 返回值借自原始文本，而不是借自索引本身：索引销毁后，只要原始文本
    /// 还在，这个切片仍然可以使用。
    pub fn source(&self) -> &'a str {
        let _ = &self;
        todo!()
    }

    /// 返回文本中单词的个数。
    ///
    /// 空文本、只有空白或只有标点的文本都返回 `0`。
    pub fn word_count(&self) -> usize {
        let _ = &self;
        todo!()
    }

    /// 返回按文本顺序出现的第一个单词。
    ///
    /// 没有任何单词时返回 `None`。
    pub fn first_word(&self) -> Option<&'a str> {
        let _ = &self;
        todo!()
    }

    /// 返回按文本顺序出现的最后一个单词。
    ///
    /// 没有任何单词时返回 `None`。
    pub fn last_word(&self) -> Option<&'a str> {
        let _ = &self;
        todo!()
    }

    /// 返回字符个数最多的单词。
    ///
    /// 行为约定：
    ///
    /// - 按 Unicode 字符（`char`）的个数比较，不按 UTF-8 字节数比较；
    /// - 多个单词的字符数并列最大时，返回最先出现的那个；
    /// - 没有任何单词时返回 `None`。
    pub fn longest_word(&self) -> Option<&'a str> {
        let _ = &self;
        todo!()
    }

    /// 统计与 `word` 完全相等的单词个数。
    ///
    /// 行为约定：
    ///
    /// - 比较区分大小写；
    /// - `word` 本身按原样比较，不按单词定义清洗标点；例如查询 `"rust,"`
    ///   不会匹配文本中的 `rust`；
    /// - 没有匹配项时返回 `0`。
    pub fn count(&self, word: &str) -> usize {
        let _ = &self;
        let _ = &word;
        todo!()
    }

    /// 返回第一个以 `prefix` 开头的单词。
    ///
    /// 行为约定：
    ///
    /// - 前缀比较区分大小写；
    /// - `prefix` 为空字符串时，匹配第一个单词；
    /// - 没有单词或没有匹配项时返回 `None`；
    /// - `prefix` 只在这次调用中被读取；返回的单词来自原始文本，不来自
    ///   `prefix`，因此前缀字符串不需要和索引活得一样久。
    pub fn first_with_prefix<'b>(&self, prefix: &'b str) -> Option<&'a str> {
        let _ = &self;
        let _ = &prefix;
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::WordIndex;

    /// 确认 `slice` 指向 `source` 内部的字节范围，而不只是内容相同。
    fn assert_slice_of(source: &str, slice: &str) {
        let source_start = source.as_ptr() as usize;
        let source_end = source_start + source.len();
        let slice_start = slice.as_ptr() as usize;
        let slice_end = slice_start + slice.len();
        assert!(
            slice_start >= source_start && slice_end <= source_end,
            "slice {slice:?} is not inside source"
        );
    }

    // ---- 空输入与边界 ----

    #[test]
    fn empty_text_has_no_words() {
        let index = WordIndex::new("");

        assert_eq!(index.source(), "");
        assert_eq!(index.word_count(), 0);
        assert_eq!(index.first_word(), None);
        assert_eq!(index.last_word(), None);
        assert_eq!(index.longest_word(), None);
        assert_eq!(index.count("rust"), 0);
        assert_eq!(index.first_with_prefix("r"), None);
    }

    #[test]
    fn whitespace_and_punctuation_only_text_has_no_words() {
        let index = WordIndex::new(" \t\n , . ! ? ， 。 “ ” ");

        assert_eq!(index.word_count(), 0);
        assert_eq!(index.first_word(), None);
        assert_eq!(index.last_word(), None);
        assert_eq!(index.longest_word(), None);
    }

    // ---- 切分与去标点 ----

    #[test]
    fn words_are_split_on_whitespace_and_trimmed_at_both_ends() {
        let index = WordIndex::new("  (Rust), rust!  [borrows];  ");

        assert_eq!(index.word_count(), 3);
        assert_eq!(index.first_word(), Some("Rust"));
        assert_eq!(index.last_word(), Some("borrows"));
        assert_eq!(index.count("Rust"), 1);
        assert_eq!(index.count("rust"), 1);
    }

    #[test]
    fn unicode_words_and_chinese_punctuation_are_handled() {
        let index = WordIndex::new("语言， 生锈。 “引用”");

        assert_eq!(index.word_count(), 3);
        assert_eq!(index.first_word(), Some("语言"));
        assert_eq!(index.last_word(), Some("引用"));
        assert_eq!(index.count("生锈"), 1);
    }

    #[test]
    fn count_matches_whole_words_case_sensitively() {
        let index = WordIndex::new("Rust rust Rust rusty");

        assert_eq!(index.count("Rust"), 2);
        assert_eq!(index.count("rust"), 1);
        assert_eq!(index.count("rus"), 0);
        assert_eq!(index.count("rusty"), 1);
        assert_eq!(index.count("rust,"), 0);
    }

    // ---- 最长单词：字符个数与并列规则 ----

    #[test]
    fn longest_word_uses_character_count_and_first_on_tie() {
        assert_eq!(WordIndex::new("é zz aa").longest_word(), Some("zz"));
        assert_eq!(WordIndex::new("生锈 abc").longest_word(), Some("abc"));
        assert_eq!(WordIndex::new("aa bb").longest_word(), Some("aa"));
        assert_eq!(WordIndex::new("one").longest_word(), Some("one"));
    }

    // ---- 前缀查询与生命周期 ----

    #[test]
    fn first_with_prefix_returns_first_match() {
        let index = WordIndex::new("apple apricot banana");

        assert_eq!(index.first_with_prefix("ap"), Some("apple"));
        assert_eq!(index.first_with_prefix("app"), Some("apple"));
        assert_eq!(index.first_with_prefix("ban"), Some("banana"));
        assert_eq!(index.first_with_prefix("x"), None);
        assert_eq!(index.first_with_prefix("Ap"), None);
    }

    #[test]
    fn empty_prefix_matches_first_word_and_missing_prefix_returns_none() {
        assert_eq!(
            WordIndex::new("alpha beta").first_with_prefix(""),
            Some("alpha")
        );
        assert_eq!(WordIndex::new("   ").first_with_prefix(""), None);
    }

    #[test]
    fn source_can_outlive_the_index() {
        let text = String::from("alpha beta");

        let saved = {
            let index = WordIndex::new(&text);
            index.source()
        };

        assert_eq!(saved, "alpha beta");
    }

    #[test]
    fn returned_words_borrow_the_original_text() {
        let text = String::from("alpha, beta!");
        let index = WordIndex::new(&text);

        assert_slice_of(&text, index.first_word().unwrap());
        assert_slice_of(&text, index.last_word().unwrap());
        assert_slice_of(&text, index.longest_word().unwrap());
    }
}
