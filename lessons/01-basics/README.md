# 第一课：第一个 Rust 程序——绑定、表达式与控制流

这是整个课程的第一课，假设你**没有学过任何编程语言**。你只需要会做最基本的电脑操作：打开文件夹、新建和保存文本文件。其余每一件工具、每一个术语，本课都会在使用前用日常语言解释。

学完本课，你将写出第一个真正能运行的 Rust 程序：一个“整数区间分析器”——给它一段整数范围，它能求和、统计满足条件的数字、把结果限制在指定上下界之内。更重要的是，你会理解代码为什么这样写，而不是背下几行语法。

预计学习时间 2～3 小时，包含练习。

## 本课定位

### 你将学会

- 认识终端、编译器、Cargo 这三件工具，并能用命令行编译、运行、测试一个程序。
- 区分 package、crate、target、module 这几个描述代码组织方式的词（都会就近解释）。
- 用 `let` 给数据起名字，理解为什么 Rust 默认不允许改、什么时候用 `mut` 允许改。
- 认识最常用的基础数据类型（整数、小数、真假值、字符等）和“类型推断”。
- 区分“表达式”和“语句”，理解一个分号为什么会改变代码的含义。
- 编写函数：给它输入（参数），让它产出结果（返回值）。
- 使用 `if`、`while`、`loop`、`for` 控制程序的执行顺序。
- 看懂 Rust 编译器最常见的报错，并按报错找到问题。

### 前置知识

无。本课是课程的起点。

### 本课暂不深入

本课的示例会用到文字（如 `"Hello"`）和函数传参，但不会系统讲解文字在内存中如何存放——那涉及 Rust 最核心的“所有权”规则，会在[第三课](../03-ownership-borrowing/README.md)专门学习。本课也不涉及自定义数据类型（[第二课](../02-data-modeling/README.md)）、集合（[第五课](../05-vec-slices/README.md)）和错误处理（[第四课](../04-option-result/README.md)）。遇到这些概念时，正文会给出当前够用的临时解释。

## 开始学习前：认识三件工具

写程序这件事，本质上是：你用一种编程语言在文本文件里写下指令，再由工具把这些指令翻译成电脑能直接执行的形式。这个过程需要三件工具。

### 终端与命令行

**终端**（terminal）是一个用键盘输入文字命令、用文字查看结果的窗口，又叫“命令行”。平时我们用鼠标点击图标让电脑做事；在终端里，我们输入一行命令、按回车，让电脑做事。

本课的所有操作都在终端里完成。一条命令通常长这样：

```bash
cargo --version
```

- `cargo` 是要运行的工具的名字。
- `--version` 是传给它的参数（附加说明），意思是“告诉我你的版本号”。

在终端中还有一个重要概念：**当前目录**（也叫工作目录），即终端“正站在”哪个文件夹里。很多命令的效果取决于当前目录。用 `cd 路径`（change directory，切换目录）可以移动，例如：

```bash
cd /home/你的用户名/learning/rust-from-zero
```

本课所有 `cargo` 命令都应当在本课程仓库的根目录（能看到 `Cargo.toml`、`lessons/` 等内容的那个目录）下运行。

### 编译器

电脑处理器只认识由 0 和 1 组成的机器指令，人类写不出也读不了那种东西。于是我们发明了人类可读的**编程语言**，再用一个叫**编译器**（compiler）的程序，把人类写的**源代码**（source code，就是你写的文本）整体翻译成电脑可执行的文件。

“先整体翻译、后执行”带来一个直接的好处：很多错误（比如写错了语法、把数字当文字用）在翻译阶段就会被编译器发现并报告，程序根本不会带着这些错误运行起来。本课第九节专门练习阅读这类报告。

Rust 的编译器叫 `rustc`。但你几乎不会直接敲 `rustc`，因为下一件工具会替你调用它。

### Cargo

一个真实项目不止一个源文件：还有版本信息、需要的外部代码库、测试、文档。**Cargo** 是 Rust 官方的项目管理工具，负责记住这些信息，并用一条命令完成编译、运行、测试、格式化（自动整理代码排版）等工作。

先确认工具已经装好。在终端运行：

```bash
cargo --version
```

如果输出了类似 `cargo 1.94.0 (...)` 的一行文字，说明一切就绪。如果提示“找不到命令”，请先按课程仓库根目录 `README.md` 的说明安装 Rust。

本课你会反复使用这几个 Cargo 命令（现在只需看一眼，后面会逐个用到）：

| 命令 | 做什么 |
|---|---|
| `cargo check -p l01_basics` | 只检查代码有没有错误，不运行。反馈最快，写代码时高频使用 |
| `cargo run -p l01_basics` | 编译并运行本课程序 |
| `cargo test -p l01_basics` | 编译并运行本课的自动测试 |
| `cargo fmt -p l01_basics` | 自动整理本课代码的排版 |

（`-p l01_basics` 的意思是“只针对名为 l01_basics 的这个 package”，package 是什么马上讲。）

## 一、从源代码到程序：Cargo 到底在管理什么

### 1.1 一个具体的问题

打开本课程仓库，你会看到二十多课的内容、近两百个文件。当你输入 `cargo run` 时，Cargo 怎么知道你想运行哪一份代码？一份代码和另一份代码之间会不会互相干扰？

为了回答这个问题，Cargo 用四个层级的概念组织代码。请对照下表和后面的目录图理解：

| 术语 | 中文解释 | 本课中的例子 |
|---|---|---|
| package（包） | 一个完整、独立的项目单元，由它自己的 `Cargo.toml` 文件描述。`Cargo.toml` 是一个文本配置文件，写着这个单元的名字、版本等信息 | `l01_basics`，也就是 `lessons/01-basics/` 这个文件夹 |
| crate（crate 是 Rust 的造词，可理解为“编译单元”） | Rust 编译器一次编译所处理的一整份代码。一个 crate 最终变成一个可执行程序，或一个供别人调用的**库**（library，一批被打包好、供其他程序使用的函数） | 本课有两个 crate：一个库 crate，一个可执行 crate |
| target（构建目标） | package 声明“请把我编译成什么”的具体目标。一个 package 可以有多个 target | 本课的库 target 和可执行 target |
| module（模块） | crate **内部**把代码分组的机制，一个文件可以是一个 module | 本课库 crate 里的 `exercise` 和 `consolidation` |

层级关系是：package 包含若干 target，每个 target 编译成一个 crate，crate 内部再用 module 划分。

再看本课 package 的真实结构：

