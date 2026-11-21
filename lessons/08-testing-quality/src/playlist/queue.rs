//! 歌单本身：容量上限、标题去重和只读查询。

use super::{PlaylistError, Track};

/// 一个有容量上限的歌单。
///
/// 歌单按添加到顺序保存曲目（先进先出地排列，不做排序）。两个字段都是
/// 私有的：容量在创建时确定、之后不再改变，曲目只能通过 `add` 追加，
/// 因此“曲目数永远不超过容量”这个条件在任何时候都成立。
///
/// # 例子
///
/// ```rust
/// use l08_testing_quality::Playlist;
///
/// let playlist = Playlist::new(3);
///
/// assert_eq!(playlist.capacity(), 3);
/// assert_eq!(playlist.len(), 0);
/// assert!(playlist.is_empty());
/// assert!(playlist.tracks().is_empty());
/// ```
///
/// 这个例子只用到脚手架已经给出的方法，所以现在就能通过。往歌单里添加
/// 曲目的例子见 [`Playlist::add`]。
#[derive(Debug)]
pub struct Playlist {
    capacity: usize,
    tracks: Vec<Track>,
}

impl Playlist {
    /// 创建一个容量为 `capacity` 的空歌单。
    ///
    /// 容量是曲目数量的上限，不是时长上限。容量为 0 也是合法歌单：它
    /// 永远装不进任何曲目，每次添加都会得到
    /// [`PlaylistError::Full`]。
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            tracks: Vec::new(),
        }
    }

    /// 返回容量，也就是最多能装多少首曲目。
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// 返回当前曲目数量。
    pub fn len(&self) -> usize {
        self.tracks.len()
    }

    /// 歌单里一首曲目也没有时返回 `true`。
    pub fn is_empty(&self) -> bool {
        self.tracks.is_empty()
    }

    /// 按添加顺序返回所有曲目的只读视图。
    ///
    /// 返回的是切片（slice，一段连续数据的借用视图），不复制曲目。
    pub fn tracks(&self) -> &[Track] {
        &self.tracks
    }

    /// 往歌单末尾追加一首曲目。
    ///
    /// # 参数
    ///
    /// - `title`：曲目标题。会先去掉两端空白（空格、制表符、换行，以及
    ///   全角空格一类的 Unicode 空白字符），保存和比较都用去掉空白后的
    ///   文本；大小写原样保留。
    /// - `seconds`：时长，单位是秒。`0` 不合法，其余 `u32` 值都合法，
    ///   包括最大值 `u32::MAX`。
    ///
    /// # 失败原因与检查顺序
    ///
    /// 四项检查按固定顺序执行，先命中的原因就是返回的错误：
    ///
    /// 1. 去掉空白后的标题为空（包括原本就是空字符串）：
    ///    [`PlaylistError::EmptyTitle`]；
    /// 2. `seconds` 等于 0：[`PlaylistError::ZeroLength`]；
    /// 3. 歌单里已有标题完全相同的曲目（比较的是各自去掉空白后的文本，
    ///    区分大小写）：[`PlaylistError::DuplicateTitle`]，附带要去重的那段
    ///    文本；
    /// 4. 曲目数量已经等于容量：[`PlaylistError::Full`]，附带容量。
    ///
    /// 顺序固定意味着：歌单已满时再添加一个重复标题，得到的是
    /// `DuplicateTitle` 而不是 `Full`；标题为空时，无论时长和容量如何，
    /// 得到的都是 `EmptyTitle`。
    ///
    /// 失败时歌单保持原样，不会留下半成品曲目。
    ///
    /// # 例子
    ///
    /// 这个例子标记为 `no_run`：它必须能编译，但不会在测试中执行，因为
    /// 它调用的方法正是本课要你实现的部分（文档测试的属性见 README）。
    ///
    /// ```rust,no_run
    /// use l08_testing_quality::{Playlist, PlaylistError};
    ///
    /// let mut playlist = Playlist::new(2);
    ///
    /// playlist.add(" 晴天 ", 269).unwrap();
    /// assert_eq!(playlist.len(), 1);
    /// assert_eq!(playlist.tracks()[0].title(), "晴天");
    ///
    /// assert_eq!(
    ///     playlist.add("晴天", 200),
    ///     Err(PlaylistError::DuplicateTitle(String::from("晴天"))),
    /// );
    ///
    /// playlist.add("七里香", 299).unwrap();
    /// assert_eq!(
    ///     playlist.add("稻香", 223),
    ///     Err(PlaylistError::Full { capacity: 2 }),
    /// );
    /// # Ok::<(), PlaylistError>(())
    /// ```
    pub fn add(&mut self, title: &str, seconds: u32) -> Result<(), PlaylistError> {
        let _ = (&self, title, seconds);
        todo!("实现 Playlist::add：去空白、检查空标题与零时长、去重、检查容量，然后追加")
    }

    /// 按标题查找一首曲目。
    ///
    /// 查找参数同样先去掉两端空白，再与每首曲目保存的标题做完全相同的
    /// 比较（区分大小写）。匹配时返回那首曲目的只读引用，不复制标题；
    /// 找不到返回 `None`。
    ///
    /// 歌单里不可能有重复标题，所以最多匹配一首。
    pub fn find(&self, title: &str) -> Option<&Track> {
        let _ = (&self, title);
        todo!("实现 Playlist::find：去空白后按标题查找，返回借用而不是复制")
    }

    /// 返回所有曲目时长之和，单位是秒。
    ///
    /// 空歌单返回 `0`。返回值类型是 `u64` 而不是 `u32`：容量很大时若干
    /// 首接近 `u32::MAX` 的曲目相加会超过 `u32` 的范围，用 `u64` 累加
    /// 才不会被截断。
    pub fn total_seconds(&self) -> u64 {
        let _ = &self;
        todo!("实现 Playlist::total_seconds：用 u64 累加每首曲目的时长")
    }

    /// 返回时长最长的那首曲目。
    ///
    /// 空歌单返回 `None`。多首曲目时长并列最长时，返回歌单中位置最靠前
    /// 的那一首；返回值是引用，不复制曲目。
    pub fn longest(&self) -> Option<&Track> {
        let _ = &self;
        todo!("实现 Playlist::longest：空歌单返回 None，并列时保留先出现的那首")
    }
}

