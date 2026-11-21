//! 课后巩固：竞速与 poll 计数。
//!
//! 主练习关心“多个作业如何被调度”；这里换一组数据和边界，专门观察
//! future 的两个细节：
//!
//! - 两个 future 同时等待时，谁先完成，输掉的那个会怎样；
//! - 一个 future 在完成前一共返回了多少次 `Pending`。
//!
//! 规格见 README「课后巩固」一节，卡住时按级查看 HINTS.md。

use std::future::Future;

/// 两个 future 竞速的结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Winner<A, B> {
    /// 左边的 future 先完成，携带它的输出。
    Left(A),
    /// 右边的 future 先完成，携带它的输出。
    Right(B),
}

/// 同时推进 `left` 和 `right`，返回先完成的那一个。
///
/// 规则：
///
/// - 每轮先 poll `left`，如果它完成了，立即返回 `Winner::Left`；
/// - 否则再 poll `right`，如果它完成了，返回 `Winner::Right`；
/// - 两边都返回 `Pending` 时，本轮结束，等待下一次 poll；
/// - 如果两个 future 在同一轮都完成，因为先检查 `left`，所以 `Left` 获胜；
/// - 一旦返回，另一个 future 会被丢弃（取消）；它已经产生的副作用不会回滚，
///   但它持有的局部变量会正常析构。
///
/// 返回类型不需要两个 future 的输出类型相同。
pub async fn first_ready<A, B>(left: A, right: B) -> Winner<A::Output, B::Output>
where
    A: Future,
    B: Future,
{
    let _ = (&left, &right);
    todo!()
}

/// 手动 poll 一个 future 直到完成，返回它的输出和期间经历的 `Pending` 次数。
///
/// - future 第一次 poll 就完成时，计数为 0；
/// - 每返回一次 `Pending`，计数加一，然后继续 poll；
/// - 输出原样返回，不复制也不修改；
/// - 本函数不限制次数；调用方应保证 future 最终会完成，否则 `block_on`
///   的总 poll 上限会以 panic 结束。
pub async fn polls_until_ready<F: Future>(future: F) -> (F::Output, usize) {
    let _ = &future;
    todo!()
}
