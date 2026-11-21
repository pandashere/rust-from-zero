# 第十二课提示：主练习与课后巩固

提示分三级，卡到哪一级看哪一级：

- **一级：思考方向**——只提示该想什么，不涉及代码结构。
- **二级：局部分解**——把任务拆成几个判断步骤，仍不写完整代码。
- **三级：接近完整的结构**——给出带空缺的骨架，你需要自己补上关键调用。

先看一级；想清楚了就不要继续往下看。三级提示故意不给出可直接提交的完整代码。

主练习推荐顺序：`NonEmpty::check` → `MaxChars::check` → `Digits::check` → `Digits::describe` → `ValueText` 两个实现 → `run_one` → `run_all`。每完成一组就运行 `cargo test -p l12_traits exercise`。

## 主练习：`NonEmpty::check`（`src/exercise.rs`）

### 一级：思考方向

“首尾空白不算内容”意味着先去掉首尾空白，再判断剩下什么。去掉之后如果什么都没有，就是 `Empty`；否则要返回“剩余字符数”。注意题目要求按字符计数，不是按字节计数——哪个方法能逐个产出字符？

### 二级：局部分解

1. 用 `trim()` 去掉字符串首尾的空白，得到一个临时的文本切片。
2. 如果这个切片是空的，返回 `Err(CheckError::Empty)`。
3. 否则数它包含多少个字符：先取字符迭代器，再数个数，返回 `Ok(数量)`。

### 三级：接近完整的结构

```rust
fn check(&self, input: &str) -> Result<Self::Value, CheckError> {
    let trimmed = input./* 去掉首尾空白的方法 */();
    if trimmed./* 判断字符串是否为空的方法 */() {
        return Err(/* 哪个错误变体 */);
    }
    Ok(trimmed./* 逐个产出字符的方法 */.count())
}
```

`"你好"` 用 `len()` 会得到 6（UTF-8 字节数），题目要的是 2。确认你用的是按字符计数的方法。

## 主练习：`MaxChars::check`

### 一级：思考方向

这个检查不去空白：首尾空格也算长度。先数出实际字符数，再和 `self.limit` 比较。相等算通过，只有严格超过才失败。失败时错误里要同时带“上限”和“实际值”。

### 二级：局部分解

1. `let actual = input.chars().count();`
2. 如果 `actual <= self.limit`，返回 `Ok(actual)`。
3. 否则返回 `Err(CheckError::TooLong { limit: self.limit, actual })`。

### 三级：接近完整的结构

```rust
fn check(&self, input: &str) -> Result<Self::Value, CheckError> {
    let actual = input./* 字符迭代器 */.count();
    if actual <= self./* 上限字段 */ {
        Ok(/* 返回什么 */)
    } else {
        Err(CheckError::TooLong {
            limit: /* 哪个字段 */,
            actual,
        })
    }
}
```

边界自查：`limit = 0` 时，空输入应该通过；任何非空输入（哪怕只有一个字符）都应该失败。

## 主练习：`Digits::check`

### 一级：思考方向

三种失败要分开报，顺序很重要：先判断空，再判断是否“全是数字”，最后才解析。如果先解析，空输入和非法字符都会变成同一种解析错误，就分不出 `Empty` 和 `NotDigits` 了。

“全是数字”指每个字符都是 ASCII 的 `0` 到 `9`，不是“看起来像数字”的全角字符。解析成功但数值太大时，才是 `TooLarge`。

### 二级：局部分解

1. `if input.is_empty() { return Err(CheckError::Empty); }`
2. 检查每个字符是否都是 ASCII 数字：`input.chars().all(|c| c.is_ascii_digit())`。只要有一个不满足，返回 `Err(CheckError::NotDigits)`。
3. 到这里输入非空且全是 ASCII 数字，调用 `input.parse::<u64>()`。这是标准库的“文本转数字”方法：成功得到 `Ok(数值)`，失败得到 `Err(解析错误)`。本步失败只可能是数值超出范围，用 `map_err` 把错误换成 `CheckError::TooLarge`；`map_err` 的作用是“只在 `Err` 时替换错误值，`Ok` 原样保留”。

### 三级：接近完整的结构

```rust
fn check(&self, input: &str) -> Result<Self::Value, CheckError> {
    if input./* 判空 */() {
        return Err(CheckError::Empty);
    }
    if !input.chars().all(|c| c./* 是否为 ASCII 数字 */()) {
        return Err(/* 哪个错误变体 */);
    }
    input
        ./* 解析成 u64 的方法 */()
        .map_err(|_| /* 解析失败对应哪个变体 */)
}
```