```text
lessons/01-basics/
  Cargo.toml            # package 的“身份证”：名字 l01_basics、版本、依赖等信息
  README.md             # 你正在读的文件
  HINTS.md              # 练习卡住时的分级提示
  src/
    lib.rs              # 库 crate 的入口，声明它包含哪些 module
    exercise.rs         # module exercise：主练习的三个函数和测试
    consolidation.rs    # module consolidation：课后巩固的两个函数和测试
    main.rs             # 可执行 crate 的入口：一个演示程序
```

两个关键约定：

- `src/lib.rs` 存在时，Cargo 自动把它当作**库** target，crate 名字与 package 相同（`l01_basics`）。库本身不能“运行”，它的函数供其他代码调用。
- `src/main.rs` 存在时，Cargo 自动把它当作**可执行** target。可执行程序必须从名叫 `main` 的函数开始运行，`main.rs` 里就写着它。本课的 `main.rs` 会调用库里的函数做一次演示。

所以“运行本课程序”实际是：编译库 crate → 编译可执行 crate（它引用了库）→ 运行可执行文件。这些 Cargo 一条命令全包了。

### 1.2 为什么命令里有 `-p l01_basics`

整个课程仓库的根目录也有一个 `Cargo.toml`，它把各课组织成一个 **workspace（工作区）**——一组共享同一个构建目录的 package 的集合。因此在仓库根目录运行命令时，要用 `-p`（package 的缩写）指明针对哪个 package，例如 `cargo test -p l01_basics` 就是“只测试第一课”。

### 1.3 `todo!()` 与三种并不矛盾的结果

打开 `src/exercise.rs`，你会看到每个练习函数的函数体里写着一个 `todo!()`。它是一个“占位符”：意思是“这里还没实现，先占住位置”。它的存在让脚手架（已经搭好结构、等你填肉的代码）能通过各种检查。请亲自验证下面三种结果——它们看起来矛盾，其实各自回答不同的问题：

1. `cargo check -p l01_basics` **成功**。因为 `todo!()` 可以暂时冒充任何类型的返回值，类型检查挑不出错。
2. `cargo run -p l01_basics` **编译成功，但运行时崩溃**。真实输出如下：

   ```text
   thread 'main' (12345) panicked at lessons/01-basics/src/exercise.rs:31:5:
   not yet implemented
   ```

   （括号里的数字是本次运行的线程编号，每次运行都不同；路径按仓库根目录显示。若先 `cd lessons/01-basics` 再运行 `cargo run`，路径会变成 `src/exercise.rs`。）

   这种“运行到一半被迫中止”的情况叫 **panic（崩溃）**：程序遇到了无法继续的状况，主动停止并报告位置。这行报告的意思是：“程序在 `src/exercise.rs` 第 31 行停下了，原因是遇到了还没实现的代码（not yet implemented）。”
3. `cargo test -p l01_basics` **编译成功，但 11 个测试全部失败**，失败原因同样是 `not yet implemented`。

也就是说：**编译通过只说明代码“写得通”，不说明“做得对”**。行为是否正确要靠测试和实际运行验证。这是本课要建立的第一个重要观念。

## 二、最小的 Rust 程序

先看一个完整的、可以运行的最小程序：

```rust
fn main() {
    println!("Hello, Rust!");
}
```

把这三行存进一个 `main.rs`，编译运行后，终端会显示一行 `Hello, Rust!`。逐部分解释：

- `fn` 是 function（函数）的缩写。**函数**是一段有名字的代码：给它一些输入，它执行一系列操作，并可以给出一个结果。定义函数用 `fn 名字(参数) { 函数体 }`。
- `main` 是一个特殊的函数名：可执行程序运行时，电脑从 `main` 函数的第一行开始执行。
- 空圆括号 `()` 表示这个函数不需要输入。
- 花括号 `{ }` 之间的部分是**函数体**，即这个函数要做的事情，按从上到下的顺序执行。
- `println!("Hello, Rust!");` 的作用是“在终端打印一行文字”（print line，打印一行）。

### `println!` 为什么带着一个感叹号

`println!` 不是普通函数，而是一个**宏**（macro）。宏是“在编译时帮你生成代码的代码”。现在不需要理解宏的原理，只需记住：**名字后面带 `!` 的调用是宏**，写法上照抄即可。（宏的原理在第二十课学习。）

### 把数据放进输出里

`println!` 的第一个参数是一段**字符串字面量**（string literal，用双引号括起来的一段固定文字）。在文字里写 `{变量名}`，输出时这个位置会被替换成该变量当前的值：

```rust
fn main() {
    let language = "Rust";
    let year = 2026;

    println!("language = {language}");
    println!("year = {year}");
    println!("{language} in {year}");
}
```

输出：

```text
language = Rust
year = 2026
Rust in 2026
```

这里的 `let` 是下一节的主题，先有个印象：它给数据起一个名字。

## 三、绑定：`let`、`mut`、shadowing 与 `const`

### 3.1 具体的问题

程序要处理数据，而数据需要名字。假设你在写一个计分程序：玩家得分一开始是 0，后来要变成 100、250。一个天真而合理的想法是：给数据起名之后，随时可以改它的值。很多语言确实默认如此。

但“随处可改”有个真实的困难：程序一长，你在第 200 行读到一个名字时，无法确定它是不是在第 57 行、第 133 行被悄悄改过。读代码的人被迫在脑中追踪每一次可能的修改， Bug（程序中的错误）常常就藏在这些意外的改动里。

Rust 的选择是：**默认不允许改，想改必须明确声明**。给数据起名字这件事，Rust 称为**绑定**（binding）——把一个名字和一个值“绑”在一起，用 `let`（“令……为……”）建立：

```rust
fn main() {
    let count = 3;
    println!("{count}");
}
```

`let count = 3;` 读作“把名字 `count` 绑定到值 `3`”。这个绑定是**不可变的**（immutable）：一旦建立，`count` 就不能再被赋新值。下面的代码无法通过编译（`rust,compile_fail` 是课程中的标记，表示“这段代码故意是错的，编译会失败”）：

```rust,compile_fail
let count = 3;
count = 4;  // 错误：不能给不可变绑定重新赋值
```

（`//` 开头到行尾是**注释**：写给人看的说明，编译器完全忽略。）

不可变带来的回报是双向的：你读代码时，只要没看到 `mut`，就能确信这个名字的值从头到尾没变过；编译器也能在你无意中写错时立刻拦住。

### 3.2 `mut`：明确声明“这个要改”

确实需要修改时（比如循环中的计数器），在 `let` 后加上 `mut`（mutable，“可变的”的缩写）：

