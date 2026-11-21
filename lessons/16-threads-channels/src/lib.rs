//! 第十六课《线程、`move` 闭包与 channel》的练习脚手架。
//!
//! - `exercise`：主练习，用固定数量的工作线程并行计算字节块校验值。
//! - `consolidation`：课后巩固，用作用域线程并行统计一组整数的范围信息。
//!
//! 两个模块里的 `todo!()` 都是学习者任务；类型定义、函数签名和测试已经
//! 准备好，不需要修改。

pub mod consolidation;
pub mod exercise;
