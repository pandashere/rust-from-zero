# 第十三课：`Box<T>`、递归类型与动态分发

第十二课学习了 trait：它描述“一个类型能做什么”。这一课换一个角度，问一个更接近内存布局的问题：**如果一份数据里还要再放一份同样形状的数据，它到底占多少空间？**

日常数据里这种“套娃”结构很常见：一个文件夹里有子文件夹，一个数学表达式里有子表达式，一条审批流程里有下一步。Rust 对这类结构有一条硬性要求：普通值的大小必须在**编译期**（把源代码翻译成机器指令的阶段，程序还没开始运行）就确定。本课要学的 `Box<T>`，就是为这个问题准备的固定大小“间接层”。

本课会反复使用两个词：**值（value）**是一份具体的数据，**类型（type）**规定这份数据能取哪些值、能做哪些操作、需要多少空间。例如 `42` 是一个值，它的类型是 `i32`。

## 本课定位

### 你将学会

- 用日常语言解释“类型大小”“栈”“堆”“指针”“间接层”这几个概念。
- 说明直接递归的类型为什么编译不过，以及编译器给出的 `E0072` 在表达什么。
- 用 `Box<T>` 定义递归 enum（枚举，一组固定选项中的某一个；递归指定义里又出现自己）和递归 struct（结构体，一组具名字段的组合）。
- 区分“借用整棵树来遍历”和“取得所有权后逐层拆开”两种递归方法。
- 理解 `Box<T>` 的独占所有权（只有它负责清理这份数据）、移动、自动析构（值离开作用域时自动清理），以及 `Deref`（允许用 `*` 取出目标的 trait）带来的自动解引用。
- 理解 `Box<dyn Trait>` 为什么能保存“具体类型在运行时（程序执行期间）才知道”的对象，以及它的代价。
- 判断什么时候该用 `Box`，什么时候普通字段、`Vec<T>` 或引用更合适。
- 阅读 `E0072`、`E0382`、`E0507`、`E0782`、`E0277` 这几类常见诊断，并按错误指向的位置修复。

### 前置知识

不需要学过其他编程语言。下面这些前课内容会在本课直接使用，每一条都先用一句话复习；忘记细节时点链接回看即可。

- **函数、`main`、`println!`、Cargo 命令**：见[第一课](../01-basics/README.md)。复习：`fn main()` 是程序入口；`cargo run -p 包名` 编译并运行；`cargo test -p 包名` 编译并运行测试。
- **struct、enum、`match`、方法与 `&self`**：见[第二课](../02-data-modeling/README.md)。复习：struct 把多个字段装在一起；enum 表示“几种情况之一”；`match` 按情况分支；方法写在 `impl` 块里，`&self` 表示“借这份数据来看，不拿走”。
- **所有权、移动（move）、借用、引用 `&T`**：见[第三课](../03-ownership-borrowing/README.md)。复习：每份数据有唯一所有者，所有者离开作用域时数据被清理；把值赋给另一个变量会把所有权移过去；`&T` 是借来看，不转移所有权。
- **`Option<T>`、`Result<T, E>`、`Ok`/`Err`**：见[第四课](../04-option-result/README.md)。复习：`Option` 表示“可能有值”；`Result` 表示“成功或失败”，成功是 `Ok(值)`，失败是 `Err(错误)`，`?` 或 `match` 可以把错误向上传递。
- **`Vec<T>`、切片、`push`、`to_vec`**：见[第五课](../05-vec-slices/README.md)。复习：`Vec<T>` 是长度可变、元素类型相同的序列；它自身大小固定，元素放在堆上的独立缓冲区里。
- **trait 与 `dyn Trait` 的初步用法**：见[第十二课](../12-traits/README.md)。复习：trait 是一组行为要求；具体类型用 `impl Trait for 类型` 兑现要求；`dyn Trait` 表示“某个实现了该 trait 的类型，具体是哪一个稍后再定”。本课第七、八节会重新解释这里够用的部分，不要求先读透第十二课。

### 本课边界

- 显式生命周期标注（描述多个引用之间有效期的关系）在[第十四课](../14-lifetimes/README.md)系统学习。本课只在“从树里借出一个值”时用到最简单的省略形式。
- 共享所有权 `Rc<T>`（引用计数指针，多个所有者共同拥有一份数据）在[第十五课](../15-smart-pointers/README.md)；可跨线程的 `Arc<T>`（原子引用计数指针）在[第十七课](../17-shared-concurrency/README.md)。本课的 `Box<T>` 始终只有一个所有者。
- 线程与并发（让多个任务同时推进的机制）不在本课范围；`Box<T>` 本身不会自动让数据可以跨线程共享。
- 本课不讨论内存分配器的内部实现，也不要求手写 `unsafe`（一段允许绕过部分安全检查的代码，第二十一课才学）。

### 如何学习本课

1. 先读第一到第三节，重点弄清“为什么直接递归会无限大”。
2. 第四、五节各有一个完整程序，先预测输出，再运行对照。
3. 第七、八节解释 `Box<dyn Trait>`；如果第十二课还没读，按本课的解释理解即可。
4. 「常见编译错误与排错」不要跳过：本课的错误几乎都来自“大小”和“所有权”两件事。
5. 完成「代码预测」后做「主练习」，再做「课后巩固」。卡住时按 [`HINTS.md`](HINTS.md) 的三级提示逐级查看。

预计用时：概念与示例约 2 小时，主练习约 1 小时，课后巩固约 40 分钟。

## 一、具体问题：表达“运算里再套运算”

先看一个具体的数学表达式：

```text
(1 + 2) * 4 - (-3)
```

它可以看成一棵倒过来的树：每个运算把下面的子表达式组合起来。

```text
          Subtract
         /        \
    Multiply       Neg
    /      \        \
  Add       4       3
 /   \
1     2
```

- **节点（node）**：树里的每一个圆圈，这里是一个数或一次运算。
- **叶子（leaf）**：没有子节点的节点，这里是数字 1、2、4、3。
- **根（root）**：最上面的节点 `Subtract`。
- **子树（subtree）**：某个节点连同它下面所有后代组成的树。

