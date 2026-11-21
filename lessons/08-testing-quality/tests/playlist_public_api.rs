//! 集成测试：作为独立的 crate 只使用 `l08_testing_quality` 的 crate 根导出，
//! 模拟一位外部用户能看到的全部接口。
//!
//! 这里**不能**写 `use l08_testing_quality::playlist::...`：`playlist` 模块
//! 对外是私有的，集成测试只比外部用户多知道 crate 根上的名字。

use l08_testing_quality::{Playlist, PlaylistError};

#[test]
fn public_api_builds_a_playlist_and_reads_it_back() {
    let mut playlist = Playlist::new(2);

    playlist.add(" 晴天 ", 269).unwrap();
    playlist.add("七里香", 299).unwrap();

    assert_eq!(playlist.capacity(), 2);
    assert_eq!(playlist.len(), 2);
    assert_eq!(playlist.tracks().len(), 2);
    assert_eq!(playlist.tracks()[0].title(), "晴天");
    assert_eq!(playlist.tracks()[1].seconds(), 299);
    assert_eq!(playlist.total_seconds(), 568);
    assert_eq!(
        playlist.longest().map(|track| track.title()),
        Some("七里香")
    );
}

#[test]
fn public_api_reports_each_failure_as_a_structured_variant() {
    let mut playlist = Playlist::new(1);
    playlist.add("晴天", 269).unwrap();

    assert_eq!(playlist.add("   ", 10), Err(PlaylistError::EmptyTitle));
    assert_eq!(playlist.add("稻香", 0), Err(PlaylistError::ZeroLength));
    assert_eq!(
        playlist.add(" 晴天", 200),
        Err(PlaylistError::DuplicateTitle(String::from("晴天")))
    );
    assert_eq!(
        playlist.add("稻香", 223),
        Err(PlaylistError::Full { capacity: 1 })
    );
    assert_eq!(playlist.len(), 1);
}

#[test]
fn public_api_searches_with_trimmed_titles_and_returns_none_when_absent() {
    let mut playlist = Playlist::new(3);
    playlist.add("Yesterday", 125).unwrap();

    let found = playlist.find(" Yesterday\t").expect("应当找到 Yesterday");

    assert_eq!(found.seconds(), 125);
    assert!(playlist.find("yesterday").is_none());
    assert!(playlist.find("Let It Be").is_none());
}
