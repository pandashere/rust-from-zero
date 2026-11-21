//! 第十九课《`async`/`.await`、`Future` 与运行时边界》的练习脚手架。
//!
//! - `runtime`：本课自带的最小 executor 和辅助 future，已经写好；
//! - `exercise`：主练习，有界并发通知调度器；
//! - `consolidation`：课后巩固，竞速与 poll 计数。
//!
//! 学习者只替换 `exercise` 和 `consolidation` 中的 `todo!()`。
//! 规格见 README，卡住时按级查看 HINTS.md。

pub mod consolidation;
pub mod exercise;
pub mod runtime;
