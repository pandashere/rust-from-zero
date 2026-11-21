# 第二十课：宏——让编译器在编译前替你生成代码

前十九课里，我们写的每一行代码都是自己敲出来的：函数、`struct`、`impl`、
测试，一项一项写。真实项目里常出现一种重复：**结构几乎一样，只有名字、
类型或数据不同**。例如十几个写法相同的测试，或者给一个结构体的每个字段
写一个几乎一样的读取方法。

复制粘贴能解决一次，但后面会出问题：改了一处忘了另一处，测试名字写错，
字段改名后旧方法还在。普通函数帮不上忙，因为这些重复发生在**代码的形状**
上，而不是运行时的值上。本课的主角**宏（macro）**就是为这种“生成代码的
重复”准备的：你写一份规则，**编译器**（compiler，把源代码翻译成可执行
程序的工具）在**编译期**按规则把调用替换成完整的代码，然后再做类型检查。

宏不是“更高级的函数”。它有自己的代价：错误信息可能指向展开后的代码，
阅读时多一层跳转，公开出去的宏还会变成长期兼容负担。所以本课有两条主线：
学会写声明宏，同时学会判断**什么时候不该写宏**。

## 本课定位

### 你将学会

- 说明宏和函数、泛型（generic，用类型占位符让同一份代码适用于多种类型的
  机制，[第十一课](../11-generics/README.md)详细讲过）各自适合解决什么问题。
- 用 `macro_rules!` 定义声明宏（declarative macro，按模式生成代码的宏）。
- 用 `expr`、`ident`、`ty`、`pat`、`path`、`literal`、`item`、`block`、`tt`
  这些**片段说明符**（fragment specifier，规定某个位置能匹配哪类语法）。
- 用 `*`、`+`、`?` 和分隔符匹配数量不定的内容，并允许尾逗号。
- 理解宏卫生（hygiene，宏内部名字与调用处名字的隔离规则）和 `$crate`。
- 避免同一个参数被求值多次，并让读者知道宏可能提前 `return`。
- 区分声明宏、derive 宏（自动为类型生成实现代码的宏）、属性（attribute，
  `#[...]` 形式的标注）宏（改写被标注项的宏）和函数式过程宏。
- 阅读常见的宏展开错误，并知道如何缩小排查范围。

### 前置知识

- 变量、函数、`if`、循环与基本类型：见[第一课](../01-basics/README.md)。
  简单复习：变量给一个值起名字；函数把一段可重复使用的计算包起来；
  `if` 按条件选择执行路径；循环重复执行一段代码。**类型**（type）规定
  一个值能表示什么数据、能做哪些操作。
- `struct`、字段、`impl`、方法与 `&self`：见[第二课](../02-data-modeling/README.md)。
  简单复习：`struct` 把几个值组合成一个新类型；`impl` 块里可以定义方法；
  `&self` 表示方法借用当前值来读，不把它移走。
- `Vec<T>`、`vec![]`、索引与 `len()`：见[第五课](../05-vec-slices/README.md)。
  简单复习：`Vec<T>` 是长度可变的同类型序列；`Vec::new()` 创建空序列；
  `vec![a, b]` 用给定元素创建序列。
- `Option`、`Result`、`Ok`/`Err` 与 `?`：见
  [第四课](../04-option-result/README.md)。简单复习：`Option<T>` 表示
  “可能有值，也可能没有”，两个变体是 `Some(v)` 和 `None`；
  `Result<T, E>` 表示“成功时是 `T`，失败时是 `E`”；`Ok(v)` 和 `Err(e)`
  是它的两个变体（variant，枚举中一种可能取值）。
- 所有权、移动与借用：见[第三课](../03-ownership-borrowing/README.md)。
  简单复习：每份数据有一个所有者，负责在合适的时候释放它；**移动**把
  所有权交给另一个变量；**借用**是临时访问数据而不取得所有权。
- 模块、crate 与 package：见[第七课](../07-modules-crates/README.md)。
  简单复习：一个 **package**（包）由 `Cargo.toml` 描述，可以包含一个库
  crate 和一个或多个可执行 crate；**crate** 是一次编译的单位；**模块**
  （module）是 crate 内部组织名字的树状结构。
- `String` 与 `&str`：见[第三课](../03-ownership-borrowing/README.md)。
  简单复习：`String` 是拥有自己文本数据的可变长字符串；`&str` 是借来看的
  一段文本，不能单独决定数据何时释放。
- 自动测试与 `#[test]`：见[第八课](../08-testing-quality/README.md)。
  简单复习：标了 `#[test]` 的函数会被测试运行器调用；**断言**（assertion，
  检查某个条件必须成立的语句）失败，或者代码调用 `panic!`（让程序立即
  中止并报告错误），都会让该测试失败。
- 闭包与迭代器：见[第九课](../09-closures/README.md)和
  [第十课](../10-iterators/README.md)。简单复习：闭包是一段可以保存起来
  以后调用的代码；**迭代器**（iterator）是按顺序访问一组值的方式。本课
  只在“求值次数”和“测试辅助”里用到很短的两者，第一次出现时会解释。

### 本课边界

本课**不要求**你写过程宏（procedural macro）。过程宏需要单独的
`proc-macro` crate、额外的依赖和更多构建配置；本课只用概念和例子说明它们
存在、适合什么场景。主练习和课后巩固都只用**标准库**（standard library，
Rust 安装时自带的代码集合），**不添加任何依赖**。

本课也不把宏当成“性能工具”或“高级写法”来推销。多数逻辑仍然应该写成
普通函数；宏只负责生成重复的**代码结构**。

### 如何学习本课

1. 先读“为什么需要宏”，再读语法。每段代码先预测行为，再看解释。
2. “真实编译诊断”一节不要跳过。宏的错误常出现在意想不到的位置，读诊断
   是主要技能。
3. 完成代码预测后，打开 `src/exercise.rs` 做主练习；卡住时按
   [`HINTS.md`](HINTS.md) 的三级提示逐级查看。
4. 主练习通过后，再做 `src/consolidation.rs` 的课后巩固。
5. 全程使用本课自己的 package 命令（**Cargo** 是 Rust 自带的构建和包管理
   工具），例如 `cargo test -p l20_macros`。

## 一、先看一个具体的重复问题

假设我们在写一个小商店程序，需要准备一批库存条目。每个条目有商品名和
数量，类型已经在 `src/exercise.rs` 里给出：

```rust
pub struct StockItem {
    pub name: String,
    pub quantity: u32,
}
```

`u32` 是只能表示非负整数的类型，范围是 0 到 4294967295；`pub` 表示这个
类型和字段允许其他模块使用（第七课讲过可见性）。库存数量用 `u32`，所以
不会出现负数。

