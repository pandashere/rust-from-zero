//! `module_path` 领域的解析器：公共入口 `parse_module_path` 和私有判定
//! 函数 `is_valid_segment`。

use super::{ModulePath, ParseModulePathError};

/// 解析 `"crate::endpoint::parser"` 形式的模块路径文本。
///
/// 行为约定：
///
/// - 先去掉整段输入两端的空白；去掉后为空则返回 `EmptyPath`；
/// - 用 `"::"` 切分，切出的每一段按从 0 开始的序号检查；
/// - 空段返回 `EmptySegment { index }`；
/// - 不满足 `is_valid_segment` 的段返回
///   `InvalidSegment { index, text }`，`text` 是该段去掉整段输入两端空白后
///   的原样内容（段内部空格不会被删除）；
/// - 全部分段合法时，按原顺序构造 [`ModulePath`]。
///
/// 注意：只有整段输入会被 `trim`，因此 `"crate:: endpoint"` 的第 1 段是
/// `" endpoint"`，开头的空格让它成为非法段；`"  crate::endpoint  "` 则
/// 可以成功。
///
/// # Examples
///
/// ```
/// use l07_modules_crates::parse_module_path;
///
/// let path = parse_module_path("crate::endpoint::parser")?;
/// assert_eq!(path.segments().len(), 3);
/// assert!(path.starts_at_crate_root());
/// # Ok::<(), l07_modules_crates::ParseModulePathError>(())
/// ```
pub fn parse_module_path(input: &str) -> Result<ModulePath, ParseModulePathError> {
    let input = input.trim();
    if input.is_empty() {
        return Err(ParseModulePathError::EmptyPath);
    }

    // 循环体实现后需要向 `segments` 追加内容；实现完成后可删除下面的属性。
    #[allow(unused_mut)]
    let mut segments = Vec::new();
    for (index, segment) in input.split("::").enumerate() {
        // 完成实现前避免“变量未使用”的警告，写好循环体后可删除。
        let _ = (index, segment);
        todo!()
    }

    Ok(ModulePath::new(segments))
}

/// 判断一段文本是否符合本课的模块名约定。
///
/// 这是为练习简化的规则，不是完整的 Rust 标识符规则：
///
/// - 不能为空；
/// - 第一个字符必须是小写 ASCII 字母或下划线 `_`；
/// - 其余字符只能是小写 ASCII 字母、数字或下划线；
/// - 至少包含一个小写 ASCII 字母（所以 `"_"`、`"__"`、`"_1"` 都不合法）。
///
/// 因此 `"parser"`、`"hash_map2"`、`"_private"` 合法；`"Parser"`、
/// `"9lives"`、`"mod-path"`、`"模块"` 不合法。
// 该函数目前只被学习者将要填写的循环体调用；循环体调用它之后，这个属性就不再需要，保留也不影响编译。
#[allow(dead_code)]
fn is_valid_segment(segment: &str) -> bool {
    let _ = segment;
    todo!()
}

#[cfg(test)]
mod tests {
    use super::{ParseModulePathError, is_valid_segment, parse_module_path};

    #[test]
    fn parses_a_rooted_three_segment_path() {
        let path = parse_module_path("crate::endpoint::parser").unwrap();

        assert_eq!(path.segments().len(), 3);
        assert_eq!(path.segments()[0], "crate");
        assert_eq!(path.segments()[2], "parser");
        assert!(path.starts_at_crate_root());
    }

    #[test]
    fn parses_a_single_relative_segment_and_trims_the_whole_input() {
        let path = parse_module_path("  main  ").unwrap();

        assert_eq!(path.segments(), ["main"]);
        assert!(!path.starts_at_crate_root());
    }

    #[test]
    fn rejects_empty_input_with_empty_path() {
        assert_eq!(parse_module_path(""), Err(ParseModulePathError::EmptyPath));
        assert_eq!(
            parse_module_path("   \n\t "),
            Err(ParseModulePathError::EmptyPath)
        );
    }

    #[test]
    fn reports_empty_segments_with_their_index() {
        assert_eq!(
            parse_module_path("::parser"),
            Err(ParseModulePathError::EmptySegment { index: 0 })
        );
        assert_eq!(
            parse_module_path("crate::"),
            Err(ParseModulePathError::EmptySegment { index: 1 })
        );
        assert_eq!(
            parse_module_path("crate::::parser"),
            Err(ParseModulePathError::EmptySegment { index: 1 })
        );
    }

    #[test]
    fn reports_invalid_segments_with_index_and_original_text() {
        assert_eq!(
            parse_module_path("crate::Parser"),
            Err(ParseModulePathError::InvalidSegment {
                index: 1,
                text: String::from("Parser"),
            })
        );
        assert_eq!(
            parse_module_path("crate::9lives"),
            Err(ParseModulePathError::InvalidSegment {
                index: 1,
                text: String::from("9lives"),
            })
        );
        assert_eq!(
            parse_module_path("crate::_1"),
            Err(ParseModulePathError::InvalidSegment {
                index: 1,
                text: String::from("_1"),
            })
        );
        assert_eq!(
            parse_module_path("crate:: endpoint"),
            Err(ParseModulePathError::InvalidSegment {
                index: 1,
                text: String::from(" endpoint"),
            })
        );
    }

    #[test]
    fn text_without_a_double_colon_is_one_invalid_segment() {
        assert_eq!(
            parse_module_path("crate:parser"),
            Err(ParseModulePathError::InvalidSegment {
                index: 0,
                text: String::from("crate:parser"),
            })
        );
    }

    #[test]
    fn private_is_valid_segment_accepts_the_documented_examples() {
        assert!(is_valid_segment("parser"));
        assert!(is_valid_segment("hash_map2"));
        assert!(is_valid_segment("_private"));

        assert!(!is_valid_segment(""));
        assert!(!is_valid_segment("Parser"));
        assert!(!is_valid_segment("9lives"));
        assert!(!is_valid_segment("_1"));
        assert!(!is_valid_segment("mod-path"));
        assert!(!is_valid_segment("模块"));
    }
}
