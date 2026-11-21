# 第四课：Option、Result、自定义错误与 `?`

很多程序的错误来自同一件事：一个函数嘴上说“我会给你一个结果”，实际上有时给不出来——列表里可能根本没有要找的东西，用户输入可能根本不是数字。如果函数的返回类型（函数签名里声明的“我会返回哪种数据”）不告诉你这一点，调用者（使用这个函数的人）只能靠猜、靠文档、靠运气。

Rust 的做法是把这两种情况直接写进类型里：用 `Option<T>` 表示“可能有值，也可能没有”；用 `Result<T, E>` 表示“可能成功给出结果，也可能失败并给出原因”。调用者拿到这样的返回值后，编译器会强制他先处理“没有”或“失败”的情况，才能用到里面的值。本课就讲这两种类型、处理它们的语法，以及如何设计自己的失败原因。

## 本课定位

### 你将学会

- 区分“没有值”和“发生错误”这两种不同的情况。
- 构造和使用 `Some(value)`、`None`、`Ok(value)`、`Err(error)`。
- 用 `match`、`if let`、`let ... else` 取出 `Option` 或 `Result` 里的值。
- 使用 `unwrap_or`、`map`、`and_then`、`map_err`、`ok_or` 这些常用的转换方法。
- 理解 `?` 运算符（operator，即语言内置的符号操作）如何传播失败，以及它为什么要求函数返回 `Option` 或 `Result`。
- 定义携带上下文信息的自定义错误 enum（枚举）。
- 判断什么时候可以用 `unwrap`/`expect` 让程序直接中止，什么时候不应该。
- 为解析函数设计清晰、可以被测试覆盖的失败路径。

### 前置知识

本课会用到前三课的内容，用到时会用一两句话复习：

- 变量绑定、函数、表达式与基本类型，见[第一课](../01-basics/README.md)。
- struct、enum、携带数据的 variant（枚举的一种取值形态）、`match` 与穷尽性检查，见[第二课](../02-data-modeling/README.md)。
- 所有权、move、`String` 与 `&str` 的区别，见[第三课](../03-ownership-borrowing/README.md)。

### 本课暂不深入

本课不会讲标准库的 `std::error::Error` trait（特征，即一组行为的约定）、错误链、第三方错误处理库或日志系统，也不展开迭代器上的大量 `Option`/`Result` 组合技巧。闭包（closure，一种写在调用现场的匿名小函数）在本课只会用到最简单的形式，系统讲解在[第九课](../09-closures/README.md)。

## 如何学习本课

1. 先跟着第一节把“为什么需要 `Option`/`Result`”想清楚，再学语法。
2. 每学一种取值的写法（`match`、`if let`、`?`），都动手改一改示例，观察编译器反应。
3. 第十四节是真实编译器报错的阅读练习，不要跳过——排错能力是本课的一半。
4. 完成代码预测题之后再做主练习；卡住时按 `HINTS.md` 的级别逐步查看提示。
5. 主练习全部通过后，再做课后巩固任务。

## 一、从一个具体问题说起：解析 `"localhost:8080"`

假设你在写一个需要连接网络服务的程序。配置里有一行文本，形如 `"localhost:8080"`：冒号前是主机名（要连接哪台机器），冒号后是端口号（一个 1 到 65535 之间的整数，标识机器上的哪个服务）。你需要写一个函数，把这行文本拆成两部分，并把端口文本变成整数。

这个函数可能遇到的情况有：

- `"localhost:8080"`——正常，拆出主机名和端口；
- `"localhost"`——没有冒号，缺信息；
- `":8080"`——主机名是空的；
- `"localhost:abc"`——端口不是数字；
- `"localhost:0"`——端口为 0，不可用；
- `""`——整行都是空的。

也就是说：**这个函数不是总能给出结果**。这是本课要解决的核心问题。

### 1.1 办法一：约定特殊值

一种常见的朴素办法是“返回特殊值表示失败”：端口解析不出来就返回 `0`，主机名找不到就返回空字符串。这个办法有三个具体问题：

1. **特殊值可能和合法值冲突**。端口号 `0` 在“解析失败”时返回，那真正的 `"localhost:0"` 算不算失败？调用者分不出来。
2. **约定只存在于文档和记忆里**。函数签名上只写着“返回一个整数”，看不出“返回 0 要当心”。三个月后你自己也会忘记这条约定。
3. **编译器帮不上忙**。调用者忘记检查特殊值，代码照常编译通过，问题留到运行时才暴露。

### 1.2 办法二：空引用与异常（背景知识，不要求掌握）

如果你没学过其他语言，这一小节只是背景介绍，读不懂细节不影响后面的学习。

其他语言里常见的两种办法各有麻烦：

- **空引用**（null，一个表示“这里什么都没指向”的特殊值）：任何引用类型的变量都“可能”是 null，但类型上看不出来。忘记判空就直接使用，程序在运行时崩溃。这类崩溃太常见，null 的发明者 Tony Hoare 称它为“价值十亿美元的错误”。
- **异常**（exception，一种“函数遇到错误就抛出一个对象，跳到远处某段代码去处理”的机制）：函数签名同样看不出它会不会抛异常、抛哪种；而且异常会跳过中间的代码，让“函数到底从哪里返回”变得难以追踪。

### 1.3 Rust 的选择：把可能性写进类型