如果手工创建三条数据，代码是：

```rust
let items = vec![
    StockItem::new("pen", 12),
    StockItem::new("paper", 3),
    StockItem::new("eraser", 0),
];
```

三条还能接受。可是设想三十条、三百条，或者这些数据分散在多个测试里：
每一行都以 `StockItem::new(` 开头，以 `),` 结尾，中间只有名字和数字不同。
这种重复有三个具体困难：

1. **函数不能改变“参数个数”。** 普通函数的参数列表在定义时就固定了。
   想要 `make_items!("pen", 12, "paper", 3, ...)` 支持任意多项，函数签名
   无法表达；泛型也只能让**类型**变化，不能让**参数个数**变化。
2. **函数调用发生在运行时。** 有些重复需要生成的是“声明”本身，例如
   一批测试函数或一批方法。这些东西必须在程序运行之前就存在，函数调用
   来不及创建它们。
3. **手工复制容易漏改。** 名字、逗号、括号、字段顺序，任何一处不一致都
   可能变成编译错误，或者更糟：测试名字重复、覆盖了另一条测试。

宏解决的正是这类问题：它接收的不是运行时的值，而是一段**源代码的片段**，
然后按规则生成新的源代码。

### 编译期和运行期

先建立两个时间概念：

- **编译期（compile time）**：编译器把你写的源文件翻译成可执行程序的阶段。
  类型检查、借用检查都发生在这里。
- **运行期（run time）**：程序已经启动、正在处理数据的阶段。

宏展开（macro expansion）发生在编译期，而且**早于类型检查**。编译器先按
宏规则把调用替换成代码，再检查替换后的完整代码。这解释了两个常见现象：

- 宏内部的类型错误，报错位置可能指向宏定义或调用处，而不是你直觉中的
  “函数体”；
- 宏可以生成 `struct`、`fn`、`impl` 这些普通函数无法在运行时创建的东西。

### token、表达式、语句、项

读宏文档时会反复遇到几个词，这里一次说清：

**语法**（syntax）指代码允许怎样组合的规则，例如函数名后面必须跟括号、
每条语句用什么分隔；**语义**（semantics）指这些组合实际表示什么行为。

- **token（词法单元，也叫标记）**：源代码被切分后的最小单位。一个名字、
  一个数字、一个逗号、一对括号，都是 token。`StockItem`、`::`、`new`、
  `(`、`"pen"` 是五个 token。
- **token tree（token 树）**：token 按括号成组后的结构。`(a, b)` 是一组，
  里面的 `a`、`b` 又是更小的单元；`[1, 2]` 也是。宏的 `tt` 片段匹配的就是
  这种带括号的单元。
- **表达式（expression）**：能计算出一个值的代码片段，例如 `1 + 2`、
  `StockItem::new("pen", 1)`、`if ready { 1 } else { 0 }`。
- **语句（statement）**：执行一个动作的代码单元，例如 `let x = 1;` 或
  `println!("hi");`。语句通常以分号结束。
- **项（item）**：写在模块或文件顶层、用来“声明某个东西”的语法单位，
  例如 `fn`、`struct`、`enum`、`impl`、`use`、`mod`。`#[test] fn ...` 也是
  一个项。

宏调用能出现在哪种位置，决定了它必须展开成哪一类语法：在表达式位置就要
展开成表达式，在项位置就要展开成项。后面会看到具体例子。

## 二、为什么 `println!` 和 `vec!` 带感叹号

你从第一课就在用宏，只是当时没有点名：

```rust
let name = "pen";
let count = 12;
println!("name={name}, count={count}");
```

`println!` 后面的 `!` 表示“这是宏调用，不是函数调用”。它需要宏，原因很具体：

- 参数个数不固定：可以打印零个、一个、五个值；
- 参数类型不固定：整数、字符串、结构体都能打印；
- 格式字符串在编译期检查：`"{name}"` 里的名字必须真的存在，类型也要能
  按对应格式输出。普通函数接收的是运行时的值，无法在编译期检查格式文本
  与参数名是否匹配。

`vec!` 也一样，支持两种不同形状的写法：

```rust
let first = vec![1, 2, 3];   // 列出每个元素
let second = vec![0; 100];   // 重复同一个值 100 次
```

这两句展开成的代码结构不同：前者依次放入三个元素，后者按“值 + 次数”
创建。普通函数没法用同一个签名同时表达这两种参数形状。宏可以分别为它们
写两条规则。

## 三、最小的声明宏

声明宏用 `macro_rules!` 定义。先看一个不接收参数的版本：

```rust
macro_rules! say_hello {
    () => {
        println!("hello");
    };
}

fn main() {
    say_hello!();
}
```

运行输出：

```text
hello
```

逐项拆开：

- `macro_rules!`：声明“下面要定义一个声明宏”。它本身不是函数，也不会在
  运行时执行。
- `say_hello`：宏的名字。调用时写 `say_hello!`，感叹号不能省。
- `()`：**匹配模式**（pattern）。这里的空括号表示“调用时括号里没有内容”。
  调用 `say_hello!()` 正好匹配。
- `=>`：后面是**展开模板**（transcription），也就是要生成的代码。
- `{ println!("hello"); }`：生成的代码。调用处会被替换成这个块。
- 规则末尾的分号 `;`：分隔多条规则；只有一条规则时也可以保留。

一个宏可以有多条规则。编译器**从上到下**依次尝试，使用第一条匹配成功的
规则；如果都不匹配，就报“no rules expected ...”。

宏定义要出现在使用之前（在同一个模块的源文件顺序里）。本课练习文件里，
宏定义都放在测试模块上方，原因就在这里。若宏加了 `#[macro_export]`，它还
会被放到 crate 根，其他 crate 可以用 `crate名::宏名!` 调用。

## 四、捕获表达式：`$value:expr`

固定输出不够用。下一个需求是“计算一个值的平方，但只计算一次”。先看宏：

```rust
macro_rules! square {
    ($value:expr) => {{
        let value = $value;
        value * value
    }};
}

fn main() {
    let mut count = 0;
    let answer = square!({
        count += 1;
        count + 3
    });

    println!("answer={answer}, count={count}");
}
```

输出：

```text
answer=16, count=1
```

这里出现了几个新符号：

- `$value`：**元变量**（metavariable），宏规则里的“占位捕获”。名字由你
  起，习惯用小写；调用时匹配到的内容可以通过 `$value` 在展开模板中使用。
- `:expr`：**片段说明符**，规定 `$value` 只能匹配一个表达式。`3 + 1`、
  `foo()`、`{ count += 1; count + 3 }` 都是表达式；单独一个 `let` 语句不是。
