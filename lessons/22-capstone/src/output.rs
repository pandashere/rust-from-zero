//! 输出层：把记录和统计结果变成稳定文本。
//!
//! 所有函数都返回 `String`，不直接打印。调用者可以把结果写到标准输出、
//! 文件或测试用的 `Vec<u8>`，格式化逻辑因此可以单独验证。

use crate::model::Record;
use crate::stats::Summary;

/// 把查询结果渲染成表格，结尾带一个换行。
///
/// 规则：
/// - 第一行固定是表头 `ID`、`CATEGORY`、`VALUE`、`TAGS`、`NOTE`；
/// - 前四列按该列最宽的内容（包括表头）用空格补齐，列之间固定两个空格；
/// - 最后一列 `NOTE` 不补齐，避免行尾出现无意义空格；
/// - 没有标签时 `TAGS` 显示 `-`，标签之间用 `,` 连接；
/// - note 为空时显示 `-`；
/// - 空记录列表只输出表头行。
///
/// 宽度按字符数而不是字节数计算，因此中文不会被当成多个字节来补齐；
/// 但字符数不是终端显示宽度，宽字符仍可能让终端中的列看起来不齐。
pub fn render_records(records: &[&Record]) -> String {
    let _ = records;
    todo!()
}

/// 渲染全局统计，结尾带一个换行。
///
/// 空集合的 `min`、`max`、`mean` 显示为 `-`；平均值固定两位小数。
pub fn render_summary(summary: &Summary) -> String {
    format!(
        "count: {}\nsum: {}\nmin: {}\nmax: {}\nmean: {}\n",
        summary.count,
        summary.sum,
        optional_i64(summary.min),
        optional_i64(summary.max),
        optional_mean(summary.mean()),
    )
}

/// 渲染按分类分组的统计，结尾带一个换行。
///
/// 表头固定为 `CATEGORY`、`COUNT`、`SUM`、`MIN`、`MAX`、`MEAN`。
/// 前五列按该列最宽的内容补齐，列之间固定两个空格；`MEAN` 不补齐。
/// 空分组只输出表头行。
pub fn render_groups(groups: &[(String, Summary)]) -> String {
    let rows: Vec<[String; 6]> = groups
        .iter()
        .map(|(category, summary)| {
            [
                category.clone(),
                summary.count.to_string(),
                summary.sum.to_string(),
                optional_i64(summary.min),
                optional_i64(summary.max),
                optional_mean(summary.mean()),
            ]
        })
        .collect();

    let mut widths = [8_usize, 5, 3, 3, 3, 4];
    for row in &rows {
        for (index, width) in widths.iter_mut().enumerate() {
            *width = (*width).max(row[index].chars().count());
        }
    }

    let headers = ["CATEGORY", "COUNT", "SUM", "MIN", "MAX", "MEAN"];
    let mut output = String::new();
    for (index, header) in headers.iter().enumerate() {
        if index == headers.len() - 1 {
            output.push_str(header);
        } else {
            output.push_str(&pad(header, widths[index]));
            output.push_str("  ");
        }
    }
    output.push('\n');

    for row in &rows {
        for (index, cell) in row.iter().enumerate() {
            if index == row.len() - 1 {
                output.push_str(cell);
            } else {
                output.push_str(&pad(cell, widths[index]));
                output.push_str("  ");
            }
        }
        output.push('\n');
    }

    output
}

fn optional_i64(value: Option<i64>) -> String {
    value.map_or_else(|| "-".to_string(), |number| number.to_string())
}

fn optional_mean(value: Option<f64>) -> String {
    value.map_or_else(|| "-".to_string(), |mean| format!("{mean:.2}"))
}

fn pad(text: &str, width: usize) -> String {
    let mut padded = String::with_capacity(text.len() + width);
    padded.push_str(text);
    let spaces = width.saturating_sub(text.chars().count());
    padded.extend(std::iter::repeat_n(' ', spaces));
    padded
}