Rust 不使用 null，也不用异常处理普通的业务失败。它用两个标准库（standard library，随语言一起发布、任何程序都可以直接使用的一组现成代码）里定义的普通 enum 来表达“可能给不出结果”：

- `Option<T>`——一个值**可能有、也可能没有**，没有不是错误，只是正常的一种结果；
- `Result<T, E>`——一个操作**可能成功、也可能失败**，失败时会附带原因。

因为它们写在返回类型里，所以：

- 调用者一看签名就知道“这个函数可能给不出结果”；
- 编译器强制调用者处理“没有”或“失败”的分支，否则代码编译不过；
- `match` 的穷尽性检查（第二课学过：编译器检查 `match` 是否覆盖了 enum 的所有 variant）会数清楚你有没有漏掉分支。

下面分别学习这两个类型。

## 二、`Option<T>`：一个“可能有、也可能没有”的容器

标准库中 `Option` 的定义可以简化理解为：

```rust
enum Option<T> {
    None,
    Some(T),
}
```

这是第二课学过的“携带数据的 enum”：

- 它有两个 variant：`None` 表示“没有值”，`Some(T)` 表示“有值，值就在里面”。
- `T` 是一个**泛型参数**（generic parameter，定义时先占位的类型名，使用时再换成具体类型）。`Option<i32>` 就是“可能有一个 `i32`”，`Option<String>` 就是“可能有一个 `String`”。同一个 `Option` 定义可以装任何类型，这就是“泛型”（generic，通用的、不绑定具体类型的写法）的意义。

构造两个值看看：

```rust
let number: Option<i32> = Some(10);
let missing: Option<i32> = None;
```

注意：`Option<i32>` 和 `i32` 是**两种不同类型**。前者是一个“里面可能有整数的盒子”，后者就是一个整数。你不能把盒子当整数用：

```rust,compile_fail
let value: Option<i32> = Some(10);
let result = value + 1; // 编译错误：Option<i32> 不能和整数相加
```

必须先打开盒子、处理“有”和“没有”两种情况，才能用到里面的整数。这正是 1.1 节“编译器帮不上忙”的反面：忘记处理 `None`，代码根本编译不过。

### 2.1 什么时候用 Option

一句话标准：**“没有”是正常结果、不需要解释原因时，用 `Option`**。例如“在列表里找第一个偶数”，找不到不是错误，只是事实如此。需要告诉调用者“为什么失败”时，用后面讲的 `Result`。

## 三、取出 Option 里的值

### 3.1 用 `match` 覆盖两种情况

```rust
fn double(value: Option<i32>) -> Option<i32> {
    match value {
        Some(number) => Some(number * 2),
        None => None,
    }
}
```

逐项读这段代码：

- `match value` 检查 `value` 当前是哪个 variant；
- `Some(number)` 这个 pattern（模式，用来匹配数据形状的写法）在匹配成功时，把 `Some` 里面的 `i32` 取出来绑定到新变量 `number` 上；
- 这个分支返回 `Some(number * 2)`，即“有值就把值翻倍再装回去”；
- `None` 分支没有值可取，直接返回 `None`。

`match` 是表达式（第一课学过：表达式会算出一个值），两个分支共同产生这个函数的返回值。如果你漏写 `None` 分支，编译器会以“pattern 没有覆盖所有情况”拒绝编译——这就是穷尽性检查在保护你。

### 3.2 `if let`：只关心一种情况

有时你只想在“有值”时做点什么，“没有”时什么都不做。写完整的 `match` 要补一个什么都不做的分支，啰嗦。`if let` 是“如果匹配上这个 pattern 就执行”的简写：

```rust
let value = Some(10);

if let Some(number) = value {
    println!("拿到 {number}");
}
```

含义是：如果 `value` 是 `Some`，就把里面的值绑定到 `number` 并执行花括号里的代码；否则整个跳过。它等价于一个只处理 `Some`、其他情况空着的 `match`。

### 3.3 `let ... else`：先排除失败，再写正常路径

还有一种常见结构：函数开头先确认“有值”，没有就直接返回，后面的代码只处理有值的情况。`let ... else` 专门支持这种写法：

```rust
fn print_number(value: Option<i32>) {
    let Some(number) = value else {
        println!("没有值");
        return;
    };

    println!("number = {number}");
}
```

读法：`let Some(number) = value` 尝试按 `Some(number)` 这个 pattern 解构 `value`；如果配不上（`value` 是 `None`），就执行 `else` 分支。`else` 分支必须“发散”（diverge，即必须让当前流程离开，不能照常走到下一行）——用 `return` 返回、用 `break`/`continue` 跳出循环，或者用 `panic!` 中止程序，都可以。匹配成功时，`number` 在之后的代码里可以直接使用。

这种写法的价值在于：失败处理缩进在 `else` 里，主逻辑不嵌套，从上到下一路读到底。

## 四、不用 match 的三个常用方法

`Option` 上有一批现成方法，能把常见的 `match` 缩成一行。先学三个。

### 4.1 `unwrap_or`：没有就给个默认值

```rust
let value = Some(10);
assert_eq!(value.unwrap_or(0), 10);

let missing: Option<i32> = None;
assert_eq!(missing.unwrap_or(0), 0);
```

`unwrap_or(默认值)`：有值时取出值，没有时使用括号里的默认值。

### 4.2 `map`：有值就转换一下

```rust
let value = Some(10);
let doubled = value.map(|number| number * 2);

assert_eq!(doubled, Some(20));
```