- `{{ ... }}`：外层花括号是规则的展开块，内层花括号让展开结果成为一个
  **块表达式**，同时也是 `let value` 的作用域边界。宏展开后大致相当于：

```rust
let answer = {
    let value = {
        count += 1;
        count + 3
    };
    value * value
};
```

为什么先写 `let value = $value;`？如果直接展开成 `$value * $value`，那么
调用 `square!({ count += 1; count + 3 })` 时，`count += 1` 会执行两次。
宏只是复制 token，不会自动帮你避免重复求值。这个主题在第十节还会展开。

### 捕获的名字和调用处的名字互不干扰

注意宏里的 `value` 和调用处的 `count` 不是一回事。宏作者可以放心使用
`value` 这个局部名字，不必担心调用者也有同名变量。这种隔离叫**宏卫生
（hygiene）**，第八节会专门解释。

## 五、多条规则与固定关键字

宏规则里的固定 token 可以当成小型语法关键字。例如一个只接受 `debug` 或
`display` 的日志宏：

```rust
macro_rules! log_value {
    (debug, $value:expr) => {
        eprintln!("[debug] {:?}", $value);
    };
    (display, $value:expr) => {
        eprintln!("[info] {}", $value);
    };
}

fn main() {
    log_value!(display, "ready");
    log_value!(debug, vec![1, 2]);
}
```

输出（`eprintln!` 写到标准错误流，终端里通常也会显示）：

```text
[info] ready
[debug] [1, 2]
```

`debug` 和 `display` 不是变量，而是规则里固定的 token：调用者必须原样写出
其中一个，否则两条规则都不匹配。`{:?}` 是“调试格式”，要求类型实现了
`Debug`；`{}` 是“显示格式”，要求实现了 `Display`。这些是标准库 trait
（行为接口）提供的格式化能力，[第十二课](../12-traits/README.md)系统讲过
trait。

固定关键字适合**少量、稳定**的分支。如果选项会不断增长，普通 `enum` 加
一个函数通常更好维护：新增分支时类型检查会提醒你处理所有情况，而宏的
关键字只是文本，漏掉一个只会在调用处报“没有匹配规则”。

## 六、常见片段说明符

`$value:expr` 中的 `expr` 只是众多片段类型之一。常用的有：

| 说明符 | 匹配的内容 | 例子 | 典型用途 |
|---|---|---|---|
| `expr` | 表达式 | `a + 1`、`make()` | 传入要计算的值 |
| `ident` | 标识符（名字） | `name`、`parse_case` | 生成函数名、字段名、变量名 |
| `ty` | 类型 | `u32`、`Vec<String>` | 生成参数或返回类型 |
| `pat` | 模式 | `Some(x)`、`1..=5` | 生成 `match` 分支 |
| `path` | 路径（带 `::` 的名字） | `std::mem::drop` | 传入函数或类型的完整名字 |
| `literal` | 字面量 | `42`、`"text"` | 只接受写死的常量 |
| `item` | 一个项 | `fn f() {}`、`struct S;` | 生成声明 |
| `block` | 一个花括号块 | `{ do_work() }` | 只接受块表达式 |
| `tt` | 一个 token tree | 一个 token 或一组括号内容 | 需要原样转发语法时 |

原则是**能用精确的说明符就用精确的**。`tt` 几乎什么都能接，看起来灵活，
但它把语法检查推迟到更深处：调用者写错时，错误可能出现在展开后的一大段
代码里，而不是清晰地指出“这里需要表达式”。`expr` 会在匹配阶段就拒绝
明显不是表达式的内容，错误更早、更准。

`stringify!` 可以把一段 token 原样变成字符串，常用于观察宏收到了什么：

```rust
fn main() {
    let name = stringify!(some_identifier);
    let message = concat!("prefix", "-", "suffix");

    println!("{name} {message}");
    assert_eq!(name, "some_identifier");
    assert_eq!(message, "prefix-suffix");
}
```

输出：

```text
some_identifier prefix-suffix
```

`stringify!` 不会计算 `some_identifier`，它只是把 token 拼成文本；`concat!`
在编译期拼接字符串字面量。两者都由标准库提供，不需要额外依赖。

这段代码用到的 `assert_eq!` 是标准库的断言宏：两个值相等时什么也不做，
不相等时让程序 `panic!` 并打印两边的值，因此很适合写在示例和测试里。

## 七、重复匹配：数量不定的输入

现在处理“任意多项”的核心工具：**重复（repetition）**。先写一个把若干
数字相加的宏：

```rust
macro_rules! sum_all {
    ($($value:expr),+ $(,)?) => {
        0 $(+ $value)+
    };
}

fn main() {
    let total = sum_all![1, 2 + 3, 4];
    println!("total={total}");
    assert_eq!(total, 10);
}
```

输出：

```text
total=10
```

拆解匹配模式 `($($value:expr),+ $(,)?)`：

- `$(` ... `)+`：把中间的内容重复**一次或多次**；
- `$value:expr`：每一轮捕获一个表达式；
- `),+` 里的 `,`：重复项之间用逗号分隔；
- `$(,)?`：在主要重复之后，允许零个或一个逗号，也就是可选的尾逗号。

展开模板里的 `0 $(+ $value)+` 表示：先写 `0`，然后对每个捕获重复写
`+ $value`。三个元素展开后等价于 `0 + 1 + (2 + 3) + 4`：`$value` 捕获的
`2 + 3` 是一个整体表达式。`0` 让“至少一项”的规则也能自然累加；如果允许
零项，可以用 `*` 并让初值承担空输入。

三个重复运算符：

- `*`：零次或多次；
- `+`：一次或多次；
- `?`：零次或一次。

分隔符可以换成其他 token，例如分号或 `=>`。分隔符必须与调用处的写法
一致；`$( $value:expr ),*` 不会接受用分号分隔的列表。

### 并行重复

一条规则里可以有多个捕获一起重复，只要它们在同一个重复层：

```rust
macro_rules! show_all {
    ($($label:expr => $value:expr),+ $(,)?) => {
        $(
            println!("{} = {}", $label, $value);
        )+
    };
}

fn main() {
    show_all!["pen" => 12, "paper" => 3];
}
```

输出：

```text
pen = 12
paper = 3
```

`$label` 和 `$value` 在每一轮里成对出现：第一轮是 `"pen"` 和 `12`，第二轮
是 `"paper"` 和 `3`。展开模板的 `$( ... )+` 里同时使用两者，编译器就知道
按同一轮配对。如果两个捕获来自**不同层级**的重复，展开时又放在同一层，
就会得到“repeats ... times, but ... repeats ... times”一类错误。

