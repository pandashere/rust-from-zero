//! 领域模型：一条记录是什么、怎样规范化，以及怎样和一行文本互相转换。
//!
//! 文本格式固定为五个字段，用竖线分隔：
//!
//! ```text
//! id|category|value|tag1;tag2|note
//! ```
//!
//! 例如：
//!
//! ```text
//! 1|book|89|rust;systems|The Rust Book
//! ```
//!
//! 选择这种格式是为了让本课不依赖外部库：一行就是一条记录，出错时可以直接
//! 指出行号。格式约束集中在这里执行，导入文件和存储文件共用同一套规则。

use std::error::Error;
use std::fmt;

/// 五个字段之间的分隔符。
pub const FIELD_SEPARATOR: char = '|';
/// `tags` 字段内部多个标签之间的分隔符。
pub const TAG_SEPARATOR: char = ';';
/// 一条记录必须包含的字段数。
pub const FIELD_COUNT: usize = 5;

/// 一条已经通过领域规则检查的记录。
///
/// 字段是公开的，方便测试和查询层读取；需要创建记录时应优先使用
/// [`Record::try_new`] 或 [`Record::parse_line`]，由它们统一执行规范化。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Record {
    /// 记录编号，在同一个 store 中唯一。
    pub id: u64,
    /// 分类。保存的是去掉两端空白后的文本，不能为空。
    pub category: String,
    /// 整数值，允许负数与 0。
    pub value: i64,
    /// 已经去重并排序的标签。
    pub tags: Vec<String>,
    /// 用户备注，原样保留（`|`、回车、换行和制表符除外）。
    pub note: String,
}

/// 记录不符合领域规则时返回的错误。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    /// 分类在去掉两端空白后为空。
    EmptyCategory,
    /// 某个字段含有会破坏单行格式或表格布局的字符。
    ForbiddenCharacter {
        /// 出问题的字段名，例如 `"note"`。
        field: &'static str,
        /// 不被允许的字符。
        character: char,
    },
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyCategory => write!(f, "category is empty after trimming"),
            Self::ForbiddenCharacter { field, character } => {
                write!(f, "{field} contains forbidden character {character:?}")
            }
        }
    }
}

impl Error for ValidationError {}

/// 一行文本无法变成 [`Record`] 时返回的错误。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseRecordError {
    /// 用 `|` 切分后的字段数量不对。
    FieldCount { found: usize },
    /// id 不是合法的 `u64`。
    InvalidId { text: String },
    /// value 不是合法的 `i64`。
    InvalidValue { text: String },
    /// 字段能读出，但不符合领域规则。
    Validation(ValidationError),
}

impl fmt::Display for ParseRecordError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldCount { found } => write!(
                f,
                "expected {FIELD_COUNT} fields separated by '|', found {found}"
            ),
            Self::InvalidId { text } => {
                write!(f, "invalid id {text:?}: expected a non-negative integer")
            }
            Self::InvalidValue { text } => {
                write!(
                    f,
                    "invalid value {text:?}: expected a 64-bit signed integer"
                )
            }
            Self::Validation(error) => write!(f, "{error}"),
        }
    }
}

impl Error for ParseRecordError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Validation(error) => Some(error),
            Self::FieldCount { .. } | Self::InvalidId { .. } | Self::InvalidValue { .. } => None,
        }
    }
}

impl From<ValidationError> for ParseRecordError {
    fn from(error: ValidationError) -> Self {
        Self::Validation(error)
    }
}

impl Record {
    /// 从五个已经分开的值创建记录，并执行全部规范化。
    ///
    /// - `category`：去掉两端空白；结果为空时返回
    ///   [`ValidationError::EmptyCategory`]；含有 `|`、回车 `\r`、换行 `\n`
    ///   或制表符 `\t` 时返回 [`ValidationError::ForbiddenCharacter`]，字段名
    ///   是 `"category"`。
    /// - `tags`：逐个去掉两端空白，转成小写，丢弃空标签；重复标签只保留
    ///   一个；最终按字典序从小到大排列。标签含有 `|`、`;`、回车 `\r`、
    ///   换行 `\n` 或制表符 `\t` 时返回
    ///   [`ValidationError::ForbiddenCharacter`]，字段名是 `"tag"`。
    /// - `note`：保留调用者给出的文本，不 trim；含有 `|`、回车 `\r`、
    ///   换行 `\n` 或制表符 `\t` 时返回
    ///   [`ValidationError::ForbiddenCharacter`]，字段名是 `"note"`。
    /// - `id` 与 `value`：直接保存，不做范围限制。
    ///
    /// # 示例
    ///
    /// ```text
    /// try_new(7, "  book ".into(), -3, vec![" Rust ".into(), "".into()], " hi ")
    /// 得到 category = "book"，tags = ["rust"]，note = " hi "。
    /// ```
    pub fn try_new(
        id: u64,
        category: String,
        value: i64,
        tags: Vec<String>,
        note: String,
    ) -> Result<Self, ValidationError> {
        let _ = (&id, &category, &value, &tags, &note);
        todo!()
    }

    /// 解析一行 `id|category|value|tags|note` 文本。
    ///
    /// - 必须恰好有 [`FIELD_COUNT`] 个字段，否则返回
    ///   [`ParseRecordError::FieldCount`]；`found` 是实际字段数。
    /// - 第一个字段必须是 `u64`，否则返回 [`ParseRecordError::InvalidId`]，
    ///   `text` 保存原始文本。
    /// - 第三个字段必须是 `i64`，否则返回
    ///   [`ParseRecordError::InvalidValue`]，`text` 保存原始文本。
    /// - 第四个字段按 `;` 拆成标签；空字符串表示没有标签。
    /// - 解析出的字段交给 [`Record::try_new`] 做规范化和领域验证，失败时
    ///   返回 [`ParseRecordError::Validation`]。
    ///
    /// 空行不是合法记录，会得到字段数为 1 的 `FieldCount`。
    pub fn parse_line(line: &str) -> Result<Self, ParseRecordError> {
        let _ = line;
        todo!()
    }

    /// 把记录写成一行文本，结尾不带换行符。
    ///
    /// 输出格式为 `id|category|value|tags|note`：标签之间用 `;` 连接，
    /// 没有标签时该位置为空字符串，例如 `1|book|3||note`。
    pub fn to_line(&self) -> String {
        let _ = self;
        todo!()
    }
}