`map` 接收一个“怎么转换”的说明：如果原来是 `Some`，就把里面的值按说明转换后重新包成 `Some`；如果原来是 `None`，就还是 `None`，转换根本不会发生。

这里的 `|number| number * 2` 是一个**闭包**（closure）：写在调用现场的一次性小函数，竖线之间是参数，竖线之后是函数体。现在你只需要会读这种最简单的形式；闭包的完整能力在[第九课](../09-closures/README.md)专门讲。

### 4.3 `and_then`：转换本身也可能没有结果

假设转换步骤本身也会返回 `Option`：

```rust
fn positive_half(value: i32) -> Option<i32> {
    if value > 0 && value % 2 == 0 {
        Some(value / 2)
    } else {
        None
    }
}

let result = Some(8).and_then(positive_half);
assert_eq!(result, Some(4));

let negative = Some(-8).and_then(positive_half);
assert_eq!(negative, None);
```

如果对 `Some(8)` 用 4.2 的 `map`，闭包返回的是 `Option<i32>`，你会得到两层盒子 `Option<Option<i32>>`。`and_then` 的作用就是把这两层压成一层：有值时调用转换函数，并直接采用它返回的 `Option` 作为整体结果。

### 4.4 初学阶段不必追求链式调用

这些方法可以首尾相接写成一长串“链式调用”。链式写法学起来有成就感，但如果你发现一条链让“失败时到底走了哪条路”变得难读，就退回 `match` 或 `let ... else`。**清晰比简短重要**，主练习也只要求行为正确，不要求形式花哨。

## 五、`Result<T, E>`：失败也要带上原因

`Option` 只能说“没有”，说不出“为什么”。解析 `"localhost:abc"` 失败时，调用者需要知道是端口不是数字、数字是什么——这时用 `Result`。标准库中的定义可以简化为：

```rust
enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

它有两个泛型参数：

- `T`：成功时给出的值的类型，装在 `Ok` 里；
- `E`：失败时给出的错误信息的类型，装在 `Err` 里。

```rust
let success: Result<i32, String> = Ok(10);
let failure: Result<i32, String> = Err(String::from("invalid input"));
```

和 `Option` 一样，`Result` 也是普通 enum，处理方式和第三节完全平行：

```rust
fn print_result(result: Result<i32, String>) {
    match result {
        Ok(value) => println!("value = {value}"),
        Err(error) => println!("error = {error}"),
    }
}
```

差别只在语义：`Err` 里的东西是给调用者看的失败原因。调用者可以显示它、据此重试、把它转换成自己的错误，或者继续往上传——选择权在调用者手里，这正是把失败写进类型的意义。

第四节讲的方法很多也有 `Result` 版本，后面用到 `map_err` 时会讲。先记住对应关系：`Option` 的“坏消息”是 `None`，`Result` 的“坏消息”是 `Err`。

## 六、标准库已经在用 Result：把文本解析成数字

字符串有一个 `parse` 方法，能把文本解析成数字，它的返回值就是 `Result`：

```rust
let parsed = "42".parse::<i32>();
```

`parse` 能解析成很多种数字类型，编译器无法替你决定目标类型，所以要用 `::<i32>` 指明。这种写在方法名后、夹在两个冒号之间的尖括号写法有个俗名叫 **turbofish**（因为 `::<>` 看起来有点像一条鱼），作用是给泛型方法指定具体类型。也可以靠变量类型标注让编译器推断，写成 `"42".parse()` 而省略 turbofish：

```rust
let parsed: Result<i32, _> = "42".parse();
```

这里的 `_` 表示“这个类型请编译器自己填”。

`parse` 返回的完整类型是 `Result<i32, ParseIntError>`：成功时 `Ok` 里是解析出的整数；失败时 `Err` 里是标准库定义的 `ParseIntError`（parse integer error，解析整数失败的错误类型）。你现在不需要了解它的内部结构，只需要知道它是一个专门的错误类型：

```rust
assert_eq!("42".parse::<i32>(), Ok(42));

let result = "forty-two".parse::<i32>();
assert!(result.is_err()); // is_err：是 Err 时返回 true
```

（`assert!` 与 `assert_eq!` 都是宏：条件不成立、或两个值不相等时，它们让程序 panic 并报告现场。测试里用它们表达“这里必须成立”。）

## 七、unwrap、expect 与 panic：什么时候可以让程序直接停下来

`Option` 和 `Result` 上都有 `unwrap` 方法：“把里面的值给我，不给就中止程序”：

```rust
let number = "42".parse::<i32>().unwrap(); // Ok(42) => number 是 42
```

如果结果是 `Err`（或 `Option` 是 `None`），`unwrap` 会触发 **panic**：程序主动中止当前运行，打印一条错误信息后退出。panic 是 Rust 里“程序遇到了无法合理继续的状况”的表达，和普通可预期的失败（用户输错、文件不存在）不是一回事。

`expect` 和 `unwrap` 一样会 panic，但允许你附上一句说明，panic 信息会带上它：

```rust
let number = "42"
    .parse::<i32>()
    .expect("hard-coded number should be valid");
