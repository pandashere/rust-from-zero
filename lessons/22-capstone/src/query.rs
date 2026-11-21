//! 查询模型：把“筛选、排序、截断”表达成一个普通值，再一次性执行。
//!
//! 查询函数只读取记录并返回引用，不复制记录本身。这样排序结果仍然指向
//! 调用者拥有的 `Vec<Record>`，所有权关系保持清晰。

use std::fmt;

use crate::model::Record;

/// `--sort` 支持的排序主键。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortKey {
    /// 按 id 从小到大（默认）。
    #[default]
    Id,
    /// 按 value 从小到大。
    Value,
    /// 按 category 的字典序。
    Category,
}

/// 一组筛选与排序条件。所有筛选条件同时成立（逻辑 AND）才保留记录。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Query {
    /// 分类精确匹配；比较前会去掉查询值两端空白。
    pub category: Option<String>,
    /// 标签匹配；查询值会去掉空白并转小写，再与规范化后的标签比较。
    pub tag: Option<String>,
    /// value 的下界，包含边界。
    pub min: Option<i64>,
    /// value 的上界，包含边界。
    pub max: Option<i64>,
    /// note 的大小写敏感子串。
    pub contains: Option<String>,
    /// 排序主键。
    pub sort: SortKey,
    /// 是否反转主键顺序。主键相等时仍然按 id 从小到大。
    pub descending: bool,
    /// 排序后最多保留多少条；`Some(0)` 得到空结果。
    pub limit: Option<usize>,
}

/// 查询条件在语义上不成立时返回的错误。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueryError {
    /// `min` 大于 `max`。
    InvalidRange { min: i64, max: i64 },
}

impl fmt::Display for QueryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRange { min, max } => {
                write!(f, "min ({min}) must not be greater than max ({max})")
            }
        }
    }
}

impl std::error::Error for QueryError {}

/// 按 `query` 筛选、排序并截断记录，返回原切片中的引用。
///
/// 执行顺序固定为：先检查 `min <= max`，再依次应用全部筛选，再排序，
/// 最后应用 `limit`。因此 `limit` 永远作用在最终顺序上。
///
/// 排序规则：
/// - 主键分别是 id、value 或 category；
/// - 主键相等时一律按 id 从小到大，保证输出与输入顺序无关；
/// - `descending` 为真时只反转主键的比较方向，不反转 id 的次级顺序。
///
/// 筛选规则：
/// - `category`：去掉两端空白后与记录的 category 完全相等；
/// - `tag`：去掉两端空白并转小写后，记录 tags 中包含它；
/// - `min`/`max`：包含边界；
/// - `contains`：在 note 中查找大小写敏感的子串，空字符串匹配所有记录。
///
/// `min > max` 时返回 [`QueryError::InvalidRange`]。
pub fn execute<'a>(records: &'a [Record], query: &Query) -> Result<Vec<&'a Record>, QueryError> {
    let _ = (records, query);
    todo!()
}
