//! 主练习：用声明宏生成库存清单，并实现一个提前返回的检查宏。
//!
//! 请只替换两个宏展开模板中的 `todo!()`（以及占位结构），不要修改类型定义、
//! 函数签名和本文件底部的测试。每个宏上方的文档注释就是它的完整行为约定。

/// 一种商品以及它的库存数量。
///
/// `name` 是商品名，`quantity` 是当前库存件数；数量为 `0` 表示暂时缺货，
/// 仍然是合法数据。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StockItem {
    pub name: String,
    pub quantity: u32,
}

impl StockItem {
    /// 用商品名和数量创建一个条目。
    ///
    /// 参数 `name` 接受任何能转换成 `String` 的文本（例如 `&str` 或
    /// `String`），这样宏既可以直接收到字符串字面量，也可以收到计算出的
    /// 文本。
    pub fn new(name: impl Into<String>, quantity: u32) -> Self {
        Self {
            name: name.into(),
            quantity,
        }
    }
}

/// 从库存中取货时可能出现的失败。
#[derive(Debug, PartialEq, Eq)]
pub enum StockError {
    /// 库存不足。`available` 是当前数量，`requested` 是请求取走的数量。
    NotEnough { available: u32, requested: u32 },
}

/// 用一组商品构造库存清单。
///
/// 支持三种调用形式（一次调用只能使用其中一种，不能混用）：
///
/// 1. 空清单：不写任何内容，得到空的 `Vec<StockItem>`；
/// 2. 全部写明数量：每项写成 `商品名 => 数量`，项之间用逗号分隔；
/// 3. 全部使用默认数量：每项只写商品名，数量按 `1` 计算。
///
/// 具体行为：
///
/// - 返回的 `Vec<StockItem>` 顺序与调用中各项的书写顺序一致；
/// - `商品名` 和 `数量` 都可以是任意表达式，但每个表达式在展开后只求值
///   一次（有副作用时只会产生一次效果）；
/// - 最后一项之后允许有一个可选的尾逗号；
/// - 数量为 `0` 是合法输入；
/// - 商品名通过 [`StockItem::new`] 转成 `String`，因此可以传字符串字面量，
///   也可以传 `String` 或其他能转换的文本。
///
/// 这个宏最终要在**别的 crate** 里也能直接调用，因此展开模板不能假设调用处
/// 已经导入了任何名称。
#[macro_export]
macro_rules! stock {
    () => {{
        // 脚手架占位：返回类型已经写对，但一执行就会 panic。
        // 实现时请用真正的展开模板替换整个 `{{ ... }}`。
        fn placeholder() -> ::std::vec::Vec<$crate::exercise::StockItem> {
            todo!()
        }
        placeholder()
    }};
    ($($name:expr => $quantity:expr),+ $(,)?) => {{
        fn placeholder() -> ::std::vec::Vec<$crate::exercise::StockItem> {
            todo!()
        }
        placeholder()
    }};
    ($($name:expr),+ $(,)?) => {{
        fn placeholder() -> ::std::vec::Vec<$crate::exercise::StockItem> {
            todo!()
        }
        placeholder()
    }};
}

/// 条件不成立时，从当前函数提前返回 `Err(错误表达式)`。
///
/// 行为约定：
///
/// - `条件` 为 `true` 时什么也不做，函数继续执行后面的语句；
/// - `条件` 为 `false` 时，宏会立即让**调用它的函数**返回
///   `Err(错误表达式)`，后面的语句不再执行；
/// - `条件` 至多求值一次；
/// - `错误表达式` 只在条件为 `false` 时求值，因此成功路径不会构造昂贵的
///   错误值；
/// - 最后允许一个可选的尾逗号。
///
/// 这是控制流宏：阅读调用处时要记得它可能直接结束当前函数。
#[macro_export]
macro_rules! ensure {
    ($condition:expr, $error:expr $(,)?) => {
        // 脚手架占位：这里的条件判断和展开结果都不是正确实现。
        // 请用真正的展开模板替换整个宏体。
        if $condition {
            todo!()
        }
    };
}

/// 从 `item` 中取走 `requested` 件商品。
///
/// 成功时返回取走之后剩余的件数；请求数量超过现有库存时返回
/// [`StockError::NotEnough`]，其中同时带上现有数量和请求数量。
/// 请求 `0` 件是合法的，剩余数量不变；请求数量正好等于库存也合法，
/// 此时剩余 `0` 件。
pub fn take_stock(item: &StockItem, requested: u32) -> Result<u32, StockError> {
    ensure!(
        requested <= item.quantity,
        StockError::NotEnough {
            available: item.quantity,
            requested,
        }
    );
    Ok(item.quantity - requested)
}