### 嵌套重复

重复可以嵌套。下面的宏按行打印一个数字表，外层重复每一行，内层重复一行
里的每个数字：

```rust
macro_rules! show_grid {
    ($([$($value:expr),* $(,)?]),+ $(,)?) => {
        $(
            println!("row:");
            $(
                println!("  {}", $value);
            )*
        )+
    };
}

fn main() {
    show_grid![[1, 2,], [3,],];
}
```

输出：

```text
row:
  1
  2
row:
  3
```

注意内层使用 `*`，所以 `[]` 这样的空行也能匹配：它只是不打印任何数字。
外层使用 `+`，要求至少有一行；完全没有行的调用要用另一条空规则处理。
行内尾逗号和整个调用的尾逗号分别由两个 `$(,)?` 负责。课后巩固的
`matrix!` 正是这个结构的变体，只是把打印换成构造 `Vec<Vec<i64>>`。

## 八、宏卫生：名字为什么不打架

看这个宏：

```rust
macro_rules! with_temp {
    ($value:expr) => {{
        let temporary = $value;
        temporary + 1
    }};
}

fn main() {
    let temporary = 100;
    let result = with_temp!(3);

    println!("result={result}, temporary={temporary}");
    assert_eq!(result, 4);
    assert_eq!(temporary, 100);
}
```

输出：

```text
result=4, temporary=100
```

宏内部绑定的 `temporary` 不会覆盖调用者的同名变量。**宏卫生（hygiene）**
指的是：宏定义中引入的标识符带有自己的语法上下文，默认不会意外解析到
调用处的同名名字；调用处传入的 token 则仍然属于调用处的上下文。

卫生能避免很多意外，但不是万能的：

- 宏可以通过捕获使用调用者传入的标识符，这本来就是它的工作；
- 宏生成的**公开项**（例如函数、结构体）会进入调用处的命名空间，名字仍
  可能冲突；
- 宏展开里调用的方法或 trait，可能要求调用处已经导入某个 trait；宏作者
  应尽量使用完整路径，减少这种隐藏依赖；
- 宏内部的 `use` 可能影响作用域，公开宏应谨慎。

本课练习要求宏在“别的 crate”里也能直接调用，就是为了逼你面对这些隐藏
依赖，而不是只在定义它的模块里碰巧能跑。

## 九、`$crate` 与导出宏

一个 crate 里定义的宏，展开后可能引用同 crate 的其他项。如果直接写
`StockItem::new`，调用者必须刚好导入了 `StockItem`，否则展开失败。宏作
者不能假设调用处的 `use` 长什么样。

`$crate` 是宏里的特殊路径前缀，指向**定义这个宏的 crate 根**。看一个完整
的两 crate 例子。

库 crate `loghelpers`：

```rust
pub mod logging {
    pub fn write(message: &str) {
        println!("[log] {message}");
    }
}

#[macro_export]
macro_rules! public_log {
    ($message:expr) => {
        $crate::logging::write($message)
    };
}
```

使用它的 crate `app`，在 `Cargo.toml` 里把依赖改名成 `renamed`：

```toml
[dependencies]
renamed = { package = "loghelpers", path = "../loghelpers" }
```

```rust
fn main() {
    renamed::public_log!("ready");
}
```

输出：

```text
[log] ready
```

即使依赖被改名，`$crate` 仍然指向 `loghelpers`，展开后的
`loghelpers::logging::write(...)` 依然正确。若当初写成
`crate::logging::write`，它在 `app` 里就会指向 `app` 自己的根，直接失败。

`#[macro_export]` 把声明宏放到 crate 根命名空间，供外部使用。注意这不是
“免费”的：宏的名字和它接受的语法一旦公开，就和公开函数签名一样，构成
兼容性承诺。不要为了少写几行就把内部宏导出。

## 十、求值次数：宏会复制 token

宏不是函数调用。函数参数在进入函数前各求值一次；宏只是把 token 复制到
展开位置，复制几次就可能求值几次。看一个错误版本：

```rust
macro_rules! bad_max {
    ($left:expr, $right:expr) => {
        if $left > $right { $left } else { $right }
    };
}
```

调用 `bad_max!(left(), right())` 时，条件里各调用一次，被选中的分支又调用
一次。用计数器观察：

代码里的 `|| { ... }` 是**闭包**（closure，一段可以保存起来、以后像函数
一样调用的代码，第九课讲过）；`Cell`（`std` 是标准库的名字）是标准库提供的
小容器，允许通过共享引用修改里面的值（第十五课讲的内部可变性工具）。

```rust
use std::cell::Cell;

fn main() {
    let left_calls = Cell::new(0);
    let right_calls = Cell::new(0);

    let left = || {
        left_calls.set(left_calls.get() + 1);
        3
    };
    let right = || {
        right_calls.set(right_calls.get() + 1);
        8
    };

    let value = bad_max!(left(), right());
    println!(
        "value={value}, left={}, right={}",
        left_calls.get(),
        right_calls.get()
    );
}
```

输出：

```text
value=8, left=1, right=2
```

`right` 被调用了两次：一次比较，一次返回。改成先各绑定一次：

```rust
macro_rules! max_once {
    ($left:expr, $right:expr) => {{
        let left = $left;
        let right = $right;
        if left > right { left } else { right }
    }};
}
```

把同一段 `main` 里的 `bad_max!` 换成 `max_once!`，输出变成：

```text
value=8, left=1, right=1
```

这里还有所有权细节：`let left = $left;` 会把值移动进局部变量。对 `i32`
这类实现了 `Copy`（按位复制即可安全复制的类型）的类型没问题；对 `String`
这类拥有堆数据（堆是 heap，运行时按需申请的内存区域，
[第三课](../03-ownership-borrowing/README.md)讲过）的类型，调用者传入的
表达式如果是一个变量，就会被移动。
宏的文档应该和函数文档一样，说明求值、移动、借用和返回语义。若不想移动，
可以考虑绑定引用，但那会改变返回类型，需要根据需求选择。

本课主练习要求每个商品名和数量表达式**只求值一次**；`ensure!` 还要求错误
表达式只在失败时求值。两者都是在训练“先想清楚 token 被复制到几条路径”。

## 十一、控制流宏：展开出 `return`

函数不能替调用者提前返回，宏可以，因为宏可以把 `return` 直接放进调用者
的函数体。一个只处理 `Option` 的例子：

