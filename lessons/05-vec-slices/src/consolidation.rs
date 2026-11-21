//! 课后巩固：分析一组考试分数。
//!
//! 这是主练习之后的独立任务：数据不同（分数而不是传感器样本），边界也
//! 不同（“没有任何分数”“加分超过满分”）。函数只借用切片，不拥有自己的
//! 集合，正好练习 `&[T]` 与 `&mut [T]` 的取舍。
//!
//! 请只替换三个函数体里的 `todo!()`，不要修改类型定义、函数签名，也不
//! 要改动本文件底部的测试。每个函数上方的注释就是它的完整行为约定。
//!
//! 完成后的验收命令：
//!
//! ```bash
//! cargo test -p l05_vec_slices consolidation
//! ```

/// 一组分数的统计结果：份数、最低分、最高分和总分。
///
/// `total` 用 `u64` 存放：分数个数很多时，累加结果可能超出 `u32`
/// 能表示的范围（0 到约 42 亿）。
#[derive(Debug, PartialEq, Eq)]
pub struct Summary {
    pub count: usize,
    pub min: u32,
    pub max: u32,
    pub total: u64,
}

/// 统计一组分数，返回份数、最低分、最高分和总分。
///
/// 行为约定：
///
/// - 输入为空（一个分数也没有）时返回 `None`，因为最低分、最高分都
///   不存在；
/// - 否则返回 `Some(Summary)`：`count` 是分数个数，`min` 和 `max`
///   分别是出现的最低分和最高分，`total` 是所有分数的加总。
pub fn summarize(scores: &[u32]) -> Option<Summary> {
    // 完成实现前避免“参数未使用”的提示，写好函数体后可删除。
    let _ = scores;
    todo!()
}

/// 选出最高的 `count` 个分数，从高到低排列后返回。
///
/// 行为约定：
///
/// - 返回的 `Vec<u32>` 按从高到低排序；并列的分数都要保留，例如
///   两个 90 分会占两个位置；
/// - `count` 大于等于分数总数时，返回全部分数（同样从高到低）；
/// - `count` 为 `0` 或输入为空时，返回空的 `Vec`；
/// - 传入的切片是借用，函数不能也不应修改它，调用之后原数据保持
///   原样。
pub fn top_scores(scores: &[u32], count: usize) -> Vec<u32> {
    // 完成实现前避免“参数未使用”的提示，写好函数体后可删除。
    let _ = (scores, count);
    todo!()
}

/// 给每个分数加上 `bonus` 分，但任何分数最高不超过 100。
///
/// 行为约定：
///
/// - 直接修改传入的切片，不返回新数据；
/// - 加上 `bonus` 后超过 100 的分数记为 100；
/// - 空切片不做任何事，也不出错。
pub fn apply_bonus(scores: &mut [u32], bonus: u32) {
    // 完成实现前避免“参数未使用”的提示，写好函数体后可删除。
    let _ = (scores, bonus);
    todo!()
}

#[cfg(test)]
mod tests {
    use super::{Summary, apply_bonus, summarize, top_scores};

    #[test]
    fn summarize_returns_none_when_there_are_no_scores() {
        assert_eq!(summarize(&[]), None);
    }

    #[test]
    fn summarize_handles_a_single_score() {
        assert_eq!(
            summarize(&[75]),
            Some(Summary {
                count: 1,
                min: 75,
                max: 75,
                total: 75,
            })
        );
    }

    #[test]
    fn summarize_reports_count_min_max_and_total() {
        assert_eq!(
            summarize(&[90, 60, 90, 45]),
            Some(Summary {
                count: 4,
                min: 45,
                max: 90,
                total: 285,
            })
        );
    }

    #[test]
    fn top_scores_returns_the_highest_scores_in_descending_order() {
        assert_eq!(top_scores(&[58, 92, 74, 92, 61], 3), [92, 92, 74]);
    }

    #[test]
    fn top_scores_handles_count_boundaries() {
        let scores = [30, 10, 20];

        assert_eq!(top_scores(&scores, 0), []);
        assert_eq!(top_scores(&scores, 3), [30, 20, 10]);
        assert_eq!(top_scores(&scores, 10), [30, 20, 10]);
        assert_eq!(top_scores(&[], 2), []);
    }

    #[test]
    fn top_scores_leaves_the_input_unchanged() {
        let scores = [58, 92, 74];

        let _ = top_scores(&scores, 2);

        assert_eq!(scores, [58, 92, 74]);
    }

    #[test]
    fn apply_bonus_adds_points_up_to_the_cap() {
        let mut scores = [95, 80, 100];

        apply_bonus(&mut scores, 10);

        assert_eq!(scores, [100, 90, 100]);
    }

    #[test]
    fn apply_bonus_accepts_an_empty_slice() {
        let mut scores: [u32; 0] = [];

        apply_bonus(&mut scores, 10);

        assert_eq!(scores, []);
    }
}
