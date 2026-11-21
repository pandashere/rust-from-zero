# 第十课提示：主练习与课后巩固

提示分三级，卡到哪一级看哪一级：

- **一级：思考方向**——只提示该想什么，不给代码结构。
- **二级：局部分解**——把任务拆成几个步骤，说明每步要处理的数据，但仍不写完整代码。
- **三级：接近完整的结构**——给出带空缺的骨架，关键调用需要你自己补上。

先看一级；想清楚了就不要再往下看。三级提示故意留下空缺，不能直接复制提交。

## 主练习：`parse_line`（`src/exercise.rs`）

### 一级：思考方向

一行文本要拆成“级别”和“正文”两段。哪个标准库字符串方法能按**第一个**
分隔符切开，并同时告诉你“找到了”或“没找到”？没找到时，错误里要放什么？

级别文本要不要先去空白？题目要求“完全相同”，这意味着 `" INFO"` 算不算
合法？正文去掉两端空白后，怎么判断它是不是空的？错误里的 `text` 和成功时
的 `message`，一个要保留原文，一个要保存加工后的文本，分别该怎么得到拥有
所有权的 `String`？

### 二级：局部分解

1. 用按第一个 `:` 切分的字符串方法，得到 `(级别文本, 正文文本)`；它返回
   `Option`，用 `ok_or` 或 `ok_or_else` 把 `None` 变成
   `ParseLogError { line, text: content 的拥有副本 }`，再用 `?` 取出两段。
2. 用 `match` 把级别文本映射成 `Level`。三个合法文本之外的分支要返回
   `Err`，错误里的 `text` 仍是整行原文，不是级别文本。
3. 对正文调用 `trim()`，检查结果是否为空；为空就返回 `Err`，同样保留整行
   原文。
4. 成功时构造 `LogEntry`：`line` 原样放入，`level` 用第 2 步的结果，
   `message` 用 `trim()` 之后的文本转成 `String`。

### 三级：接近完整的结构

```rust
pub fn parse_line(line: usize, content: &str) -> Result<LogEntry, ParseLogError> {
    let (level_text, message_text) = content
        .split_once(':')
        ./* None 变成带 line 和 content 副本的错误 */(...)?;

    let level = match level_text {
        "INFO" => Level::Info,
        "WARN" => Level::Warn,
        "ERROR" => Level::Error,
        _ => return Err(/* 保留整行原文的错误 */),
    };

    let message = message_text.trim();
    if message.is_empty() {
        return Err(/* 保留整行原文的错误 */);
    }

    Ok(LogEntry {
        line,
        level,
        message: /* 把 message 变成拥有所有权的 String */,
    })
}
```

## 主练习：`parse_all`（`src/exercise.rs`）

### 一级：思考方向

输入是一个字符串切片，每一行需要编号。哪个适配器能同时给出位置和元素？
位置从 0 开始，而题目要求行号从 1 开始，差多少？

每一行解析出来都是 `Result`。`collect` 除了能收进 `Vec`，还能把一串
`Result` 收成一个整体的 `Result`。这种情况下，第一个 `Err` 出现时会发生
什么？这正好符合题目“返回最靠前的错误”的要求吗？

### 二级：局部分解

1. 对 `lines` 调用 `iter()`，得到 `&str` 的迭代器；接 `enumerate()`，
   得到 `(index, line)`。
2. 用 `map` 把每一对变成 `parse_line(index + 1, line)`，此时元素类型是
   `Result<LogEntry, ParseLogError>`。
3. 用 `collect` 把这些 `Result` 收成 `Result<Vec<LogEntry>, ParseLogError>`。
   目标类型要写清楚，可以用 turbofish。

### 三级：接近完整的结构

```rust
pub fn parse_all(lines: &[&str]) -> Result<Vec<LogEntry>, ParseLogError> {
    lines
        .iter()
        .enumerate()
        .map(|(index, line)| /* 用从 1 开始的行号调用 parse_line */)
        .collect::</* 目标类型：整体 Result */>()
}
```

## 主练习：`partition_logs`（`src/exercise.rs`）

### 一级：思考方向

这次不能遇错就停，成功项和错误项都要留下，而且各自保持输入顺序。你手头
有两个“篮子”，每解析一行就看它属于哪个篮子。普通 `for` 循环能不能直接
做这件事？如果坚持用迭代器，`fold` 的累积值可以是什么？

注意行号仍然从 1 开始，和 `parse_all` 一样需要 `enumerate`。

### 二级：局部分解

1. 准备两个空 `Vec`：一个放 `LogEntry`，一个放 `ParseLogError`。
2. 遍历 `lines.iter().enumerate()`，对每一行调用 `parse_line(index + 1, line)`。
3. 用 `match` 看结果是 `Ok(entry)` 还是 `Err(error)`：前者推进第一个篮子，
   后者推进第二个篮子。
4. 遍历结束后把两个篮子组成元组返回。

如果使用 `fold`，累积值就是 `(Vec<LogEntry>, Vec<ParseLogError>)`，每轮
把新元素加进对应的一边再交回。

### 三级：接近完整的结构

```rust
pub fn partition_logs(lines: &[&str]) -> (Vec<LogEntry>, Vec<ParseLogError>) {
    let mut entries = Vec::new();
    let mut errors = Vec::new();

    for (index, line) in lines.iter().enumerate() {
        match /* 解析第 index + 1 行 */ {
            Ok(entry) => entries.push(entry),
            Err(error) => /* 放进另一个篮子 */,
        }
    }

    (entries, errors)
}
```

