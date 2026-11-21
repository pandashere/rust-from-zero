//! 第二十一课：`unsafe`、裸指针与 FFI 的练习脚手架。
//!
//! - [`exercise`]：主练习，用裸指针实现受检的可变切片拆分；
//! - [`ffi`]：C ABI 字节计数函数的声明、替身实现与安全包装；
//! - [`consolidation`]：课后巩固，把 C 字符串读取成拥有型 Rust 文本。
//!
//! 每个 `todo!()` 都是学习者负责实现的部分。实现前行为测试会失败，这是
//! 学习流程的一部分；规格见 README「主练习」与「课后巩固」，卡住时按级
//! 查看 HINTS.md。

pub mod consolidation;
pub mod exercise;
pub mod ffi;

pub use consolidation::{NameError, read_c_name};
pub use exercise::split_at_mut_checked;
pub use ffi::count_byte;
