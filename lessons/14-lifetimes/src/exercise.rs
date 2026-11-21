//! 主练习：零复制配置目录。
//!
//! 程序把一份配置文本整体读入一个 `String`。目录不需要为每一行的键和值
//! 再分配新的 `String`，只需要记住它们在原文本中的位置。因此 `Record`
//! 和 `Catalog` 都带生命周期参数：它们借用原始文本，并受原始文本的有效
//! 范围约束。
//!
//! 请只替换各个函数和方法体里的 `todo!()`，不要修改类型定义、函数签名、
//! 文档注释或文件底部的测试。每个待实现项上方的注释就是它的完整行为
//! 约定。
//!
//! 完成后验收：
//!
//! ```bash
//! cargo test -p l14_lifetimes exercise
//! ```

/// 一条配置记录，键和值都是原始输入中的文本片段。
#[derive(Debug, PartialEq, Eq)]
pub struct Record<'a> {
    /// `=` 左侧去掉两端空白后的键。
    pub key: &'a str,
    /// `=` 右侧去掉两端空白后的值。
    pub value: &'a str,
}

/// 按输入顺序保存配置记录的目录；重复键全部保留。
// 脚手架阶段还没有方法读取这个字段，先只对这个类型放行 dead_code。
#[allow(dead_code)]
#[derive(Debug)]
pub struct Catalog<'a> {
    records: Vec<Record<'a>>,
}

/// 把一行配置解析成一条借用 `line` 的记录。
///
/// 行为约定：
///
/// - 先去掉整行两端的空白；去掉后为空，或以 `#` 开头，返回 `None`；
/// - 用第一个 `=` 把行分成键和值；没有 `=` 时返回 `None`；
/// - 键和值各自去掉两端空白；键为空时返回 `None`；
/// - 值允许为空；第二个及之后的 `=` 都属于值，不再切分；
/// - 成功时返回的 `key` 和 `value` 都是 `line` 内部的连续片段，不复制
///   文本。
pub fn parse_record(line: &str) -> Option<Record<'_>> {
    let _ = &line;
    todo!()
}

impl<'a> Catalog<'a> {
    /// 按输入顺序收集 `source` 中的所有有效记录。
    ///
    /// 行为约定：
    ///
    /// - 每一行按 [`parse_record`] 的规则解析；无效行直接跳过；
    /// - 有效记录保持输入顺序，重复键不去重；
    /// - 记录中的键和值都借用 `source`，不复制成 `String`；
    /// - `source` 为空，或只有无效行时，返回空目录。
    pub fn parse(source: &'a str) -> Self {
        let _ = &source;
        todo!()
    }

    /// 返回第一条键与 `key` 完全相等的记录的值。
    ///
    /// 行为约定：
    ///
    /// - 比较区分大小写，必须逐字符完全相等；
    /// - 重复键返回最先出现的那条记录的值；
    /// - 没有匹配项时返回 `None`；
    /// - `key` 只在这次查找中被读取；返回的切片来自目录保存的原始文本，
    ///   不来自 `key`，因此查询字符串不需要和目录活得一样久。
    pub fn get(&self, key: &str) -> Option<&str> {
        let _ = &self;
        let _ = &key;
        todo!()
    }

    /// 按输入顺序返回全部记录的只读切片，包括重复键。
    ///
    /// 返回的是目录内部记录的视图，不复制记录；没有记录时返回空切片。
    pub fn records(&self) -> &[Record<'a>] {
        let _ = &self;
        todo!()
    }