```rust
macro_rules! require_some {
    ($option:expr, $message:expr $(,)?) => {
        match $option {
            Some(value) => value,
            None => {
                eprintln!("{}", $message);
                return None;
            }
        }
    };
}

fn first_positive(values: &[i64]) -> Option<i64> {
    let value = require_some!(
        values.iter().copied().find(|value| *value > 0),
        "no positive value"
    );
    Some(value)
}

fn main() {
    println!("{:?}", first_positive(&[-3, 4, 5]));
    println!("{:?}", first_positive(&[-3, -4]));
}
```

输出：

```text
Some(4)
no positive value
None
```

这段代码里，`&[i64]` 是借来看的一段整数（**切片** slice，第五课讲过），
`i64` 是可以表示正数和负数的 64 位整数。`iter().copied().find(...)` 依次
查看每个数字，返回第一个满足条件的值；没找到就是 `None`。

当 `find` 返回 `None` 时，宏展开出的 `return None;` 结束的是
`first_positive`，不是宏（宏本身没有“函数体”）。这是控制流宏的关键：
**读调用处时必须知道它可能提前结束当前函数**。名字和文档要把这件事说
清楚，例如 `ensure!`、`bail!` 这类名字通常暗示“可能返回”。

对比一下普通写法：

```rust
let Some(value) = values.iter().copied().find(|value| *value > 0) else {
    eprintln!("no positive value");
    return None;
};
```

`let Some(value) = ... else { ... };` 的意思是：如果匹配 `Some(value)`，
就绑定 `value` 并继续；否则执行 `else` 块，而 `else` 块必须用 `return`、
`break` 或 `panic!` 提前离开当前函数。

这种 `let ... else` 写法没有宏，控制流一眼可见，通常是更清楚的选择。
控制流宏适合重复出现、模式稳定的检查；只出现一两次时，直接写 `if` 或
`?` 往往更好。

`?` 运算符也能提前返回错误，但它要求函数返回 `Result` 或 `Option`；对
`Result` 来说，它还会把错误交给 `From` 转换。若你想返回自定义错误、附带
上下文，或者错误构造比较昂贵、只在失败时才做，自定义宏或显式 `if` 仍然
有意义。选择顺序建议是：**普通 `if`/`?` → 辅助函数 → 宏**。

## 十二、生成项：方法、测试与声明

宏不只能生成表达式，还能生成项。下面这个宏为结构体字段生成读取方法：

```rust
macro_rules! make_getter {
    ($method:ident, $field:ident, $ty:ty) => {
        fn $method(&self) -> &$ty {
            &self.$field
        }
    };
}

struct User {
    name: String,
}

impl User {
    make_getter!(name, name, String);
}

fn main() {
    let user = User {
        name: String::from("Ada"),
    };
    println!("user name = {}", user.name());
}
```

输出：

```text
user name = Ada
```

`make_getter!` 必须写在 `impl` 块里，因为它展开成的是一个**关联函数**
（associated function，写在 `impl` 里的函数）；在这里它是一个方法，因为
第一个参数是 `&self`。`$method:ident` 捕获方法名，`$field:ident` 捕获字段
名，`$ty:ty` 捕获返回类型。三个位置分别选择了最贴近目标的片段类型：方法名
和字段名必须是标识符，类型位置用 `ty` 表达“这里要一个类型”。若图省事全用
`tt`，语法检查会被推迟，错误也更难读。

这种宏确实能减少机械重复，但也要比较代价：调用处只看到一行
`make_getter!(...)`，读者必须知道宏展开成什么；字段改名时，宏参数可能
不会自动跟着改；IDE（集成开发环境，带编辑、跳转和补全功能的编程工具）的
“查找引用”也可能看不到生成的方法。字段少的时候，
手写两三个方法通常更清楚。

### 完整示例：生成多个测试

宏最常见的正当用途之一是表驱动测试（table-driven tests）：一张“输入 →
期望输出”的表，每行生成一个测试函数。

```rust
macro_rules! table_tests {
    (
        mod $module:ident for $function:path {
            $(
                $name:ident: $input:expr => $expected:expr
            ),+ $(,)?
        }
    ) => {
        mod $module {
            use super::*;

            $(
                #[test]
                fn $name() {
                    let actual = $function($input);
                    assert_eq!(actual, $expected, "case `{}`", stringify!($name));
                }
            )+
        }
    };
}

fn double(value: i32) -> i32 {
    value * 2
}

table_tests! {
    mod double_cases for double {
        zero: 0 => 0,
        positive: 4 => 8,
        negative: -3 => -6,
    }
}
```

运行 `cargo test`，会看到三个测试：

```text
running 3 tests
test double_cases::negative ... ok
test double_cases::positive ... ok
test double_cases::zero ... ok
```

拆解：

- `mod $module:ident`：捕获模块名，生成 `mod double_cases { ... }`；
- `for $function:path`：捕获一个函数路径，例如 `double` 或
  `crate::math::double`；
- `$name:ident: $input:expr => $expected:expr`：每行捕获测试名、输入和期望；
- `$( ... ),+ $(,)?`：允许一行或多行，并允许尾逗号；
- `#[test] fn $name()`：在项位置生成完整的测试函数；
- `assert_eq!(actual, $expected, "case `{}`", stringify!($name))`：失败时
  额外打印用例名，方便定位是哪一行数据出错；
- `use super::*;`：生成的模块是独立作用域，需要把外层（通常是测试模块）
  的名字带进来，否则 `double` 这样的相对路径找不到。

`table_tests!` 的调用必须位于**项位置**（模块里、其他项旁边），不能写在
函数体中间期待它生成“半个函数”。宏调用所在位置期待哪类语法，展开就必须
提供哪类语法。

这类宏的取舍：它省掉了大量重复的 `#[test] fn ...` 外壳，但错误位置会指向
宏展开；测试名和数据的对应关系需要靠 `stringify!` 或断言消息补足。生成
少量测试时，手写往往更直接；测试表很长、格式固定时，宏才明显划算。

## 十三、过程宏：另一类宏

声明宏基于 token 模式匹配。Rust 还有**过程宏（procedural macro）**：它们
本质上是 Rust 函数，接收一段 token 流，返回另一段 token 流，可以执行任意
Rust 代码来生成结果。常见三类：

- **derive 宏**：为 `struct`/`enum` 自动生成实现，例如
  `#[derive(Debug, Clone)]`。`Debug` 让值可以用 `{:?}` 打印，`Clone` 让值
  可以显式复制。
- **属性宏**：改写或替换被标注的项，例如测试框架提供的
  `#[tokio::test]`（需要外部依赖，本课不引入），或某些 Web 框架的路由标注。
- **函数式过程宏**：调用形式像声明宏，例如 `sql!("select ...")`，但内部是
  函数在处理 token。

