//! 本课自带的最小异步运行时，只用于教学和测试。
//!
//! 生产代码应使用 Tokio 等成熟运行时：它们会在没有任务可运行时挂起线程，
//! 并由操作系统事件唤醒，而不是像这里这样反复让出 CPU。本模块故意保持
//! 小巧，让 `poll`、`Pending`、`Ready` 和取消行为都能直接观察。

use std::future::Future;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll, Wake, Waker};
use std::thread;

/// 单个 future 最多被 poll 的次数。
///
/// 如果 future 依赖真实定时器或网络，而当前环境又没有这些事件来源，
/// 这个上限会把“永远等下去”变成一条明确的 panic，方便定位问题。
const POLL_BUDGET: usize = 100_000;

/// 一个什么都不做的 waker。
///
/// 真实 executor 的 waker 会把任务重新放进待运行队列，然后让线程休眠。
/// 本课的 `block_on` 在 `Pending` 后直接继续轮询，所以 waker 只需满足
/// `Future::poll` 的类型要求。
struct NoopWaker;

impl Wake for NoopWaker {
    fn wake(self: Arc<Self>) {}

    fn wake_by_ref(self: &Arc<Self>) {}
}

fn noop_waker() -> Waker {
    Waker::from(Arc::new(NoopWaker))
}

/// 在当前线程反复 poll 一个 future，直到它完成并返回结果。
///
/// 这是本课唯一的“入口”：测试和演示都用它把异步代码真正跑起来。
/// 它不创建新线程，也不处理真实 I/O；它只是一个最小的 executor（执行器，
/// 负责反复推进 future 的循环）。
pub fn block_on<F: Future>(future: F) -> F::Output {
    let waker = noop_waker();
    let mut context = Context::from_waker(&waker);
    let mut future = Box::pin(future);

    for _ in 0..POLL_BUDGET {
        if let Poll::Ready(output) = future.as_mut().poll(&mut context) {
            return output;
        }
        // 真实 executor 会在这里等待 waker 通知；教学运行时直接让出 CPU，
        // 这样测试不依赖墙钟时间，结果也可重复。
        thread::yield_now();
    }

    panic!(
        "future 在 {POLL_BUDGET} 次 poll 后仍未完成：检查它是否依赖真实的定时器、网络或永远不返回的等待"
    );
}

/// 一个“先让出一次、再完成”的 future。
///
/// 第一次 poll 返回 `Pending`，第二次 poll 返回 `Ready(())`。测试用它模拟
/// “需要等待几步才能完成”的操作，而不必真的睡眠若干秒。
#[derive(Debug, Clone, Copy, Default)]
pub struct YieldNow {
    yielded: bool,
}

impl Future for YieldNow {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.yielded {
            Poll::Ready(())
        } else {
            self.yielded = true;
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

/// 创建一个 `YieldNow`：它会在第一次 poll 时挂起，第二次 poll 时完成。
pub fn yield_now() -> YieldNow {
    YieldNow::default()
}

/// 同时推进一组 future，按输入顺序返回它们的结果。
///
/// 每次 poll 会依次询问每个尚未完成的 future。一个 future 返回 `Pending`
/// 不会阻止后面的 future 被 poll，因此它们的时间是重叠的（并发）。
/// 这不是操作系统线程的并行：所有 poll 都发生在调用 `block_on` 的线程上。
#[derive(Debug)]
pub struct JoinAll<F: Future> {
    futures: Vec<Option<Pin<Box<F>>>>,
    results: Vec<Option<F::Output>>,
    remaining: usize,
}

// `JoinAll` 内部的每个 future 都装在 `Pin<Box<_>>` 里，在自己的堆位置上一动不动；
// 结构体本身被移动时只搬运这些指针和普通结果值，因此整体可以安全地声明为 `Unpin`。
// 这样 `poll` 里就能用 `get_mut` 直接访问字段，而不用 `unsafe`。
impl<F: Future> Unpin for JoinAll<F> {}

impl<F: Future> JoinAll<F> {
    fn new(futures: Vec<F>) -> Self {
        let remaining = futures.len();
        let results = std::iter::repeat_with(|| None).take(remaining).collect();
        let futures = futures
            .into_iter()
            .map(|future| Some(Box::pin(future)))
            .collect();
        Self {
            futures,
            results,
            remaining,
        }
    }
}

impl<F: Future> Future for JoinAll<F> {
    type Output = Vec<F::Output>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();

        for (index, slot) in this.futures.iter_mut().enumerate() {
            let Some(future) = slot else {
                continue;
            };

            if let Poll::Ready(output) = future.as_mut().poll(cx) {
                this.results[index] = Some(output);
                *slot = None;
                this.remaining -= 1;
            }
        }

        if this.remaining == 0 {
            let results = this
                .results
                .iter_mut()
                .map(|slot| slot.take().expect("所有 future 都已完成"))
                .collect();
            Poll::Ready(results)
        } else {
            Poll::Pending
        }
    }
}

/// 把一组 future 包装成一个 future，等它们全部完成。
pub fn join_all<F: Future>(futures: Vec<F>) -> JoinAll<F> {
    JoinAll::new(futures)
}

/// 表示被包装的 future 在 poll 过程中 panic 了。
///
/// 生产运行时会把独立任务放在自己的执行单元里，panic 通过 `JoinHandle`
/// 的 `JoinError` 报告。本课没有独立任务，因此用这个类型把 panic 变成
/// 普通返回值，避免它直接终止整个 `block_on`。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TaskPanic;

/// 捕获 future 在一次 poll 中发生的 panic。
#[derive(Debug)]
pub struct CatchPanic<F: Future> {
    future: Pin<Box<F>>,
}

impl<F: Future> CatchPanic<F> {
    fn new(future: F) -> Self {
        Self {
            future: Box::pin(future),
        }
    }
}

// 同 `JoinAll`：被包裹的 future 固定在堆上，结构体自身移动不会移动它。
impl<F: Future> Unpin for CatchPanic<F> {}

impl<F: Future> Future for CatchPanic<F> {
    type Output = Result<F::Output, TaskPanic>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let future = &mut self.get_mut().future;

        // `catch_unwind` 要求被包裹的闭包实现 `UnwindSafe`。future 内部的
        // 状态在 panic 后会被丢弃，我们不会再使用它，因此这里的断言是安全的。
        match catch_unwind(AssertUnwindSafe(|| future.as_mut().poll(cx))) {
            Ok(Poll::Ready(output)) => Poll::Ready(Ok(output)),
            Ok(Poll::Pending) => Poll::Pending,
            Err(_) => Poll::Ready(Err(TaskPanic)),
        }
    }
}

/// 把 future 的 panic 转成 `Result`，正常完成时返回 `Ok`。
pub fn catch_panic<F: Future>(future: F) -> CatchPanic<F> {
    CatchPanic::new(future)
}
