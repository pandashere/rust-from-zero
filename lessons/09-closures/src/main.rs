//! 第九课的演示入口：把闭包传给事件处理管线，并观察捕获的环境。
//!
//! 在完成 `exercise.rs` 里的 `todo!()` 之前，运行本程序会在第一次调用
//! `apply_all` 时中止（panic）——这是预期现象。先去做练习，再回来看
//! 完整输出。

use l09_closures::exercise::{
    Event, apply_all, first_match, make_threshold_filter, process_events, run_once,
};

fn main() {
    let mut values = [1, 2, 3];
    apply_all(&mut values, |value| value * 10);
    println!("apply_all: {values:?}");

    let found = first_match(&values, |value| value > 15);
    println!("first_match: {found:?}");

    let completed = run_once(|| String::from("done"));
    println!("run_once: {completed}");

    let events = vec![
        Event { id: 1, value: 5 },
        Event { id: 2, value: 10 },
        Event { id: 3, value: 15 },
    ];
    let filter = make_threshold_filter(10);
    let mut handled = 0;
    let result: Result<Vec<String>, &str> = process_events(&events, filter, |event| {
        handled += 1;
        Ok(format!("event {} has value {}", event.id, event.value))
    });

    println!("process_events: {result:?}");
    println!("handler calls: {handled}");
}