为什么 `" 12"` 是 `NotDigits` 而不是解析成功？因为空格不是 ASCII 数字，第二步就会拦下。

## 主练习：`Digits::describe` 与 `ValueText`

### 一级：思考方向

`describe` 的返回值在注释里已经写死，不需要拼接任何字段。`ValueText` 的两个实现都是“把数字变成十进制文本”；标准库里 `usize` 和 `u64` 已经有现成的转换方法。

### 二级：局部分解

1. `Digits::describe` 直接返回注释中规定的那个固定字符串。
2. `usize::value_text` 和 `u64::value_text` 都调用数字自带的文本转换方法，返回 `String`。

### 三级：接近完整的结构

```rust
// Digits
fn describe(&self) -> String {
    String::from("/* 注释里规定的完整文字 */")
}

// ValueText for usize / u64
fn value_text(&self) -> String {
    self./* 转成十进制文本的方法 */()
}
```

`to_string()` 来自 `Display` trait；本课不要求你手写数字转文本的算法。

## 主练习：`run_one`

### 一级：思考方向

先运行检查。检查失败时，整个函数应该直接返回那个错误，不要构造半个 `Outcome`。成功时再取三样东西：名称的副本、检查产出的值、说明文字。

### 二级：局部分解

1. 调用 `check.check(input)`，用 `?` 把错误直接传播出去，成功值绑定到 `value`。
2. `label` 用 `check.label().to_string()` 复制一份 `String`。
3. `description` 用 `check.describe()`。
4. 返回 `Ok(Outcome { label, value, description })`。

### 三级：接近完整的结构

```rust
fn run_one<C: Check>(check: &C, input: &str) -> Result<Outcome<C::Value>, CheckError> {
    let value = check./* 运行检查的方法 */(input)?;
    Ok(Outcome {
        label: check./* 取名称 */.to_string(),
        value,
        description: check./* 取说明 */(),
    })
}
```

`?` 在这里的作用：如果 `check` 返回 `Err(e)`，整个 `run_one` 立刻返回 `Err(e)`；只有 `Ok(v)` 才会继续往下走。

## 主练习：`run_all`

### 一级：思考方向

这是“对队列里的每一项做同一件事，并保持顺序”的需求。每一项的失败不应该中断循环：`result_text` 已经把成功和失败都装进了 `Result`，你只需要把每个 `Result` 收集起来。

### 二级：局部分解

1. 遍历 `checks` 的引用：`checks.iter()`。
2. 对每个 `check` 调用 `check.result_text(input)`。
3. 用 `collect()` 把迭代器收集成 `Vec<Result<String, CheckError>>` 并返回。

### 三级：接近完整的结构

```rust
fn run_all(checks: &[Box<dyn ErasedCheck>], input: &str) -> Vec<Result<String, CheckError>> {
    checks
        .iter()
        .map(|check| check./* 擦除层的方法 */(input))
        .collect()
}
```

不要在这里写 `?`：`run_all` 的返回类型不是 `Result`，而且题目要求某一项失败后继续运行后面的检查。

## 课后巩固：`Rectangle` 的 `area` 与 `scale`

### 一级：思考方向

面积就是宽乘高。缩放要先判断因子是否大于 `0`：不合法时立刻返回错误，不能先改字段再检查，否则失败路径会留下改了一半的数据。合法时宽和高都乘以因子。

### 二级：局部分解

1. `area`：返回 `self.width * self.height`。
2. `scale`：`if factor <= 0.0 { return Err(ShapeError::NonPositiveFactor); }`。
3. 然后 `self.width *= factor; self.height *= factor;`，最后 `Ok(())`。

### 三级：接近完整的结构

```rust
fn area(&self) -> f64 {
    self./* 宽字段 */ * self./* 高字段 */
}

fn scale(&mut self, factor: f64) -> Result<(), ShapeError> {
    if factor <= 0.0 {
        return Err(/* 哪个错误变体 */);
    }
    self.width *= factor;
    self./* 另一个字段 */ *= factor;
    Ok(())
}
```

为什么先检查再修改？测试会验证失败后面积仍是原值。

## 课后巩固：`Circle` 的 `area`、`scale` 与 `describe`

### 一级：思考方向

圆面积是 π 乘半径的平方。缩放只改半径。`describe` 要覆盖默认实现，格式在注释里：中文括号、半径两位小数。`format!` 的 `{:.2}` 能把浮点数保留两位小数。

### 二级：局部分解