过程宏必须定义在单独的 `proc-macro` crate 中（`Cargo.toml` 里写
`proc-macro = true`），不能和普通库写在同一个 crate 里。它们通常需要
`syn`、`quote` 这类解析和生成 token 的依赖，构建配置也更复杂。

过程宏看到的是**语法 token**，不是编译器完整的类型信息。它们能做声明宏
做不到的复杂解析，但代价是：编译更慢、错误更难定位、依赖更多、公开
API（application programming interface，这里指库对外提供的名字和用法）
更难演进。多数项目里，derive 宏来自成熟库，业务代码只需要使用，不需要
自己写。

本课不引入过程宏依赖。你要掌握的是：看到 `#[derive(...)]` 或
`#[some_attribute]` 时知道它可能在编译期生成代码；以及在没有强需求时，
不要为了“高级”去自建过程宏。

## 十四、真实编译诊断

宏的错误常出现在两个位置：调用处，或宏定义里的匹配模式。下面几段输出是
真实的 `rustc`（Rust 编译器）/Cargo 诊断（版本 1.94），用来说明怎么读。

### 1. 调用的形状和所有规则都不匹配

```rust
use l20_macros::stock;

fn main() {
    let items = stock!["pen" => 12, "paper", "ink"];
    println!("{items:?}");
}
```

诊断：

```text
error: no rules expected `,`
  --> src/main.rs:4:44
   |
 4 |     let items = stock!["pen" => 12, "paper", "ink"];
   |                                            ^ no rules expected this token in macro call
   |
note: while trying to match `=>`
  --> src/exercise.rs:67:19
   |
67 |     ($($name:expr => $quantity:expr),+ $(,)?) => {{
   |                   ^^
```

读法：

- 第一行说明在匹配某条规则时遇到了没料到的 `,`；
- `note` 指出当时正在尝试哪条规则，以及它期待的 token（这里期待 `=>`）。
  说明 `"paper"` 被当成了“名字”，接下来应该写 `=> 数量`，却遇到了逗号；
- 规则是**整段匹配**的，不会把两种形式自动拼接。要支持混合形式，需要
  另外设计规则（本课不要求）。

### 2. 调用提前结束

```rust
use l20_macros::stock;

fn main() {
    let items = stock!["pen" => 12, "paper"];
    println!("{items:?}");
}
```

诊断：

```text
error: unexpected end of macro invocation
  --> src/main.rs:4:44
   |
 4 |     let items = stock!["pen" => 12, "paper"];
   |                                            ^ missing tokens in macro arguments
   |
note: while trying to match `=>`
  --> src/exercise.rs:67:19
   |
67 |     ($($name:expr => $quantity:expr),+ $(,)?) => {{
   |                   ^^
```

同样是“形状不对”：第二项已经开始匹配 `$name:expr`，规则期待后面的 `=>`，
但调用已经结束。

### 3. 展开后类型不匹配

```rust
use l20_macros::stock;

fn main() {
    let items = stock!["pen" => "many"];
    println!("{items:?}");
}
```

数量位置传了字符串。假设 `stock!` 的展开模板已经正确调用
`StockItem::new`，它的第二个参数要求 `u32`，就会得到：

```text
error[E0308]: mismatched types
  --> src/main.rs:4:33
   |
 4 |     let items = stock!["pen" => "many"];
   |                 ----------------^^^^^^-
   |                 |               |
   |                 |               expected `u32`, found `&str`
   |                 arguments to this function are incorrect
   |
note: associated function defined here
  --> src/exercise.rs:22:12
   |
22 |     pub fn new(name: impl Into<String>, quantity: u32) -> Self {
   |            ^^^
```

错误不一定写在“宏调用”上，而会指向展开后的函数调用。看到宏相关的类型
错误时，先把宏展开在脑中还原成普通代码，再按普通类型错误处理。

### 4. 重复层级不一致

```rust
macro_rules! mismatch {
    ($($left:expr),+ ; $($right:expr),*) => {
        vec![$(($left, $right)),*]
    };
}

fn main() {
    let _ = mismatch!(1, 2; 3);
}
```

诊断：

```text
error: meta-variable `left` repeats 2 times, but `right` repeats 1 time
 --> src/main.rs:3:15
  |
3 |         vec![$(($left, $right)),*]
  |               ^^^^^^^^^^^^^^^^^
```

`$left` 来自“至少一次”的重复，`$right` 来自“零次或多次”的另一个重复；
在展开时把它们放进同一个 `$( ... )*`，编译器无法知道怎样配对。解决方向
不是加括号，而是让两个捕获来自同一个重复层，或者分别写两个重复块。

### 5. 用 `compile_error!` 给自定义提示

如果某种输入永远不该被接受，可以主动生成一条清楚的错误：

```rust
macro_rules! mode {
    (fast) => {
        true
    };
    (safe) => {
        false
    };
    ($other:tt) => {
        compile_error!("expected `fast` or `safe`")
    };
}

fn main() {
    let _fast = mode!(fast);
    let unknown = mode!(turbo);
    println!("{unknown}");
}
```

调用 `mode!(turbo)` 时：

```text
error: expected `fast` or `safe`
  --> src/main.rs:9:9
   |
 9 |         compile_error!("expected `fast` or `safe`")
   |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
...
15 |     let unknown = mode!(turbo);
   |                   ------------ in this macro invocation
   |
   = note: this error originates in the macro `mode` (in Nightly builds, run with `-Z macro-backtrace` to show the backtrace)
```

`compile_error!` 适合“语法上能匹配，但**语义**（semantics，代码实际表示的
行为）上不允许”的情况。注意它在
`mode!` 位于表达式位置时，末尾不要多写分号，否则可能额外触发
“trailing semicolon in macro used in expression position”的前向兼容错误。
把它放在语句位置或项位置时，分号则更自然。

### 6. 忘记感叹号

```rust
use l20_macros::stock;

fn main() {
    let items = stock["pen"];
    println!("{items:?}");
}
```

诊断（节选）：

```text
error[E0423]: expected value, found macro `stock`
 --> src/main.rs:4:17
  |
4 |     let items = stock["pen"];
  |                 ^^^^^ not a value
  |
help: use `!` to invoke the macro
  |
4 |     let items = stock!["pen"];
  |                      +
```

编译器知道 `stock` 是一个宏，但这里被当成值使用。宏调用必须带 `!`。

### 测试失败时先看 panic 位置

脚手架尚未实现时，运行测试会看到类似：

```text
thread 'exercise::tests::stock_accepts_empty_list' panicked at src/exercise.rs:...:
not yet implemented
```