#[cfg(test)]
mod tests {
    use super::{Playlist, PlaylistError};

    /// 一个已经装好两首曲目的歌单，供多个测试复用。
    fn sample_playlist() -> Playlist {
        let mut playlist = Playlist::new(4);
        playlist.add("晴天", 269).unwrap();
        playlist.add("七里香", 299).unwrap();
        playlist
    }

    // ---- 脚手架已给出的行为：新歌单的初始状态 ----

    #[test]
    fn new_playlist_starts_empty_with_the_requested_capacity() {
        let playlist = Playlist::new(4);

        assert_eq!(playlist.capacity(), 4);
        assert_eq!(playlist.len(), 0);
        assert!(playlist.is_empty());
        assert!(playlist.tracks().is_empty());
    }

    // ---- 正常行为 ----

    #[test]
    fn add_stores_the_trimmed_title_and_seconds_in_order() {
        let playlist = sample_playlist();
        let titles: Vec<&str> = playlist
            .tracks()
            .iter()
            .map(|track| track.title())
            .collect();

        assert_eq!(titles, ["晴天", "七里香"]);
        assert_eq!(playlist.len(), 2);
        assert!(!playlist.is_empty());
        assert_eq!(playlist.tracks()[0].seconds(), 269);
        assert_eq!(playlist.tracks()[1].seconds(), 299);
    }

    #[test]
    fn add_trims_whitespace_on_both_sides_of_the_title() {
        let mut playlist = Playlist::new(2);

        playlist.add("\t 稻香 \n", 223).unwrap();

        assert_eq!(playlist.tracks()[0].title(), "稻香");
    }

    #[test]
    fn add_accepts_the_largest_u32_length() {
        let mut playlist = Playlist::new(1);

        assert_eq!(playlist.add("超长曲目", u32::MAX), Ok(()));
        assert_eq!(playlist.tracks()[0].seconds(), u32::MAX);
    }

