//! 第八课《测试、文档测试与质量门禁》的练习脚手架。
//!
//! 本课的练习只有一个领域：一个有容量上限的歌单。
//!
//! - `playlist`：歌单的数据类型、错误类型和查询逻辑。四个方法留给学习者
//!   实现，其余骨架、自动测试和文档示例都已经给出。
//! - `statistics`：课后巩固用的平均数与中位数，同样留给学习者实现。
//!
//! 所有行为测试都已提前写好。实现之前，测试会在对应的 `todo!()` 处以
//! `not yet implemented` 失败；全部实现之后，整套测试应当通过。

mod playlist;
pub mod statistics;

pub use playlist::{Playlist, PlaylistError, Track};