这是 `todo!()` 的固定信息，说明执行到了学习者负责的位置，不是测试写错。
实现一部分后，某些测试可能从“panic”变成“断言失败”；那时再按行为约定
检查数据、顺序和边界。

## 十五、代码预测

先自己回答，再展开解析。

### 预测 1

使用第四节的 `square!`：

```rust
let mut count = 0;
let result = square!({
    count += 1;
    count
});
```

`count += 1` 执行几次？`result` 是多少？

<details><summary>解析</summary>

一次。`square!` 先把整个表达式求值并绑定到内部的 `value`，之后使用该值
两次。`count` 从 0 变成 1，表达式的结果是 1，所以 `result = 1 * 1 = 1`。
若宏直接展开成 `$value * $value`，`count += 1` 会执行两次，结果也会不同。

</details>

### 预测 2

使用 `bad_max!` 和 `max_once!`，两个参数都是带计数器的函数调用：

```rust
let value = bad_max!(left(), right());
```

假设 `left()` 返回 3，`right()` 返回 8。`right` 被调用几次？换成
`max_once!` 后呢？

<details><summary>解析</summary>

`bad_max!` 中 `right` 两次：一次在 `$left > $right` 比较里，一次在
`else { $right }` 返回时。`max_once!` 先分别绑定 `left` 和 `right`，之后
只用局部变量，因此各一次。

</details>

### 预测 3

`table_tests!` 里为什么测试名用 `$name:ident`，而不是 `$name:expr`？

<details><summary>解析</summary>

因为捕获值要放在 `fn $name()` 的函数名位置。这个位置只接受标识符，不接受
表达式片段：即使用 `$name:expr` 捕获一个裸名字，展开到函数名位置时编译器
也会报“expected identifier, found metavariable”。`ident` 明确表示“这里要
一个名字”，能在匹配阶段就限定形状。选择片段说明符时，要看捕获值最终落在
哪类语法位置。

</details>

### 预测 4

```rust
let items = stock!["pen" => 12, "paper"];
```

这段调用能匹配 `stock!` 的哪条规则？为什么？

<details><summary>解析</summary>

三条规则都不匹配。空规则要求括号里没有内容；带 `=>` 的规则要求每一项都写
`名字 => 数量`，第二项缺少 `=>`；只有名字的规则要求所有项都不带 `=>`，
第一项又不满足。宏规则不会“逐项混合匹配”，一次调用必须整体符合某一条
规则。主练习因此要求：一次调用要么全写数量，要么全用默认数量。

</details>

## 十六、主练习：用宏生成库存清单

主练习在 `src/exercise.rs`，包含两个宏和一个已经写好的函数。你只需要实现
宏的展开部分；类型、函数签名和测试不要改。

### 目标

1. 实现 `stock!`：根据三种调用形式生成 `Vec<StockItem>`。
2. 实现 `ensure!`：条件不成立时从调用者函数提前返回 `Err(...)`。
3. 让 `take_stock` 在 `ensure!` 的支持下正确工作（函数体已给出）。

### 输入与期望输出

类型定义（已给出）：

```rust
pub struct StockItem {
    pub name: String,
    pub quantity: u32,
}

pub enum StockError {
    NotEnough { available: u32, requested: u32 },
}
```

`stock!` 的调用与结果：

```rust
let a = stock!["pen" => 12, "paper" => 3];
// 结果：vec![StockItem::new("pen", 12), StockItem::new("paper", 3)]

let b = stock!["ink", "ruler"];
// 结果：vec![StockItem::new("ink", 1), StockItem::new("ruler", 1)]

let c: Vec<StockItem> = stock![];
// 结果：空 Vec

let d = stock!["pen" => 12,];
let e = stock!["pen",];
// 都允许一个可选的尾逗号，结果分别只有一项
```

列表顺序必须保持书写顺序。商品名可以是任意表达式，只要它能交给
`StockItem::new`；数量可以是任意 `u32` 表达式。数量 `0` 合法。

每个商品名表达式和每个数量表达式在展开后**只能出现一次**，因此最多求值
一次。测试用带计数器的块表达式检查这一点。

`ensure!` 的调用与结果：

```rust
ensure!(condition, error_expression);
ensure!(condition, error_expression,); // 允许尾逗号
```

- 条件为 `true`：什么也不做，函数继续；
- 条件为 `false`：调用者函数立即返回 `Err(error_expression)`；
- 错误表达式只在失败时求值，成功路径不构造错误值。

`take_stock` 的行为（函数体已给出）：

```rust
take_stock(&StockItem::new("pen", 10), 3)  // Ok(7)
take_stock(&StockItem::new("pen", 10), 10) // Ok(0)
take_stock(&StockItem::new("pen", 10), 0)  // Ok(10)
take_stock(&StockItem::new("pen", 2), 5)   // Err(NotEnough { available: 2, requested: 5 })
```

### 约束
- 不修改 `StockItem`、`StockError`、`take_stock` 的签名和文件底部测试。
- 不添加依赖，不修改 `Cargo.toml`。
- 宏展开不能依赖调用处的 `use`：构造条目时使用 `$crate::exercise::...`
  这样的完整路径。
- 不要用 `unwrap`、`expect` 或 `panic!` 处理正常的库存不足；错误通过
  `Result` 返回。
- 每个捕获在展开模板里最多出现一次。

### 推荐完成顺序

1. 先读 `src/exercise.rs` 顶部的文档注释，运行
   `cargo test -p l20_macros stock_accepts_empty_list`，确认失败原因是
   `todo!()` 的 panic。
2. 实现 `stock!` 的空清单规则，让 `stock_accepts_empty_list` 通过。
3. 实现写明数量的规则，让 `stock_builds_items_with_explicit_quantities`、
   `stock_keeps_zero_quantities`、`stock_accepts_trailing_commas` 通过。
4. 实现默认数量的规则，让 `stock_defaults_quantity_to_one` 通过。
5. 检查求值次数，让 `stock_evaluates_each_expression_once` 通过。
6. 实现 `ensure!`，让四个 `take_stock_*` 测试和三个 `ensure_*` 测试通过。
7. 运行完整命令，包括外部 crate 的集成测试。

### 实现前会失败的测试

`stock!`、`ensure!` 尚未实现时，以下 13 个单元测试都会失败（大多直接
panic，个别表现为断言不符）：

```text
stock_builds_items_with_explicit_quantities
stock_defaults_quantity_to_one
stock_accepts_trailing_commas
stock_accepts_empty_list
stock_keeps_zero_quantities
stock_evaluates_each_expression_once
take_stock_returns_remaining_quantity
take_stock_allows_taking_everything
take_stock_allows_zero_request
take_stock_reports_shortage_with_both_numbers
ensure_returns_early_with_the_error
ensure_does_not_build_the_error_on_success
ensure_evaluates_the_condition_once_and_accepts_a_trailing_comma
```

