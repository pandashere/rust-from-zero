//! 主练习：泛型双元素容器与 const generic 定长窗口。
//!
//! `Pair<T>` 练习最普通的泛型 struct、泛型方法和“只给部分类型提供的方法”。
//! `FixedWindow<T, N>` 把数组长度放进类型里，练习 const 泛型参数和空数组
//! 边界。
//!
//! 请只替换方法体里的 `todo!()`，不要修改类型定义和方法签名，也不要改动
//! 文件底部的测试。每个方法上方的注释就是它的完整行为约定。

/// 一个装有两个同类型值的容器，先放 `left`，再放 `right`。
///
/// `T` 是类型参数：`Pair<i32>` 的两个字段都是 `i32`，`Pair<String>` 的
/// 两个字段都是 `String`。同一个 `Pair` 的两个字段必须是同一种类型。
#[derive(Debug, PartialEq, Eq)]
pub struct Pair<T> {
    left: T,
    right: T,
}

impl<T> Pair<T> {
    /// 用两个值创建一个 `Pair`，第一个成为 `left`，第二个成为 `right`。
    ///
    /// 两个值会按传入顺序保存；函数不要求 `T` 能复制或比较。
    pub fn new(left: T, right: T) -> Self {
        // 完成实现前避免“参数未使用”的提示，写好方法体后可删除。
        let _ = (&left, &right);
        todo!()
    }

    /// 以借用元组 `(&left, &right)` 的形式查看两个字段。
    ///
    /// 返回的是两个只读引用，调用者不能通过它们修改容器，原 `Pair` 在
    /// 调用之后仍然可以继续使用。
    pub fn as_tuple(&self) -> (&T, &T) {
        todo!()
    }

    /// 用同一个转换函数依次处理两个字段，返回元素类型可能改变的 `Pair`。
    ///
    /// 行为约定：
    ///
    /// - 先处理 `left`，再处理 `right`，转换函数恰好被调用两次；
    /// - 转换函数可以修改自己捕获的外部状态，所以参数接收 `FnMut`
    ///   （可重复调用的闭包类型，第 9 课讲过）；
    /// - 函数消费 `self`，两个字段被移动进转换函数，不要求 `T: Clone`；
    /// - 返回值中的两个字段类型由转换函数的返回类型决定，可以和 `T`
    ///   不同。
    pub fn map<U>(self, mut operation: impl FnMut(T) -> U) -> Pair<U> {
        let _ = &mut operation;
        todo!()
    }

    /// 用两个各自独立的转换函数分别处理 `left` 和 `right`。
    ///
    /// 行为约定：
    ///
    /// - `left_operation` 只处理原来的 `left`，`right_operation` 只处理
    ///   原来的 `right`，两个函数各调用一次；
    /// - 两个参数接收 `FnOnce`（最多调用一次的闭包类型），因此它们都可以
    ///   消费自己捕获的拥有型数据；
    /// - 函数消费 `self`，两个字段被移动，不要求 `T: Clone`；
    /// - 两个返回值必须是同一种类型 `U`，结果的 `left` 来自
    ///   `left_operation`，`right` 来自 `right_operation`。
    pub fn map_each<U, F, G>(self, left_operation: F, right_operation: G) -> Pair<U>
    where
        F: FnOnce(T) -> U,
        G: FnOnce(T) -> U,
    {
        let _ = (&left_operation, &right_operation);
        todo!()
    }

    /// 交换左右两个字段，返回新的 `Pair`。
    ///
    /// 函数消费 `self`，两个值只是换到另一个字段，没有发生复制。因此这个
    /// 方法不需要 `T: Clone`，对“不能复制的类型”同样有效。
    pub fn swap(self) -> Pair<T> {
        todo!()
    }
}

impl<T: Ord> Pair<T> {
    /// 返回较大字段的只读引用；两个字段相等时返回 `right` 的引用。
    ///
    /// `Ord`（ordering，全序比较）是“类型支持 `<`、`>`、`max` 等比较”的
    /// 能力约定。因此整个 impl 块里的方法只对能比较大小的 `T` 存在。
    ///
    /// 函数只借用字段，不复制也不移动它们，原 `Pair` 在调用之后仍可继续
    /// 使用。
    pub fn max_ref(&self) -> &T {
        todo!()
    }
}

impl<T: Clone> Pair<T> {
    /// 返回一个新的 `Pair`：两个字段都是原 `left` 字段的独立副本。
    ///
    /// `Clone` 是“可以显式复制一份值”的能力约定。复制出来的是新值：
    /// 修改结果不会影响原 `Pair`。
    pub fn duplicated_left(&self) -> Pair<T> {
        todo!()
    }
}

