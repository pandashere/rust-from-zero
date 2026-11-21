# 第九课提示：主练习与课后巩固

提示分三级，卡到哪一级看哪一级：

- **一级：思考方向**——只提示该想什么，不涉及代码结构。
- **二级：局部分解**——把任务拆成几个步骤，仍不写完整代码。
- **三级：接近完整的结构**——给出带空缺的骨架，你需要自己补上关键调用。

先看一级；想清楚了就不要继续往下看。三级提示故意不给出可直接提交的完整代码。所有 `todo!()` 都在 `src/` 里，测试不要改。

主练习建议按 README 的顺序完成：`run_once`、`first_match`、`make_threshold_filter`、`apply_all`，最后 `process_events`。课后巩固的三个函数可以按 `make_counter`、`make_labeler`、`make_outbox_summary` 的顺序做。

## 主练习：`run_once`（`src/exercise.rs`）

### 一级：思考方向

参数是 `FnOnce() -> String`，意思是“最多调用一次，调用后返回一个 `String`”。函数自己只需要调用它一次并把结果交回去。不要提前 `clone` 或想办法保留闭包，因为调用方已经允许它被消耗。

### 二级：局部分解

1. 直接调用参数。
2. 把调用产生的 `String` 作为函数返回值。
3. 不需要 `mut`，因为 `FnOnce` 的调用会拿走闭包值本身，而不是借它。

### 三级：接近完整的结构

```rust
pub fn run_once(operation: impl FnOnce() -> String) -> String {
    /* 调用一次并返回结果 */
}
```

想一想：如果这里写两次 `operation()`，为什么第二次会编译失败？

## 主练习：`first_match`（`src/exercise.rs`）

### 一级：思考方向

需要同时知道“第几个位置”和“那个位置上的值”。从下标 0 开始，逐个检查；一旦某个值让 `predicate` 返回 `true`，立刻返回这两个信息。检查完还没有，就返回 `None`。注意提前返回，不要把所有元素都检查完再决定。

### 二级：局部分解

1. 用一个 `usize` 变量记录当前下标，从 0 开始。
2. 用 `while` 循环，条件是下标小于 `values.len()`。
3. 用 `values[index]` 取出当前值（`i32` 是 `Copy`，取出来不会影响切片）。
4. 调用 `predicate(当前值)`；为 `true` 时返回 `Some((下标, 当前值))`。
5. 每轮结束把下标加 1；循环正常结束后返回 `None`。

> 本课还没讲迭代器。用 `while` 循环维护下标最直接；如果写成 `for index in 0..values.len()` 再按下标取值，Clippy 可能建议改为迭代器。下一课会讲 `values.iter().enumerate()`。

### 三级：接近完整的结构

```rust
pub fn first_match(values: &[i32], predicate: impl Fn(i32) -> bool) -> Option<(usize, i32)> {
    let mut index = 0;
    while index < values.len() {
        let value = /* 取出当前元素 */;
        if /* 条件 */ {
            return Some((index, value));
        }
        index += 1;
    }
    None
}
```

检查两个边界：`values` 为空时循环一次也不执行，返回什么？第一个元素就匹配时，循环体执行几次？

## 主练习：`make_threshold_filter`（`src/exercise.rs`）

### 一级：思考方向

函数要返回一个闭包，而闭包需要记住参数 `threshold`。`threshold` 是函数的局部变量，函数返回后它就不存在了，所以闭包不能只借它。用什么关键字让闭包把值带进自己的字段？比较时注意题目要求“大于或等于”。

### 二级：局部分解

1. 返回一个接收 `&Event` 的闭包。
2. 在闭包前加 `move`，把 `threshold` 按值捕获（`i32` 是 `Copy`，会复制一份）。
3. 闭包体访问 `event.value`，与 `threshold` 用“大于或等于”比较，返回 `bool`。

### 三级：接近完整的结构

```rust
pub fn make_threshold_filter(threshold: i32) -> impl Fn(&Event) -> bool {
    move |event| event.value /* 与 threshold 比较的运算符 */ threshold
}
```

如果去掉 `move`，编译器会报哪一类错误？回想 README 第七节“为什么返回闭包通常需要 move”。

## 主练习：`apply_all`（`src/exercise.rs`）

### 一级：思考方向

需要就地修改切片里的每个 `i32`。`for` 循环遍历 `&mut [i32]` 时，循环变量是什么类型？怎样通过它改到切片里的原值？`operation` 是 `FnMut`，每次调用都允许它修改自己的捕获状态。

### 二级：局部分解

1. 用 `for value in values` 遍历可变切片；此时 `value` 是 `&mut i32`。
2. 用 `*value` 读出旧值，传给 `operation`。
3. 把 `operation` 的返回值写回 `*value`。
4. 循环自然结束，不需要额外返回；空切片时循环体一次也不执行。

### 三级：接近完整的结构

```rust
pub fn apply_all(values: &mut [i32], mut operation: impl FnMut(i32) -> i32) {
    for value in values {
        *value = operation(/* 旧值 */);
    }
}
```

参数绑定上的 `mut` 是给谁用的？如果删掉它，调用 `operation` 会报什么错？

## 主练习：`process_events`（`src/exercise.rs`）

### 一级：思考方向

