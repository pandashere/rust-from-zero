//! `module_path` 领域模块：课后巩固练习。
//!
//! 与 `endpoint` 一样，本模块自身私有，三个子模块也私有；外部只能看到
//! crate root 通过 `pub use` 提供的 `ModulePath`、`ParseModulePathError`
//! 和 `parse_module_path`。
//!
//! 这里解析的是模块路径文本，例如 `"crate::endpoint::parser"`。规则是本课
//! 为练习简化的“小写模块名”约定，见 `parser.rs` 中每个函数的文档。

mod error;
mod model;
mod parser;

pub use error::ParseModulePathError;
pub use model::ModulePath;
pub use parser::parse_module_path;