在 Rust 里，可以用 enum 表示“一个表达式是这几种情况之一”：

```rust,compile_fail
enum Expr {
    Literal(i64),
    Neg(Expr),
    Add(Expr, Expr),
}

fn main() {}
```

逐项解释：

- `i64` 是 64 位有符号整数：能表示从 -9,223,372,036,854,775,808 到 9,223,372,036,854,775,807 的整数。`i` 表示 signed integer（有符号整数），`64` 表示 64 个二进制位。
- `Literal(i64)` 是“一个字面量数字”，括号里的 `i64` 是这个变体携带的数据。
- `Neg(Expr)` 是“取相反数”，里面放一个子表达式。
- `Add(Expr, Expr)` 是“加法”，里面放左右两个子表达式。
- 这种“类型定义里又出现自己”的结构叫**递归类型（recursive type）**。

`Expr` 在定义里包含 `Expr`，`Expr` 又包含 `Expr`……从数学上看可以一直写下去。问题在于：Rust 的普通值必须在编译期有确定的字节数（**字节**是内存的计量单位，一个 `i64` 占 8 个字节）。编译器要回答“一个 `Expr` 占多少字节”，却得到这样的方程：

```text
Expr 的大小 >= Neg 里那个 Expr 的大小
Neg 里那个 Expr 的大小 >= 另一个 Expr 的大小
……
```

永远算不出一个有限答案。把上面的代码保存成文件交给 `rustc`（Rust 编译器），会得到真实的编译错误（以下输出来自 rustc 1.94.0，行号可能随你的版本变化）：

```text
error[E0072]: recursive type `Expr` has infinite size
 --> recursive.rs:1:1
  |
1 | enum Expr {
  | ^^^^^^^^^
2 |     Literal(i64),
3 |     Neg(Expr),
  |         ---- recursive without indirection
  |
help: insert some indirection (e.g., a `Box`, `Rc`, or `&`) to break the cycle
  |
3 |     Neg(Box<Expr>),
  |         ++++    +

error: aborting due to 1 previous error
```

关键短语是 `recursive without indirection`：递归字段缺少**间接层（indirection）**。错误不是“递归逻辑不允许”，而是“递归字段不能是一整个内嵌的值”。编译器甚至在 help 里直接给出了修复方向。

为什么普通值非得有确定大小？因为编译器要为局部变量预留栈空间、为结构体字段计算偏移量（每个字段距离结构体开头多少字节）、为函数参数决定如何传递。给它一个“大小取决于自己”的类型，这些工作都无法完成。这不是 Rust 故意为难人，而是所有需要提前安排内存的语言都要面对的问题。

## 二、背景：栈、堆、指针与 `Box<T>`

要理解“间接层”，先要分清两种存放数据的地方。

### 2.1 栈与堆

- **栈（stack）**：函数调用时自动使用的一块内存区域。每调用一个函数，就为它的局部变量分配一小块**栈帧（stack frame）**；函数返回时整块回收。栈上的数据大小必须固定，分配和回收非常快。
- **堆（heap）**：一块更大的、由程序在运行时按需申请和归还的内存区域。堆上的数据大小可以随运行情况变化，但申请和释放需要额外工作。

第三课已经用过这两个概念：`i32`（32 位有符号整数）、`bool`（只能是 `true` 或 `false` 的布尔类型）这类值直接放在栈上；`Vec<i32>` 的“指针、长度、容量”三个字段放在栈上，真正的元素放在堆上。`Box<T>` 也是同样的思路，只是它只装一个 `T`。

### 2.2 指针与间接层

**指针（pointer）**是一个内存地址，本质是一个固定大小的整数（在常见 64 位平台上占 8 字节），记录“东西在哪里”，而不是“东西本身”。

**间接层**就是“先存地址，再顺着地址找过去”。地址的大小是固定的，所以无论它指向的东西多大，存地址的这个位置大小都固定。

`Box<T>` 是标准库（随 Rust 一起安装、不需要额外下载的代码库）提供的**拥有型堆指针**：

- “拥有型”表示 `Box<T>` 独占它指向的那个 `T`，负责在适当的时候清理；
- “堆指针”表示 `T` 被放在堆上，`Box<T>` 自己只保存地址。

```rust
fn main() {
    let number = Box::new(42_i32);
    println!("number = {}", *number);
    println!("Box 的大小 = {} 字节", std::mem::size_of::<Box<i32>>());
    println!("i32 的大小 = {} 字节", std::mem::size_of::<i32>());
}
```

输出：

```text
number = 42
Box 的大小 = 8 字节
i32 的大小 = 4 字节
```

`Box::new(42_i32)` 做两件事：在堆上放一个 `i32`，返回一个指向它的 `Box<i32>`。`*number` 里的 `*` 是**解引用（dereference）**：顺着地址取出里面的值。

内存关系可以画成这样：

```text
栈上的变量 number                堆上的数据
+------------------+           +--------+
| 地址 0x...       |---------->|   42   |
+------------------+           +--------+
   Box<i32>，8 字节               i32，4 字节
```

注意 `Box<T>` 自己并不是 `T`。`Box<i32>` 占 8 字节，`i32` 占 4 字节，两者是不同的类型。

### 2.3 `Box` 的所有权与移动

`Box<T>` 不是 `Copy` 类型。`Copy` 表示“赋值时自动复制一份，原变量仍然可用”，例如 `i32`。`Box` 被赋值给另一个变量时，发生的是**移动（move）**：所有权转过去，原变量失效。例子里的 `String` 是“拥有文本内容、长度可变”的字符串类型，第三课已经用过。

```rust,compile_fail
fn main() {
    let first = Box::new(String::from("data"));
    let second = first;
    println!("{}", first);
    println!("{}", second);
}
```

真实诊断：

```text
error[E0382]: borrow of moved value: `first`
 --> moved.rs:4:20
  |
2 |     let first = Box::new(String::from("data"));
  |         ----- move occurs because `first` has type `Box<String>`, which does not implement the `Copy` trait
3 |     let second = first;
  |                  ----- value moved here
4 |     println!("{}", first);
  |                    ^^^^^ value borrowed here after move
  |
help: consider cloning the value if the performance cost is acceptable
  |
3 |     let second = first.clone();
  |                       ++++++++
```