准备一个结果 `Vec<String>`。按顺序遍历事件：先问过滤器“要不要处理”，只有返回 `true` 才调用处理器。处理器返回 `Ok` 就把文本追加进结果；返回 `Err` 就立刻结束整个函数。`?` 正好能表达“成功取值，失败提前返回”。注意 `events` 为空时两个闭包都不应该被调用。

### 二级：局部分解

1. 创建 `let mut results = Vec::new();`。
2. `for event in events` 按顺序遍历；`event` 是 `&Event`。
3. 先调用 `filter(event)`；为 `false` 时用 `continue` 或 `if` 跳过本轮，绝不能调用 `handler`。
4. 为 `true` 时调用 `handler(event)`，得到 `Result<String, E>`。
5. 用 `?` 取出 `Ok` 里的文本并 `push` 到 `results`；遇到 `Err` 时 `?` 会自动返回错误。
6. 循环结束后返回 `Ok(results)`；空输入时 `results` 保持为空。

### 三级：接近完整的结构

```rust
pub fn process_events<E>(
    events: &[Event],
    mut filter: impl FnMut(&Event) -> bool,
    mut handler: impl FnMut(&Event) -> Result<String, E>,
) -> Result<Vec<String>, E> {
    let mut results = Vec::new();

    for event in events {
        if /* 过滤器是否通过 */ {
            results.push(/* 调用处理器并传播错误 */);
        }
    }

    Ok(results)
}
```

如果先调用 `handler` 再调用 `filter`，哪个测试会失败？如果忘了 `?`，错误类型会变成什么？

## 课后巩固：`make_counter`（`src/consolidation.rs`）

### 一级：思考方向

编号器必须在两次调用之间记住“下一个编号”。这个状态不能放在函数外面让所有编号器共享，而要成为返回闭包自己的字段。每次调用先交还当前值，再把内部状态推进一格；推进时要防止 `u32` 溢出。

### 二级：局部分解

1. 在函数里创建 `let mut current = start;`。
2. 返回一个 `move` 闭包，把 `current` 按值捕获进闭包。
3. 闭包每次调用：先把 `current` 复制到 `value`，作为本次返回值。
4. 用“饱和加法”把 `current` 加 1：到达 `u32::MAX` 后保持最大值，不再溢出。
5. 返回 `value`。

### 三级：接近完整的结构

```rust
pub fn make_counter(start: u32) -> impl FnMut() -> u32 {
    let mut current = start;

    move || {
        let value = current;
        current = current./* 加 1 但到最大值就停住的方法 */(1);
        value
    }
}
```

如果先加一再返回，第一次调用会得到什么？和题目要求的“第一次返回 `start`”差在哪？

## 课后巩固：`make_labeler`（`src/consolidation.rs`）

### 一级：思考方向

返回的闭包要在多次调用中使用同一个前缀。`prefix` 是 `String`，不是 `Copy`；如果只借用它，函数返回后前缀就失效了。让闭包拥有它。文本格式是“前缀、冒号、空格、原文”，按这个顺序拼接。

### 二级：局部分解

1. 返回一个接收 `&str`、返回 `String` 的闭包。
2. 在闭包前加 `move`，把 `prefix` 移动进闭包。
3. 闭包体用 `format!` 把 `prefix`、冒号、空格和 `text` 拼成一个新的 `String`。
4. 不要修改 `prefix`，也不要把它移出闭包，这样每次调用都能继续使用。

### 三级：接近完整的结构

```rust
pub fn make_labeler(prefix: String) -> impl Fn(&str) -> String {
    move |text| /* 用 prefix、": " 和 text 拼出新 String */
}
```

第二次调用时，`prefix` 还在吗？为什么这个闭包实现 `Fn` 而不是 `FnOnce`？

## 课后巩固：`make_outbox_summary`（`src/consolidation.rs`）

### 一级：思考方向

闭包要接管整批消息，并在被调用时把它们连接成一个字符串。`Vec<String>` 不是 `Copy`，函数返回后原变量就没了，所以闭包必须拥有它。连接时保留原顺序，用题目指定的分隔符；空批次要得到空字符串。`Vec<String>` 有一个现成的连接方法，分隔符作为参数传入。

### 二级：局部分解

1. 返回一个无参数、返回 `String` 的闭包。
2. 在闭包前加 `move`，把 `messages` 移动进闭包。
3. 在闭包体里调用 `messages` 的连接方法，分隔符是 `" | "`。
4. 连接方法会保留每个元素的内容和顺序；空 `Vec` 连接后自然是空 `String`。
5. 返回类型写 `impl FnOnce`，表示调用方只应调用一次。

### 三级：接近完整的结构

```rust
pub fn make_outbox_summary(messages: Vec<String>) -> impl FnOnce() -> String {
    move || messages./* 用 " | " 连接所有消息的方法 */(" | ")
}
```

为什么这里可以用 `FnOnce`，而 `make_labeler` 要返回 `Fn`？如果调用方拿到汇总闭包后调用两次，编译器会怎样？

## 卡住时

1. 先回到 README 对应小节，确认行为约定，不要从测试字面量反推实现。
2. 用 `cargo test -p l09_closures --no-run` 确认只是 `todo!()` 没实现，而不是签名或测试被改坏。
3. 用 `cargo test -p l09_closures <测试名>` 只跑一个测试，读第一条失败信息。
4. 仍然卡住时，把你的思路和报错一起带给 AI 助教；助教会按这三级的顺序提示，而不是直接给完整答案。