```

什么时候可以用它们？

- **测试代码**里，失败本来就意味着测试失败；
- **示例代码**里，为了让主线更短；
- 程序自己保证的**不变量**（invariant，由代码逻辑保证永远成立的条件），如果它破裂说明程序本身有 bug，继续运行没有意义。

什么时候不该用？处理**用户输入、文件、网络**这类你无法控制、失败完全可预期的数据时。这些场景失败是正常的，应该用 `Result` 把失败交还给调用者，而不是让整个程序中止。主练习明确禁止使用 `unwrap`/`expect`/`panic!` 处理输入，就是要练习正确的做法。

## 八、自定义错误类型：让失败原因可以被程序判断

回到第一节的解析函数。给 `Err` 装一个 `String` 描述文字是最简单的起步：

```rust
fn parse_age(input: &str) -> Result<u8, String> {
    // ...
}
```

但 `String` 错误有个实际麻烦：调用者想区分“输入为空”和“数字太大”，只能去比较错误文本。一旦哪天你把文案从 `"empty"` 改成 `"input is empty"`，所有比较旧文案的代码悄悄失效，编译器不会提醒。

第二课学过的 enum 正好解决这个问题——给每种失败原因一个 variant，需要上下文的 variant 就携带数据：

```rust
#[derive(Debug, PartialEq, Eq)]
enum AgeError {
    Empty,
    InvalidNumber(String),
    TooLarge(u32),
}
```

复习一下：`#[derive(...)]` 是让编译器自动为这个类型生成指定能力的写法。`Debug` 让它能用 `{:?}` 打印（便于调试和测试失败时显示）；`PartialEq` 和 `Eq` 让它能用 `==` 比较（测试里断言“返回的错误恰好是这个 variant”就靠它）。

用 enum 做错误类型的好处：

- **variant 明确列出所有失败原因**，一看类型就知道这个函数可能怎么失败；
- **每种错误只携带自己需要的上下文**：`InvalidNumber` 带上无法解析的原文，`TooLarge` 带上超出的数值，`Empty` 什么都不用带；
- **测试可以断言具体 variant**，不依赖文案；
- **调用者可以 `match` 穷尽处理**，将来你给 enum 增加新的 variant，编译器会逼所有 `match` 它的地方更新——第十四节会演示这个报错。

## 九、`?`：一行写完“失败就交给调用者”

### 9.1 没有 `?` 的世界

写一个函数：把文本解析成整数再翻倍。解析失败时，函数自己也失败：

```rust
fn parse_and_double(input: &str) -> Result<i32, std::num::ParseIntError> {
    let number = match input.parse::<i32>() {
        Ok(number) => number,
        Err(error) => return Err(error),
    };

    Ok(number * 2)
}
```

中间的 `match` 只做一件事：`Ok` 就取值继续，`Err` 就原样返回给调用者。这个模式在错误处理代码里出现得极其频繁，每次都写四行很啰嗦。

### 9.2 `?` 就是这个 match 的简写

```rust
fn parse_and_double(input: &str) -> Result<i32, std::num::ParseIntError> {
    let number = input.parse::<i32>()?;
    Ok(number * 2)
}
```

写在表达式末尾的 `?` 是 Rust 的**错误传播运算符**（propagation，传播就是“把失败往上传给调用者”）。它的心智模型只有两行：

```text
表达式是 Ok(value)  => 取出 value，赋给左边，继续往下执行
表达式是 Err(error) => 当前函数立刻 return Err(error)
```

所以 `"abc"` 会让 `parse_and_double` 在 `?` 那一行就直接返回 `Err`，`Ok(number * 2)` 根本不会执行。

注意 `?` **不是异常**：失败仍然通过普通的函数返回值显式传递，函数签名上写得清清楚楚，控制流也只是“提前 return”，没有跳到远处的神秘处理代码。

### 9.3 `?` 也能用在 Option 上

```rust
fn first_character(text: Option<&str>) -> Option<char> {
    let text = text?;
    text.chars().next()
}
```

对 `Option`，`?` 的规则对应为：

```text
表达式是 Some(value) => 取出 value，继续执行
表达式是 None        => 当前函数立刻 return None
```

（`chars().next()` 先取得字符的迭代器再取第一个字符；`next` 返回 `Option<char>`，因为空字符串没有第一个字符。迭代器在第十课系统学习，这里只需知道“取第一个，可能取不到”。）

### 9.4 为什么 `?` 要求函数返回兼容的类型

下面这段代码编译不过：

```rust,compile_fail
fn parse_number(input: &str) -> i32 {
    let number = input.parse::<i32>()?;
    number
}
```

想一下为什么：`?` 遇到 `Err` 时需要从函数提前返回这个错误，但函数声明的返回类型是 `i32`——一个整数里根本没有地方装错误信息。返回类型没有给失败留通道，`?` 就无处传播。所以规则是：**在返回 `Result` 的函数里，`?` 可以作用于 `Result`；在返回 `Option` 的函数里，`?` 可以作用于 `Option`**。第十四节有这条真实报错及其读法。

修复方式就是给函数一个能装失败的返回类型：

```rust
fn parse_number(input: &str) -> Result<i32, std::num::ParseIntError> {
    let number = input.parse::<i32>()?;
    Ok(number)
}
```

## 十、map_err：把别人的错误翻译成自己的错误

上一个例子里函数原样返回了标准库的 `ParseIntError`。但你自己的函数通常希望返回第八节那样的自定义错误——这时需要“翻译”：

