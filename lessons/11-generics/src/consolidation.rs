//! 课后巩固：固定容量的后进先出栈。
//!
//! 这是主练习之后的独立任务。数据和主练习不同（这里保存一组会变化的
//! 值，而不是两个固定字段或一个只读窗口），边界也不同（容量为 0、栈已
//! 满、栈为空），正好练习 const 泛型参数、泛型 impl 和“失败时把值还给
//! 调用者”的错误处理。
//!
//! 请只替换方法体里的 `todo!()`，不要修改类型定义和方法签名，也不要改动
//! 文件底部的测试。每个方法上方的注释就是它的完整行为约定。
//!
//! 完成后的验收命令：
//!
//! ```bash
//! cargo test -p l11_generics consolidation
//! ```

/// 一个最多保存 `N` 个值的栈，后放入的值先被取出。
///
/// `N` 是 const 泛型参数：`Stack<String, 2>` 最多保存两个 `String`，
/// `Stack<i32, 0>` 一个也保存不了。内部用 `Vec<T>` 存放已经进入栈的
/// 值，但对外容量上限由 `N` 决定。
#[derive(Debug)]
pub struct Stack<T, const N: usize> {
    values: Vec<T>,
}

impl<T, const N: usize> Stack<T, N> {
    /// 创建一个空栈，容量上限为 `N`。
    ///
    /// 这个方法已经提供，不需要修改。
    pub fn new() -> Self {
        Self {
            values: Vec::with_capacity(N),
        }
    }

    /// 返回容量上限 `N`。
    ///
    /// 这个方法已经提供，不需要修改。
    pub fn capacity(&self) -> usize {
        N
    }

    /// 返回当前保存的值的个数。
    ///
    /// 这个方法已经提供，不需要修改。
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// 当前没有保存任何值时返回 `true`。
    ///
    /// 这个方法已经提供，不需要修改。
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// 把一个值放到栈顶。
    ///
    /// 行为约定：
    ///
    /// - 栈未满时保存该值并返回 `Ok(())`，它成为新的栈顶；
    /// - 栈已满（当前个数等于 `N`）时**不保存**该值，把值原样放进
    ///   `Err` 返回给调用者，栈里原有的内容保持不变；
    /// - `N` 为 0 时任何一次放入都会失败，返回 `Err(value)`；
    /// - 函数接收值的所有权；成功时值归栈所有，失败时所有权回到调用者。
    pub fn push(&mut self, value: T) -> Result<(), T> {
        let _ = &value;
        todo!()
    }

    /// 取出并返回栈顶的值；栈为空时返回 `None`。
    ///
    /// 成功时值被移出栈，栈的个数减少 1。返回值拥有这个值，调用者可以
    /// 继续使用它。
    pub fn pop(&mut self) -> Option<T> {
        todo!()
    }

    /// 查看栈顶的值但不取出；栈为空时返回 `None`。
    ///
    /// 返回的是只读引用。调用之后栈的内容和个数都不变。
    pub fn peek(&self) -> Option<&T> {
        todo!()
    }
}

impl<T, const N: usize> Default for Stack<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::Stack;

    #[test]
    fn new_stack_is_empty_and_has_its_declared_capacity() {
        let stack = Stack::<i32, 3>::new();

        assert_eq!(stack.capacity(), 3);
        assert_eq!(stack.len(), 0);
        assert!(stack.is_empty());
    }

    #[test]
    fn push_and_pop_follow_last_in_first_out() {
        let mut stack = Stack::<i32, 3>::new();

        assert_eq!(stack.push(10), Ok(()));
        assert_eq!(stack.push(20), Ok(()));
        assert_eq!(stack.push(30), Ok(()));

        assert_eq!(stack.pop(), Some(30));
        assert_eq!(stack.pop(), Some(20));
        assert_eq!(stack.pop(), Some(10));
        assert_eq!(stack.pop(), None);
    }

    #[test]
    fn peek_shows_the_top_without_removing_it() {
        let mut stack = Stack::<i32, 2>::new();
        stack.push(10).unwrap();
        stack.push(20).unwrap();

        assert_eq!(stack.peek(), Some(&20));
        assert_eq!(stack.len(), 2);
        assert_eq!(stack.pop(), Some(20));
        assert_eq!(stack.peek(), Some(&10));
    }

    #[test]
    fn push_returns_the_value_when_the_stack_is_full() {
        let mut stack = Stack::<i32, 2>::new();
        stack.push(1).unwrap();
        stack.push(2).unwrap();

        assert_eq!(stack.push(3), Err(3));
        assert_eq!(stack.len(), 2);
    }

    #[test]
    fn full_stack_keeps_its_earlier_values() {
        let mut stack = Stack::<String, 2>::new();
        stack.push(String::from("first")).unwrap();
        stack.push(String::from("second")).unwrap();

        let rejected = stack.push(String::from("third"));

        assert_eq!(rejected, Err(String::from("third")));
        assert_eq!(stack.pop(), Some(String::from("second")));
        assert_eq!(stack.pop(), Some(String::from("first")));
    }

    #[test]
    fn zero_capacity_stack_rejects_every_push() {
        let mut stack = Stack::<i32, 0>::new();

        assert_eq!(stack.push(7), Err(7));
        assert_eq!(stack.len(), 0);
        assert_eq!(stack.peek(), None);
        assert_eq!(stack.pop(), None);
    }

    #[test]
    fn empty_stack_returns_none_from_pop_and_peek() {
        let mut stack = Stack::<String, 2>::new();

        assert_eq!(stack.peek(), None);
        assert_eq!(stack.pop(), None);
        assert!(stack.is_empty());
    }

    #[test]
    fn stack_can_hold_values_that_cannot_be_copied() {
        let mut stack = Stack::<String, 2>::new();

        stack.push(String::from("hello")).unwrap();

        let taken = stack.pop();
        assert_eq!(taken, Some(String::from("hello")));
        assert_eq!(stack.pop(), None);
    }
}