    // ---- 错误行为 ----

    #[test]
    fn add_rejects_titles_that_are_empty_after_trimming() {
        let mut playlist = Playlist::new(2);

        assert_eq!(playlist.add("", 200), Err(PlaylistError::EmptyTitle));
        assert_eq!(
            playlist.add("   \t\n ", 200),
            Err(PlaylistError::EmptyTitle)
        );
        // 全角空格 U+3000 也属于空白字符。
        assert_eq!(playlist.add("　", 200), Err(PlaylistError::EmptyTitle));
        assert!(playlist.is_empty());
    }

    #[test]
    fn add_rejects_zero_seconds() {
        let mut playlist = Playlist::new(2);

        assert_eq!(playlist.add("静音", 0), Err(PlaylistError::ZeroLength));
        assert!(playlist.is_empty());
    }

    #[test]
    fn add_rejects_a_duplicate_title_after_trimming() {
        let mut playlist = Playlist::new(3);

        playlist.add("晴天", 269).unwrap();

        assert_eq!(
            playlist.add("  晴天  ", 200),
            Err(PlaylistError::DuplicateTitle(String::from("晴天")))
        );
        assert_eq!(playlist.len(), 1);
    }

    #[test]
    fn add_treats_title_comparison_as_case_sensitive() {
        let mut playlist = Playlist::new(2);

        playlist.add("Yesterday", 125).unwrap();

        assert_eq!(playlist.add("yesterday", 125), Ok(()));
        assert_eq!(playlist.len(), 2);
    }

    #[test]
    fn add_reports_full_with_the_capacity_when_no_room_is_left() {
        let mut playlist = Playlist::new(2);

        playlist.add("晴天", 269).unwrap();
        playlist.add("七里香", 299).unwrap();

        assert_eq!(
            playlist.add("稻香", 223),
            Err(PlaylistError::Full { capacity: 2 })
        );
        assert_eq!(playlist.len(), 2);
        assert_eq!(playlist.tracks()[1].title(), "七里香");
    }

    #[test]
    fn a_zero_capacity_playlist_rejects_every_track() {
        let mut playlist = Playlist::new(0);

        assert_eq!(
            playlist.add("晴天", 269),
            Err(PlaylistError::Full { capacity: 0 })
        );
        assert!(playlist.is_empty());
    }

    // ---- 错误优先级：检查顺序是对外契约的一部分 ----

    #[test]
    fn duplicate_title_is_reported_before_a_full_playlist() {
        let mut playlist = Playlist::new(1);
        playlist.add("晴天", 269).unwrap();

        assert_eq!(
            playlist.add(" 晴天 ", 200),
            Err(PlaylistError::DuplicateTitle(String::from("晴天")))
        );
    }

    #[test]
    fn empty_title_is_reported_before_zero_length_and_full() {
        let mut playlist = Playlist::new(0);

        assert_eq!(playlist.add("  ", 0), Err(PlaylistError::EmptyTitle));
    }

    #[test]
    fn zero_length_is_reported_before_duplicate_and_full() {
        let mut playlist = Playlist::new(1);
        playlist.add("晴天", 269).unwrap();

        assert_eq!(playlist.add("晴天", 0), Err(PlaylistError::ZeroLength));
    }

    // ---- 查找 ----

    #[test]
    fn find_returns_the_stored_track_and_none_for_unknown_titles() {
        let playlist = sample_playlist();

        let found = playlist.find("七里香").expect("七里香 在歌单里");

        assert_eq!(found.title(), "七里香");
        assert_eq!(found.seconds(), 299);
        assert!(playlist.find("稻香").is_none());
    }

    #[test]
    fn find_trims_the_argument_but_stays_case_sensitive() {
        let mut playlist = Playlist::new(1);
        playlist.add("Yesterday", 125).unwrap();

        assert!(playlist.find("  Yesterday\t").is_some());
        assert!(playlist.find("yesterday").is_none());
    }