```rust
#[derive(Debug, PartialEq, Eq)]
enum PortError {
    Invalid(String),
}

fn parse_port(input: &str) -> Result<u16, PortError> {
    input
        .parse::<u16>()
        .map_err(|_| PortError::Invalid(input.to_owned()))
}
```

`map_err` 和第四节的 `map` 对称，只是作用在失败那一侧：

- 结果是 `Ok`：原样通过，闭包不会执行；
- 结果是 `Err`：调用闭包，把原来的错误换成闭包算出的新错误。

闭包参数写 `_` 表示“原来的错误（那个 `ParseIntError`）我不需要用”，因为我们选择自己保存原始输入文本作为上下文——`input.to_owned()` 把借用的 `&str` 复制成拥有的 `String`（第三课的所有权知识）。保留导致失败的原文，比只说一句“端口无效”对排查问题更有用。

这个例子里 `map_err` 的结果直接就是函数返回值。更常见的写法是在 `map_err(...)` 后面再加 `?`：翻译完错误类型之后，顺手用 `?` 完成“成功取值、失败返回”。第十三节的完整示例会用到这个组合。

> 进阶说明（现在不用掌握）：当上下游错误类型不同时，`?` 还能通过名为 `From` 的 trait 自动转换错误类型。本课统一用 `map_err` 显式转换，因为显式写法不需要新知识，也更容易看清转换发生在哪一行。

## 十一、ok_or：把“没有”升级成“失败”

有些标准库方法返回 `Option`——比如 `find` 在文本里查找子串的位置，找不到就返回 `None`：

```rust
let index = "localhost:8080".find(':'); // Option<usize>，Some(9)
```

但在你的解析函数里，“找不到冒号”不是“正常缺失”，而是“输入格式不对”，应该成为 `Err`。`ok_or` 就完成这个转换：

```rust
let index = input.find(':').ok_or(ParseError::MissingSeparator)?;
```

规则：`Some(value)` 变成 `Ok(value)`，`None` 变成括号里给出的 `Err`。后面紧跟 `?`，于是找不到冒号时函数直接带着 `MissingSeparator` 返回。

它有一个孪生方法 `ok_or_else`，接收一个闭包，只在 `None` 时才调用闭包构造错误；当构造错误本身代价较高（比如要拼接长字符串）时用它可以避免浪费。本课构造的都是小错误，用 `ok_or` 即可。

## 十二、错误里为什么常放 String 而不是 &str

观察第八节的错误 enum：`InvalidNumber(String)` 装的是拥有的 `String`，不是借用的 `&str`。这是第三课所有权知识的直接应用：

- 错误值经常要**活过输入文本**。输入可能是一个临时字符串，函数返回后就被释放；如果错误里存着指向它的 `&str`，编译器会要求错误类型标注生命周期（lifetime，引用必须保持有效的范围，第十四课系统讲），错误类型随之变得复杂。
- 错误值可能要在函数之间层层传递，最后被打印或记录。**拥有**自己的文本，错误就能自由地传到任何地方。

代价是多一次堆上的分配和复制。对错误路径来说这通常值得：错误本来就不频繁，而诊断信息丢了就很难找问题。

## 十三、完整示例：解析百分比

把本课到目前为止的内容组合起来，写一个比主练习小的完整程序：把 `"75"` 这样的文本解析成 0 到 100 的百分比。下面的程序可以原样保存成文件运行：

```rust
#[derive(Debug, PartialEq, Eq)]
enum PercentError {
    Empty,
    InvalidNumber(String),
    OutOfRange(u16),
}

fn parse_percent(input: &str) -> Result<u8, PercentError> {
    let trimmed = input.trim();

    if trimmed.is_empty() {
        return Err(PercentError::Empty);
    }

    let number = trimmed
        .parse::<u16>()
        .map_err(|_| PercentError::InvalidNumber(trimmed.to_owned()))?;

    if number > 100 {
        return Err(PercentError::OutOfRange(number));
    }

    Ok(number as u8)
}

fn main() {
    for input in ["75", " 100 ", "abc", "120"] {
        match parse_percent(input) {
            Ok(value) => println!("{input:?} => {value}%"),
            Err(error) => println!("{input:?} => {error:?}"),
        }
    }
}
```

运行输出：

```text
"75" => 75%
" 100 " => 100%
"abc" => InvalidNumber("abc")
"120" => OutOfRange(120)
```

逐段分析 `parse_percent`：

1. **先 trim，再判空**。`"  "` 这种全是空白的输入，去掉两端空白后就是空字符串，应该报 `Empty` 而不是扔给数字解析器。`trim` 返回去掉两端空白后的 `&str`。
2. **解析成 `u16` 而不是直接解析成 `u8`**。目标范围是 0 到 100，但如果直接解析 `u8`，`"120"` 和 `"300"` 都会笼统地变成“数字解析失败”，无法区分“数字本身写不出来”和“数字合法但超范围”。先用能装下 0 到 65535 的 `u16` 解析，就能用两个独立 variant 分别报告这两种情况。
3. **`map_err` 翻译错误并保留原文**。解析失败时，把标准库的 `ParseIntError` 换成 `InvalidNumber`，同时把 trim 后的输入文本存进去，调用者能看到用户到底输了什么。
4. **`?` 完成传播**。`map_err` 之后紧跟 `?`：解析成功就取出数字继续，失败就带着翻译好的错误提前返回。
5. **范围检查通过后，`u16` 转 `u8` 才安全**。能走到这里，数字一定在 0 到 100 之间，`as u8`（类型转换写法）不会丢信息。
6. **调用者决定怎么展示错误**。`main` 里用 `match` 分别处理成功和失败。真实程序里这里可能是提示用户重新输入，而不是打印。

