//! `module_path` 领域的错误类型。

/// 解析模块路径文本时可能出现的失败原因。
///
/// `index` 一律是从 0 开始的分段序号：`"crate::endpoint"` 中 `crate` 是
/// 第 0 段，`endpoint` 是第 1 段。带上序号和原文，调用者才能指出究竟是
/// 哪一段写错了。
#[derive(Debug, PartialEq, Eq)]
pub enum ParseModulePathError {
    /// 去掉两端空白后，整段输入是空文本。
    EmptyPath,
    /// 找到了分段位置，但某一段是空文本，例如 `"a::"` 或 `"a::::b"`。
    EmptySegment {
        /// 空段的位置，从 0 开始。
        index: usize,
    },
    /// 某一段不满足本课的模块名约定：以小写 ASCII 字母或下划线开头，
    /// 只含小写 ASCII 字母、数字和下划线，并且至少有一个小写字母。
    InvalidSegment {
        /// 非法段的位置，从 0 开始。
        index: usize,
        /// 非法段的原始文本（已去掉整段输入两端的空白，但段内部保持原样）。
        text: String,
    },
}
