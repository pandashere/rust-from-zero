//! 主练习：可求值、可遍历的算术表达式树。
//!
//! 一个表达式要么是数字，要么由更小的表达式组合而成。为了让“组合”这一
//! 关系有一个固定大小的落脚点，递归字段都放在 `Box` 后面。
//!
//! 请只替换方法体里的 `todo!()`，不要修改类型定义、方法签名或文件底部的
//! 测试。每个方法上方的注释就是它的完整行为约定。

/// 求值过程中可能出现的、可以恢复的错误。
#[derive(Debug, PartialEq, Eq)]
pub enum EvalError {
    /// 除法的右操作数等于 0。
    DivisionByZero,
    /// 结果超出 `i64` 能表示的范围。
    Overflow,
}

/// 一棵算术表达式树。
///
/// - `Literal` 是叶子，直接保存一个 64 位有符号整数（`i64`）；
/// - `Neg` 是一元节点，保存唯一的一棵子树；
/// - 其余四个变体是二元节点，依次保存左、右两棵子树。
///
/// 所有递归字段都是 `Box<Expr>`：节点里存的是指向堆上子树的指针，而不是
/// 另一棵完整的树。因此 `Expr` 自身的大小是固定的。
#[derive(Debug)]
pub enum Expr {
    Literal(i64),
    Neg(Box<Expr>),
    Add(Box<Expr>, Box<Expr>),
    Subtract(Box<Expr>, Box<Expr>),
    Multiply(Box<Expr>, Box<Expr>),
    Divide(Box<Expr>, Box<Expr>),
}

impl Expr {
    /// 对整棵树求值。
    ///
    /// 行为约定：
    ///
    /// - `Literal(value)` 返回 `Ok(value)`；
    /// - `Neg(inner)` 返回子树值的相反数；
    /// - `Add`、`Subtract`、`Multiply`、`Divide` 分别返回左子树值与右子树
    ///   值的和、差、积、商；
    /// - 整数除法向 0 截断：`-7 / 2` 的结果是 `-3`，不是 `-4`；
    /// - 先计算左子树，再计算右子树。任何一棵子树出错就立刻返回那个错误，
    ///   不再计算其余部分；左右都出错时报告左子树的错误；
    /// - 除数为 0 返回 `Err(EvalError::DivisionByZero)`；
    /// - 结果不能用 `i64` 表示时返回 `Err(EvalError::Overflow)`，包括加法、
    ///   减法、乘法溢出，对 `i64::MIN` 取相反数，以及 `i64::MIN / -1`。
    ///
    /// 本方法只借用树，调用后原表达式仍然完整可用。
    pub fn eval(&self) -> Result<i64, EvalError> {
        todo!()
    }

    /// 返回树的深度（从根到最远叶子经过的节点数）。
    ///
    /// 行为约定：
    ///
    /// - 单个 `Literal` 的深度是 1；
    /// - `Neg` 的深度是 `1 + 子树的深度`；
    /// - 二元节点的深度是 `1 + 左右子树中较大的深度`；
    /// - 左右子树深度相同时取哪一个都得到同一个结果。
    ///
    /// 本方法只借用树。
    pub fn depth(&self) -> usize {
        todo!()
    }

    /// 按从左到右的顺序收集所有字面量，返回一个新 `Vec`。
    ///
    /// 行为约定：
    ///
    /// - 对每个节点先访问左子树，再访问右子树；`Neg` 先进入它唯一的子树；
    /// - 返回值的长度等于树中 `Literal` 节点的个数；
    /// - 相同的数字出现几次就收集几次，不排序、不去重；
    /// - 只借用树，返回后原表达式仍可继续 `eval`、`depth` 或再次收集。
    ///
    /// 本方法不应克隆任何子树，只复制叶子里的小整数。
    pub fn literals(&self) -> Vec<i64> {
        todo!()
    }

    /// 消费整棵树，按与 [`Expr::literals`] 相同的顺序收集字面量。
    ///
    /// 行为约定：
    ///
    /// - 返回值与对同一棵树调用 `literals` 完全相同；
    /// - 调用时取得树的所有权，调用之后原变量不能再使用；
    /// - 逐个节点把 `i64` 移出，不克隆子树，也不为了借用而保留整棵树。
    pub fn into_literals(self) -> Vec<i64> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::{EvalError, Expr};
    use std::mem::size_of;

    /// 把 `Expr::Literal` 包成子树指针，减少测试里的嵌套括号。
    fn lit(value: i64) -> Box<Expr> {
        Box::new(Expr::Literal(value))
    }

    /// `(1 + 2) * 4 - (-3)`：深度 4，值 15，字面量 `[1, 2, 4, 3]`。
    fn sample_tree() -> Expr {
        Expr::Subtract(
            Box::new(Expr::Multiply(Box::new(Expr::Add(lit(1), lit(2))), lit(4))),
            Box::new(Expr::Neg(lit(3))),
        )
    }

    // ---- 概念测试：不调用待实现方法，脚手架阶段就应通过 ----

    #[test]
    fn box_has_a_fixed_size_however_deep_the_tree_is() {
        assert_eq!(size_of::<Box<Expr>>(), size_of::<usize>());
        assert!(size_of::<Expr>() > size_of::<Box<Expr>>());
    }

