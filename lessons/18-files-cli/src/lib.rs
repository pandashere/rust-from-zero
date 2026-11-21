//! 第十八课《文件、路径、命令行参数与输入输出错误》的练习脚手架。
//!
//! - `exercise`：主练习，实现带开关的文件统计命令 `rstat`。
//! - `consolidation`：课后巩固，整理文本文件并写入另一个路径。
//!
//! 真正的命令行入口在 `src/main.rs`。它只负责连接进程参数、标准输出、
//! 错误输出和退出码，具体行为都在本库中实现，方便自动测试。

pub mod consolidation;
pub mod exercise;

#[cfg(test)]
mod test_support;
