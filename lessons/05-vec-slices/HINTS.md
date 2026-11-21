# 第五课提示：主练习与课后巩固

提示分三级，卡到哪一级看哪一级：

- **一级：思考方向**——只提示该想什么，不涉及代码结构。
- **二级：局部分解**——把方法拆成几个判断步骤，仍不写完整代码。
- **三级：接近完整的结构**——给出带空缺的骨架，你需要自己补上关键调用。

先看一级；想清楚了就不要继续往下看。三级提示故意不给出可直接提交的完整代码。

主练习按 README 推荐的顺序完成方法：先 `new` 和 `capacity`，再 `values`、`push`（先只处理未满的情况）、`latest`，然后补 `push` 的两个分支、`extend`、`clear_negative`，最后 `split_recent`。

## 主练习：`new` 与 `capacity`（`src/exercise.rs`）

### 一级：思考方向

`Window` 有两个字段，`new` 的职责就是把它们都填上。想想：新窗口里应该放几个样本？哪个字段直接来自参数？创建空 `Vec` 的两种写法（`Vec::new` 和 `Vec::with_capacity`）在这里用哪个更合适——容量已知，能不能省掉之后的扩容？

### 二级：局部分解

1. 用 `Vec::with_capacity(capacity)` 创建缓冲区：容量已经知道，提前申请可以避免之后的扩容搬家。
2. 用结构体字面量 `Self { values: ..., capacity }` 把两个字段装好返回。参数名和字段名相同时，`capacity` 一个名字就是 `capacity: capacity` 的简写。
3. `capacity()` 方法只需返回自己的 `capacity` 字段。

### 三级：接近完整的结构

```rust
pub fn new(capacity: usize) -> Self {
    Self {
        values: Vec::/* 提前申请 capacity 个位置的方法 */(capacity),
        capacity,
    }
}

pub fn capacity(&self) -> usize {
    self./* 哪个字段记录了容量 */
}
```

## 主练习：`values`、`push` 与 `latest`

### 一级：思考方向

`values()` 要返回“整段数据的只读视图”——`Vec` 的哪个引用形式正好就是这个视图？`push` 有三种情况：容量为 0、未满、已满。最旧（最早进入）的样本在 Vec 的哪一端？`latest()` 要找“最后一个元素”，标准库里有没有直接叫这个名字的方法？它返回的元素是引用，而函数要返回 `Option<i32>`，差在哪一步？

### 二级：局部分解

1. `values()`：`&self.values` 的类型是 `&Vec<i32>`，它能自动转换成 `&[i32]`——直接返回即可，不需要复制。
2. `push(value)`：
   - 容量为 0：直接返回，什么都不做；
   - `values.len() == capacity`：删除位置 0（最旧的样本）再追加；
   - 其余情况：直接追加。
3. `latest()`：`self.values.last()` 返回 `Option<&i32>`（最后一个元素的引用）。`i32` 是 `Copy` 类型，用一个方法把引用里的值复制出来，把 `Option<&i32>` 变成 `Option<i32>`。

### 三级：接近完整的结构

```rust
pub fn values(&self) -> &[i32] {
    &self./* 保存样本的字段 */
}

pub fn push(&mut self, value: i32) {
    if self.capacity == 0 {
        return;
    }
    if self.values.len() == self.capacity {
        self.values./* 删除位置 0 的元素的方法 */(0);
    }
    self.values./* 追加到末尾的方法 */(value);
}

pub fn latest(&self) -> Option<i32> {
    self.values
        ./* 取最后一个元素引用的方法 */()
        ./* 把 Option<&i32> 里的值复制出来 */
}
```

回想 README 第六节：为什么不能直接写 `self.values.last()` 就返回？`Option<&i32>` 和 `Option<i32>` 的所有权差别是什么？

## 主练习：`extend`

### 一级：思考方向

注释说“效果等同于逐个调用 `push`”——这就是全部提示。参数 `impl IntoIterator<Item = i32>` 可以直接用 `for` 循环逐个取出。

### 二级：局部分解

1. `for value in values` 逐个取出样本。
2. 循环体里调用 `self.push(value)`，淘汰逻辑全部由 `push` 负责，这里不要重复写。

### 三级：接近完整的结构

```rust
pub fn extend(&mut self, values: impl IntoIterator<Item = i32>) {
    for value in values {
        self./* 复用哪个方法 */;
    }
}
```

## 主练习：`clear_negative`

### 一级：思考方向

这是“按条件批量删除、保持顺序”的需求。README 第九节讲过一个专门做这件事的方法，它接收一个闭包，闭包返回 `true` 就保留。想想闭包参数是什么类型（元素本身还是元素的引用？），判断条件应该怎么写。

### 二级：局部分解

1. 调用 `self.values.retain(...)`。
2. 闭包写成 `|value| ...`，其中 `value` 是 `&i32`（元素的引用，因为 retain 只是“看一眼”再决定，不能拿走元素）。
3. 保留条件是“不小于 0”：对 `&i32` 解引用后与 0 比较。

### 三级：接近完整的结构

```rust
pub fn clear_negative(&mut self) {
    self.values.retain(|value| /* value 是 &i32，什么条件表示“保留” */);
}
```

如果写“边递增索引边 `remove`”，会踩到 README 第九节末尾说的哪个坑？

## 主练习：`split_recent`

### 一级：思考方向

