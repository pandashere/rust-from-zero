//! 演示入口：用几组真实输入展示歌单的公开行为。
//!
//! `Playlist::add` 是本课要你实现的方法，所以在完成它之前运行
//! `cargo run -p l08_testing_quality` 会在第一次添加曲目时 panic，
//! 报错原文是 `not yet implemented`。这是脚手架的预期状态。
//!
//! 完成实现后，这个程序会依次打印：成功添加的曲目、四类错误、查找结果
//! 和时长合计。

use l08_testing_quality::{Playlist, PlaylistError};

fn main() {
    let mut playlist = Playlist::new(3);

    let additions = [
        ("  晴天  ", 269),
        ("七里香", 299),
        ("稻香", 223),
        ("晴天", 200), // 重复标题
    ];

    for (title, seconds) in additions {
        match playlist.add(title, seconds) {
            Ok(()) => println!("添加成功：{title:?}（{seconds} 秒）"),
            Err(error) => println!("添加失败：{title:?} => {error:?}"),
        }
    }

    println!("容量 {}，已装 {} 首", playlist.capacity(), playlist.len());
    println!("时长合计 {} 秒", playlist.total_seconds());

    for track in playlist.tracks() {
        println!("  {} — {} 秒", track.title(), track.seconds());
    }

    for query in ["七里香", " 稻香 ", "不存在的歌"] {
        match playlist.find(query) {
            Some(track) => println!("查找到 {query:?} => {} 秒", track.seconds()),
            None => println!("没有找到 {query:?}"),
        }
    }

    let longest = playlist.longest().map(|track| track.title());
    println!("最长曲目：{longest:?}");

    // 展示错误类型可以被外部代码匹配：这里只关心“歌单满了”这一种。
    match playlist.add("额外的歌", 100) {
        Err(PlaylistError::Full { capacity }) => println!("歌单已满，容量 {capacity}"),
        other => println!("意外结果：{other:?}"),
    }
}
