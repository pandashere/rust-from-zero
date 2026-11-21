//! 主练习：有界样本窗口。
//!
//! 传感器每秒产生一个整数样本，程序只需要保留最近的一小段。`Window`
//! 就是这样一个窗口：它有固定容量，放满之后每进入一个新样本，就丢弃
//! 最旧的一个。
//!
//! 请只替换各个方法体里的 `todo!()`，不要修改类型定义和方法签名，也
//! 不要改动本文件底部的测试。每个方法上方的注释就是它的完整行为约定。

/// 有固定容量上限的样本窗口，按从旧到新的顺序保存最近的样本。
// 字段在方法实现（todo!()）完成后才会被读取，先对这个类型单独放行。
#[allow(dead_code)]
#[derive(Debug)]
pub struct Window {
    values: Vec<i32>,
    capacity: usize,
}

impl Window {
    /// 创建一个容量为 `capacity` 的空窗口。
    ///
    /// 行为约定：
    ///
    /// - 新窗口里没有任何样本：`values()` 返回空切片，`latest()` 返回
    ///   `None`；
    /// - 窗口此后最多同时保存 `capacity` 个样本；`capacity` 为 `0` 表示
    ///   这个窗口丢弃一切输入。
    pub fn new(capacity: usize) -> Self {
        // 完成实现前避免“参数未使用”的提示，写好方法体后可删除。
        let _ = capacity;
        todo!()
    }

    /// 把一个新样本放进窗口。
    ///
    /// 行为约定：
    ///
    /// - 容量为 `0` 时忽略这次输入，窗口保持不变；
    /// - 窗口未满（当前样本数小于容量）时，新样本成为最新（最后）一个；
    /// - 窗口已满时，先丢弃最旧（最早进入）的样本，再放入新样本，其余
    ///   样本的先后顺序保持不变。
    pub fn push(&mut self, value: i32) {
        // 完成实现前避免“参数未使用”的提示，写好方法体后可删除。
        let _ = value;
        todo!()
    }

    /// 按从旧到新的顺序返回当前所有样本的只读视图。
    ///
    /// 返回的是借用：调用者只能读取，不能通过返回值增删样本，也不会
    /// 复制整批数据。
    pub fn values(&self) -> &[i32] {
        todo!()
    }

    /// 返回最新（最后进入）的样本。
    ///
    /// 窗口为空时返回 `None`；否则返回 `Some(最新样本)`。返回值是复制
    /// 出来的整数，不借用窗口内部数据，因此拿到结果之后可以继续修改
    /// 窗口。
    pub fn latest(&self) -> Option<i32> {
        todo!()
    }

    /// 删除窗口中所有小于 0 的样本。
    ///
    /// 行为约定：
    ///
    /// - 保留的样本先后顺序不变；
    /// - 删除后窗口可能变短，但容量不变，也不会补充新样本。
    pub fn clear_negative(&mut self) {
        todo!()
    }

    /// 按顺序把一批样本依次放入窗口，效果等同于逐个调用 `push`。
    ///
    /// 样本多于容量时，只有最后进入的那些样本留在窗口中，数量不超过
    /// 容量。
    ///
    /// 参数 `impl IntoIterator<Item = i32>` 读作“任何能逐个产出 `i32`
    /// 的东西”：数组、`Vec<i32>` 等都能直接传入。这个写法的细节在
    /// 第 11 课讲解，这里只需要用 `for` 循环逐个取出样本。
    pub fn extend(&mut self, values: impl IntoIterator<Item = i32>) {
        // 完成实现前避免“参数未使用”的提示，写好方法体后可删除。
        let _ = values;
        todo!()
    }

    /// 把当前样本切成“较旧部分”和“最近部分”两段只读视图。
    ///
    /// 返回 `(较旧部分, 最近部分)`：
    ///
    /// - 最近部分包含最多 `count` 个最新的样本，顺序保持从旧到新；
    /// - 较旧部分包含其余所有样本；
    /// - `count` 大于等于样本总数时，较旧部分为空，最近部分是全部
    ///   样本；
    /// - `count` 为 `0` 时，最近部分为空。
    pub fn split_recent(&self, count: usize) -> (&[i32], &[i32]) {
        // 完成实现前避免“参数未使用”的提示，写好方法体后可删除。
        let _ = count;
        todo!()
    }

    /// 返回窗口的容量，即最多能同时保存的样本数。
    pub fn capacity(&self) -> usize {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::Window;

    #[test]
    fn new_window_is_empty() {
        let window = Window::new(3);

        assert_eq!(window.capacity(), 3);
        assert_eq!(window.values(), []);
        assert_eq!(window.latest(), None);
    }

    #[test]
    fn zero_capacity_window_ignores_all_input() {
        let mut window = Window::new(0);

        window.push(1);
        window.extend([2, 3, 4]);

        assert_eq!(window.values(), []);
        assert_eq!(window.latest(), None);
    }

    #[test]
    fn push_fills_the_window_up_to_its_capacity() {
        let mut window = Window::new(3);

        window.push(10);
        window.push(20);
        window.push(30);

        assert_eq!(window.values(), [10, 20, 30]);
        assert_eq!(window.latest(), Some(30));
    }

    #[test]
    fn push_evicts_the_oldest_sample_first() {
        let mut window = Window::new(3);
        window.extend([10, 20, 30]);

        window.push(40);
        window.push(50);

        assert_eq!(window.values(), [30, 40, 50]);
        assert_eq!(window.latest(), Some(50));
    }

    #[test]
    fn extend_keeps_only_the_most_recent_samples() {
        let mut window = Window::new(3);

        window.extend([1, 2, 3, 4, 5]);

        assert_eq!(window.values(), [3, 4, 5]);
    }

    #[test]
    fn clear_negative_removes_only_negative_samples() {
        let mut window = Window::new(6);
        window.extend([-3, 2, -1, 0, 5, -8]);

        window.clear_negative();

        assert_eq!(window.values(), [2, 0, 5]);
    }

    #[test]
    fn clear_negative_keeps_the_capacity_unchanged() {
        let mut window = Window::new(4);
        window.extend([-1, -2]);

        window.clear_negative();
        window.extend([7, 8, 9, 10, 11]);

        assert_eq!(window.capacity(), 4);
        assert_eq!(window.values(), [8, 9, 10, 11]);
    }

    #[test]
    fn split_recent_partitions_old_and_recent_samples() {
        let mut window = Window::new(4);
        window.extend([10, 20, 30, 40]);

        assert_eq!(window.split_recent(0), (&[10, 20, 30, 40][..], &[][..]));
        assert_eq!(window.split_recent(2), (&[10, 20][..], &[30, 40][..]));
        assert_eq!(window.split_recent(4), (&[][..], &[10, 20, 30, 40][..]));
        assert_eq!(window.split_recent(10), (&[][..], &[10, 20, 30, 40][..]));
    }

    #[test]
    fn reading_the_window_does_not_consume_it() {
        let mut window = Window::new(3);
        window.extend([1, 2]);

        let snapshot = window.values();
        assert_eq!(snapshot, [1, 2]);

        window.push(3);
        assert_eq!(window.values(), [1, 2, 3]);
    }
}