```rust
fn main() {
    let mut count = 3;
    count = count + 1;  // 取出 count 的旧值加 1，再把结果重新绑定给 count
    count += 1;         // “+=”是“在原有基础上加”的简写

    println!("{count}"); // 输出 5
}
```

`mut` 只允许改值，不允许改值的**类型**（类型的严格定义见下一节，先理解为“数据的种类”）。数字的名字不能被改绑到一段文字上：

```rust,compile_fail
let mut value = 10;
value = "ten";  // 错误：10 是整数，"ten" 是文字，种类不同
```

### 3.3 shadowing（遮蔽）：用同一个名字建立一个新绑定

另一种“变”的方式是干脆再写一次 `let`：

```rust
fn main() {
    let value = 10;
    let value = value + 5;  // 右边的 value 还是旧绑定（10），左边建立了新绑定（15）
    let value = value * 2;  // 同理，新绑定的值是 30

    println!("{value}"); // 输出 30
}
```

第二个 `let value = ...` 不是“修改”旧绑定，而是**建立一个全新的同名绑定**，从此旧绑定被**遮蔽**（shadowing）——名字 `value` 从此指向新值，旧值再也够不着了。

因为是全新绑定，所以类型也可以变：

```rust
fn main() {
    let spaces = "   ";        // 一段文字
    let spaces = spaces.len(); // 一个数字：这段文字的长度。len() 表示“求长度”

    println!("{spaces}"); // 输出 3
}

```

什么时候用哪个？一个经验：

- 一个状态要**反复**更新（如循环计数、累加结果）：用 `mut`，因为“它确实是同一个东西在变”。
- 一个数据经过**少数几步**加工、每步之后旧形态就没用了：可以用 shadowing，逐步换个更贴切的新形态。

### 3.4 `const`：编译期就定死的常量

有些值是程序里永远固定的规则参数，比如“最多重试 3 次”。这时用 `const`（constant，常量）：

```rust
const MAX_RETRIES: u32 = 3;
```

与 `let` 的区别：

- `const` 必须显式写出类型（`:` 后面的 `u32`，下一节解释）。
- 它的值必须在编译期就能算出来，不能依赖运行时才有的数据。
- 命名惯例是全大写、单词间用下划线。

日常代码里绝大多数“不会变的值”用普通 `let` 就够了；只有真正的固定规则参数才用 `const`。

### 3.5 先报名字，后给值

Rust 允许先声明一个绑定、稍后再赋值，但编译器会严格检查：**在读取之前，每条可能的执行路径都必须给它赋过值**：

```rust
fn main() {
    let value: i32;   // 先声明：名字 value，类型 i32，暂时没值

    if 2 > 1 {
        value = 10;
    } else {
        value = 20;
    }

    println!("{value}");
}
```

如果删掉 `else` 分支，编译器会拒绝：万一条件不成立，`value` 就是没被赋过值的空名字，读取它没有意义。Rust 不允许程序读到这种“空值”，这类隐患在编译期就被消除了。

不过本例更地道的写法是让 `if` 直接产出值（见第七节）：`let value = if 2 > 1 { 10 } else { 20 };`。

## 四、基础类型与类型推断

### 4.1 什么是类型，什么是推断

**类型**（type）是“数据的种类”：它决定一份数据占多少内存、能做什么操作。`3` 是整数，`"3"` 是文字，`3.0` 是小数——对人类来说它们是“三”的三种写法，对电脑来说是三种不同种类的数据，能做的事完全不同（文字不能开平方，整数可以）。

Rust 是**静态类型**语言：每个数据、每个表达式的类型在**编译期**就全部确定，而不是等到运行时才看。配合的是**类型推断**（type inference）：大多数时候你不用手写类型，编译器根据上下文自动推断。注意推断不等于“没有类型”——类型一直在，只是编译器帮你写了。

### 4.2 整数类型

整数类型按两个维度区分：能不能为负、占多少二进制位（bit，位越多能表示的数越大）。名字规律是：`i`（integer，整数，可正可负）或 `u`（unsigned，无符号，即只能是非负）+ 位数。

| 可正可负 | 只能非负 | 位数 | 范围示例 |
|---|---|---|---|
| `i8` | `u8` | 8 | `u8`：0 到 255 |
| `i16` | `u16` | 16 | |
| `i32` | `u32` | 32 | `i32`：约 ±21 亿 |
| `i64` | `u64` | 64 | |
| `i128` | `u128` | 128 | |

没有任何其他线索时，Rust 把整数字面量推断为 `i32`——它在多数场景下够用且高效：

```rust
fn main() {
    let a = 10;            // 推断为 i32
    let b: i64 = 10;       // 显式写明类型：冒号后跟类型名
    let c = 10_u32;        // 字面量后面直接带类型后缀
    let big = 1_400_000_000_u64; // 下划线只是千分位分隔符，方便阅读，不影响值

    println!("{a} {b} {c} {big}");
}
```

怎么选？默认 `i32`；“数量、次数”这类不可能为负的语义，用 `u32` 之类的无符号类型能更好地表达意图（本课练习里的计数函数就返回 `u32`）。

### 4.3 Rust 不悄悄转换数值类型

不同种类的整数不能直接混算：

```rust,compile_fail
let signed: i32 = 10;
let unsigned: u32 = 20;
let total = signed + unsigned; // 错误：i32 和 u32 不能直接相加
```

这不是刁难。“悄悄转换”意味着编译器替你猜——猜错了就是隐蔽的 Bug。Rust 要求你显式写清意图，用 `as` 做**显式类型转换**：

```rust
fn main() {
    let signed: i32 = 10;
    let unsigned: u32 = 20;
    let total = signed + unsigned as i32; // “把 unsigned 当作 i32 来用”

    println!("{total}"); // 30
}
```

注意 `as` 可能改变值（比如一个很大的 `u32` 转成 `i32` 会变负）。所以看到类型不匹配时，先想清楚“这两个类型为什么不同、转换是否合理”，不要用 `as` 硬压报错。能报告失败的转换方式（`try_from`）会在[第四课](../04-option-result/README.md)学习。

### 4.4 整数溢出

每种整数都有范围上限。一个 `u8` 最大是 255，那 `255 + 1` 会发生什么？这叫**溢出**（overflow）。Rust 在开发时默认的构建模式下会让程序 panic 来暴露这个问题；标准库还提供了 `checked_add`（加不了就如实报告失败）等显式方法，它们涉及[第四课](../04-option-result/README.md)的 `Option` 类型。本课练习保证输入足够小、不会溢出，你只需要知道“整数有界”这件事。

