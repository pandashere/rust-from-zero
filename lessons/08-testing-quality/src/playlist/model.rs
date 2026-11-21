//! 单首曲目的数据。

/// 歌单中的一首曲目：标题加上时长（单位是秒）。
///
/// 字段是私有的，crate（Rust 的一次编译单位，这里是 library）外部只能
/// 通过 `title()` 和 `seconds()` 读取。曲目只由 `Playlist::add` 在检查
/// 通过之后创建，因此每个存在的 `Track` 都满足“标题非空、时长大于 0”
/// 的规则——这就是第七课讲的不变量（invariant，始终成立的条件）。
///
/// # 字段不能从外部直接构造
///
/// 下面这段代码故意无法编译，因为两个字段都是私有的，外部代码只能走
/// `Playlist::add` 这条经过检查的路径：
///
/// ```compile_fail
/// use l08_testing_quality::Track;
///
/// let track = Track { title: String::from("晴天"), seconds: 269 };
/// ```
#[derive(Debug, PartialEq, Eq)]
pub struct Track {
    title: String,
    seconds: u32,
}

impl Track {
    /// 歌单内部使用的构造器：调用者必须已经检查过两个参数。
    #[allow(dead_code)] // 学习者完成 `Playlist::add` 之后会用到它。
    pub(super) fn new(title: String, seconds: u32) -> Self {
        Self { title, seconds }
    }

    /// 返回标题，也就是添加时去掉两端空白后的文本。
    pub fn title(&self) -> &str {
        &self.title
    }

    /// 返回时长，单位是秒。
    pub fn seconds(&self) -> u32 {
        self.seconds
    }
}