## 主练习：`error_messages`（`src/exercise.rs`）

### 一级：思考方向

只需要读，不需要改，也不需要搬走元素。哪个入口产出 `&LogEntry`？怎样只
留下 `ERROR` 级别？`LogEntry` 的 `message` 是 `String`，但返回值要求
`Vec<&str>`：哪个方法能从 `String` 借出 `&str`，而不复制文本？

### 二级：局部分解

1. 对 `entries` 调用 `iter()`，得到 `&LogEntry`。
2. 用 `filter` 留下 `level == Level::Error` 的条目；`Level` 可以比较相等。
3. 用 `map` 把每个条目变成 `entry.message.as_str()`，元素类型就从
   `&LogEntry` 变成了 `&str`。
4. 用 `collect` 收进 `Vec<&str>`。

### 三级：接近完整的结构

```rust
pub fn error_messages(entries: &[LogEntry]) -> Vec<&str> {
    entries
        .iter()
        .filter(|entry| /* 只留下 ERROR */)
        .map(|entry| /* 借出消息文本，不复制 */)
        .collect()
}
```

## 主练习：`summarize`（`src/exercise.rs`）

### 一级：思考方向

这是“按类别计数”：每遇到一个条目，就在对应类别上加 1。第六课学过
`HashMap` 的 `entry` 写法，可以先取到值的可变引用，再加一。键是
`Level`，它实现了 `Copy`，所以从 `&LogEntry` 取出级别时不需要克隆整个
条目。

计数可以用 `fold`，累积值是 `HashMap`；也可以直接 `for`。哪种更容易读？
题目没有强制链式。

### 二级：局部分解

1. 创建一个空 `HashMap<Level, usize>`。
2. 遍历 `entries.iter()`，对每个条目：
   - 用 `entry.level` 作为键；
   - 用 `entry(...).or_insert(0)` 拿到计数器；
   - 对计数器解引用并加 1。
3. 把整个 map 返回。空输入自然得到空 map，不需要特判。

### 三级：接近完整的结构

```rust
pub fn summarize(entries: &[LogEntry]) -> HashMap<Level, usize> {
    let mut counts = HashMap::new();

    for entry in entries {
        let counter = counts
            .entry(/* 用级别作为键 */)
            .or_insert(0);
        /* 让计数器加 1 */
    }

    counts
}
```

## 主练习：`first_error`（`src/exercise.rs`）

### 一级：思考方向

“第一条满足条件的元素”正是短路查找的用途。哪个消费者会在找到后立即
停止，并返回 `Option<元素>`？对 `iter()` 来说，元素是 `&LogEntry`，所以
找到时返回的应该是 `Option<&LogEntry>`，不需要复制。

闭包收到的参数是什么类型？它需要比较 `entry.level` 和 `Level::Error`。
`Level` 实现了 `PartialEq`，可以直接用 `==`。

### 二级：局部分解

1. 对 `entries` 调用 `iter()`。
2. 调用 `find`，闭包判断 `entry.level == Level::Error`。
3. 直接把 `find` 的结果作为函数返回值；没有错误条目时它自然是 `None`。

### 三级：接近完整的结构

```rust
pub fn first_error(entries: &[LogEntry]) -> Option<&LogEntry> {
    entries
        .iter()
        .find(|entry| /* 判断级别是不是 ERROR */)
}
```

## 课后巩固：`Countdown`（`src/consolidation.rs`）

### 一级：思考方向

迭代器只需要记住“下一个要产出的数字”。第一次 `next` 返回起始值；之后
每次返回比上一次小 1 的值；产出 `1` 之后，剩余数量变成 0。

要小心两个边界：

- 起始值为 0 时，第一次调用就应该返回 `None`；
- 返回 `None` 之后不能再减，否则会下溢（无符号整数是只能表示 0 和正整数的类型，减到 0 以下会 panic
  或回绕，取决于编译配置）。只在“还有值”时递减，就不会有这个问题。

`size_hint` 问的是“还能产出多少个”，不是“起始值是多少”。对一个倒计时，
剩余数量就是字段里的值本身。

### 二级：局部分解

1. `new(start)`：把 `start` 存进 `remaining` 字段。
2. `next`：
   - 如果 `remaining == 0`，直接返回 `None`，不要修改任何状态；
   - 否则把 `remaining` 的当前值保存下来；
   - 把 `remaining` 减 1；
   - 返回保存下来的值，包在 `Some` 里。
3. `size_hint`：
   - 剩余数量就是 `remaining`；
   - 返回 `(数量, Some(数量))`，两边都用 `usize`；
   - 注意把 `u32` 转成 `usize`。

### 三级：接近完整的结构

```rust
pub fn new(start: u32) -> Self {
    Self {
        remaining: /* 保存起始值 */,
    }
}

fn next(&mut self) -> Option<Self::Item> {
    if /* 已经没有值 */ {
        return None;
    }

    let value = /* 当前要产出的值 */;
    /* 让剩余数量减 1 */;
    Some(value)
}

fn size_hint(&self) -> (usize, Option<usize>) {
    let remaining = /* 把剩余数量转成 usize */;
    (remaining, Some(remaining))
}
```

## 课后巩固：自测问题

做完后用自己的话回答：

1. 为什么 `next` 在 `remaining == 0` 时必须直接返回，而不能先减 1？
2. 如果 `size_hint` 返回 `(0, None)`，`len()` 还能工作吗？为什么？
3. 为什么 `Countdown::new(0)` 不需要在 `new` 里特判？
