//! `module_path` 领域的数据模型：类型公开，字段私有，构造器只开放给父模块。

/// 一个已经通过校验的模块路径，由按顺序排列的若干段组成。
///
/// 字段保持私有，因此 crate 外的调用者无法跳过校验直接拼出一个
/// `ModulePath`；只能通过 [`parse_module_path`](crate::parse_module_path)
/// 获得它，再用 `segments()`、`starts_at_crate_root()` 读取内容。
///
/// 不变量：至少有一段，且每一段都满足本课的模块名约定。
#[derive(Debug, PartialEq, Eq)]
pub struct ModulePath {
    segments: Vec<String>,
}

impl ModulePath {
    /// 用已经校验过的分段构造路径。只开放给父模块 `module_path` 的范围：
    /// 兄弟模块 `parser` 校验完每一段后可以调用它，crate 外看不到它。
    pub(super) fn new(segments: Vec<String>) -> Self {
        Self { segments }
    }

    /// 返回全部段的只读借用（`&[String]`），顺序与输入文本一致。
    ///
    /// 本方法只借出已有数据，不复制字符串、不分配新的 `Vec`。调用者可以
    /// 按下标读取，也可以把切片交给需要 `&[String]` 的函数。
    pub fn segments(&self) -> &[String] {
        todo!()
    }

    /// 判断路径是否以 `crate` 作为第一段。
    ///
    /// 只看第一段，不要求整条路径等于 `crate`；`"crate::endpoint"` 返回
    /// `true`，`"endpoint::crate"` 返回 `false`。
    pub fn starts_at_crate_root(&self) -> bool {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::ModulePath;

    fn sample() -> ModulePath {
        ModulePath::new(vec![
            String::from("crate"),
            String::from("endpoint"),
            String::from("parser"),
        ])
    }

    #[test]
    fn segments_returns_every_segment_in_input_order() {
        let path = sample();
        let segments: &[String] = path.segments();

        assert_eq!(segments.len(), 3);
        assert_eq!(segments[0], "crate");
        assert_eq!(segments[1], "endpoint");
        assert_eq!(segments[2], "parser");
    }

    #[test]
    fn starts_at_crate_root_checks_only_the_first_segment() {
        assert!(sample().starts_at_crate_root());

        let relative = ModulePath::new(vec![String::from("endpoint"), String::from("crate")]);
        assert!(!relative.starts_at_crate_root());
    }
}