impl Pair<String> {
    /// 用 `separator` 连接两个字符串，返回 `left + separator + right`。
    ///
    /// 这个方法只存在于 `Pair<String>`，`Pair<i32>` 没有它。`separator`
    /// 为空字符串时，返回两个字符串直接相连的结果。
    ///
    /// 函数只借用内容并生成新的 `String`，不修改原来的两个字符串。
    pub fn joined(&self, separator: &str) -> String {
        let _ = separator;
        todo!()
    }
}

/// 一个长度由类型参数决定的定长窗口，内部按原顺序保存 `N` 个值。
///
/// `N` 是 const 泛型参数（编译期常量参数）：`FixedWindow<i32, 3>` 和
/// `FixedWindow<i32, 5>` 是两个不同的类型，数组长度在编译期就确定。
#[derive(Debug, PartialEq, Eq)]
pub struct FixedWindow<T, const N: usize> {
    values: [T; N],
}

impl<T, const N: usize> FixedWindow<T, N> {
    /// 用一个长度恰好为 `N` 的数组创建窗口，保持数组原有顺序。
    ///
    /// 这个方法已经提供，不需要修改。
    pub fn new(values: [T; N]) -> Self {
        Self { values }
    }

    /// 返回第一个元素的只读引用；窗口长度为 0 时返回 `None`。
    ///
    /// 长度为 0 是正常输入，不是程序错误，所以用 `Option` 表示“没有
    /// 第一个元素”，调用者可以通过返回的 `None` 平静地处理它。
    pub fn first(&self) -> Option<&T> {
        todo!()
    }

    /// 返回最后一个元素的只读引用；窗口长度为 0 时返回 `None`。
    ///
    /// 窗口长度为 1 时，第一个和最后一个元素是同一个。
    pub fn last(&self) -> Option<&T> {
        todo!()
    }

    /// 把所有元素按原顺序作为只读切片返回。
    ///
    /// 返回的是借用，不复制数组；空窗口返回空切片。
    pub fn as_slice(&self) -> &[T] {
        todo!()
    }