## 十四、阅读真实编译器报错

下面的报错都来自真实编译（rustc，Rust 编译器；输出中的错误码如 `E0369` 可以用 `rustc --explain E0369` 查看官方详细解释）。学会读报错比记住语法更重要。

### 14.1 把 Option 当数字用：E0369

```rust,compile_fail
let value: Option<i32> = Some(10);
let result = value + 1;
```

```text
error[E0369]: cannot add `{integer}` to `Option<i32>`
   --> src/main.rs:3:24
    |
  3 |     let result = value + 1;
    |                  ----- ^ - {integer}
    |                  |
    |                  Option<i32>
```

读法：错误码 `E0369` 表示“这个运算符不支持这些操作数类型”。编译器标出了加号两边的类型：左边是 `Option<i32>`，右边是整数。修复方向不是给 `Option` 找加法，而是**先取出里面的值**——用 `match`、`unwrap_or` 或 `?`，取决于“没有值”时你想怎么办。

### 14.2 在没有失败通道的函数里用 `?`：E0277

```rust,compile_fail
fn parse_number(input: &str) -> i32 {
    let number = input.parse::<i32>()?;
    number
}
```

```text
error[E0277]: the `?` operator can only be used in a function that returns `Result` or `Option` (or another type that implements `FromResidual`)
 --> src/main.rs:2:38
  |
1 | fn parse_number(input: &str) -> i32 {
  | ----------------------------------- this function should return `Result` or `Option` to accept `?`
2 |     let number = input.parse::<i32>()?;
  |                                      ^ cannot use the `?` operator in a function that returns `i32`
```

读法：编译器直接说明了规则——`?` 只能用在返回 `Result` 或 `Option` 的函数里（`FromResidual` 是支撑这条规则的内部 trait，现在忽略）。修复方向：把返回类型改成 `Result<i32, 某个错误类型>`，成功路径用 `Ok(...)` 包起来。

### 14.3 混淆 Option 和 Result：E0308

```rust,compile_fail
fn find() -> Option<i32> {
    Err("missing")
}
```

```text
error[E0308]: mismatched types
 --> src/main.rs:2:5
  |
1 | fn find() -> Option<i32> {
  |              ----------- expected `Option<i32>` because of return type
2 |     Err("missing")
  |     ^^^^^^^^^^^^^^ expected `Option<i32>`, found `Result<_, &str>`
```

读法：`E0308`（mismatched types，类型不匹配）是最常见的错误码，意思是“这里声明的类型和你给的值对不上”。编译器会告诉你“期望什么、实际是什么”。记住对应关系：`Option` 的“没有”是 `None`，`Result` 的“失败”是 `Err(...)`，两者不能混用；想把 `Option` 变成 `Result` 要用第十一节的 `ok_or`。

### 14.4 match 漏掉 variant：E0004

```rust,compile_fail
enum ParseEndpointError {
    EmptyInput,
    MissingSeparator,
    EmptyHost,
}

fn describe(error: ParseEndpointError) -> &'static str {
    match error {
        ParseEndpointError::EmptyInput => "empty",
        ParseEndpointError::MissingSeparator => "no separator",
    }
}
```

```text
error[E0004]: non-exhaustive patterns: `ParseEndpointError::EmptyHost` not covered
 --> src/main.rs:8:11
  |
8 |     match error {
  |           ^^^^^ pattern `ParseEndpointError::EmptyHost` not covered
```

读法：`E0004` 是穷尽性检查（第二课）在报错：`match` 没有覆盖 `EmptyHost`。这正是第八节说的“给错误 enum 新增 variant 后，编译器逼你更新所有处理它的地方”。不要为了省事用 `_` 兜底吞掉所有没列出的 variant——那会把这个保护关掉，新增错误类型时编译器就再也提醒不了你。

### 14.5 认识测试失败输出

还没实现练习时运行 `cargo test -p l04_option_result`，你会看到每个测试都失败，原因类似：

```text
test exercise::tests::parse_port_accepts_valid_port_numbers ... FAILED
...
thread 'exercise::tests::parse_port_accepts_valid_port_numbers' (12345) panicked at lessons/04-option-result/src/exercise.rs:49:5:
not yet implemented
```

（括号里的数字是本次运行的线程编号，每次运行都不同；路径按仓库根目录显示，在本课目录里运行时会变成 `src/exercise.rs`。）

读法：第一行告诉你哪个测试失败（模块路径 `exercise::tests::` 加测试名）；`panicked at` 给出中止位置，`not yet implemented` 是 `todo!()` 触发的 panic 信息——意思是“这里是留给你的作业”。**测试因 `todo!()` 失败是预期状态**；如果换成编译错误，或者测试报出别的原因，才说明有问题。

## 十五、代码预测

先写下你的预测和理由，再展开核对。

### 题目 1

```rust
let value: Option<i32> = None;
let result = value.unwrap_or(5);
```

`result` 是几？

<details>
<summary>查看解析</summary>

`result` 是 `5`。`unwrap_or` 在 `None` 时返回括号里的默认值。

</details>

### 题目 2

```rust
let value = Some(3).map(|number| number * 2);
```

`value` 是什么？类型和值都要说。