### 4.5 小数、真假值与字符

```rust
fn main() {
    let ratio: f64 = 0.75;   // f64：64 位浮点数（floating point，小数的近似表示）
    let enabled: bool = true; // bool：布尔值，只有 true（真）和 false（假）两种取值
    let letter: char = 'R';   // char：单个字符，用单引号
    let chinese: char = '锈'; // 一个汉字也是一个 char

    println!("{ratio} {enabled} {letter} {chinese}");
}
```

要点：

- 小数字面量默认推断为 `f64`。浮点数是**近似**值（`0.1 + 0.2` 不精确等于 `0.3`），本课不用它做精确计算。
- `bool` 只有两个值，是 `if` 等判断的唯一合法条件类型（详见第七节）。
- `char` 用单引号，双引号是字符串。一个 `char` 是一个 **Unicode**（统一码，给全世界每个文字符号编号的标准）字符，所以汉字、emoji 都算一个 `char`，但它不一定只占一个字节。

### 4.6 tuple 与 array：两种“打包”方式

**tuple（元组）**把固定数量、类型可以不同的几个值打包成一个值，用圆括号：

```rust
fn main() {
    let point: (i32, i32) = (3, 5);
    let x = point.0;              // 用 .序号 取成员，序号从 0 开始数
    let (x_again, y_again) = point; // 也可以一次拆开

    println!("{x} {x_again} {y_again}"); // 3 3 5
}
```

**array（数组）**装固定数量、类型相同的值，用方括号。`[i32; 4]` 读作“4 个 i32 的数组”，长度是类型的一部分：

```rust
fn main() {
    let values: [i32; 4] = [10, 20, 30, 40];
    let first = values[0];   // 用 [序号] 取元素，序号从 0 开始
    let zeros = [0_i32; 5];  // 简写：5 个 0

    println!("{first} {}", zeros[4]); // 10 0
}
```

（`println!` 里写 `{}` 空花括号时，按顺序取后面的参数填进去。）

两种打包的长度都在编译期固定。长度能变化的列表 `Vec` 在[第五课](../05-vec-slices/README.md)学习。

### 4.7 单位类型 `()`：“没有有用结果”的值

有的操作做了事，但没有什么结果值得交出，比如“打印一行字”。Rust 用**单位类型** `()`（读作 unit）表示这种情况：它只有一个值，也写作 `()`，含义是“执行完了，仅此而已”。

```rust
fn print_message() {   // 没写返回值类型，等价于 -> ()
    println!("done");
}
```

记住 `()`，下一节和第九节的报错都会用到它。

## 五、表达式、语句与分号

### 5.1 具体的问题

考虑这个需求：根据会员与否决定折扣金额，并存入一个名字。很多语言里你只能“先声明、再在两个分支里分别赋值”。Rust 提供了一种更直接的说法：让**判断本身产出一个值**。要理解它，先要分清两个概念。

**表达式**（expression）是“算得出一个值”的代码。下面每一行都是表达式：

```rust
1 + 2                    // 值是 3
true                     // 值是 true
double(4)                // 函数调用，值是它的返回值
if ready { 10 } else { 20 } // if 也是表达式，值是被执行分支的值
```

（这些是表达式片段，不是完整程序：`double` 和 `ready` 是假想的函数与变量，只用来展示表达式的形状。）

**语句**（statement）是“执行一个动作、本身不产出值”的代码。`let count = 3;` 就是语句：它建立绑定，但你不能写 `let x = (let count = 3);`。

### 5.2 分号会“吃掉”值

给表达式末尾加上分号，它的值就被丢弃，整行变成一个语句。而花括号包起来的**代码块**（block）本身也是表达式，它的值是块内最后一个不带分号的表达式的值：

```rust
fn main() {
    let value = {
        let base = 6;
        base * 2   // 块的最后一句没有分号，它的值 12 成为整个块的值
    };

    println!("{value}"); // 12
}
```

对比——只是多了一个分号，整个代码就编译不过：

```rust,compile_fail
let value = {
    let base = 6;
    base * 2;  // 值被分号丢弃，块现在只产出 ()
};
```

此时块的值是 `()`，`value` 的类型也成了 `()`。所以 Rust 的分号不是可有可无的“行尾装饰”，它改变代码的含义。

### 5.3 函数返回值中的分号

这个规则最常见的踩坑处是函数返回：

```rust
fn good() -> i32 {
    42        // 函数体最后的表达式（无分号）就是返回值
}
```

```rust,compile_fail
fn bad() -> i32 {
    42;       // 值被丢弃，函数实际交出 ()，与声明的 i32 对不上
}
```

第九节会展示这段代码的真实报错。

## 六、函数

### 6.1 具体的问题

我们已经能写一串按顺序执行的代码。但如果“计算折扣”的逻辑要在程序的五个地方用到，复制五份意味着改的时候要改五处——迟早漏一处。函数解决这个问题：**把一段逻辑起个名字，需要时按名字调用，逻辑只写一份**。

### 6.2 函数签名

```rust
fn add(left: i32, right: i32) -> i32 {
    left + right
}
```

第一行叫**函数签名**（signature），是这份逻辑的“合同”：

- 函数名 `add`；
- **参数**（parameter）：调用时必须提供的输入，每个都写明名字和类型。Rust 不允许省略参数类型；
- `-> i32`：**返回类型**，函数承诺交出的结果的类型。

签名之后花括号里是函数体。函数体最后一个不带分号的表达式的值，就是函数的返回值——这叫**尾表达式返回**，是 Rust 的惯用写法。

### 6.3 `return`：提前交卷

需要在函数中途结束时用 `return`（返回）：

```rust
fn absolute(value: i32) -> i32 {
    if value < 0 {
        return -value; // 提前结束，后面的代码不再执行
    }

    value // 正常路径：尾表达式
}
```

惯例是：正常路径用尾表达式，提前退出用 `return`。

### 6.4 参数在函数体内也是绑定

参数名在函数体内就是一个绑定，也可以声明成可变的：

```rust
fn countdown(mut current: i32) {
    while current > 0 {
        println!("{current}");
        current -= 1; // “-=”是“在原有基础上减”的简写
    }
}
```

修改的是函数自己手里的这份数据，不会影响调用方。为什么传进来的整数可以放心地各自独立使用，涉及“所有权”和 `Copy`，[第三课](../03-ownership-borrowing/README.md)会完整解释。

## 七、控制流

**控制流**（control flow）指“代码按什么顺序、在什么条件下执行”，主要是两类：**分支**（根据条件选择执行哪段）和**循环**（重复执行一段）。