    /// 返回位置 `index` 上的元素；位置超出 `0..N` 时返回 `None`。
    ///
    /// 位置从 0 开始。`index` 等于 `N`、大于 `N` 或取 `usize` 的最大值，
    /// 都应该得到 `None`，而不是让程序中止。`N` 为 0 时任何位置都
    /// 越界，因此总是返回 `None`。
    pub fn get(&self, index: usize) -> Option<&T> {
        let _ = &index;
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::{FixedWindow, Pair};

    /// 一个故意不实现 `Clone` 的类型，用来证明某些方法只靠移动就能工作。
    #[derive(Debug, PartialEq, Eq)]
    struct Token(u64);

    // ---- 概念测试：不调用待实现方法，脚手架阶段就应通过 ----

    #[test]
    fn pair_fields_support_equality_directly() {
        assert_eq!(Pair { left: 1, right: 2 }, Pair { left: 1, right: 2 });
        assert_ne!(Pair { left: 1, right: 2 }, Pair { left: 2, right: 1 });
    }

    #[test]
    fn fixed_window_size_includes_the_array_inline() {
        assert_eq!(std::mem::size_of::<FixedWindow<u8, 16>>(), 16);
        assert_eq!(std::mem::size_of::<FixedWindow<u8, 32>>(), 32);
    }

    // ---- new / as_tuple ----

    #[test]
    fn new_and_as_tuple_expose_both_fields() {
        let pair = Pair::new(3, 9);

        assert_eq!(pair.as_tuple(), (&3, &9));
    }

    #[test]
    fn as_tuple_only_borrows_so_the_pair_stays_usable() {
        let pair = Pair::new(String::from("a"), String::from("b"));

        let (left, right) = pair.as_tuple();
        assert_eq!(left, "a");
        assert_eq!(right, "b");
        assert_eq!(pair.as_tuple(), (&String::from("a"), &String::from("b")));
    }

    // ---- swap ----

    #[test]
    fn swap_exchanges_the_fields() {
        assert_eq!(Pair::new(1, 2).swap(), Pair { left: 2, right: 1 });
    }

    #[test]
    fn swap_moves_values_that_cannot_be_cloned() {
        let swapped = Pair::new(Token(1), Token(2)).swap();

        assert_eq!(
            swapped,
            Pair {
                left: Token(2),
                right: Token(1),
            }
        );
    }

    // ---- map ----

    #[test]
    fn map_applies_the_same_fnmut_to_both_fields() {
        let mut seen = Vec::new();

        let result = Pair::new(1, 2).map(|value| {
            seen.push(value);
            value * 10
        });

        assert_eq!(
            result,
            Pair {
                left: 10,
                right: 20
            }
        );
        assert_eq!(seen, [1, 2]);
    }

    #[test]
    fn map_can_change_the_element_type() {
        let result = Pair::new(1, 2).map(|value| format!("v{value}"));

        assert_eq!(
            result,
            Pair {
                left: String::from("v1"),
                right: String::from("v2"),
            }
        );
    }

    #[test]
    fn map_moves_values_that_cannot_be_cloned() {
        let result = Pair::new(Token(1), Token(2)).map(|token| token.0);

        assert_eq!(result, Pair { left: 1, right: 2 });
    }

    // ---- map_each ----

    #[test]
    fn map_each_uses_a_separate_closure_for_each_field() {
        let result = Pair::new(1, 2).map_each(|value| value + 100, |value| value + 200);

        assert_eq!(
            result,
            Pair {
                left: 101,
                right: 202
            }
        );
    }

    #[test]
    fn map_each_allows_each_fnonce_to_consume_its_own_capture() {
        let left_prefix = String::from("L");
        let right_prefix = String::from("R");

        let result = Pair::new(1, 2).map_each(
            move |value| left_prefix + &value.to_string(),
            move |value| right_prefix + &value.to_string(),
        );

        assert_eq!(
            result,
            Pair {
                left: String::from("L1"),
                right: String::from("R2"),
            }
        );
    }

    // ---- 不同粒度的 bound ----

    #[test]
    fn max_ref_borrows_the_larger_field() {
        let pair = Pair::new(3, 9);

        assert_eq!(pair.max_ref(), &9);
        assert_eq!(pair.as_tuple(), (&3, &9));
    }

    #[test]
    fn max_ref_returns_the_right_field_when_values_are_equal() {
        let pair = Pair::new(7, 7);
        let (left, right) = pair.as_tuple();

        assert!(std::ptr::eq(pair.max_ref(), right));
        assert!(!std::ptr::eq(pair.max_ref(), left));
    }

    #[test]
    fn duplicated_left_copies_the_left_field_into_both_fields() {
        let pair = Pair::new(String::from("a"), String::from("b"));

        assert_eq!(
            pair.duplicated_left(),
            Pair {
                left: String::from("a"),
                right: String::from("a"),
            }
        );
    }

    #[test]
    fn duplicated_left_does_not_change_the_original() {
        let pair = Pair::new(String::from("a"), String::from("b"));
        let mut duplicated = pair.duplicated_left();

        duplicated.right.push('!');

        assert_eq!(duplicated.left, "a");
        assert_eq!(duplicated.right, "a!");
        assert_eq!(pair.as_tuple(), (&String::from("a"), &String::from("b")));
    }

    #[test]
    fn joined_inserts_the_separator_between_the_strings() {
        let pair = Pair::new(String::from("ab"), String::from("cd"));

        assert_eq!(pair.joined("-"), "ab-cd");
    }

    #[test]
    fn joined_with_an_empty_separator_concatenates() {
        let pair = Pair::new(String::from("ab"), String::from("cd"));

        assert_eq!(pair.joined(""), "abcd");
    }

    // ---- FixedWindow ----

    #[test]
    fn fixed_window_exposes_first_last_and_slice() {
        let window = FixedWindow::new([10, 20, 30]);

        assert_eq!(window.first(), Some(&10));
        assert_eq!(window.last(), Some(&30));
        assert_eq!(window.as_slice(), &[10, 20, 30]);
    }

    #[test]
    fn fixed_window_of_one_has_the_same_first_and_last() {
        let window = FixedWindow::new([7]);

        assert_eq!(window.first(), Some(&7));
        assert_eq!(window.last(), Some(&7));
    }

    #[test]
    fn fixed_window_get_returns_none_outside_its_range() {
        let window = FixedWindow::new([10, 20, 30]);

        assert_eq!(window.get(0), Some(&10));
        assert_eq!(window.get(2), Some(&30));
        assert_eq!(window.get(3), None);
        assert_eq!(window.get(usize::MAX), None);
    }

    #[test]
    fn fixed_window_of_zero_is_empty_and_never_panics() {
        let window = FixedWindow::<i32, 0>::new([]);

        assert_eq!(window.first(), None);
        assert_eq!(window.last(), None);
        assert_eq!(window.as_slice(), &[]);
        assert_eq!(window.get(0), None);
    }
}