要点：

- 移动不是“把堆上的字符串复制一份”。`String` 的内容留在原处，只是“谁负责清理它”的记录从 `first` 换成了 `second`。
- 移动之后继续用 `first` 会被编译器阻止。这是好事：如果两个变量都以为自己拥有同一块堆内存，清理时就会重复释放。
- 编译器建议的 `.clone()` 会真的复制数据；是否需要复制要看语义，不能为了消除错误就无条件克隆。

### 2.4 `Box` 离开作用域时发生什么

Rust 在值离开作用域时自动调用**析构（drop）**逻辑：先销毁 `Box` 里的 `T`，再把它占用的堆内存归还给分配器（负责管理堆内存、按请求分配和回收空间的程序组件）。你不需要写 `free`，也不需要记得手动释放。

```rust
struct Trace(String);

impl Drop for Trace {
    fn drop(&mut self) {
        println!("销毁 {}", self.0);
    }
}

fn main() {
    let _boxed = Box::new(Trace(String::from("堆上的值")));
    println!("main 结束前");
}
```

输出：

```text
main 结束前
销毁 堆上的值
```

`Drop` 是一个 trait（行为接口），实现它的 `drop` 方法就能在值被清理时执行代码。`Box` 内部会自动完成“先销毁内容、再释放内存”，所以 `Box` 里的值也会按同样的规则析构。同一个作用域里的局部变量按声明的相反顺序清理：后声明的先清理。

## 三、用 `Box` 打断无限递归

现在把递归字段改成 `Box<Expr>`：

```rust
#[derive(Debug)]
enum Expr {
    Literal(i64),
    Neg(Box<Expr>),
    Add(Box<Expr>, Box<Expr>),
}
```

enum 需要在内存里记录“当前到底是哪一个变体”，这个记录通常叫**判别标签（discriminant）**，本质是一个小整数。每个节点的大小变成：

- `Literal`：判别标签 + 8 字节整数；
- `Neg`：判别标签 + 一个 `Box<Expr>`（固定大小的地址）；
- `Add`：判别标签 + 两个 `Box<Expr>`。

enum 的大小取“最大的那个变体”再加上判别标签（编译器可能重排字段以减少浪费，但总量仍是固定的）。`Expr` 不再直接包含一个完整的 `Expr`，只包含指向堆上子树的**地址**，所以大小有限。

```rust
use std::mem::size_of;

#[derive(Debug)]
enum Expr {
    Literal(i64),
    Neg(Box<Expr>),
    Add(Box<Expr>, Box<Expr>),
}

fn main() {
    println!("Box<Expr> = {} 字节", size_of::<Box<Expr>>());
    println!("Expr = {} 字节", size_of::<Expr>());

    let expression = Expr::Add(
        Box::new(Expr::Literal(1)),
        Box::new(Expr::Neg(Box::new(Expr::Literal(2)))),
    );
    println!("{expression:?}");
}
```

在一台常见 64 位机器上，输出类似：

```text
Box<Expr> = 8 字节
Expr = 24 字节
Add(Literal(1), Neg(Literal(2)))
```

`Expr` 的具体字节数会随编译器和平台变化，但**它不会随着树的深度增长**。`Add` 变体里只有两个地址；更深的结构放在堆上，通过地址一层层找到。

构造时要注意 `Box::new` 的嵌套：

```rust
let expression = Expr::Add(
    Box::new(Expr::Literal(1)),
    Box::new(Expr::Neg(Box::new(Expr::Literal(2)))),
);
```

- `Expr::Literal(1)` 是一个 `Expr`；
- `Box::new(...)` 把它放到堆上，得到 `Box<Expr>`；
- 外层的 `Expr::Add` 需要两个 `Box<Expr>`，所以左右两边都包了 `Box::new`；
- `Neg` 里面还要再放一个子表达式，于是出现 `Box::new(Expr::Neg(Box::new(...)))` 这种嵌套。

这里也有一条重要边界：只有**递归路径**需要 `Box`。`Literal(i64)` 里的 `i64` 不是递归的，直接内嵌即可。不要因为“`Box` 看起来高级”就给所有字段都加 `Box`。

## 四、完整示例：借用一条任务链

递归类型不只用于数学表达式。下面是一条“构建步骤”链：每一步有一个名字，并指向下一步，最后以 `Done` 结束。这个例子演示**只借用树**的遍历。

新建一个临时 Cargo 项目（或者把代码放进本课的 `main.rs` 试验），运行：

```rust
#[derive(Debug)]
enum Step {
    Done,
    Next { label: String, rest: Box<Step> },
}

impl Step {
    // 数一数还有多少步。`&self` 表示只借用，不消费。
    fn count(&self) -> usize {
        match self {
            Step::Done => 0,
            Step::Next { rest, .. } => 1 + rest.count(),
        }
    }

    // 按顺序收集每一步的名字。
    fn labels(&self) -> Vec<String> {
        match self {
            Step::Done => Vec::new(),
            Step::Next { label, rest } => {
                let mut labels = vec![label.clone()];
                labels.extend(rest.labels());
                labels
            }
        }
    }
}

fn main() {
    let steps = Step::Next {
        label: String::from("编译"),
        rest: Box::new(Step::Next {
            label: String::from("测试"),
            rest: Box::new(Step::Next {
                label: String::from("发布"),
                rest: Box::new(Step::Done),
            }),
        }),
    };

    println!("步骤数: {}", steps.count());
    println!("步骤: {:?}", steps.labels());
    println!("再次统计: {}", steps.count());
}
```

输出：

```text
步骤数: 3
步骤: ["编译", "测试", "发布"]
再次统计: 3
```

逐点解释：