<details>
<summary>查看解析</summary>

`value` 是 `Some(6)`，类型是 `Option<i32>`。`map` 保留外层结构：只转换 `Some` 里面的值，结果仍然包在 `Option` 里。

</details>

### 题目 3

```rust
fn parse(input: &str) -> Result<i32, std::num::ParseIntError> {
    let value = input.parse::<i32>()?;
    Ok(value + 1)
}
```

输入 `"abc"` 时，`value + 1` 这一行会执行吗？

<details>
<summary>查看解析</summary>

不会。`?` 看到 `Err` 的那一刻就从函数返回了这个 `Err`，后面的代码不会执行。

</details>

### 题目 4

```rust
let result: Result<i32, &str> = Err("bad");
let value = result.unwrap();
```

会发生什么？

<details>
<summary>查看解析</summary>

程序 panic 并中止，panic 信息里带着 `Err` 的内容 `"bad"`。对可预期的输入错误这样做是不合适的——失败应该通过 `Result` 交还给调用者。

</details>

### 题目 5

为什么错误 enum 的 variant 里常存 `String` 而不是 `&str`？

<details>
<summary>查看解析</summary>

因为错误值可能要活得比输入文本更久，还要在函数之间传递。存借用的 `&str` 会强迫错误类型携带输入的生命周期，让签名复杂化；存拥有的 `String`，错误就独立持有诊断文本，可以传到任何地方。

</details>

## 十六、主练习：解析 host:port

现在回到第一节的问题，亲手实现它。练习在 `src/exercise.rs`，类型定义已经给出，你要实现两个函数（把函数体里的 `todo!()` 换成你的代码）：

```rust
pub fn parse_port(text: &str) -> Result<u16, ParseEndpointError>;
pub fn parse_endpoint(input: &str) -> Result<Endpoint, ParseEndpointError>;
```

成功类型和错误类型：

```rust
pub struct Endpoint {
    pub host: String,
    pub port: u16,
}

pub enum ParseEndpointError {
    EmptyInput,          // 整个输入 trim 后为空
    MissingSeparator,    // 找不到 `:`
    EmptyHost,           // 主机名部分 trim 后为空
    InvalidPort(String), // 端口文本不是合法 u16，附带该文本
    ZeroPort,            // 端口数字为 0
}
```

### 16.1 行为规则（判断按此顺序，命中即返回）

1. 整个 `input` 去掉两端空白后为空 → `EmptyInput`。
2. 找不到第一个 `:` → `MissingSeparator`。
3. 第一个 `:` 之前的主机名部分，单独去掉两端空白后为空 → `EmptyHost`。
4. 第一个 `:` 之后的全部内容按端口规则处理（交给 `parse_port`）：trim 后解析不成 `u16` → `InvalidPort(去掉空白后的端口文本)`；解析结果是 `0` → `ZeroPort`；解析成功且非零 → 得到端口号。
5. 全部通过 → `Ok(Endpoint)`，`host` 是去掉两端空白后的主机名，`port` 是端口号。

注意规则 4：输入 `"host:8080:extra"` 时，第一个 `:` 之后的内容是 `"8080:extra"`，它不是合法数字，所以是 `InvalidPort("8080:extra")`。

### 16.2 输入与期望输出

| 输入 | 期望结果 |
| --- | --- |
| `"localhost:8080"` | `Ok(Endpoint { host: "localhost", port: 8080 })` |
| `" example.com : 443 "` | `Ok(Endpoint { host: "example.com", port: 443 })` |
| `""` 或 `"   "` | `Err(EmptyInput)` |
| `"localhost"` | `Err(MissingSeparator)` |
| `":8080"` | `Err(EmptyHost)` |
| `"localhost:abc"` | `Err(InvalidPort("abc"))` |
| `"localhost:70000"` | `Err(InvalidPort("70000"))` |
| `"localhost:"` | `Err(InvalidPort(""))` |
| `"localhost:8080:extra"` | `Err(InvalidPort("8080:extra"))` |
| `"localhost:0"` | `Err(ZeroPort)` |

（表中 `host: "..."` 指 `String` 内容的相等比较。）

### 16.3 约束

- 只替换两个函数体里的 `todo!()`；不修改类型定义、函数签名和测试。
- 不允许用 `unwrap`、`expect` 或 `panic!` 处理输入——所有失败都必须通过返回值表达。
- `parse_endpoint` 中至少使用一次 `?` 传播失败（本课的核心技能）。

### 16.4 推荐完成顺序

1. 先实现 `parse_port`：它的输入输出单纯，测完它再往上走。
2. 再实现 `parse_endpoint`：先处理空输入，再切分冒号、检查主机名，最后调用 `parse_port` 并组装 `Endpoint`。
3. 每完成一个函数就跑一次测试，不要攒到最后。

### 16.5 哪些测试会先失败

实现前，`src/exercise.rs` 里的 9 个测试全部因 `todo!()` 失败（见 14.5 的读法）。实现过程中可以先只跑单个测试，例如：

```bash
cargo test -p l04_option_result parse_port_accepts
```

### 16.6 验收命令

```bash
cargo fmt -p l04_option_result -- --check
cargo check -p l04_option_result --all-targets
cargo test -p l04_option_result
```

`cargo run -p l04_option_result` 会用几组输入演示你的 `parse_endpoint`：完成前它会在第一组输入处因 `todo!()` 中止，完成后应看到每种输入对应的解析结果或错误 variant。

