//! 课后巩固：实现一个自定义倒计时迭代器。
//!
//! 主练习处理的是日志文本；这个任务换成数字序列，边界也不同：起始值是
//! `0` 时一个元素都不产生，取完最后一个值之后还要继续返回“没有值”。
//!
//! 请只替换本文件中 `todo!()` 所在的三处函数体，不要修改类型定义、函数
//! 签名、`ExactSizeIterator` 的实现和文件底部的测试。
//!
//! 完成后的验收命令：
//!
//! ```bash
//! cargo test -p l10_iterators consolidation
//! ```

/// 从某个起始值开始、每次减 1 的倒计时迭代器。
///
/// 例如 `Countdown::new(3)` 依次产出 `3`、`2`、`1`，然后永远返回“没有
/// 值”。起始值为 `0` 时不产出任何值。
#[derive(Debug, Clone)]
pub struct Countdown {
    // 这个字段在 `todo!()` 被替换后才会被读取；完成实现后可以删除下面的
    // `allow` 注释。
    #[allow(dead_code)]
    remaining: u32,
}

impl Countdown {
    /// 创建一个倒计时迭代器。
    ///
    /// `start` 是第一次调用 `next` 时产出的值；之后每次调用都产出比上一次
    /// 小 1 的值，直到产出 `1`。`start` 为 `0` 时迭代器一开始就是结束状态。
    pub fn new(start: u32) -> Self {
        // 完成实现前避免“参数未使用”的提示，写好函数体后可删除。
        let _ = start;
        todo!()
    }
}

impl Iterator for Countdown {
    type Item = u32;

    /// 产出下一个值：还剩数字时交出当前值并把剩余数量减 1；已经结束时交出
    /// `None`。
    ///
    /// 结束之后状态不再变化，因此可以反复调用，结果始终是 `None`。
    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }

    /// 返回“还能产出多少个值”的提示。
    ///
    /// 下界和上界都必须等于剩余值的真实数量：例如刚创建 `Countdown::new(3)`
    /// 时返回 `(3, Some(3))`；取走一个之后返回 `(2, Some(2))`；取完之后返回
    /// `(0, Some(0))`。
    fn size_hint(&self) -> (usize, Option<usize>) {
        todo!()
    }
}

// 这个标记表示“剩余数量总是可以精确算出来”。它没有需要实现的方法：默认的
// `len()` 会读取上面的 `size_hint`，所以只要提示准确，`len()` 就准确。
impl ExactSizeIterator for Countdown {}

#[cfg(test)]
mod tests {
    use super::Countdown;

    #[test]
    fn countdown_yields_start_down_to_one() {
        let mut countdown = Countdown::new(3);

        assert_eq!(countdown.next(), Some(3));
        assert_eq!(countdown.next(), Some(2));
        assert_eq!(countdown.next(), Some(1));
        assert_eq!(countdown.next(), None);
    }

    #[test]
    fn countdown_from_zero_is_empty() {
        assert_eq!(Countdown::new(0).next(), None);
        assert_eq!(Countdown::new(0).size_hint(), (0, Some(0)));
    }

    #[test]
    fn countdown_from_one_yields_exactly_one_value() {
        assert_eq!(Countdown::new(1).collect::<Vec<_>>(), [1]);
    }

    #[test]
    fn countdown_stays_finished_after_returning_none() {
        let mut countdown = Countdown::new(1);

        assert_eq!(countdown.next(), Some(1));
        assert_eq!(countdown.next(), None);
        assert_eq!(countdown.next(), None);
        assert_eq!(countdown.next(), None);
    }

    #[test]
    fn countdown_works_in_a_for_loop() {
        let mut seen = Vec::new();

        for value in Countdown::new(4) {
            seen.push(value);
        }

        assert_eq!(seen, [4, 3, 2, 1]);
    }

    #[test]
    fn countdown_size_hint_tracks_remaining_values() {
        let mut countdown = Countdown::new(3);

        assert_eq!(countdown.size_hint(), (3, Some(3)));
        countdown.next();
        assert_eq!(countdown.size_hint(), (2, Some(2)));
        countdown.next();
        countdown.next();
        assert_eq!(countdown.size_hint(), (0, Some(0)));
    }

    #[test]
    fn countdown_len_matches_remaining_values() {
        let mut countdown = Countdown::new(3);

        assert_eq!(countdown.len(), 3);
        countdown.next();
        assert_eq!(countdown.len(), 2);
    }

    #[test]
    fn countdown_can_drive_adapters() {
        let doubled_evens: Vec<u32> = Countdown::new(6)
            .filter(|value| value % 2 == 0)
            .map(|value| value * 10)
            .collect();

        assert_eq!(doubled_evens, [60, 40, 20]);
    }

    #[test]
    fn countdown_handles_the_largest_u32_without_overflow() {
        let mut countdown = Countdown::new(u32::MAX);

        assert_eq!(countdown.next(), Some(u32::MAX));
        assert_eq!(countdown.next(), Some(u32::MAX - 1));
    }
}