1. `Step::Next { label, rest }` 是一个带字段的 enum 变体。`rest` 的类型是 `Box<Step>`，保存“下一步”。
2. 递归函数必须有**终止情况（base case）**：`Step::Done` 直接返回 0 或空 Vec，不再调用自己。没有终止情况，函数会一直调用下去，最终耗尽调用栈。
3. `match self` 中 `self` 的类型是 `&Step`。Rust 的**模式匹配（match）**会自动在借用值上匹配，所以 `rest` 拿到的是 `&Box<Step>`，不是把 `Box` 移走。
4. `rest.count()` 看起来像在 `Box<Step>` 上调用 `Step::count`。编译器会自动解引用，找到 `Box` 里面的 `Step`；这一机制在第六节解释。
5. `label` 是 `&String`。`label.clone()` 复制出一个新的 `String`，因为返回的 `Vec<String>` 需要自己拥有内容；原链表中的名字仍然保留。
6. 两次调用 `steps.count()` 都成功，说明 `&self` 方法没有消费树。

补一个类型：`count` 返回的 `usize` 是 Rust 用来表示长度、下标和计数的无符号整数（只能表示 0 及以上的整数）。它的大小随平台变化，在常见 64 位平台上占 8 字节；本课只需要把它当作“非负计数”。

### 递归的代价：调用栈

每进入一层递归，程序就在**调用栈（call stack）**上压入一个栈帧。树很深时，栈帧会不断累积，最终可能**栈溢出（stack overflow）**，程序直接中止。递归数据结构不要求算法也必须递归：如果输入深度不可控，可以改用显式的 `Vec` 当工作栈，用循环逐层处理。本课练习的树深度较小，递归写法足够；但要把“深度可能很大”当成设计风险记住。

## 五、消费递归结构：把数据拆出来

上一节的方法都接收 `&self`，只借看。另一类需求是“把树拆掉，把里面的数据全部拿出来”。这时方法接收 `self`（按值取得所有权），匹配时可以把字段**移出**节点。

下面是同一个 `Step` 类型的另一个方法（片段：省略了上面的 `enum Step` 和 `main`；`impl Step` 里的方法可以放在同一个 `impl` 块中）：

```rust
fn into_labels(self) -> Vec<String> {
    let mut labels = Vec::new();
    let mut current = self;

    loop {
        match current {
            Step::Done => break,
            Step::Next { label, rest } => {
                labels.push(label);
                current = *rest;
            }
        }
    }

    labels
}
```

关键一行是 `current = *rest`：

- `rest` 的类型是 `Box<Step>`；
- `*rest` 把堆上的 `Step` **移动**出来，交给 `current`；
- 这个 `Box` 随后被丢弃，释放它管理的那块堆内存，但不会再次销毁已经移走的 `Step`；
- 循环继续处理新的 `current`，直到 `Done`。

因为 `self` 是按值传入的，整棵树归这个方法所有，它有权把每个节点拆开。这也是 `into_literals` 这类方法能“不克隆就拿到所有数据”的原因。

如果方法只接收 `&self`，就不能这样移出：

```rust
fn take_rest(list: &Step) -> Step {
    match list {
        Step::Next { rest, .. } => **rest,
        Step::Done => Step::Done,
    }
}
```

真实诊断：

```text
error[E0507]: cannot move out of `**tail` which is behind a shared reference
 --> borrowmove.rs:8:32
  |
8 |         List::Cons(_, tail) => **tail,
  |                                ^^^^^^ move occurs because `**tail` has type `List`, which does not implement the `Copy` trait
```

（这份输出用的是链表例子，`tail` 的位置对应上面的 `rest`。）翻译：`**rest` 想从“借来的数据”后面把值搬走，但共享引用 `&T` 只允许读取，不允许改变或搬走它指向的内容。否则原所有者会突然失去一部分数据。

可行的方向取决于你真正想要什么：

- 只想看内容：返回引用 `&T`，或者把需要的小数据复制出来（`i64` 是 `Copy`，复制很便宜）；
- 需要一份独立副本：显式 `clone()`，并接受复制成本；
- 需要拿走原数据：让方法接收 `self`，由调用者交出所有权；
- 字段是 `Option<T>` 时，可以用 `Option::take` 从可变位置“换出”值，留下 `None`。`Option` 的细节见[第四课](../04-option-result/README.md)。

不要为了通过编译就随手 `clone()`。先问：这个函数应该借用、复制，还是消费？这个选择会直接影响调用者的使用方式和程序的性能。

## 六、`Deref` 与自动解引用

`Box<T>` 能像 `T` 一样使用，靠的是标准库 trait `Deref`。trait 在这里可以先理解成“一份行为要求”：实现了 `Deref` 的类型，可以用 `*` 取出它指向的目标。

下面示例用 `assert_eq!` 检查结果：它是标准库提供的宏（宏是编译时展开代码的工具，第一课的 `println!` 就是宏），比较两个值是否相等；不相等就让程序带着错误信息中止。程序能正常跑完，就表示这些检查都通过了。

```rust
fn text_len(text: &str) -> usize {
    text.len()
}

fn main() {
    let text = Box::new(String::from("rust"));
    assert_eq!(text_len(&text), 4);
    assert_eq!(text.len(), 4);
}
```

`&text` 的类型是 `&Box<String>`，而 `text_len` 需要 `&str`。编译器会沿着 `Deref` 一层层转换借用：

```text
&Box<String>  ->  &String  ->  &str
```

这叫**解引用强制转换（deref coercion）**。它只改变“借用的视角”，不会把 `String` 从 `Box` 里搬出来，也不会新建字符串内容。`text.len()` 同理：编译器自动找到 `String` 的 `len` 方法。

再区分三种写法：

- `boxed`：类型 `Box<T>`，拥有里面的 `T`；
- `*boxed`：类型 `T`。如果按值使用，会把 `T` 移出 `Box`（`Box` 允许这样做）；如果 `T` 不是 `Copy`，之后 `boxed` 不能再用；
- `&*boxed`：类型 `&T`，只是借用里面的值，不移出。

自动解引用不是“任意隐式类型转换”。它主要发生在方法调用和编译器已知的借用转换位置，并且依赖类型实现了 `Deref`。遇到“为什么这里能调用那个方法”的问题，先写出表达式的静态类型（编译器在编译期确定的类型，不考虑运行时具体值），再检查中间经过了哪些 `Deref`。

