//! 歌单（playlist）库：容量上限、标题去重和只读查询。
//!
//! 模块分工与第七课相同：`model` 保存数据，`error` 保存失败原因，
//! `queue` 保存歌单本身；本层只对外重新导出公共项。`mod` 声明和
//! `pub use` 重新导出的规则见[第七课](../../../07-modules-crates/README.md)。

mod error;
mod model;
mod queue;

pub use error::PlaylistError;
pub use model::Track;
pub use queue::Playlist;
