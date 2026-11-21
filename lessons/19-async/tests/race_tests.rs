//! 课后巩固的行为测试：竞速、取消与 poll 计数。
//!
//! 与主练习不同，这里没有作业、服务或重试；数据是两个 future 以及它们的
//! 输出。测试同样完全确定，不访问网络，也不依赖墙钟时间。

use std::cell::Cell;
use std::rc::Rc;

use l19_async::consolidation::{Winner, first_ready, polls_until_ready};
use l19_async::runtime::{block_on, yield_now};

async fn after_yields<T>(yields: usize, value: T) -> T {
    for _ in 0..yields {
        yield_now().await;
    }
    value
}

struct DropCounter(Rc<Cell<usize>>);

impl Drop for DropCounter {
    fn drop(&mut self) {
        self.0.set(self.0.get() + 1);
    }
}

#[test]
fn left_wins_when_both_are_ready_on_the_first_poll() {
    let winner = block_on(first_ready(async { 1 }, async { 2 }));

    assert_eq!(winner, Winner::Left(1));
}

#[test]
fn right_wins_when_left_is_still_pending() {
    let winner = block_on(first_ready(after_yields(2, "左"), after_yields(0, "右")));

    assert_eq!(winner, Winner::Right("右"));
}

#[test]
fn left_wins_a_tie_on_the_same_poll() {
    let winner = block_on(first_ready(after_yields(1, "左"), after_yields(1, "右")));

    assert_eq!(winner, Winner::Left("左"));
}

#[test]
fn outputs_of_the_two_futures_may_have_different_types() {
    let winner = block_on(first_ready(after_yields(2, 1_u8), async {
        String::from("右")
    }));

    assert_eq!(winner, Winner::Right(String::from("右")));
}

#[test]
fn loser_is_dropped_when_left_wins_immediately() {
    let drops = Rc::new(Cell::new(0));
    let guard = DropCounter(Rc::clone(&drops));
    let slow = async move {
        let _guard = guard;
        for _ in 0..5 {
            yield_now().await;
        }
        99
    };

    let winner = block_on(first_ready(async { 7 }, slow));

    assert_eq!(winner, Winner::Left(7));
    assert_eq!(drops.get(), 1);
}

#[test]
fn loser_is_dropped_when_right_wins() {
    let drops = Rc::new(Cell::new(0));
    let guard = DropCounter(Rc::clone(&drops));
    let slow = async move {
        let _guard = guard;
        for _ in 0..5 {
            yield_now().await;
        }
        99
    };

    let winner = block_on(first_ready(slow, async { 8 }));

    assert_eq!(winner, Winner::Right(8));
    assert_eq!(drops.get(), 1);
}

#[test]
fn polls_until_ready_returns_zero_for_an_immediately_ready_future() {
    let (value, pending_polls) = block_on(polls_until_ready(async { "立即" }));

    assert_eq!(value, "立即");
    assert_eq!(pending_polls, 0);
}

#[test]
fn polls_until_ready_counts_every_pending_poll() {
    let (value, pending_polls) = block_on(polls_until_ready(after_yields(3, 42)));

    assert_eq!(value, 42);
    assert_eq!(pending_polls, 3);
}

#[test]
fn polls_until_ready_returns_owned_output() {
    let (value, pending_polls) = block_on(polls_until_ready(after_yields(0, String::from("文本"))));

    assert_eq!(value, "文本");
    assert_eq!(pending_polls, 0);
}