## 七、动态大小类型与 `Sized`

前面说“普通值必须有固定大小”。有些类型偏偏没有：

- `str`：一段文本，长度在运行时才知道；
- `[T]`：一段同类型元素的切片（一段连续数据的借用视图，第五课讲过），元素个数在运行时才知道；
- `dyn Trait`：某个实现了 trait 的类型，具体是哪个类型、占多少字节，可能到运行时才知道。

这些叫**动态大小类型（dynamically sized type，DST）**。它们不能直接作为局部变量按值存在，因为你没法为“大小未知的东西”预留栈空间。它们通常出现在引用或智能指针后面：

```rust
let text: &str = "hello";
let numbers: &[i32] = &[1, 2, 3];
```

`&str` 和 `&[i32]` 是**胖指针（fat pointer）**：除了地址，还带一个长度，所以“指向的数据有多长”在运行时是已知的。指针本身大小固定，因此可以放在栈上。

Rust 有一个标记 trait 叫 `Sized`，表示“大小在编译期已知”。泛型参数默认隐含 `Sized` 要求；泛型（generic）可以先理解为“把类型写成参数，让同一段代码适配多种类型”，第十一课会系统讲。

```rust
fn consume<T>(value: T) {
    // 等价于 fn consume<T: Sized>(value: T)
}
```

如果想让函数接收“可能不是 `Sized` 的借用值”，可以写 `T: ?Sized`。`?Sized` 不是“要求不定长”，而是“取消默认的 `Sized` 限制，定长和不定长都接受”：

```rust
fn byte_len<T: ?Sized>(value: &T) -> usize {
    std::mem::size_of_val(value)
}

fn main() {
    let text: &str = "hello";
    assert_eq!(byte_len(text), 5);
    assert_eq!(byte_len(&42_i32), 4);
}
```

`Box<T>` 是少数允许 `T: ?Sized` 的类型：`Box<str>`、`Box<[i32]>`、`Box<dyn Trait>` 都是合法类型。对于 `Box<dyn Trait>`，`Box` 里保存“指向具体对象的指针 + 指向方法表的元数据”，于是 `Box` 自身有固定大小。

如果把 `dyn Trait` 直接当字段类型，会得到“大小未知”的错误：

```rust,compile_fail
trait Render {
    fn render(&self) -> String;
}

struct Screen {
    item: dyn Render,
}

fn main() {
    println!("{}", std::mem::size_of::<Screen>());
}
```

诊断里会附带一个 `'static`。它是生命周期标注的一种，表示“这个借用可以持续到程序结束”；这里先把它当作编译器给 trait object 附加的默认限制，完整规则见第十四课。真实诊断（节选）：