## 十七、课后巩固：解析时长文本

巩固任务在 `src/consolidation.rs`，数据与边界都和主练习不同：把 `"90s"`（90 秒）、`"5m"`（5 分钟）、`"2h"`（2 小时）这样的时长文本换算成总秒数。你需要实现：

```rust
pub fn unit_seconds(unit: &str) -> Option<u64>;
pub fn parse_duration(input: &str) -> Result<u64, DurationError>;
```

- `unit_seconds`：认出 `"s"`、`"m"`、`"h"` 分别相当于 1、60、3600 秒，其他单位（包括空文本）返回“没有值”。它返回 `Option` 而不是 `Result`，因为“不认识这个单位”在查询函数里只是正常的没有结果；到了 `parse_duration` 的语境里，它才升级为错误。
- `parse_duration` 的行为规则（按此顺序）：整个输入 trim 后为空 → `DurationError::Empty`；开头没有数字 → `MissingNumber`；数字部分超出 `u64` 能表示的范围 → `InvalidNumber(数字文本)`；只有数字没有单位 → `MissingUnit`；单位 trim 后不被支持 → `UnknownUnit(单位文本)`；数值换算成秒时超出 `u64` 范围 → `Overflow`；否则返回 `Ok(总秒数)`。

示例：

| 输入 | 期望结果 |
| --- | --- |
| `"90s"` | `Ok(90)` |
| `" 2h "` | `Ok(7200)` |
| `"0s"` | `Ok(0)` |
| `""` | `Err(Empty)` |
| `"s"` | `Err(MissingNumber)` |
| `"90"` | `Err(MissingUnit)` |
| `"10d"` | `Err(UnknownUnit("d"))` |
| `"99999999999999999999999999s"` | `Err(InvalidNumber("99999999999999999999999999"))` |
| `"18446744073709551615h"` | `Err(Overflow)` |

最后两行是新边界：一个数大到 `u64`（无符号 64 位整数，最大约 1844 亿亿）都装不下，和数装得下、但乘上 3600 后装不下，是两种不同的失败，对应不同 variant。约束与主练习相同：不用 `unwrap`/`expect`/`panic!`，至少用一次 `?`。

验收命令：

```bash
cargo test -p l04_option_result consolidation
```

卡住时先看 `HINTS.md` 的一级提示。这个任务会用到主练习没用到的一点：`char_indices` 可以边遍历字符边拿到位置，`find` 返回 `Option`，乘法的“可能溢出”版本叫 `checked_mul`、也返回 `Option`——它们都是本课“用类型表达可能没有结果”的实例。

## 十八、常见误区

- **用特殊值同时表示正常结果和失败**，例如用端口 `0` 表示解析失败。这正是一开始要摆脱的写法。
- **对用户输入直接 `unwrap`**。用户输错是日常，程序不该因此中止。
- **为了能写 `?`，把所有错误都改成 `String`**。这丢掉了 enum 错误的可判断性和穷尽性检查。
- **丢掉导致失败的原始输入**。`InvalidPort(String)` 存原文，就是为了出错时能告诉用户“你写的这段不对”。
- **把 `None` 当 `Err` 用、把 `Err` 当 `None` 用**。它们是不同类型，混淆会得到 E0308（见 14.3）。
- **用 `_` 吞掉错误 enum 的所有剩余 variant**。新增错误类型时，编译器将无法再提醒你更新处理代码（见 14.4）。

## 十九、本课总结

1. `Option<T>` 表示“可能有值、可能没有”，`Result<T, E>` 表示“可能成功、可能失败并带原因”；选择标准是“没有”是否需要解释。
2. 两者都是普通 enum，必须先处理外层状态（`Some`/`None`、`Ok`/`Err`）才能用到里面的值——编译器强制这一点，这正是它们比特殊值和 null 安全的原因。
3. `?` 是“失败就提前 return”的简写，错误通过普通返回值显式传播，不是异常；它要求函数返回兼容的 `Option` 或 `Result`。
4. 自定义错误 enum 让失败原因结构化、可比较、可被穷尽处理；`map_err` 负责翻译错误类型，`ok_or` 负责把 `Option` 升级为 `Result`。
5. `unwrap`/`expect` 会把 `Err`/`None` 变成 panic，只适合测试、示例和程序自身保证的不变量。
6. 错误类型也有所有权问题：错误通常拥有 `String` 形式的诊断文本，以便活过输入、自由传递。

## 二十、课后复盘

用自己的话回答，答不出就回对应小节重读：

1. “没有值”和“发生错误”的区别是什么？各对应哪个类型？
2. 为什么不能对 `Option<i32>` 直接做加法？编译器用什么方式阻止你？
3. `?` 遇到 `Err` 时具体做了什么？它和异常有什么不同？
4. 为什么练习中 `InvalidPort` 存 `String` 而不是 `&str`？
5. 什么情况下 `unwrap` 是合理的？举两个不合理的场景。
6. 给错误 enum 新增一个 variant 后，所有 `match` 它的地方会怎样？这为什么是好事？

## 下一课预告

[第五课](../05-vec-slices/README.md)：学习 `Vec<T>`（可增长的列表）、slice（对一段连续数据的借用视图）和集合的所有权，并把本课的 `Result` 用在批量数据处理上——一批输入里有的能解析、有的不能，程序该怎么汇总。
