//! 第五课的演示入口：模拟一组温度样本进入有界窗口。
//!
//! 在完成 `exercise.rs` 中的 `todo!()` 之前，运行本程序会在第一次
//! 调用窗口方法时中止（panic）——这是预期现象，先去做练习，再回来
//! 看完整输出。

use l05_vec_slices::exercise::Window;

fn main() {
    // 容量为 5 的窗口：只保留最近 5 个样本。
    let mut window = Window::new(5);
    window.extend([18, -3, 21, 19, 24]);
    window.push(26);

    println!("当前窗口: {:?}", window.values());
    println!("最新样本: {:?}", window.latest());

    // 负数是无效读数，清理掉。
    window.clear_negative();
    println!("清理负数后: {:?}", window.values());

    let (older, recent) = window.split_recent(2);
    println!("较旧部分: {older:?}");
    println!("最近两个: {recent:?}");
}
