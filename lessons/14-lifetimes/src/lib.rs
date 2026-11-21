//! 第十四课《生命周期、借用关系与零复制文本 API》的练习脚手架。
//!
//! - `exercise`：主练习，从一段配置文本建立零复制的键值目录。
//! - `consolidation`：课后巩固，为一段普通文本建立借用的单词索引。
//!
//! 两个模块都只保存对原始文本的引用，不把键、值或单词复制成新的
//! `String`。完成练习后可以运行：
//!
//! ```bash
//! cargo test -p l14_lifetimes
//! cargo run -p l14_lifetimes
//! ```

pub mod consolidation;
pub mod exercise;