1. `area`：`std::f64::consts::PI * self.radius * self.radius`。
2. `scale`：同样先判断 `factor <= 0.0`，再 `self.radius *= factor`。
3. `describe`：`format!("圆形（半径 {:.2}）", self.radius)`。

### 三级：接近完整的结构

```rust
fn area(&self) -> f64 {
    std::f64::consts::/* π 常量名 */ * self.radius * self.radius
}

fn scale(&mut self, factor: f64) -> Result<(), ShapeError> {
    if factor <= 0.0 {
        return Err(/* 错误变体 */);
    }
    self./* 半径字段 */ *= factor;
    Ok(())
}

fn describe(&self) -> String {
    format!("/* 固定前缀 */（半径 {:.2}）", self./* 半径字段 */)
}
```

注意 `describe` 的返回类型是 `String`，不是 `&str`；`format!` 正好返回 `String`。

## 课后巩固：`total_area` 与 `largest_name`

### 一级：思考方向

`total_area` 是求和：空切片自然得到 `0.0`，可以用迭代器的 `sum()`。`largest_name` 需要记住“当前最大”的那一项：空切片返回 `None`；非空时从第一项开始，遇到严格更大的才替换。用严格大于才能保证并列时保留最先出现的。

### 二级：局部分解

1. `total_area`：`shapes.iter().map(|shape| shape.area()).sum()`。
2. `largest_name`：先取 `shapes.first()?`，把它作为当前最大；遍历其余项，如果 `shape.area() > best.area()` 就更新 `best`；最后返回 `Some(best.name().to_string())`。
3. 空切片的 `first()` 返回 `None`，用 `?` 直接得到 `None`。

### 三级：接近完整的结构

```rust
fn total_area(shapes: &[Box<dyn Shape>]) -> f64 {
    shapes.iter().map(|shape| shape./* 面积方法 */()).sum()
}

fn largest_name(shapes: &[Box<dyn Shape>]) -> Option<String> {
    let mut best = shapes./* 取第一项的方法 */()?;
    for shape in shapes {
        if shape.area() > best.area() {
            best = /* 更新成当前项 */;
        }
    }
    Some(best./* 取名称 */.to_string())
}
```

`?` 在返回 `Option` 的函数里遇到 `None` 会直接返回 `None`；这和 `Result` 里的 `?` 是同一个思路。

## 课后巩固：`scale_all`

### 一级：思考方向

这个函数有两个层次：先校验整个调用的参数，再逐项执行。非法因子必须在任何修改之前拦下；合法时对每个 trait object 调用 `scale`。因为 `scale` 返回 `Result`，循环里可以用 `?` 把错误传出去。

### 二级：局部分解

1. `if factor <= 0.0 { return Err(ShapeError::NonPositiveFactor); }`
2. 遍历 `shapes.iter_mut()`，对每个 `shape` 调用 `shape.scale(factor)?`。
3. 循环结束返回 `Ok(())`。

### 三级：接近完整的结构

```rust
fn scale_all(shapes: &mut [Box<dyn Shape>], factor: f64) -> Result<(), ShapeError> {
    if factor <= 0.0 {
        return Err(/* 错误变体 */);
    }
    for shape in shapes./* 可变引用迭代方法 */() {
        shape./* 缩放方法 */(factor)?;
    }
    Ok(())
}
```

为什么这里要 `iter_mut()` 而不是 `iter()`？因为 `scale` 需要 `&mut self`，只读引用无法调用它。

## 课后巩固：`build_report` 与 `Report` 的 `Display`

### 一级：思考方向

`build_report` 把每个图形的 `describe()` 收集成一个 `Vec<String>`，再包进 `Report`。`Display` 的输出是“每行一个元素”：把 `Vec<String>` 用一个换行符连接起来；空 Vec 连接后就是空字符串，正好满足边界要求。

### 二级：局部分解

1. `build_report`：`shapes.iter().map(|shape| shape.describe()).collect()`，然后用 `Report(...)` 包起来。
2. `Display::fmt`：用 `self.0.join("\n")` 得到连接后的字符串，再用 `write!` 写进格式化器 `f`。`join` 只在元素之间插入分隔符，末尾不会多加换行。

### 三级：接近完整的结构

```rust
fn build_report(shapes: &[Box<dyn Shape>]) -> Report {
    Report(
        shapes
            .iter()
            .map(|shape| shape./* 说明方法 */())
            .collect(),
    )
}

impl Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self./* 元组字段 */.join(/* 分隔符 */))
    }
}
```

不要用 `join` 之外的方式手动在每行后追加换行，否则最后一行会多出一个换行，测试会失败。