    #[test]
    fn find_on_an_empty_playlist_returns_none() {
        let playlist = Playlist::new(3);

        assert!(playlist.find("晴天").is_none());
    }

    // ---- 时长合计 ----

    #[test]
    fn total_seconds_of_an_empty_playlist_is_zero() {
        assert_eq!(Playlist::new(2).total_seconds(), 0);
    }

    #[test]
    fn total_seconds_sums_every_track() {
        assert_eq!(sample_playlist().total_seconds(), 269 + 299);
    }

    #[test]
    fn total_seconds_does_not_overflow_when_the_sum_exceeds_u32() {
        let mut playlist = Playlist::new(3);
        playlist.add("甲", u32::MAX).unwrap();
        playlist.add("乙", u32::MAX).unwrap();

        assert_eq!(
            playlist.total_seconds(),
            u64::from(u32::MAX) + u64::from(u32::MAX)
        );
    }

    // ---- 最长曲目 ----

    #[test]
    fn longest_of_an_empty_playlist_is_none() {
        assert!(Playlist::new(2).longest().is_none());
    }

    #[test]
    fn longest_returns_the_track_with_the_most_seconds() {
        let mut playlist = Playlist::new(4);
        playlist.add("短", 100).unwrap();
        playlist.add("长", 400).unwrap();
        playlist.add("中", 200).unwrap();

        assert_eq!(playlist.longest().map(|track| track.title()), Some("长"));
    }

    #[test]
    fn longest_keeps_the_earliest_track_when_lengths_are_tied() {
        let mut playlist = Playlist::new(3);
        playlist.add("先", 300).unwrap();
        playlist.add("后", 300).unwrap();

        assert_eq!(playlist.longest().map(|track| track.title()), Some("先"));
    }

    // ---- 表格驱动：同一行为的多种输入放在一张表里 ----

    #[test]
    fn add_outcome_table_covers_normal_and_error_cases() {
        // 一个输入 case：先按 `existing` 预置曲目，再添加 `title`/`seconds`，
        // 期望得到 `expected`。
        struct Case {
            name: &'static str,
            existing: &'static [(&'static str, u32)],
            title: &'static str,
            seconds: u32,
            expected: Result<(), PlaylistError>,
        }

        let cases = [
            Case {
                name: "普通标题",
                existing: &[],
                title: "稻香",
                seconds: 223,
                expected: Ok(()),
            },
            Case {
                name: "两端空白与制表符",
                existing: &[],
                title: " \t稻香\n ",
                seconds: 223,
                expected: Ok(()),
            },
            Case {
                name: "Unicode 标题",
                existing: &[],
                title: "君の名は",
                seconds: 250,
                expected: Ok(()),
            },
            Case {
                name: "单字符标题",
                existing: &[],
                title: "A",
                seconds: 1,
                expected: Ok(()),
            },
            Case {
                name: "空标题",
                existing: &[],
                title: "",
                seconds: 10,
                expected: Err(PlaylistError::EmptyTitle),
            },
            Case {
                name: "零时长",
                existing: &[],
                title: "静音",
                seconds: 0,
                expected: Err(PlaylistError::ZeroLength),
            },
            Case {
                name: "重复标题（去空白后相同）",
                existing: &[("晴天", 269)],
                title: "晴天 ",
                seconds: 269,
                expected: Err(PlaylistError::DuplicateTitle(String::from("晴天"))),
            },
            Case {
                name: "已满且标题不重复",
                existing: &[("晴天", 269)],
                title: "稻香",
                seconds: 223,
                expected: Err(PlaylistError::Full { capacity: 1 }),
            },
        ];

        for case in cases {
            // 每个 case 都从自己的歌单开始：测试之间没有共享状态，单独
            // 运行或整组运行结果一致。
            let mut playlist = Playlist::new(case.existing.len().max(1));
            for (title, seconds) in case.existing {
                playlist.add(title, *seconds).unwrap();
            }

            let actual = playlist.add(case.title, case.seconds);

            assert_eq!(
                actual, case.expected,
                "case {:?}，标题 {:?}，时长 {}",
                case.name, case.title, case.seconds
            );
        }
    }
}