#[cfg(test)]
mod tests {
    use super::{StockError, StockItem, take_stock};
    use std::cell::Cell;

    #[test]
    fn stock_builds_items_with_explicit_quantities() {
        let items = stock!["pen" => 12, "paper" => 3];
        assert_eq!(
            items,
            vec![StockItem::new("pen", 12), StockItem::new("paper", 3)]
        );
    }

    #[test]
    fn stock_defaults_quantity_to_one() {
        let items = stock!["pen", "ink", "ruler"];
        assert_eq!(
            items,
            vec![
                StockItem::new("pen", 1),
                StockItem::new("ink", 1),
                StockItem::new("ruler", 1),
            ]
        );
    }

    #[test]
    fn stock_accepts_trailing_commas() {
        let explicit = stock!["pen" => 12, "paper" => 3,];
        let defaults = stock!["pen", "paper",];
        assert_eq!(explicit.len(), 2);
        assert_eq!(defaults.len(), 2);
        assert_eq!(explicit[0], StockItem::new("pen", 12));
        assert_eq!(defaults[0], StockItem::new("pen", 1));
    }

    #[test]
    fn stock_accepts_empty_list() {
        let items: Vec<StockItem> = stock![];
        assert!(items.is_empty());
    }

    #[test]
    fn stock_keeps_zero_quantities() {
        let items = stock!["eraser" => 0];
        assert_eq!(items, vec![StockItem::new("eraser", 0)]);
    }

    #[test]
    fn stock_evaluates_each_expression_once() {
        let name_calls = Cell::new(0);
        let quantity_calls = Cell::new(0);
        let items = stock![
            {
                name_calls.set(name_calls.get() + 1);
                "pen"
            } => {
                quantity_calls.set(quantity_calls.get() + 1);
                5
            },
            {
                name_calls.set(name_calls.get() + 1);
                "paper"
            } => {
                quantity_calls.set(quantity_calls.get() + 1);
                2
            },
        ];

        assert_eq!(name_calls.get(), 2);
        assert_eq!(quantity_calls.get(), 2);
        assert_eq!(items[0], StockItem::new("pen", 5));
        assert_eq!(items[1], StockItem::new("paper", 2));
    }

    #[test]
    fn take_stock_returns_remaining_quantity() {
        let item = StockItem::new("pen", 10);
        assert_eq!(take_stock(&item, 3), Ok(7));
    }

    #[test]
    fn take_stock_allows_taking_everything() {
        let item = StockItem::new("pen", 10);
        assert_eq!(take_stock(&item, 10), Ok(0));
    }

    #[test]
    fn take_stock_allows_zero_request() {
        let item = StockItem::new("pen", 10);
        assert_eq!(take_stock(&item, 0), Ok(10));
    }

    #[test]
    fn take_stock_reports_shortage_with_both_numbers() {
        let item = StockItem::new("pen", 2);
        assert_eq!(
            take_stock(&item, 5),
            Err(StockError::NotEnough {
                available: 2,
                requested: 5
            })
        );
        assert_eq!(
            take_stock(&StockItem::new("ink", 0), 1),
            Err(StockError::NotEnough {
                available: 0,
                requested: 1
            })
        );
    }

    #[test]
    fn ensure_returns_early_with_the_error() {
        let calls = Cell::new(0);
        let run = || -> Result<(), StockError> {
            ensure!(false, {
                calls.set(calls.get() + 1);
                StockError::NotEnough {
                    available: 0,
                    requested: 1,
                }
            });
            Ok(())
        };

        assert_eq!(
            run(),
            Err(StockError::NotEnough {
                available: 0,
                requested: 1
            })
        );
        assert_eq!(calls.get(), 1);
    }

    #[test]
    fn ensure_does_not_build_the_error_on_success() {
        let calls = Cell::new(0);
        let run = || -> Result<(), StockError> {
            ensure!(true, {
                calls.set(calls.get() + 1);
                StockError::NotEnough {
                    available: 0,
                    requested: 1,
                }
            });
            Ok(())
        };

        assert_eq!(run(), Ok(()));
        assert_eq!(calls.get(), 0);
    }

    #[test]
    fn ensure_evaluates_the_condition_once_and_accepts_a_trailing_comma() {
        let calls = Cell::new(0);
        let run = || -> Result<(), StockError> {
            ensure!(
                {
                    calls.set(calls.get() + 1);
                    true
                },
                StockError::NotEnough {
                    available: 0,
                    requested: 1,
                },
            );
            Ok(())
        };

        assert_eq!(run(), Ok(()));
        assert_eq!(calls.get(), 1);
    }
}