```text
error[E0277]: the size for values of type `(dyn Render + 'static)` cannot be known at compilation time
  --> dst2.rs:10:40
   |
10 |     println!("{}", std::mem::size_of::<Screen>());
   |                                        ^^^^^^ doesn't have a size known at compile-time
   |
   = help: within `Screen`, the trait `Sized` is not implemented for `(dyn Render + 'static)`
note: required because it appears within the type `Screen`
```

翻译：`Screen` 里有一个大小未知的字段，所以 `Screen` 的大小也无法确定；而 `size_of` 要求类型 `Sized`。`'static` 是生命周期标注的一种，表示“这个借用可以持续到程序结束”；它的完整规则在第十四课，这里只要知道它是编译器附加的默认限制。

## 八、`Box<dyn Trait>` 与动态分发

第十二课介绍过 trait：它规定一组方法签名，具体类型来实现。例如：

```rust
trait Render {
    fn render(&self) -> String;
}
```

现在假设程序里有一组“能渲染成文本”的东西：有的是纯文本，有的是数字，未来还可能加入图片、图表。它们的共同点是都能 `render`，但具体类型不同、大小不同。用一个 `Vec<具体类型>` 装不下不同的类型；用 enum 则每加一种就要改 enum 和所有 `match`。

这时可以用 `Box<dyn Render>`：每个元素都是一个拥有具体对象的盒子，盒子里保存“指向对象的指针 + 方法表”。方法表（通常叫 **vtable**，virtual method table）记录了“对这个具体类型，`render` 应该调用哪段代码”。

完整示例：

```rust
trait Render {
    fn render(&self) -> String;
}

struct Text(String);
struct Number(i64);

impl Render for Text {
    fn render(&self) -> String {
        self.0.clone()
    }
}

impl Render for Number {
    fn render(&self) -> String {
        format!("#{}", self.0)
    }
}

fn main() {
    let items: Vec<Box<dyn Render>> = vec![
        Box::new(Text(String::from("hello"))),
        Box::new(Number(42)),
    ];

    for item in &items {
        println!("{}", item.render());
    }

    println!("一个 Box<dyn Render> 占 {} 字节", std::mem::size_of::<Box<dyn Render>>());
}
```

输出：

```text
hello
#42
一个 Box<dyn Render> 占 16 字节
```

逐点解释：

1. `Vec<Box<dyn Render>>` 里每个元素的类型都是 `Box<dyn Render>`，大小固定；盒子里面可以装不同具体类型。
2. `Box::new(Text(...))` 得到 `Box<Text>`，在需要 `Box<dyn Render>` 的地方会自动转换成 **trait object（特征对象，即“一个实现了该 trait 的值 + 方法表”）**。这个转换叫**向上转型（unsize coercion）**，因为它把“具体大小”变成了“运行时才知道大小”。
3. `item.render()` 调用的是 trait 方法。编译器在运行时通过 vtable 找到 `Text::render` 或 `Number::render`，这叫**动态分发（dynamic dispatch）**。
4. `Box<dyn Render>` 通常是 16 字节：8 字节指向数据，8 字节指向方法表（具体布局由平台和编译器决定，不要把它当作稳定协议）。
5. `Box` 在这里同时解决两件事：给不同大小的对象提供统一的固定大小外壳；独占拥有对象，保证对象活得足够久。

如果漏写 `dyn`，会得到：

```text
error[E0782]: expected a type, found a trait
 --> nody.rs:5:18
  |
5 | fn make() -> Box<Render> {
  |                  ^^^^^^
  |
help: you can add the `dyn` keyword if you want a trait object
  |
5 | fn make() -> Box<dyn Render> {
  |                  +++
```

在 Rust 2024 中，`Render` 是 trait 的名字，不是类型；`dyn Render` 才是“某个实现了 `Render` 的类型”这个类型。

### 动态分发的代价与替代方案

- 每次调用要经过一次间接跳转，通常比静态调用慢一点；
- 编译器更难内联和优化；
- 具体类型的信息被隐藏，某些泛型操作无法使用。

如果类型集合是封闭的、数量不多，enum 加 `match` 往往更简单，也能让编译器检查“有没有漏掉一种情况”。泛型（第十一课）适合“调用方知道具体类型、只是希望代码复用”的场景。`Box<dyn Trait>` 适合“运行时才能决定装什么、需要一个统一容器”的场景。

## 九、`Box` 不总是正确答案

`Box` 是工具，不是“更高级的写法”。先问需求，再选类型：

| 需求 | 更合适的选择 | 原因 |
| --- | --- | --- |
| 递归结构里恰好一个子节点 | `Box<T>` | 提供固定大小的间接层，并表达独占所有权 |
| 子节点数量不固定、类型相同 | `Vec<T>` | 控制结构大小固定，元素在堆上连续存放 |
| 子节点可能没有 | `Option<Box<T>>` | `Option` 表达“有或没有”，`Box` 提供间接层 |
| 只是想读取别处已有的数据 | `&T` 或 `&[T]` | 不取得所有权，不分配新内存 |
| 需要运行时多态（同一接口按具体类型执行不同实现） | `Box<dyn Trait>` | 统一外壳 + vtable 动态分发 |
| 普通的小字段 | 直接内嵌 | 少一次堆分配和一次指针跳转 |

`Option<Box<T>>` 的读法是“可能有、也可能没有的一个盒子”。`Option` 在[第四课](../04-option-result/README.md)讲过：`Some(值)` 表示有，`None` 表示没有。树节点如果允许“左孩子为空”，就会用到它。

常见误区：

- **`Box<Vec<T>>`**：`Vec<T>` 本身已经是固定大小的控制结构，再套一层 `Box` 只是多一次堆分配和一次跳转，通常没有收益。
- **`Box<String>`**：`String` 也已经是固定大小的控制结构；除非确实需要“把整个 String 值固定在某个堆地址上”，否则直接存 `String` 更简单。
- **以为 `Box` 能共享**：`Box<T>` 仍然只有一个所有者。需要多个所有者时用 `Rc<T>`（第十五课）或 `Arc<T>`（第十七课），而不是复制 `Box`。
- **以为 `Box` 自动线程安全**：`Box<T>` 能不能跨线程，取决于 `T` 是否满足 `Send`/`Sync`，与“在堆上”无关。
- **以为堆一定比栈快或慢**：堆分配有额外成本；小值直接放在栈上通常更简单也更快。选择 `Box` 的理由应是“需要间接层/动态大小/独占堆对象”，而不是“堆听起来更厉害”。

## 十、常见编译错误与排错

下面每一条都给出真实诊断的核心部分和排查顺序。

### 10.1 `E0072`：递归类型无限大

```text
error[E0072]: recursive type `Expr` has infinite size
  |
3 |     Neg(Expr),
  |         ---- recursive without indirection
```

排查：

1. 找到错误指向的递归字段；
2. 看它是否直接内嵌了自身类型；
3. 在递归路径上加入 `Box<T>`、`Vec<T>`、`Rc<T>` 或引用，任选一种符合语义的间接层；
4. 不要给整个 enum 外面套 `Box` 就完事，要看具体哪个字段造成循环。

### 10.2 `E0382`：使用了已经移动的值

```text
error[E0382]: borrow of moved value: `first`
  |
3 |     let second = first;
  |                  ----- value moved here
4 |     println!("{}", first);
  |                    ^^^^^ value borrowed here after move
```

排查：

1. 找到“value moved here”的那一行，确认所有权去了哪里；
2. 问清楚：后面真的还需要原变量吗？
3. 需要两份独立数据就 `clone()`；只需要看就用 `&` 借用；只需要传下去就不要再保留原变量。

### 10.3 `E0507`：不能从共享引用后面移出值

```text
error[E0507]: cannot move out of `**tail` which is behind a shared reference
  |
8 |         List::Cons(_, tail) => **tail,
  |                                ^^^^^^ move occurs because `**tail` has type `List`, which does not implement the `Copy` trait
```

排查：

1. 当前方法的接收者是 `&self` 还是 `self`？
2. 如果只是读取，返回引用或复制 `Copy` 小值；
3. 如果必须拿走，改成接收 `self`，或者用 `Option::take` 从 `&mut` 位置换出；
4. 不要用 `clone()` 掩盖所有权设计问题。

### 10.4 `E0782`：把 trait 名当成类型

```text
error[E0782]: expected a type, found a trait
  |
5 | fn make() -> Box<Render> {
  |                  ^^^^^^
help: you can add the `dyn` keyword if you want a trait object
```

修复：写 `Box<dyn Render>`。同理，`&Render` 要写 `&dyn Render`。

### 10.5 `E0277`：大小在编译期未知

```text
error[E0277]: the size for values of type `(dyn Render + 'static)` cannot be known at compilation time
```

排查：

1. 是不是把 `str`、`[T]`、`dyn Trait` 直接当成了字段或局部变量类型？
2. 放到 `&`、`Box`、`Rc` 等指针后面；
3. 泛型函数如果只需借用，考虑 `T: ?Sized`。

### 10.6 运行期问题：递归太深导致栈溢出

编译能通过，运行时却可能出现：

```text
thread 'main' has overflowed its stack
fatal runtime error: stack overflow
```

这不是借用检查错误，而是调用栈被递归层数耗尽。处理方式：限制输入深度；或者改用显式 `Vec` 作为工作栈的循环算法。递归数据结构本身没有问题，问题在算法是否假设了“深度一定很小”。

### 10.7 测试失败：`not yet implemented`

第一次运行本课测试时，会看到大量失败，这是脚手架的正常状态，不是测试写错：

```text
running 35 tests
test exercise::tests::box_has_a_fixed_size_however_deep_the_tree_is ... ok
test exercise::tests::literal_evaluates_to_itself ... FAILED
...
thread 'exercise::tests::literal_evaluates_to_itself' panicked at lessons/13-box-recursion/src/exercise.rs:54:9:
not yet implemented
...
test result: FAILED. 1 passed; 34 failed
```

`todo!()` 一旦被执行就会 panic（程序中止），并打印 `not yet implemented`。主练习的 23 个测试中，只有不调用待实现方法的 `box_has_a_fixed_size_however_deep_the_tree_is` 能通过；其余 22 个以及课后巩固的 12 个都会因为 `todo!()` 失败。实现后应全部通过。

### 10.8 一个查看表达式类型的小技巧

不确定某个表达式的静态类型时，可以故意把它赋给 `()`：

```rust
let boxed = Box::new(1);
let _: () = boxed;
```

编译器会报：

```text
error[E0308]: mismatched types
expected `()`, found `Box<i32>`
```

错误信息里就写出了真实类型。确认类型后，再决定用 `boxed`、`*boxed` 还是 `&*boxed`。

## 十一、代码预测

先不要运行，写下答案，再展开对照。

### 预测 1

```rust
let boxed = Box::new(vec![1, 2, 3]);
let moved = boxed;
let length = moved.len();
println!("{length}");
```

<details><summary>解析</summary>

能够编译。`boxed` 的所有权移动到 `moved`，之后 `boxed` 不能再用。`moved.len()` 通过自动解引用借用内部的 `Vec`，不会消费它。输出 `3`。

如果把最后一行改成 `println!("{length} {}", boxed.len());`，会因为借用已移动的值而报 `E0382`。

</details>

### 预测 2

```rust
let boxed = Box::new(String::from("rust"));
let text: &str = &boxed;
println!("{text}");
```

<details><summary>解析</summary>

能够编译。`&boxed` 的类型是 `&Box<String>`，经过 `Box<String> -> String -> str` 的解引用链转换成 `&str`。这里只借用，不移动 `String`，也不复制字符串内容。

</details>

### 预测 3

```rust
let mut boxed = Box::new(1_i32);
let copied = *boxed;
*boxed = 2;
println!("{copied} {boxed}");
```

<details><summary>解析</summary>

能够编译，输出 `1 2`。`i32` 是 `Copy`，`*boxed` 复制出值 1，`boxed` 仍然拥有堆上的整数；随后把堆上的值改成 2。如果换成 `Box<String>`，`*boxed` 会移动字符串，之后再给 `*boxed` 赋值就需要重新放入一个 `String`，代码含义完全不同。

</details>

### 预测 4

```rust
enum List {
    Cons(i32, Box<List>),
    Nil,
}

let list = List::Cons(1, Box::new(List::Cons(2, Box::new(List::Nil))));
println!("{}", std::mem::size_of::<List>());
```

<details><summary>解析</summary>

能够编译。指针大小固定，`List` 不会随链表长度增长。在常见 64 位平台上 `List` 通常是 16 字节（判别标签 + `i32` + 一个 8 字节指针，具体布局由编译器决定）。如果把 `Box<List>` 改回 `List`，就会得到 `E0072`。

</details>

## 十二、主练习：可求值、可遍历的算术表达式树

### 12.1 目标与输入输出

打开 `src/exercise.rs`。类型 `Expr` 已经定义好，包含字面量、取负、加、减、乘、除六种变体，递归字段全部是 `Box<Expr>`。

练习使用的真实输入是这棵树：

```rust
Expr::Subtract(
    Box::new(Expr::Multiply(
        Box::new(Expr::Add(
            Box::new(Expr::Literal(1)),
            Box::new(Expr::Literal(2)),
        )),
        Box::new(Expr::Literal(4)),
    )),
    Box::new(Expr::Neg(Box::new(Expr::Literal(3)))),
)
```

也就是数学式 `(1 + 2) * 4 - (-3)`。期望结果：

```text
eval()          -> Ok(15)
depth()         -> 4
literals()      -> [1, 2, 4, 3]
into_literals() -> [1, 2, 4, 3]
```

错误输入示例：

```text
Divide(Literal(7), Literal(0))          -> Err(DivisionByZero)
Add(Literal(i64::MAX), Literal(1))      -> Err(Overflow)
Neg(Literal(i64::MIN))                  -> Err(Overflow)
Divide(Literal(i64::MIN), Literal(-1))  -> Err(Overflow)
```

### 12.2 需要完成的方法

| 方法 | 接收者 | 作用 | 失败方式 |
| --- | --- | --- | --- |
| `eval(&self)` | 借用整棵树 | 递归求值 | 返回 `Err(EvalError)` |
| `depth(&self)` | 借用整棵树 | 统计深度 | 无失败情况 |
| `literals(&self)` | 借用整棵树 | 从左到右收集字面量 | 无失败情况 |
| `into_literals(self)` | 取得所有权 | 消费整棵树，收集字面量 | 无失败情况 |

完整的行为约定写在 `src/exercise.rs` 每个方法上方的注释里。特别注意：

- `eval` 先算左边，再算右边；左边出错就返回左边的错误，不再计算右边；
- 整数除法向 0 截断；
- 所有会溢出 `i64` 的运算都返回 `Err(EvalError::Overflow)`，不能让程序 panic；
- `depth` 中单个字面量深度为 1，二元节点取左右较大深度再加 1；
- `literals` 不消费树，调用后还能继续 `eval`、`depth` 和再次收集；
- `into_literals` 必须消费树，逐个节点移出 `i64`，不能为了借用而保留整棵树，也不能克隆整棵子树。

### 12.3 约束

- 只修改 `src/exercise.rs` 中四个方法体里的 `todo!()`；不要改类型定义、方法签名和已有测试。
- 不要引入新的依赖。
- 不要使用 `unsafe`。
- 在 `eval`、`depth`、`literals` 中不要克隆整棵树；`i64` 是 `Copy`，复制叶子里的整数是允许的。
- `into_literals` 接收 `self`，应真正消费树，而不是调用 `self.literals()` 后返回。
- 允许使用标准库的检查型算术方法（名字里带 `checked_` 的方法），它们把溢出变成 `Option`，不满足条件时返回 `None`。

### 12.4 推荐完成顺序

1. 先实现 `eval` 的普通递归：只处理 `Literal` 和几个运算，先让 `sample_tree` 得到 15。
2. 再补 `eval` 的错误分支：除零、溢出、错误先后顺序。
3. 实现 `depth`，先写字面量，再写一元和二元节点。
4. 实现 `literals`，用一个 `Vec` 累积结果，想清楚先左后右。
5. 最后实现 `into_literals`，匹配时让模式把 `Box` 里的值移出来。
6. 运行 `cargo run -p l13_box_recursion` 对照演示输出。

### 12.5 测试与验收

主练习共有 23 个测试，覆盖：

- 正常：字面量、嵌套取负、混合运算、整数除法截断；
- 边界：深度 1、左右深度不同、`i64::MIN`、`i64::MIN / -1`；
- 错误：除零、加/减/乘溢出、对 `i64::MIN` 取负、左错误优先、右错误向上传递；
- 所有权：借用后仍可求值、`into_literals` 结果与借用版一致；
- 概念：`Box<Expr>` 大小固定。

在实现前运行，只有概念测试通过，其余 22 个因 `todo!()` 失败。实现后运行：

```bash
cargo test -p l13_box_recursion exercise
cargo run -p l13_box_recursion
cargo fmt -p l13_box_recursion -- --check
cargo clippy -p l13_box_recursion --all-targets -- -D warnings
```

`cargo test -p l13_box_recursion exercise` 里的 `exercise` 是测试名过滤器，只运行主练习相关测试；不带过滤器的 `cargo test -p l13_box_recursion` 会连同课后巩固一起运行。

## 十三、课后巩固：目录树统计与查找

主练习用 `Box` 表示“恰好一个/两个子表达式”。巩固任务换成“子节点数量不固定”的目录树，递归字段改用 `Vec<Entry>`。数据和边界都与主练习不同。

打开 `src/consolidation.rs`。示例目录：

例子里文件大小使用 `u64`：无符号 64 位整数，只能表示 0 到 18,446,744,073,709,551,615。文件大小不会是负数，所以这里不用 `i64`。

```text
root/
├── src/
│   ├── main.rs   (120 字节)
│   └── util.rs   (30 字节)
├── notes/        (空目录)
└── README.md     (200 字节)
```

期望结果：

```text
total_size()  -> 350
file_count()  -> 3
max_depth()   -> 3
find_path("util.rs") -> Some(["root", "src", "util.rs"])
find_path("notes")   -> Some(["root", "notes"])
find_path("missing.rs") -> None
```

需要完成四个方法：

- `total_size(&self)`：文件返回自己的大小，目录返回所有子项之和；空目录为 0。
- `file_count(&self)`：文件计 1，目录累加子项；空目录为 0，目录本身不计入。
- `max_depth(&self)`：文件和空目录深度为 1；非空目录是 `1 + 子项最大深度`。
- `find_path(&self, target)`：先比较当前条目，再按 `children` 顺序深度优先查找；返回从当前条目到目标的完整路径；找不到返回 `None`；同名条目返回最先遇到的。

边界要求：

- 空目录的深度是 1，不是 0；
- 大小为 0 的文件仍然计入文件个数；
- 两个不同分支有同名文件时，返回存储顺序靠前的那一个；
- 名字可能包含中文等非 ASCII 字符，路径必须原样保留；
- 两个文件大小相加可以恰好达到 `u64::MAX`，不能因为中间计算而溢出。

巩固任务共有 12 个测试，覆盖正常统计、空目录、深层分支、零字节文件、`u64` 上界、Unicode 名字、根节点查找、空目录查找、同名优先级和缺失返回 `None`。验收命令：

```bash
cargo test -p l13_box_recursion consolidation
```

也可以直接运行全部测试：

```bash
cargo test -p l13_box_recursion
```

卡住时看 [`HINTS.md`](HINTS.md) 的课后巩固部分，同样分三级。

## 十四、本课总结

- Rust 的普通值需要编译期已知的大小；递归字段直接内嵌自身会导致无限大小（`E0072`）。
- `Box<T>` 是固定大小的拥有型堆指针，用地址把递归“接起来”，让类型大小有限。
- `Box<T>` 独占所有权；移动 `Box` 只转移所有权，不复制堆上的 `T`；离开作用域时自动析构并释放堆内存。
- `&self` 方法适合借用遍历；`self` 方法适合消费整棵树并逐层移出数据。不能从共享引用后面移出非 `Copy` 值（`E0507`）。
- `Deref` 让借用和方法调用可以自动穿过 `Box`；“自动”不等于任意隐式转换。
- `str`、`[T]`、`dyn Trait` 是动态大小类型，通常放在引用或智能指针后面；`Box<dyn Trait>` 用数据指针加 vtable 支持动态分发。
- 递归路径上恰好一个子节点用 `Box`，数量不定用 `Vec`，可能没有用 `Option<Box<T>>`，只是借用用 `&T`。
- 递归算法受调用栈深度限制；深度不可控时改用显式工作栈的循环。

## 下一课

本课的主练习里，`literals(&self) -> Vec<i64>` 返回的是复制出来的整数；如果想让方法返回树内部的引用，例如 `fn first(&self) -> Option<&i64>`，返回值的有效期就与输入借用绑在一起。下一课[第十四课：生命周期](../14-lifetimes/README.md)会系统解释这种关系：生命周期不负责“让引用活得更久”，而是描述多个引用之间必须满足的有效期约束。