    // ---- eval：正常输入 ----

    #[test]
    fn literal_evaluates_to_itself() {
        assert_eq!(Expr::Literal(-7).eval(), Ok(-7));
    }

    #[test]
    fn each_negation_flips_the_sign() {
        assert_eq!(Expr::Neg(lit(5)).eval(), Ok(-5));
        assert_eq!(Expr::Neg(Box::new(Expr::Neg(lit(5)))).eval(), Ok(5));
    }

    #[test]
    fn sample_tree_evaluates_with_mixed_operators() {
        assert_eq!(sample_tree().eval(), Ok(15));
    }

    #[test]
    fn integer_division_truncates_toward_zero() {
        assert_eq!(Expr::Divide(lit(-7), lit(2)).eval(), Ok(-3));
    }

    // ---- eval：错误与边界 ----

    #[test]
    fn division_by_zero_is_reported() {
        assert_eq!(
            Expr::Divide(lit(7), lit(0)).eval(),
            Err(EvalError::DivisionByZero)
        );
    }

    #[test]
    fn addition_overflow_is_reported() {
        assert_eq!(
            Expr::Add(lit(i64::MAX), lit(1)).eval(),
            Err(EvalError::Overflow)
        );
    }

    #[test]
    fn subtraction_and_multiplication_overflow_are_reported() {
        assert_eq!(
            Expr::Subtract(lit(i64::MIN), lit(1)).eval(),
            Err(EvalError::Overflow)
        );
        assert_eq!(
            Expr::Multiply(lit(i64::MAX), lit(2)).eval(),
            Err(EvalError::Overflow)
        );
    }

    #[test]
    fn negating_the_smallest_integer_overflows() {
        assert_eq!(Expr::Neg(lit(i64::MIN)).eval(), Err(EvalError::Overflow));
    }

    #[test]
    fn dividing_the_smallest_integer_by_negative_one_overflows() {
        assert_eq!(
            Expr::Divide(lit(i64::MIN), lit(-1)).eval(),
            Err(EvalError::Overflow)
        );
    }

    #[test]
    fn an_error_in_the_left_subtree_is_reported_first() {
        let expression = Expr::Add(
            Box::new(Expr::Divide(lit(1), lit(0))),
            Box::new(Expr::Neg(lit(i64::MIN))),
        );
        assert_eq!(expression.eval(), Err(EvalError::DivisionByZero));
    }

    #[test]
    fn an_error_in_the_right_subtree_is_propagated() {
        let expression = Expr::Add(lit(1), Box::new(Expr::Divide(lit(1), lit(0))));
        assert_eq!(expression.eval(), Err(EvalError::DivisionByZero));
    }

    // ---- depth ----

    #[test]
    fn a_literal_has_depth_one() {
        assert_eq!(Expr::Literal(7).depth(), 1);
    }

    #[test]
    fn a_unary_node_adds_one_level() {
        assert_eq!(Expr::Neg(lit(1)).depth(), 2);
    }

    #[test]
    fn depth_follows_the_deeper_subtree() {
        let expression = Expr::Add(lit(1), Box::new(Expr::Neg(Box::new(Expr::Neg(lit(2))))));
        assert_eq!(expression.depth(), 4);
    }

    #[test]
    fn the_sample_tree_has_depth_four() {
        assert_eq!(sample_tree().depth(), 4);
    }

    #[test]
    fn every_binary_operator_adds_one_level() {
        assert_eq!(Expr::Subtract(lit(1), lit(2)).depth(), 2);
        assert_eq!(Expr::Multiply(lit(1), lit(2)).depth(), 2);
        assert_eq!(Expr::Divide(lit(1), lit(2)).depth(), 2);
    }

    // ---- literals：借用遍历 ----

    #[test]
    fn a_literal_collects_itself() {
        assert_eq!(Expr::Literal(9).literals(), vec![9]);
    }

    #[test]
    fn literals_are_collected_in_left_to_right_order() {
        assert_eq!(sample_tree().literals(), vec![1, 2, 4, 3]);
    }

    #[test]
    fn literals_follow_the_evaluation_order_of_nested_operands() {
        let expression = Expr::Subtract(
            Box::new(Expr::Divide(lit(10), Box::new(Expr::Add(lit(4), lit(1))))),
            Box::new(Expr::Neg(lit(2))),
        );
        assert_eq!(expression.literals(), vec![10, 4, 1, 2]);
    }

    #[test]
    fn borrowing_literals_leaves_the_tree_usable() {
        let expression = sample_tree();
        assert_eq!(expression.literals(), vec![1, 2, 4, 3]);
        assert_eq!(expression.eval(), Ok(15));
        assert_eq!(expression.depth(), 4);
        assert_eq!(expression.literals(), vec![1, 2, 4, 3]);
    }

    // ---- into_literals：消费遍历 ----

    #[test]
    fn into_literals_matches_the_borrowing_version() {
        assert_eq!(sample_tree().into_literals(), vec![1, 2, 4, 3]);
    }

    #[test]
    fn into_literals_works_for_a_single_literal() {
        assert_eq!(Expr::Literal(-5).into_literals(), vec![-5]);
    }
}