### 7.1 `if`：条件必须是 `bool`

```rust
fn main() {
    let number = 3;

    if number > 0 {
        println!("positive");
    } else if number < 0 {
        println!("negative");
    } else {
        println!("zero");
    }
}
```

`if` 后面跟一个 `bool` 条件，条件为 `true` 执行第一个分支，否则看 `else if`，都不命中走 `else`（`else if` 和 `else` 都可省略）。

一个常见的新手疑问：能不能直接写 `if number`（意思是“number 不是 0 就算真”）？不能：

```rust,compile_fail
if number {
    println!("non-zero");
}
```

Rust 拒绝猜测你的意图，因为“非 0 即真”掩盖过太多 Bug（比如把一个长度误判成开关）。想表达什么就写清楚：`if number != 0`（`!=` 表示“不等于”；相应地，`==` 表示“等于”——判断相等用两个等号，一个等号是赋值）。

需要同时检查多个条件时，用 `&&`（并且：两个条件都成立）和 `||`（或者：至少一个成立）把两个 `bool` 条件连起来，例如 `if number > 0 && number % 2 == 0`。

### 7.2 `if` 是表达式：判断本身可以产出值

回到本节开头的问题——根据条件得到一个值，在 Rust 里可以直接写：

```rust
fn main() {
    let number = -8;
    let sign = if number < 0 { -1 } else { 1 };

    println!("{sign}"); // -1
}
```

因为结果要放进同一个绑定，所以各分支产出的类型必须一致：

```rust,compile_fail
let result = if true { 10 } else { "ten" }; // 错误：一个分支给整数，一个给文字
```

这个要求很合理：编译器必须能在编译期确定 `result` 的类型，而到底走哪个分支要运行时才知道，所以两个分支的类型必须相同，才能保证“无论走哪边，类型都说得通”。

### 7.3 `loop`：无条件重复，直到 `break`

```rust
fn main() {
    let mut attempts = 0;

    loop {
        attempts += 1;

        if attempts == 3 {
            break; // 结束循环
        }
    }

    println!("{attempts}"); // 3
}
```

`loop` 无条件重复执行循环体，唯一的出口是 `break`（跳出循环）——所以它能明确表达“我不知道要循环多少轮，反正做到满足条件为止”。`continue` 则是“跳过本轮剩下的部分，直接开始下一轮”。

`loop` 也能产出值：`break` 后面跟一个表达式，这个值就成为整个 `loop` 表达式的结果：

```rust
fn main() {
    let mut number = 0;

    let answer = loop {
        number += 1;

        if number == 5 {
            break number * 10; // 带着 50 跳出
        }
    };

    println!("{answer}"); // 50
}
```

### 7.4 `while`：条件不满足就停

已知“继续的条件”时，用 `while`（当……的时候）更直接：

```rust
fn main() {
    let mut current = 1;
    let mut total = 0;

    while current <= 4 { // 每轮开始前检查条件（“<=”表示小于等于）
        total += current;
        current += 1;    // 推动状态前进，否则条件永远成立，循环永不结束
    }

    println!("{total}"); // 10
}
```

每轮开始时检查条件，条件不成立就结束。一个能正确结束的循环需要四要素：**初始状态、继续条件、每轮推进状态、循环后使用结果**。漏掉“推进”是最常见的死循环来源（`current` 永远不变，条件永远成立）。

### 7.5 `for`：逐个取来用

要“把一段数字里每个都取来用一遍”，`for`（对于每一个）最省事：

```rust
fn main() {
    for value in 1..4 {   // 1..4 是一个 range（区间）：从 1 到 4，不含 4
        println!("{value}"); // 依次输出 1、2、3
    }

    for value in 1..=4 {  // ..= 表示包含右端点
        println!("{value}"); // 依次输出 1、2、3、4
    }
}
```

本课主练习特意要求用 `while` 或 `loop`，目的是先亲手练习管理循环状态；`for` 背后更强大的“迭代器”机制在[第十课](../10-iterators/README.md)展开。

### 7.6 取余 `%` 与循环内分支

循环里常配合分支做筛选。`%` 是**取余**运算（modulo，求除法的余数）：`7 % 3` 得 `1`。一个数对 `2` 取余为 `0`，就说明它是偶数：

```rust
fn main() {
    let mut value = 0;

    while value < 10 {
        value += 1;

        if value % 2 == 0 {
            continue; // 偶数：跳过本轮剩余部分
        }

        if value > 7 {
            break; // 大于 7：结束整个循环
        }

        println!("{value}"); // 依次输出 1、3、5、7
    }
}
```

## 八、作用域：名字在哪里有效

花括号 `{ }` 还划定**作用域**（scope）：一个名字只在声明它的那对花括号之内（含内部嵌套的花括号）有效：

```rust,compile_fail
let outer = 10;

{
    let inner = 20;
    println!("{}", outer + inner); // 可以：内层能看见外层的名字
}

println!("{inner}"); // 错误：inner 只存在于上面那对花括号里
```

作用域的好处是名字用完即失效，互不污染。以后还会学到，作用域同时决定数据何时被自动清理，那是[第三课](../03-ownership-borrowing/README.md)的核心。本课先建立“名字只在它的花括号里可见”的认识。

## 九、怎样阅读 Rust 编译器诊断

编译器拒绝代码时给出的报告叫**诊断**（diagnostic）。新手常犯的错是只看最后一行、或看到红色文字就慌。一份诊断包含固定几部分，按顺序读：

1. 第一行：错误编号和一句话摘要，如 `error[E0384]: cannot assign twice to immutable variable`。
2. 带箭头的代码位置：`--> 文件名:行号:列号`。
3. 类型对比：`expected ... found ...`（期望什么、实际是什么）。
4. `help:` 给出的修改建议——**先理解再采纳**，不要照抄不理解的建议。

下面五个真实报错都来自本节对应的错误代码（在 Rust 1.94 上实际编译产生）。

### 9.1 修改不可变绑定：E0384

代码：

```rust,compile_fail
fn main() {
    let count = 1;
    count += 1;
    println!("{count}");
}
```

真实诊断：

```text
error[E0384]: cannot assign twice to immutable variable `count`
 --> main.rs:3:5
  |
2 |     let count = 1;
  |         ----- first assignment to `count`
3 |     count += 1;
  |     ^^^^^^^^^^ cannot assign twice to immutable variable
  |
help: consider making this binding mutable
  |
2 |     let mut count = 1;
  |         +++
```

