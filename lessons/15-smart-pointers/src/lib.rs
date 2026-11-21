//! 第十五课《`Rc`、`Weak`、`RefCell` 与内部可变性》的练习脚手架。
//!
//! - `exercise`：主练习，实现一棵可以共享、可以从任意节点向上查找的组织树。
//! - `consolidation`：课后巩固，实现一个只弱引用订阅者的消息频道。
//!
//! 两个模块底部的测试都已写好。完成实现之前，行为测试会在 `todo!()`
//! 处失败，这是学习流程的一部分。

pub mod consolidation;
pub mod exercise;