    /// 返回 UTF-8 字节数最多的值。
    ///
    /// 行为约定：
    ///
    /// - 按值的 UTF-8 字节数比较，而不是按字符个数比较；
    /// - 多个值的字节数并列最大时，返回最先出现的那个值；
    /// - 目录为空时返回 `None`。
    pub fn longest_value(&self) -> Option<&str> {
        let _ = &self;
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::{Catalog, Record, parse_record};

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

    // ---- 概念测试：不依赖待实现函数，初始即应通过 ----

    #[test]
    fn record_literal_borrows_source_without_copying() {
        let source = String::from("language=rust");
        let record = Record {
            key: &source[..8],
            value: &source[9..],
        };

        assert_eq!(record.key, "language");
        assert_eq!(record.value, "rust");
        assert_slice_of(&source, record.key);
        assert_slice_of(&source, record.value);
    }

    #[test]
    fn owned_string_satisfies_static_bound() {
        fn require_static<T: 'static>(_: T) {}

        let owned = String::from("owned locally");
        require_static(owned);
    }

    // ---- 单行解析：正常、边界与错误输入 ----

    #[test]
    fn parse_record_trims_key_and_value() {
        let line = String::from("  language = rust  ");
        let record = parse_record(&line).unwrap();

        assert_eq!(
            record,
            Record {
                key: "language",
                value: "rust"
            }
        );
        assert_slice_of(&line, record.key);
        assert_slice_of(&line, record.value);
    }

    #[test]
    fn parse_record_rejects_blank_comment_and_invalid_lines() {
        for line in [
            "",
            "   ",
            "# comment",
            "  # comment",
            "missing separator",
            " = no key",
        ] {
            assert!(parse_record(line).is_none(), "input: {line:?}");
        }
    }

    #[test]
    fn parse_record_allows_empty_value_and_later_equals_signs() {
        assert_eq!(
            parse_record("empty ="),
            Some(Record {
                key: "empty",
                value: ""
            })
        );
        assert_eq!(
            parse_record("token = a=b"),
            Some(Record {
                key: "token",
                value: "a=b"
            })
        );
    }

    #[test]
    fn parse_record_preserves_unicode_content() {
        assert_eq!(
            parse_record(" 语言 = 生锈 "),
            Some(Record {
                key: "语言",
                value: "生锈"
            })
        );
    }

    // ---- 目录行为：顺序、重复键、空输入 ----

    #[test]
    fn catalog_filters_invalid_lines_and_preserves_order() {
        let source = "\
# settings
language = rust

invalid line
 = no key
editor=helix
language = oxidized
empty =
token = a=b
";
        let catalog = Catalog::parse(source);

        assert_eq!(
            catalog.records(),
            &[
                Record {
                    key: "language",
                    value: "rust"
                },
                Record {
                    key: "editor",
                    value: "helix"
                },
                Record {
                    key: "language",
                    value: "oxidized"
                },
                Record {
                    key: "empty",
                    value: ""
                },
                Record {
                    key: "token",
                    value: "a=b"
                },
            ]
        );
    }

    #[test]
    fn catalog_keeps_duplicate_keys_and_get_returns_the_first() {
        let source = String::from("language=rust\nlanguage=oxidized");
        let catalog = Catalog::parse(&source);

        assert_eq!(catalog.get("language"), Some("rust"));
        assert_eq!(catalog.get("LANGUAGE"), None);
        assert_eq!(catalog.get("missing"), None);
    }

    #[test]
    fn get_query_does_not_need_to_outlive_the_catalog() {
        let source = String::from("language=rust\nlanguage=oxidized");
        let catalog = Catalog::parse(&source);

        let value = {
            let query = String::from("language");
            catalog.get(&query).unwrap()
        };

        assert_eq!(value, "rust");
    }

    #[test]
    fn empty_catalog_has_no_records_or_values() {
        for source in ["", "# comment\n\ninvalid\n = no key\n", "   \n\t\n"] {
            let catalog = Catalog::parse(source);
            assert!(catalog.records().is_empty(), "input: {source:?}");
            assert_eq!(catalog.get("anything"), None, "input: {source:?}");
            assert_eq!(catalog.longest_value(), None, "input: {source:?}");
        }
    }

    // ---- 最长值：字节长度与并列规则 ----

    #[test]
    fn longest_value_compares_utf8_byte_length() {
        let source = "accent=é\nletters=zz";
        let catalog = Catalog::parse(source);
        assert_eq!(catalog.longest_value(), Some("é"));

        let source = "language=生锈\neditor=helix";
        let catalog = Catalog::parse(source);
        assert_eq!(catalog.longest_value(), Some("生锈"));
    }

    #[test]
    fn longest_value_returns_first_on_tie() {
        let catalog = Catalog::parse("first=aa\nsecond=bb\nthird=c");
        assert_eq!(catalog.longest_value(), Some("aa"));
    }

    // ---- 零复制：字段必须落在原输入缓冲区中 ----

    #[test]
    fn record_fields_are_slices_of_original_source() {
        let source = String::from("language = rust\nempty =");
        let catalog = Catalog::parse(&source);

        for record in catalog.records() {
            assert_slice_of(&source, record.key);
            assert_slice_of(&source, record.value);
        }
    }
}