逐行读：摘要“不能给不可变变量 `count` 赋值两次”；标注指出第一次赋值在第 2 行、再次赋值在第 3 行；`help` 建议加 `mut`。

但注意：**修复方向不总是加 `mut`**。先问自己：这个值真的需要反复更新吗？如果只是做一次变换，`let count = count + 1;`（shadowing）往往更贴切；如果那次赋值本来就是误写，应该删掉。

### 9.2 分号导致返回 `()`：E0308

代码：

```rust,compile_fail
fn answer() -> i32 {
    42;
}

fn main() {
    println!("{}", answer());
}
```

真实诊断：

```text
error[E0308]: mismatched types
 --> main.rs:1:16
  |
1 | fn answer() -> i32 {
  |    ------      ^^^ expected `i32`, found `()`
  |    |
  |    implicitly returns `()` as its body has no tail or `return` expression
2 |     42;
  |       - help: remove this semicolon to return this value
```

E0308 是“类型不匹配”（mismatched types）的通用编号，会出现很多场景。这里读法是：函数声明期望返回 `i32`，但函数体没有尾表达式（因为 `42` 后面多了分号），实际返回 `()`。`help` 正对症：删掉那个分号。

以后看到 `expected 某类型, found ()`，第一反应就是检查是不是哪里多了分号、或忘了返回值。

### 9.3 条件不是布尔值：E0308

```rust,compile_fail
fn main() {
    let number = 3;
    if number {
        println!("non-zero");
    }
}
```

真实诊断：

```text
error[E0308]: mismatched types
 --> main.rs:3:8
  |
3 |     if number {
  |        ^^^^^^ expected `bool`, found integer
```

修复不是“把整数硬转成 bool”，而是写出真正的条件：`if number != 0`。

### 9.4 `if` 两个分支类型不同：E0308

```rust,compile_fail
fn main() {
    let value = if true { 1 } else { "one" };
    println!("{value}");
}
```

真实诊断：

```text
error[E0308]: `if` and `else` have incompatible types
 --> main.rs:2:38
  |
2 |     let value = if true { 1 } else { "one" };
  |                           -          ^^^^^ expected integer, found `&str`
  |                           |
  |                           expected because of this
```

`&str` 是字符串字面量的类型（精确含义在[第三课](../03-ownership-borrowing/README.md)讲）。修复思路：让两个分支给出同一种类型，比如都返回数字，或都返回文字。

### 9.5 名字不在作用域里：E0425

```rust,compile_fail
fn main() {
    let outer = 10;
    {
        let inner = 20;
        println!("{}", outer + inner);
    }
    println!("{inner}");
}
```

真实诊断：

```text
error[E0425]: cannot find value `inner` in this scope
 --> main.rs:7:16
  |
7 |     println!("{inner}");
  |                ^^^^^
  |
help: the binding `inner` is available in a different scope in the same function
 --> main.rs:4:13
  |
4 |         let inner = 20;
  |             ^^^^^
```

“在当前作用域找不到 `inner`”，`help` 还指出了它实际声明的位置。看到 E0425 先检查：名字拼写、声明位置、以及你是不是在花括号外用了花括号内的名字。

### 9.6 编译器管不了的事：死循环

下面的代码能通过编译，但永远运行不完：

```rust,no_run
fn main() {
    let current = 1;

    while current <= 3 {
        println!("{current}");
    }
}
```

`current` 从不变化，条件永远成立。编译器检查“写得通不通”，不证明“逻辑对不对”——这是测试和你的推理要负责的部分。如果程序疑似死循环，在终端按 `Ctrl + C`（按住 Ctrl 再按 C）可以强制结束它。

## 十、完整示例：订单价格计算

这个示例把前面所有概念组合起来（它与主练习不同，不泄露练习答案）。先完整读一遍代码，再看逐条分析：

```rust
const FREE_SHIPPING_THRESHOLD: i32 = 100; // 满 100 免运费

fn discounted_price(unit_price: i32, quantity: i32, is_member: bool) -> i32 {
    let subtotal = unit_price * quantity;
    let discount = if is_member { subtotal / 10 } else { 0 }; // if 直接产出值
    subtotal - discount
}

fn shipping_fee(order_total: i32) -> i32 {
    if order_total >= FREE_SHIPPING_THRESHOLD {
        0
    } else {
        12
    }
}

fn sum_odd_below(limit: i32) -> i32 {
    let mut current = 1;
    let mut total = 0;

    while current < limit {
        total += current;
        current += 2; // 每次加 2，只访问奇数
    }

    total
}

fn main() {
    let item_total = discounted_price(30, 4, true);
    let delivery = shipping_fee(item_total);
    let final_total = item_total + delivery;

    println!("item total = {item_total}");
    println!("shipping = {delivery}");
    println!("final total = {final_total}");
    println!("odd sum below 8 = {}", sum_odd_below(8));
}
```

逐条分析：

1. 免运费门槛是固定规则参数，所以用 `const`，全大写命名。
2. `subtotal` 和 `discount` 算出来之后不再变，所以用普通 `let`，不写 `mut`。
3. `discount` 直接接收 `if` 表达式的结果，不需要“先声明再在两分支里赋值”。
4. `discounted_price` 和 `shipping_fee` 都用尾表达式返回，没有多余的 `return`。
5. `sum_odd_below` 里的 `current`（当前加到哪个奇数）和 `total`（累计结果）每轮都要更新，所以用 `mut`；`current += 2` 保证条件 `current < limit` 最终会不成立，循环能结束。

自己动手推一遍输出，再对照：

```text
item total = 108
shipping = 0
final total = 108
odd sum below 8 = 16
```

（`30 * 4 = 120`，会员减 `12` 得 `108`；`108 >= 100` 免运费；8 以下的奇数是 1、3、5、7，和为 16。）

## 十一、代码预测与解析

Rust 学习很依赖“先预测编译器会接受还是拒绝、输出会是什么”的能力。请先遮住答案作答，再展开核对。

### 题目 1：shadowing

```rust
let x = 4;
let x = x + 1;
let x = x * 2;
println!("{x}");
```

输出什么？需要 `mut` 吗？

<details>
<summary>查看解析</summary>

输出 `10`。三个 `let` 依次建立了三个同名绑定，后一个遮蔽前一个；没有任何一个绑定被“重新赋值”，所以不需要 `mut`。

</details>

### 题目 2：块表达式与分号

```rust
let value = {
    let base = 6;
    base * 2
};
println!("{value}");
```

输出什么？把 `base * 2` 改成 `base * 2;` 后会怎样？

<details>
<summary>查看解析</summary>

