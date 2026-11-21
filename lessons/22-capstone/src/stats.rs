//! 统计模型：对一组记录求数量、总和、最小值、最大值和平均值。
//!
//! 总和用 `i128` 保存。`i64` 的加法在接近极值时可能溢出，而 `i128` 能容纳
//! 大量 `i64` 之和；本课在这里先选择更宽的类型，而不是依赖输入规模“应该
//! 不会太大”的假设。

use crate::model::Record;

/// 一组记录的聚合结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Summary {
    /// 记录条数。
    pub count: u64,
    /// 所有 value 的和；空集合为 0。
    pub sum: i128,
    /// 最小 value；空集合为 `None`。
    pub min: Option<i64>,
    /// 最大 value；空集合为 `None`。
    pub max: Option<i64>,
}

impl Summary {
    /// 计算记录的数量、总和、最小值和最大值。
    ///
    /// 空切片得到 `count = 0`、`sum = 0`、`min = None`、`max = None`。
    /// 单条记录的最小值和最大值相同。value 可以为负数或 0。
    pub fn summarize(records: &[Record]) -> Self {
        let _ = records;
        todo!()
    }

    /// 平均值；`count` 为 0 时返回 `None`。
    ///
    /// 结果用 `f64` 表示，因此极大整数可能损失精度；这是展示层可接受的
    /// 近似，原始总和仍保存在 [`Summary::sum`]。
    pub fn mean(&self) -> Option<f64> {
        let _ = self;
        todo!()
    }
}

/// 按 category 分组统计，结果按分类名从小到大排列。
///
/// 每条记录只进入一个组。空输入得到空 `Vec`。结果按分类名升序，且不
/// 依赖哈希表那种不保证顺序的迭代方式。
pub fn group_by_category(records: &[Record]) -> Vec<(String, Summary)> {
    let _ = records;
    todo!()
}
