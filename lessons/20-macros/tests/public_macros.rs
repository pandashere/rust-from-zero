//! 从本 package 的外部（集成测试相当于另一个 crate）调用三个宏。
//!
//! 这些测试检查宏在展开时是否依赖调用处的 `use`：这里只写了 crate 名和宏名，
//! 没有把 `StockItem` 或其他辅助名称导入当前作用域。

use l20_macros::exercise::{StockError, StockItem};

#[test]
fn stock_works_outside_its_crate() {
    let explicit = l20_macros::stock!["pen" => 2, "paper" => 1];
    let defaults = l20_macros::stock!["ink", "ruler"];

    assert_eq!(
        explicit,
        vec![StockItem::new("pen", 2), StockItem::new("paper", 1)]
    );
    assert_eq!(
        defaults,
        vec![StockItem::new("ink", 1), StockItem::new("ruler", 1)]
    );
}

#[test]
fn ensure_works_outside_its_crate() {
    fn check(condition: bool) -> Result<(), StockError> {
        l20_macros::ensure!(
            condition,
            StockError::NotEnough {
                available: 0,
                requested: 1,
            }
        );
        Ok(())
    }

    assert_eq!(check(true), Ok(()));
    assert_eq!(
        check(false),
        Err(StockError::NotEnough {
            available: 0,
            requested: 1,
        })
    );
}

#[test]
fn matrix_works_outside_its_crate() {
    let grid = l20_macros::matrix![[1, 2], [3], []];

    assert_eq!(grid, vec![vec![1, 2], vec![3], vec![]]);
}