原代码输出 `12`：`base * 2` 是块的尾表达式，块的值就是 `12`。加了分号后值被丢弃，块的类型变成 `()`，`value` 的类型也成了 `()`；`()` 无法被 `println!` 正常显示，代码编译失败。

</details>

### 题目 3：`if` 表达式

```rust
let number = 7;
let category = if number % 2 == 0 { "even" } else { "odd" };
println!("{category}");
```

`category` 的值是什么？这种写法和“先声明再赋值”相比好在哪里？

<details>
<summary>查看解析</summary>

值是 `"odd"`：`7 % 2` 得 `1`，条件不成立，走 `else` 分支。这种写法让 `category` 在创建的一刻就有确定的值，且保持不可变；“先声明后赋值”则存在一段“名字已存在但还没值”的空档，依赖编译器帮你检查每条路径都赋过值。

</details>

### 题目 4：`loop` 的值

```rust
let mut value = 0;

let result = loop {
    value += 1;
    if value == 3 {
        break value * 4;
    }
};

println!("{result}");
```

<details>
<summary>查看解析</summary>

输出 `12`。第三轮时 `value == 3`，`break value * 4` 结束循环并把 `12` 作为整个 `loop` 表达式的值交给 `result`。

</details>

### 题目 5：区间边界

```rust
let mut total = 0;

for value in 1..=3 {
    total += value;
}

println!("{total}");
```

<details>
<summary>查看解析</summary>

输出 `6`：`1..=3` 包含右端点，依次产生 1、2、3。若写成 `1..3`（不含右端点），结果是 `3`。

</details>

## 十二、主练习：整数区间分析器

练习文件是 `src/exercise.rs`。你的任务是替换三个函数体里的 `todo!()`。**不要修改函数签名，不要改动文件底部的测试。**

### 12.1 练习目标

三个函数各练一种核心能力：

- `clamp`：把 `if` 当作产出值的表达式。
- `sum_inclusive`：亲手维护循环的位置和累积状态。
- `count_multiples`：先拦截非法输入，再做循环筛选统计。

它们只用整数，避免在第一课引入还没学的概念。

三个函数的输入都涉及“闭区间”。**闭区间** `[start, end]` 指数学上 `start <= value <= end` 的所有整数，两端都包含。代码里写作 `start..=end` 也是这个意思。

### 12.2 函数一：`clamp`

签名（已写好，不要改）：

```rust
pub fn clamp(value: i32, minimum: i32, maximum: i32) -> i32
```

（`pub` 是 public，“公开的”的缩写：表示其他 module 也能调用这个函数。module 系统会在[第七课](../07-modules-crates/README.md)细讲，现在照原样保留即可。）

**输入**：一个值 `value`，以及区间的下界 `minimum`、上界 `maximum`。可以假设 `minimum <= maximum`。

**输出**：把 `value` 限制在 `[minimum, maximum]` 内后的结果——低于下界返回下界，高于上界返回上界，否则返回 `value` 本身。

行为示例：

| 调用 | 期望结果 | 说明 |
|---|---:|---|
| `clamp(-4, 0, 10)` | `0` | 低于下界 |
| `clamp(7, 0, 10)` | `7` | 在区间内 |
| `clamp(14, 0, 10)` | `10` | 高于上界 |
| `clamp(10, 0, 10)` | `10` | 恰好等于上界，属于“在区间内” |

边界情况：`value` 恰好等于某一端时返回原值；`minimum == maximum` 时任何输入都被限制到那个唯一的值。

### 12.3 函数二：`sum_inclusive`

签名：

```rust
pub fn sum_inclusive(start: i32, end: i32) -> i32
```

**输入**：闭区间 `[start, end]`。

**输出**：区间内所有整数的和；区间为空（`start > end`）时返回 `0`。

行为示例：

| 调用 | 参与求和的值 | 期望结果 |
|---|---|---:|
| `sum_inclusive(1, 4)` | 1, 2, 3, 4 | `10` |
| `sum_inclusive(-2, 2)` | -2, -1, 0, 1, 2 | `0` |
| `sum_inclusive(5, 3)` | （空区间） | `0` |
| `sum_inclusive(7, 7)` | 7 | `7` |

**约束**：必须使用 `while` 或 `loop` 循环逐项累加，不许用等差数列求和公式——本练习的目的就是练循环状态管理。

### 12.4 函数三：`count_multiples`

签名：

```rust
pub fn count_multiples(start: i32, end: i32, divisor: i32) -> u32
```

**输入**：闭区间 `[start, end]` 和一个除数 `divisor`。

**输出**：区间内满足 `value % divisor == 0`（能被 `divisor` 整除）的整数的个数。个数不会是负数，所以返回类型是 `u32`。

**错误行为**：两种输入没有可统计的对象，直接返回 `0`：

- 区间为空（`start > end`）；
- `divisor == 0`——注意，对 0 取余在数学上没有意义，在 Rust 里会让程序 panic，所以必须在任何 `%` 运算**之前**拦截。

行为示例：

| 调用 | 满足条件的值 | 期望结果 |
|---|---|---:|
| `count_multiples(1, 10, 2)` | 2, 4, 6, 8, 10 | `5` |
| `count_multiples(-3, 3, 3)` | -3, 0, 3 | `3`（0 能被任何非零数整除） |
| `count_multiples(5, 3, 2)` | （空区间） | `0` |
| `count_multiples(1, 10, 0)` | （除数非法） | `0` |

### 12.5 实现约束

- 至少在一个函数里把 `if` 当作表达式使用（`clamp` 是天然的位置）。
- 两个区间函数都必须用 `while` 或 `loop`，不要用 `for`。
- 不使用还没学的特性：迭代器、递归（函数调用自己）、外部库。
- 不删除、不注释、不修改任何测试。
- 可以假设输入足够小，累加不会溢出 `i32`。

### 12.6 推荐完成顺序

1. 先实现 `clamp`，运行 `cargo test -p l01_basics clamp` 只看它的三个测试。
2. 实现 `sum_inclusive`：先想“循环前要记住什么”，再写循环，确认空区间情况自然成立。
3. 实现 `count_multiples`：先写两个非法输入的拦截，再写循环。
4. 全部测试通过后运行 `cargo fmt -p l01_basics`，最后完整验收。

### 12.7 测试在检查什么

测试（`src/exercise.rs` 底部的 `mod tests`）已经提前写好，共 7 个，覆盖三类情况：

- **正常输入**：如 `clamp(7, 0, 10)`、`sum_inclusive(1, 4)`。
- **边界情况**：恰好等于上下界、单元素区间、跨零区间、区间内没有满足条件的值。
- **错误/异常输入**：空区间、除数为 0。