`tests/public_macros.rs` 里还有 3 个集成测试，它们从**另一个 crate** 的
角度调用宏；实现正确后才会通过：

```text
stock_works_outside_its_crate
ensure_works_outside_its_crate
matrix_works_outside_its_crate
```

其中 `matrix_works_outside_its_crate` 依赖课后巩固的 `matrix!`，所以先做
主练习时它仍会失败，这是预期现象。

### 验收命令

```bash
cargo test -p l20_macros exercise
cargo test -p l20_macros
cargo test -p l20_macros --no-run
cargo clippy -p l20_macros --all-targets -- -D warnings
cargo fmt -p l20_macros -- --check
```

第一条只跑主练习相关单元测试；第二条会跑全部测试（含课后巩固与集成测试）。
实现完成前，失败是学习流程的一部分；但**编译错误和测试自身错误**不算，
需要先修好。

## 十七、课后巩固：生成二维数字表

课后巩固在 `src/consolidation.rs`，数据和边界都与主练习不同：这次是数字
表格，涉及嵌套重复、空行和长短不一的行。

### 目标

实现 `matrix!`，根据方括号行生成 `Vec<Vec<i64>>`。

### 输入与期望输出

```rust
let a = matrix![[1, 2, 3], [4, 5, 6]];
// vec![vec![1, 2, 3], vec![4, 5, 6]]

let b = matrix![[1], [2, 3, 4], [5, 6]];
// 行长度可以不同

let c = matrix![[], [7], []];
// 允许空行：vec![vec![], vec![7], vec![]]

let d: Vec<Vec<i64>> = matrix![];
// 空表：没有任何行

let e = matrix![[1, 2,], [3,],];
// 行内尾逗号和最后一行后的尾逗号都允许
```

每个数字可以是任意 `i64` 表达式（包括负数、加减乘除），并且只求值一次。
行的顺序必须保持书写顺序。

### 约束与推荐顺序

- 不修改宏的调用形式、测试和文件说明。
- 不添加依赖。
- 先让空表规则通过，再处理“一行或多行”，最后检查空行和两层尾逗号。
- 展开时不要把数字捕获写两次。

### 验收命令

```bash
cargo test -p l20_macros consolidation
cargo test -p l20_macros matrix
```

课后巩固有 7 个测试：

```text
matrix_builds_rows_in_order
matrix_accepts_ragged_rows
matrix_accepts_empty_rows
matrix_accepts_empty_matrix
matrix_accepts_trailing_commas
matrix_keeps_negative_and_computed_values
matrix_evaluates_each_value_once
```

## 十八、常见误区

1. **把宏当函数。** 宏在编译期展开，不会在运行时“调用”；宏里的
   `return` 结束的是调用者函数。看到 `!` 就要意识到代码形状可能改变。
2. **为了省三行代码写宏。** 宏会增加阅读跳转、错误定位和兼容性成本。
   输入是运行时值、参数个数固定时，普通函数更合适。
3. **过度使用 `tt`。** `tt` 什么都能接，但会把错误推到展开后。优先
   `expr`、`ident`、`ty`、`path` 等精确说明符。
4. **重复层级对不上。** 匹配时两个捕获在不同重复里，展开时却放进同一个
   重复块，会得到“repeats ... times”错误。先画外层/内层结构再写模板。
5. **参数被求值多次。** 展开模板里同一个 `$value` 出现两次，就会求值两次。
   需要多次使用时，先绑定到局部变量，并想清楚移动和借用。
6. **忘记尾逗号。** 调用者很自然会写 `stock!["pen" => 1,]`。匹配模式里加
   `$(,)?` 才能接受；展开模板不需要为它生成代码。
7. **以为卫生是万能的。** 宏内部名字不会意外捕获调用者变量，但宏生成的
   公开项仍可能重名，展开里用到的 trait 仍可能要求调用处已导入。
8. **依赖调用处的 `use`。** 宏在别的 crate 展开时，调用处不会自动导入你
   模块里的名字。使用 `$crate` 和完整路径，并用外部集成测试验证。
9. **在错误的语法位置调用宏。** 宏调用所在位置决定它必须展开成表达式、
   语句还是项。不能在函数体中间期待它生成“半个 `match`”再和周围拼接。
10. **把复杂业务逻辑塞进宏。** 宏适合生成薄结构；算法、校验和错误处理
    留在普通函数里，才能获得正常的类型错误、测试和调试体验。
11. **为了让测试通过而改测试。** 测试描述的是公开行为；改测试等于改需求。
    卡住时看 `HINTS.md`，不要删断言。

## 十九、本课总结

- 函数处理运行时的值，泛型抽象类型关系，宏处理**语法**并生成代码。
- `macro_rules!` 通过规则匹配 token；规则从上到下尝试，整段匹配。
- `expr`、`ident`、`ty`、`pat`、`path`、`literal`、`item`、`block`、`tt`
  规定捕获的语法类别；越精确，错误越早、越清楚。
- `$( ... )*`、`$( ... )+`、`$( ... )?` 处理数量不定的输入；分隔符和
  `$(,)?` 决定调用形状，重复层决定捕获如何配对。
- 宏卫生隔离宏内部名字与调用处名字，但不阻止公开项重名或隐藏的 trait
  依赖；导出宏用 `$crate` 引用定义 crate 的内部项。
- 宏会复制 token：同一个捕获出现多次就会求值多次；控制流宏会从调用者
  函数提前返回，必须在名字和文档里说清楚。
- 过程宏分为 derive、属性和函数式三类，通常位于单独的 `proc-macro` crate，
  需要额外依赖；多数场景优先使用函数、泛型和现成的 derive。
- 判断标准不是“能不能写成宏”，而是“重复是否发生在语法结构上、收益是否
  大于阅读和错误定位的成本”。

## 下一课

宏仍然受**安全 Rust**（safe Rust，不写 `unsafe` 的普通 Rust 代码）的规则
约束：它生成的是普通 Rust 代码，借用检查、类型检查一个都不会少。下一课
进入 [`unsafe`、裸指针与 FFI](../21-unsafe-ffi/README.md)：`unsafe` 是
标注“这段操作需要程序员额外证明安全条件”的关键字；裸指针（raw pointer）
是不自动保证指向有效数据的地址；FFI（外部函数接口）是与其他语言互相调用
的机制。下一课的重点不是“关闭编译器”，而是在少数操作上由程序员证明额外
不变量，并用安全接口把证明封装起来。
