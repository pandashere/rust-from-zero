//! 第二十二课《综合命令行数据工具》的练习脚手架。
//!
//! 程序名是 `datakit`，二进制由同一个 package 产生。阅读顺序：
//!
//! - [`model`]：一条记录及它的文本格式；
//! - [`query`]：筛选、排序、截断；
//! - [`stats`]：聚合与分组；
//! - [`store`]：读写存储文件；
//! - [`output`]：稳定文本输出；
//! - [`cli`]：参数解析；
//! - [`app`]：把以上模块连接成命令；
//! - [`sessions`]：课后巩固用的第二份数据脚手架。
//!
//! 学习者负责的函数以 `todo!()` 标记，具体行为写在函数文档里。测试只
//! 检查公开行为，因此内部实现可以自由重构。

pub mod app;
pub mod cli;
pub mod error;
pub mod model;
pub mod output;
pub mod query;
pub mod sessions;
pub mod stats;
pub mod store;