测试里反复出现 `assert_eq!(实际结果, 期望结果)`：两者不相等时，测试失败并打印两边的值。它是标准库提供的宏（名字带 `!`），现在只需要会读，不需要自己写。

（文件里的 `#[test]` 标记一个测试函数，`#[cfg(test)] mod tests` 表示这块代码只在运行测试时编译，测试开头的 `use super::...` 是把上面的函数引入作用域。这些标记在[第七课](../07-modules-crates/README.md)和[第八课](../08-testing-quality/README.md)讲解，现在照原样保留即可。）

实现前运行测试，7 个测试全部因 `todo!()` 而失败（panic: not yet implemented），这是正常的起点。

### 12.8 验收命令

在仓库根目录依次运行：

```bash
cargo fmt -p l01_basics -- --check   # 排版符合规范
cargo check -p l01_basics --all-targets # 编译检查通过
cargo test -p l01_basics             # 主练习 7 个测试通过（巩固任务的 4 个此时仍失败，属正常）
```

（`--all-targets` 表示“连测试代码一起检查”；测试代码平时不编译进程序，需要特别指明。）

卡住时不要硬扛：打开本课 `HINTS.md`，每个函数都有三级提示，从“思考方向”到“接近完整的结构”逐级查看。

## 十三、课后巩固：数字步数与数位之和

完成主练习和复盘后，做这组巩固任务。它不使用区间数据，边界情况也不同，目的是检验你是否真正掌握了“循环 + 状态 + 非法输入处理”，而不是记住了上一题的样子。

练习文件是 `src/consolidation.rs`，同样只替换 `todo!()`，签名和测试不要动。

### 13.1 函数一：`collatz_steps`

签名：

```rust
pub fn collatz_steps(start: u32) -> u32
```

**规则**：从 `start` 出发反复变换——当前数字是偶数就除以 2，是奇数就乘以 3 再加 1——直到第一次得到 `1`。

**输出**：需要的变换次数。

**边界与错误行为**：`start` 已经是 `1` 时返回 `0`；`start == 0` 永远到不了 `1`，是非法输入，直接返回 `0`（不做任何变换）。

行为示例：

| 调用 | 变换过程 | 期望结果 |
|---|---|---:|
| `collatz_steps(6)` | 6→3→10→5→16→8→4→2→1 | `8` |
| `collatz_steps(1)` | 无需变换 | `0` |
| `collatz_steps(0)` | 非法输入 | `0` |

这个函数和主练习最大的不同：**循环次数事先未知**，它就是你要算的答案。

### 13.2 函数二：`sum_of_digits`

签名：

```rust
pub fn sum_of_digits(value: i32) -> u32
```

**输出**：`value` 的十进制写法中每一位数字的和，忽略正负号。可以假设输入不是 `i32` 的最小值。

行为示例：

| 调用 | 计算 | 期望结果 |
|---|---|---:|
| `sum_of_digits(123)` | 1 + 2 + 3 | `6` |
| `sum_of_digits(-123)` | 忽略符号后同上 | `6` |
| `sum_of_digits(0)` | 没有非零位 | `0` |
| `sum_of_digits(907)` | 9 + 0 + 7 | `16` |

（提示：`% 10` 能取出最后一位，`/ 10` 能去掉最后一位。更完整的拆解见 `HINTS.md`。）

### 13.3 验收命令

```bash
cargo test -p l01_basics consolidation
```

这会按名字筛选，只运行巩固任务的 4 个测试。全部通过，本课才算真正完成。

## 十四、常见误区

### 误区 1：值会变化就用 shadowing

shadowing 是“建立一个新绑定”，适合少数几步的形态转换。循环计数、累加这种**反复**更新的状态，`mut` 才如实表达了意图。

### 误区 2：每个分支都写 `return`

`return` 没有错，但通篇 `return` 掩盖了 Rust 的表达式风格。正常路径用尾表达式，把 `return` 留给真正的提前退出。

### 误区 3：分号只是格式

分号会把表达式的值丢弃，把 `i32` 变成 `()`。看到 `expected i32, found ()` 先检查分号。

### 误区 4：类型不匹配就用 `as` 压掉

`as` 是显式转换，可能改变值。先想清楚类型为什么不同、转换合不合理，再动手。

### 误区 5：只检查循环条件，忘了推进状态

正确循环四要素：初始状态、继续条件、循环体、状态推进。缺了推进就是死循环。

### 误区 6：`cargo check` 过了就完成了

`todo!()` 也能通过编译。编译检查、测试、实际运行回答三个不同的问题：写得通吗？做得对吗？真能用吗？

## 十五、本课总结

需要长期记住的不是关键词，而是这些模型：

1. package 是项目的单元，target 决定要编译成什么，crate 是一次编译的产物，module 是 crate 内部的分组。
2. `let` 默认建立不可变绑定；`mut` 允许修改同一绑定；shadowing 是建立一个遮蔽旧名的新绑定；`const` 是编译期固定的规则参数。
3. Rust 在编译期就确定每个值的类型；推断只是帮你省去书写，不代表类型不存在。
4. 代码块、`if`、`loop`、函数调用都可以是产出值的表达式；分号会丢弃值。
5. 函数签名是合同：参数和返回值类型都写在明处。
6. 循环要自己管理状态推进；编译通过不代表逻辑正确。
7. `cargo check`、`cargo test`、`cargo run` 分别回答“写得通”“做得对”“真能用”。

## 十六、课后复盘

完成练习后，合上讲义，用自己的话回答（每题一两句即可）：

1. `let mut value = 1; value = 2;` 和 `let value = 1; let value = 2;` 有什么区别？
2. 为什么 `fn answer() -> i32 { 42; }` 编译失败？报错里会出现哪个类型？
3. 为什么 Rust 不接受 `if number { ... }`？
4. `cargo check` 通过后，测试为什么仍可能失败？
5. 在区间求和的循环里，哪些绑定需要 `mut`，哪些不需要，为什么？

把稳定的理解写进仓库的 `notes/concepts.md`；如果某类错误以后又犯了，记到 `notes/mistakes.md`。

## 下一课预告

[第二课：数据建模](../02-data-modeling/README.md)将学习 struct（结构体，把几个相关的值组合成一个自定义类型）、enum（枚举，表达“几种情况之一”）和方法。到那时你就能描述“一个学生”“一笔订单”这样的真实数据，而不仅是零散的整数。本课的函数、表达式和控制流知识会在那里直接使用。