要在一个位置把 Vec 切成前后两段。标准库里有一个方法：给它一个索引，返回 `(&[T], &[T])` 两段切片——名字就暗示了“在某个位置切开”。难点只在算出这个分界索引：`count` 比样本总数还大时，直接用减法会算出“负数”，而 `usize` 没有负数，运行时会 panic。有没有一种“减到 0 为止”的减法？

### 二级：局部分解

1. 分界索引 = `样本总数 - count`，但结果不能小于 0：用 `saturating_sub`（饱和减法，减到 0 就停住）代替普通减法。
2. 对 `self.values` 调用 `split_at(分界索引)`，它恰好返回 `(&[i32], &[i32])`，前段是较旧部分，后段是最近部分，顺序与题目要求一致。
3. 验证三个边界：`count == 0` 时分界在哪？`count == len` 时呢？`count > len` 时 `saturating_sub` 给出什么？

### 三级：接近完整的结构

```rust
pub fn split_recent(&self, count: usize) -> (&[i32], &[i32]) {
    let split_index = self.values.len()./* 减到 0 为止的减法 */(count);
    self.values./* 在索引处切成两段切片的方法 */(split_index)
}
```

为什么这里不能用普通的 `-`？用 `count = 10`、长度 `= 4` 在脑子里走一遍。

## 课后巩固：`summarize`（`src/consolidation.rs`）

### 一级：思考方向

空输入返回 `None`，所以第一件事是判断空。非空时要收集四个数字：份数、最低分、最高分、总分。最低分和最高分可以从第一个分数开始，逐个比较更新；总分累加时要从 `u32` 转换成 `u64`，想想在哪一步转换最不容易溢出。

### 二级：局部分解

1. `if scores.is_empty() { return None; }`。
2. 初始化：`min` 和 `max` 都从 `scores[0]` 开始（非空时第一个位置一定存在），`total` 从 `0u64` 开始。
3. 遍历切片中的每个分数：
   - 比当前 `min` 小就更新 `min`，比当前 `max` 大就更新 `max`；
   - 用 `u64::from(score)` 或 `score as u64` 转成 `u64` 后累加进 `total`。
4. 返回 `Some(Summary { count: scores.len(), min, max, total })`。

### 三级：接近完整的结构

```rust
pub fn summarize(scores: &[u32]) -> Option<Summary> {
    if scores.is_empty() {
        return None;
    }

    let mut min = scores[0];
    let mut max = scores[0];
    let mut total: u64 = 0;

    for &score in scores {
        if score < min {
            /* 更新什么 */
        }
        if score > max {
            /* 更新什么 */
        }
        total += /* 把 u32 分数变成 u64 再累加 */;
    }

    Some(Summary {
        count: /* 份数从哪来 */,
        min,
        max,
        total,
    })
}
```

`for &score in scores` 里的 `&score` 是模式：它把“元素的引用”直接拆成“元素的值”，循环体里 `score` 就是 `u32`。

## 课后巩固：`top_scores`

### 一级：思考方向

排序会修改数据，而传入的是只读切片——所以第一步是复制出一个自己拥有的 `Vec`，之后随便怎么排。标准库 `Vec` 的 `sort()` 默认从小到大排，而题目要从大到小返回；有没有反向排序的方法，或者排完再反转的方法？最后只要前 `count` 个，`count` 超界时怎么处理才不会越界？

### 二级：局部分解

1. 用 `scores.to_vec()` 把切片复制成一个自己拥有的 `Vec<u32>`。
2. 排序：`sort()` 是升序；排完再用 `reverse()` 把顺序反过来，就是从高到低。
3. 截取前 `count` 个：用 `truncate(count)`——它只在 `count` 小于当前长度时才截断，否则原样保留，正好覆盖“count 超过总数返回全部”的边界。
4. 返回这个 Vec。

### 三级：接近完整的结构

```rust
pub fn top_scores(scores: &[u32], count: usize) -> Vec<u32> {
    let mut sorted = scores./* 把切片复制成 Vec 的方法 */();
    sorted./* 排序（升序）*/();
    sorted./* 把顺序反过来，变成从高到低 */();
    sorted./* 只保留前 count 个，count 超界时保持原样 */(count);
    sorted
}
```

为什么直接对 `scores` 排序不可能通过编译？切片和 Vec 的所有权差别在这里起了什么作用？

## 课后巩固：`apply_bonus`

### 一级：思考方向

要就地修改切片的每个元素，需要元素的**可变引用**——`for` 遍历 `&mut [u32]` 的哪种写法能拿到它？加分后要封顶在 100：标准库有没有“取两个值中较小者”的方法？

### 二级：局部分解

1. `for score in scores.iter_mut()`（或等价的 `for score in &mut *scores` / 直接 `for score in scores`，因为 `&mut [u32]` 本身可以迭代出可变引用）。
2. 循环体里 `score` 是 `&mut u32`。先加上 `bonus`：`*score += bonus;`。
3. 再封顶：`*score = (*score).min(100);`，或者用 `if *score > 100 { *score = 100; }`。

### 三级：接近完整的结构

```rust
pub fn apply_bonus(scores: &mut [u32], bonus: u32) {
    for score in scores.iter_mut() {
        *score += bonus;
        *score = /* score 指向的值和 100 取较小者 */;
    }
}
```

为什么这里用 `iter_mut()` 而不是 `iter()`？如果写错了会得到哪条借用错误？
