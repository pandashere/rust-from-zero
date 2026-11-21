# 第九课：闭包——把一段逻辑当作值传递

假设你写了一个处理整数列表的程序。今天需要把每个数乘 2，明天需要把每个数加上 10，后天需要只挑出大于 20 的数，再后天还需要一边处理一边统计“到底处理了几个”。

如果每换一种规则就写一个新函数，代码会变成一堆只差一两行的 `double`、`add_ten`、`is_large`。更麻烦的是“统计处理了几个”：这个计数必须从第一次调用一直记到最后一次，而普通的具名函数（用 `fn` 定义、有名字的函数）没有地方存放这种只属于某一次任务的设置。

本课要学的**闭包（closure）**就是为解决这类问题而设计的：它是一段**可以像数字、文本一样被保存、传递和返回的逻辑**，并且能**记住定义它时周围环境里的变量**。学会闭包之后，你才能读懂下一课的 `map`、`filter`，也才能理解线程、回调（callback，交给别人在合适时机调用的函数）和很多标准库函数的参数。

## 本课定位

### 你将学会

- 写出没有参数、一个参数和多个参数的闭包，并知道什么时候必须写类型。
- 把闭包理解成“编译器生成的匿名结构体实例”：**匿名结构体**是源码里没有名字、由编译器自动生成的结构体；捕获的变量就是它的字段。
- 区分三种捕获方式：只读借用、可变借用、按值拥有（`move`）。
- 理解 `Fn`、`FnMut`、`FnOnce` 三个 trait 分别要求“怎样访问闭包保存的环境”。
- 给函数参数选择最弱的必要约束，而不是一律要求 `Fn`。
- 从函数返回闭包，理解为什么返回类型写 `impl Fn(...)`、为什么通常要加 `move`。
- 区分闭包和函数指针 `fn(...)`，知道各自不能做什么。
- 读懂闭包相关的最常见编译错误，并知道往哪个方向修。

### 前置知识

本课默认你完成了前八课。下面用一两句话复习会用到的内容；不熟悉时请点链接回到对应课程。

- **函数、参数、返回值、表达式、变量**：见[第一课](../01-basics/README.md)。复习：函数是一段有名字、可以被反复调用的代码；参数是调用时交给它的值；返回值是它交回给调用者的值；表达式是能算出一个值的写法。
- **结构体、字段、方法、`&self` 与 `&mut self`**：见[第二课](../02-data-modeling/README.md)。复习：结构体把几个相关的值装在一起，装进去的每一项叫字段；`&self` 表示方法只借数据来看，`&mut self` 表示方法要借来修改。
- **所有权、移动（move）、`Copy`、共享引用 `&T`、可变引用 `&mut T`**：见[第三课](../03-ownership-borrowing/README.md)。复习：每份数据同一时间只有一个所有者，所有者负责在数据不再使用时释放它；把非 `Copy` 类型的值交给别人会让原变量失效，这叫移动；`&T` 是“借来看”，可以有多个；`&mut T` 是“借来改”，同一时间只能有一个。
- **`Option`、`Result`、`?`**：见[第四课](../04-option-result/README.md)。复习：`Option<T>` 表示“可能有值（`Some`），也可能没有（`None`）”；`Result<T, E>` 表示“成功（`Ok`）或失败（`Err`）”；`?` 在失败时提前返回错误。
- **`Vec`、切片 `&[T]`、`&mut [T]`**：见[第五课](../05-vec-slices/README.md)。复习：`Vec<T>` 是长度可变的列表；`&[T]` 是对一段连续数据的只读视图；`&mut [T]` 是可以修改的视图。
- **测试与 `cargo test`**：见[第八课](../08-testing-quality/README.md)。复习：`#[test]` 标记一个测试函数；测试失败会打印哪一行断言没有通过。

### 本课边界

本课不系统讲以下内容，遇到时会给出“当前够用”的解释并指出后续课程：

- 迭代器（iterator，能按顺序逐个产出元素的值）的 `map`、`filter` 等方法链：下一课[第十课](../10-iterators/README.md)专门讲。本课只在需要时使用最简单的 `iter()` 或 `for` 循环。
- 泛型（generic，让同一份代码适用于多种类型的写法）的完整规则、`impl Trait` 的所有用法：第 11 课[泛型](../11-generics/README.md)和第 12 课[trait](../12-traits/README.md)会系统展开。本课把 `Fn` 当作“能被调用的能力”来用，只解释到能读懂函数签名（参数类型和返回类型的声明）为止。
- 线程（thread，程序内部可以同时推进的一段执行流程）、`Send`/`Sync`（描述一个类型能否安全地跨线程传递或共享的约束）：第 16 课[线程](../16-threads-channels/README.md)会讲。本课提到线程时只用来说明 `move` 的一个动机。
- 显式生命周期（一个值或引用能合法使用的时间范围）标注：第 14 课[生命周期](../14-lifetimes/README.md)会讲。本课用“闭包不能比它借用的数据活得更久”这一句话解释相关报错。

本课不引入任何第三方依赖。练习只需要 Rust 标准库（安装 Rust 时自带、可以直接使用的代码库）。

## 如何学习本课

1. 先读第一到第三节，在纸上画出“闭包保存了哪些变量、以什么方式保存”。
2. 每个代码块先自己预测输出或能否编译，再看解释。
3. 第十节的编译错误不要跳过。能读懂报错，比背语法更能节省时间。
4. 做完第十一节的代码预测，再打开 `src/exercise.rs` 做主练习。
5. 主练习完成后做 `src/consolidation.rs` 的课后巩固。
6. 卡住时按顺序看 `HINTS.md` 的三级提示，不要一开始就看第三级。

预计用时：概念与示例约 1.5 小时，主练习约 1 小时，课后巩固约 40 分钟。

## 一、为什么需要闭包

### 1.1 旧办法：每换一种规则就写一个新函数

先只用前八课的知识写一个“把列表里每个数都处理一遍”的函数：

```rust
fn double(value: i32) -> i32 {
    value * 2
}

fn add_ten(value: i32) -> i32 {
    value + 10
}

fn apply_all(values: &mut [i32], operation: fn(i32) -> i32) {
    for value in values {
        *value = operation(*value);
    }
}

fn main() {
    let mut numbers = [1, 2, 3];
    apply_all(&mut numbers, double);
    println!("{numbers:?}");
}
```

输出：

```text
[2, 4, 6]
```

这里出现了两个新名词：

- **函数指针（function pointer）**：类型写成 `fn(i32) -> i32`，读作“指向一个接收 `i32`、返回 `i32` 的函数的地址”。**地址**是内存中某个位置的编号；函数指针只记录“代码在哪里”，不记录任何数据。注意小写的 `fn` 是类型名；定义函数时用的 `fn` 是关键字，两者长得像但角色不同。
- **把函数当参数**：`double` 这个函数名可以直接传给 `apply_all`，因为它的形状正好符合 `fn(i32) -> i32`。

这段代码能工作，但它把“做什么”固定成了一种**没有设置的函数**。如果明天要“每个数加 10”，可以再写 `add_ten`；如果后天要“每个数加 20”，难道再写 `add_twenty`？你当然可以让函数多接收一个参数：

```rust
fn add_amount(value: i32, amount: i32) -> i32 {
    value + amount
}
```

但 `apply_all` 的参数类型是 `fn(i32) -> i32`，只接收一个参数，`add_amount` 塞不进去。要么改 `apply_all` 的签名（参数类型和返回类型的声明），让它额外接收 `amount`，要么为每一种设置写一个包装函数。每增加一种“设置”，接口就多一个参数，最后 `apply_all` 会变得又长又难用。

### 1.2 具名函数没有地方保存“这一次任务的设置”

再看一个更棘手的需求：处理每个元素时，统计“已经处理了几个”。

```rust
fn apply_all_counting(values: &mut [i32], operation: fn(i32, usize) -> i32) -> usize {
    let mut count = 0;
    for value in values {
        count += 1;
        *value = operation(*value, count);
    }
    count
}
```

这样写能工作，但计数被迫变成 `operation` 的第二个参数，所有处理函数都必须关心它。若还有“阈值”“前缀”“已经重试了几次”“上一次的错误”等设置，参数会继续膨胀。全局可变变量可以绕过参数，但它让状态对所有代码可见，测试之间还会互相影响，不是好办法。

问题的根源是：**普通函数只有代码，没有属于自己的存储空间来保存一次任务的环境。** 我们需要的值是“一段逻辑 + 这段逻辑要用的设置 + 这段逻辑自己的小状态”。

### 1.3 闭包：把逻辑和环境一起保存

闭包把这三样东西合成一个值：

```rust
fn main() {
    let threshold = 10;

    let is_large = |value: i32| value > threshold;

    assert!(is_large(12));
    assert!(!is_large(3));
}
```

逐项解释：

- `|value: i32|` 是参数列表。闭包用一对竖线 `|` 包住参数，而不是圆括号。
- `value > threshold` 是闭包体，也就是每次调用时要执行的表达式。
- `threshold` 不在参数列表里，它是闭包**捕获（capture，记住并使用）**的外部变量。
- `is_large` 是一个变量，它的值就是闭包。闭包可以像 `i32`、`String` 一样被保存和传递。
- `is_large(12)` 是**调用（call）**这个闭包：在值后面加圆括号并传入参数。

把闭包传给 `apply_all`，只需要把参数类型从函数指针换成“任何能这样调用的值”：

```rust
fn apply_all(values: &mut [i32], operation: impl Fn(i32) -> i32) {
    for value in values {
        *value = operation(*value);
    }
}

fn main() {
    let offset = 10;
    let mut numbers = [1, 2, 3];

    apply_all(&mut numbers, |value| value + offset);

    println!("{numbers:?}");
}
```

输出：

```text
[11, 12, 13]
```

`impl Fn(i32) -> i32` 读作“某个实现了 `Fn(i32) -> i32` 能力的类型”。目前只需要把它理解成“可以接收一个 `i32`、返回一个 `i32` 的可调用值”：

- **trait（特征）**：Rust 用来描述“一个类型具备哪些行为”的机制。`Fn` 是标准库里的一个 trait，表示“这个值可以像函数一样被调用”。trait 的完整规则在第 12 课，本课只需要会读这种参数类型。
- **`impl Trait`**：在参数位置表示“接受任何实现了这个 trait 的具体类型”，在返回位置表示“返回某个实现了这个 trait 的具体类型，但不在签名里写出它的名字”。第 11、12 课会展开。

闭包解决了具名函数解决不了的事：**逻辑和它要用的环境被绑定在同一个值里。** 它没有解决的事也要说清楚：闭包仍然受所有权和借用规则约束；捕获外部变量会让借用关系变长；它也不是“自动线程安全”的魔法。

## 二、闭包的基本语法

### 2.1 从零参数到多参数

```rust
fn main() {
    // 没有参数：一对空的竖线
    let answer = || 42;

    // 一个参数：类型可以省略，由使用处推断
    let double = |value: i32| value * 2;

    // 多个参数：用逗号分隔
    let add = |left: i32, right: i32| left + right;

    // 函数体不止一个表达式时，用大括号包住，最后一行是返回值
    let describe = |value: i32| {
        let doubled = value * 2;
        format!("{value} doubled is {doubled}")
    };

    assert_eq!(answer(), 42);
    assert_eq!(double(4), 8);
    assert_eq!(add(2, 3), 5);
    assert_eq!(describe(5), "5 doubled is 10");
}
```

`format!` 是标准库提供的文本模板宏（宏：在编译前按固定规则展开代码的工具；这里只需要会调用它）：`"{value}"` 会被替换成变量 `value` 的内容，返回一个 `String`。前几课已经用过它。

### 2.2 与具名函数 `fn` 的差别

```rust
fn double_named(value: i32) -> i32 {
    value * 2
}

fn main() {
    let double_closure = |value: i32| value * 2;

    assert_eq!(double_named(3), 6);
    assert_eq!(double_closure(3), 6);
}
```

两者的行为在这里一样，但有四个关键差别：

| 对比项 | 具名函数 `fn double(...)` | 闭包 `\|value\| ...` |
| --- | --- | --- |
| 有没有名字 | 有 | 没有；类型名由编译器生成 |
| 能不能捕获外部变量 | 不能 | 能 |
| 参数和返回类型 | 签名必须写完整 | 通常可以省略，由上下文推断 |
| 出现在哪里 | 定义在模块或函数里 | 是一个表达式，可以出现在任何“值”能出现的位置 |

最后一条很重要：闭包可以放在变量右边、函数实参里、数组元素里，也可以作为返回值。具名函数不是表达式，不能像值一样随便搬动（函数名可以转换成函数指针，但那是另一回事）。

### 2.3 类型推断：闭包不是泛型函数

闭包的参数类型可以省略，但省略不等于“对任何类型都能用”。编译器会在**第一次确定类型**时把参数类型固定下来，之后所有调用必须使用同一类型：

```rust
fn main() {
    let show = |value| println!("{value}");

    show(1);                        // 第一次调用：value 被固定为整数
    show(String::from("text"));     // 第二次调用：类型不匹配
}
```

编译器的关键诊断是：

```text
error[E0308]: mismatched types
5 |     show(String::from("text"));
  |          ^^^^^^^^^^^^^^^^^^^^ expected integer, found `String`
  |
note: expected because the closure was earlier called with an argument of type `{integer}`
```

这条规则常被误解。泛型函数 `fn show<T: Display>(value: T)` 会为每种实参类型分别生成代码；闭包只有一个具体类型，推断只发生一次。

如果闭包既没有被调用，上下文也没有给出足够信息，编译器就不知道参数该是什么类型：

```rust
fn main() {
    let double = |value| value * 2;
}
```

```text
error[E0284]: type annotations needed
3 |     let double = |value| value * 2;
  |                   ^^^^^        - type must be known at this point
  |
  = note: cannot satisfy `<_ as Mul<i32>>::Output == _`
help: consider giving this closure parameter an explicit type
```

`*` 来自 `Mul` trait，很多类型都能相乘，编译器无法替你做选择。加上类型标注即可：

```rust
fn main() {
    let double = |value: i32| value * 2;
    assert_eq!(double(4), 8);
}
```

也可以先把它传给一个明确要求 `Fn(i32) -> i32` 的函数，由那个位置提供类型信息。

## 三、闭包是一个值

### 3.1 存在变量里、传进函数、从函数返回

```rust
fn apply(value: i32, operation: impl Fn(i32) -> i32) -> i32 {
    operation(value)
}

fn main() {
    let increment = |value: i32| value + 1;

    let first = increment;          // 把闭包值交给另一个变量
    let result = apply(3, first);   // 再把闭包值传给函数

    assert_eq!(result, 4);
}
```

这里的 `let first = increment;` 会不会让 `increment` 失效？会不会像 `String` 一样发生移动？答案取决于这个闭包捕获了什么、是否实现 `Copy`（第三课介绍过：`Copy` 类型的赋值会复制，其他类型会移动）；第四、五节会解释捕获方式如何影响这一点。现在先记住：**闭包值和其他值一样，遵守所有权规则。**

### 3.2 编译器生成的“匿名结构体”

理解闭包最有用的模型是：

> 编译器为每一个闭包表达式（也就是源码里写下 `|参数| 表达式` 的那段代码）生成一个没有名字的结构体；闭包捕获的变量成为这个结构体的字段；闭包值就是这个结构体的一个实例。

例如：

```rust
fn main() {
    let threshold = 10;
    let is_large = |value: i32| value > threshold;

    assert!(is_large(12));
}
```

概念上，编译器生成的东西大致相当于：

```rust,ignore
struct __Closure<'a> {
    threshold: &'a i32, // 捕获的变量成为字段
}

impl<'a> Fn<(i32,)> for __Closure<'a> {
    type Output = bool;
    // 真实实现细节与这里不同，这里只表达“字段 + 调用方法”
}
```

真实的类型没有名字，trait 的实现细节也更复杂，但这个模型能直接推出下面这些结论：

- **闭包有确定的大小**：大小等于捕获字段的大小之和。不捕获任何变量的闭包是零大小的。
- **闭包有所有权**：谁持有闭包值，谁就持有它捕获的数据。
- **闭包有生命周期**：按引用捕获时，闭包不能比被引用的数据活得更久。
- **字段会随闭包一起释放**：按值捕获的 `String` 在闭包被释放时释放。
- **每个闭包表达式类型不同**：不同位置的两个闭包表达式会生成两个不同的匿名结构体。

用 `std::mem::size_of_val` 可以看到“捕获的变量就是字段”：

```rust
fn main() {
    let value = 0_i32;
    let empty = || 42;
    let capturing = move || value;

    assert_eq!(std::mem::size_of_val(&empty), 0);
    assert_eq!(std::mem::size_of_val(&capturing), 4);

    assert_eq!(capturing(), 0);
}
```

`size_of_val` 接收一个引用，返回这个值占多少**字节**（内存大小的基本单位）。`i32` 占 4 字节，所以捕获一个 `i32` 的闭包占 4 字节。不捕获任何东西的闭包没有任何字段，占 0 字节。

### 3.3 每个闭包表达式都是不同的类型

即使两个闭包的参数和返回值完全一样，它们也是两个不同的具体类型：

```rust
fn main() {
    let mut operations = Vec::new();

    operations.push(|value: i32| value + 1);
    operations.push(|value: i32| value * 2);
}
```

```text
error[E0308]: mismatched types
5 |     operations.push(|value: i32| value * 2);
  |                     ^^^^^^^^^^^^^^^^^^^^^^ expected closure, found a different closure
  |
  = note: no two closures, even if identical, have the same type
```

`Vec<T>` 要求所有元素是同一个类型。第一个 `push` 让 `operations` 变成“装着第一种匿名闭包类型的 `Vec`”，第二个闭包是另一种类型，所以塞不进去。这不是编译器故意刁难，而是结构体模型的直接结果：你在两个位置生成了两个不同的 `struct`。

如果确实需要在一个集合里放不同的闭包，常见方案有三种：

1. **都不捕获环境时，转成函数指针 `fn(i32) -> i32`**：类型统一，不分配堆内存。**堆**是程序运行时按需申请、由程序自己负责释放的一块内存区域。
2. **用 `Box<dyn Fn(i32) -> i32>`**：`Box` 是一个“拥有堆上数据”的指针；`dyn Fn(...)` 表示“某种实现了 `Fn` 的类型，具体是哪种运行时才确定”。每个闭包单独装箱，调用时经过一次间接跳转，这叫**动态分发**（运行时根据值的实际类型决定执行哪段代码），与之相对的是**静态分发**（编译期就确定调用目标）。`Box` 和 `dyn` 的细节在第 12、13 课。
3. **用枚举把行为编码成数据**：枚举（enum）是“只能取若干有限种类之一”的类型；例如 `enum Operation { Add(i32), Mul(i32) }`，再写一个 `apply` 方法用 `match` 按种类处理。没有堆分配，但每新增一种操作都要改枚举和所有相关的 `match`。

选哪种取决于你是否真的需要“运行时才决定放哪种闭包”。单个调用点通常用泛型或 `impl Fn` 就够了，不必提前装箱。

## 四、捕获环境的三种方式

闭包要使用外部变量，就必须把变量“放进”自己的字段。放法有三种：借来看、借来改、直接拥有。编译器会根据闭包体**实际怎么使用**这个变量，自动选择能满足需求的最弱方式。

### 4.1 只读：共享借用

```rust
fn main() {
    let threshold = 10;
    let is_large = |value: i32| value > threshold;

    assert!(is_large(12));
    println!("{threshold}"); // 仍然可以直接读
}
```

闭包体只读取 `threshold`，所以编译器让闭包的字段保存一个 `&i32`，相当于在创建闭包时做了一次共享借用。共享借用不排斥其他共享借用，所以闭包存续期间仍然可以直接读 `threshold`。

但不能再修改 `threshold`：

```rust
fn main() {
    let mut threshold = 10;
    let is_large = |value: i32| value > threshold;

    threshold = 20; // 不能编译
    assert!(is_large(12));
}
```

```text
error[E0506]: cannot assign to `threshold` because it is borrowed
 --> src/main.rs:5:5
  |
3 |     let is_large = |value: i32| value > threshold;
  |                    ------------         --------- borrow occurs due to use in closure
  |                    |
  |                    `threshold` is borrowed here
4 |
5 |     threshold = 20;
  |     ^^^^^^^^^^^^^^ `threshold` is assigned to here but it was already borrowed
6 |     assert!(is_large(12));
  |             -------- borrow later used here
```

原因：闭包在调用时会读取 `threshold`，所以它一直持有 `threshold` 的共享借用，直到 `is_large` 最后一次使用。`threshold = 20` 是一次写入，需要独占访问，与尚未结束的共享借用冲突；编译器因此拒绝。

修复要看意图：如果希望闭包使用新阈值，就先改完再创建闭包；如果闭包已经用完，就让它的最后一次使用早于修改。

### 4.2 修改：可变借用

```rust
fn main() {
    let mut calls = 0;
    let mut count_call = || {
        calls += 1;
        calls
    };

    assert_eq!(count_call(), 1);
    assert_eq!(count_call(), 2);
}
```

闭包体修改了 `calls`，所以字段是 `&mut i32`。这带来两个后果。

**第一，`&mut` 是独占的。** 闭包还需要再次调用时，外部不能直接碰 `calls`：

```rust
fn main() {
    let mut calls = 0;
    let mut count_call = || calls += 1;

    calls = 10; // 不能编译
    count_call();
}
```

**第二，闭包变量本身必须用 `mut` 声明。** 调用一个会修改环境的闭包，相当于对它调用一个需要 `&mut self` 的方法；对一个值调用 `&mut self` 方法，要求这个值本身可变：

```rust
fn main() {
    let mut calls = 0;
    let count_call = || calls += 1; // 少了 mut

    count_call(); // 不能编译
}
```

```text
error[E0596]: cannot borrow `count_call` as mutable, as it is not declared as mutable
```

修复是写成 `let mut count_call`。注意两个 `mut` 的含义不同：

- `let mut calls` 允许闭包修改 `calls`；
- `let mut count_call` 允许以可变方式调用闭包本身。

### 4.3 按值：`move`

默认情况下，编译器优先借用。`move` 关键字改变这一点：它强制闭包**按值捕获**，把变量移动进闭包（`Copy` 类型则是复制）：

```rust
fn main() {
    let text = String::from("owned");

    let print = move || println!("{text}");

    print();
    print();
    println!("闭包仍然可以调用多次");
}
```

`text` 在闭包创建时就被移动进闭包，闭包成为这个 `String` 的所有者。闭包体只是借用 `text` 打印，没有把它移出去，所以这个闭包仍然可以调用任意多次。这一点非常重要，第五节会再次强调：**`move` 不等于“只能调用一次”。**

对 `Copy` 类型，`move` 捕获的是副本，原变量仍然可用：

```rust
fn main() {
    let number = 42;
    let show = move || println!("{number}");

    show();
    println!("{number}"); // i32 是 Copy，原值仍可用
}
```

对非 `Copy` 类型，原变量会失效：

```rust
fn main() {
    let text = String::from("owned");
    let print = move || println!("{text}");

    print();
    println!("{text}"); // 不能编译：borrow of moved value
}
```

```text
error[E0382]: borrow of moved value: `text`
  |
3 |     let text = String::from("owned");
  |         ---- move occurs because `text` has type `String`, which does not implement the `Copy` trait
4 |     let print = move || println!("{text}");
  |                 -------            ---- variable moved due to use in closure
  |                 |
  |                 value moved into closure here
...
6 |     println!("{text}");
  |                ^^^^ value borrowed here after move
```

如果两边都需要数据，明确 `clone` 一份给闭包（`clone` 是“复制一份拥有型数据”的方法），而不是为了消掉报错到处乱加 `move`：

```rust
fn main() {
    let text = String::from("owned");
    let text_for_closure = text.clone();
    let print = move || println!("{text_for_closure}");

    print();
    println!("{text}");
}
```

先想清楚闭包是否真的需要独立拥有一份数据。只读场景下，共享借用往往就够了。

### 4.4 捕获是精确的

闭包只捕获它实际用到的变量，甚至可能只捕获用到的字段：

```rust
struct Config {
    name: String,
    limit: usize,
}

fn main() {
    let config = Config {
        name: String::from("demo"),
        limit: 10,
    };

    let show_limit = || println!("limit = {}", config.limit);

    show_limit();
    println!("{}", config.name); // name 没有被捕获，仍可使用
}
```

闭包只借用了 `config.limit` 这个字段路径，所以 `config.name` 不受影响。阅读借用错误时，注意“是哪个字段以什么方式被捕获”，不要只看外层变量名。

### 4.5 借用持续到闭包最后一次使用

按引用捕获的闭包，字段里存着引用。引用会一直有效到闭包最后一次被使用。下面的代码可以编译：

```rust
fn main() {
    let mut values = vec![1, 2, 3];

    let mut append = || values.push(4);
    append();
    println!("{values:?}"); // append 最后一次使用已经过去，借用结束
}
```

但如果闭包之后还要用，`println!` 就夹在借用期间：

```rust
fn main() {
    let mut values = vec![1, 2, 3];

    let mut append = || values.push(4);
    println!("{values:?}"); // 不能编译
    append();
}
```

```text
error[E0502]: cannot borrow `values` as immutable because it is also borrowed as mutable
  |
4 |     let mut append = || values.push(4);
  |                      -- ------ first borrow occurs due to use of `values` in closure
  |                      |
  |                      mutable borrow occurs here
5 |     println!("{values:?}");
  |                ^^^^^^ immutable borrow occurs here
6 |     append();
  |     ------ mutable borrow later used here
```

分析这类错误时，标出闭包变量的**最后一次使用**。借用从创建点持续到那里，而不是简单地“到大括号结束”。修复可以调整顺序、缩小闭包作用域，或者用 `drop(append)` 提前结束闭包值（`drop` 立即丢弃一个值，并触发它的清理逻辑）。

## 五、`Fn`、`FnMut`、`FnOnce`

### 5.1 三个 trait 描述“调用时需要怎样访问环境”

调用闭包看起来像函数调用，实际是在调用闭包类型实现的方法。三个 trait 的区别只在接收者：

```text
trait      调用方法的接收者    调用时可以做什么        至少能调用几次
FnOnce     self                可以移出捕获值          一次
FnMut      &mut self           可以修改捕获值          多次
Fn         &self               只能读取捕获值          多次
```

- **`FnOnce`**：调用会拿走整个闭包值（`self`），所以闭包里的字段可以被移出。任何闭包都至少能这样调用一次。
- **`FnMut`**：调用只借用闭包值的可变引用（`&mut self`），所以不能把字段移出，但可以修改字段。这样的闭包可以调用多次。
- **`Fn`**：调用只借用闭包值的共享引用（`&self`），所以既不能移出，也不能修改字段，只能读取。这样的闭包可以调用多次。

三者是能力层级关系：

```text
Fn  ⊂  FnMut  ⊂  FnOnce
```

读法是：一个 `Fn` 闭包也可以出现在要求 `FnMut` 或 `FnOnce` 的位置；一个 `FnMut` 闭包也可以出现在要求 `FnOnce` 的位置。反过来不成立。设计 API 时要求越弱，能接收的闭包越多。

### 5.2 编译器如何决定实现哪些 trait

不需要手动声明。编译器看闭包体如何使用每个捕获值，选择能满足需求的最强实现：

```rust
fn main() {
    let text = String::from("rust");
    let read = || text.len();          // 只读：Fn + FnMut + FnOnce
    assert_eq!(read(), 4);

    let mut buffer = String::from("rust");
    let mut write = || buffer.push('!'); // 修改：FnMut + FnOnce
    write();

    let owned = String::from("rust");
    let consume = || drop(owned);       // 移出：只有 FnOnce
    consume();
}
```

注意第三个闭包没有写 `move`：`drop(owned)` 需要取得 `owned` 的所有权，按引用捕获无法满足，所以编译器自动按值捕获。`move` 的作用只是把“能借用就借用”的默认行为改成“一律按值”，它不会让闭包变得“更弱”。

### 5.3 `FnOnce` 的典型用法

`FnOnce` 适合“这个回调只会被调用一次”的场景。接收方只调用一次，就应该声明 `FnOnce`，这样连会消耗捕获数据的闭包也能传进来：

```rust
fn run_once(operation: impl FnOnce() -> String) -> String {
    operation()
}

fn main() {
    let payload = String::from("done");
    let result = run_once(move || payload);

    assert_eq!(result, "done");
}
```

这里的闭包把捕获的 `String` 作为返回值移出环境，因此只实现 `FnOnce`。`run_once` 只调用一次，所以完全够用。

### 5.4 要求更弱的约束

需要多次调用且允许闭包带状态时，用 `FnMut`；只读共享调用时，用 `Fn`：

```rust
fn call_twice(mut operation: impl FnMut()) {
    operation();
    operation();
}

fn main() {
    let mut count = 0;
    call_twice(|| count += 1);
    assert_eq!(count, 2);
}
```

`call_twice` 的参数绑定必须写 `mut`，因为调用 `FnMut` 闭包需要可变借用它。如果写成 `fn call_twice(operation: impl Fn())`，上面这个会修改 `count` 的闭包就传不进去，编译器会报 E0525（第十节）。

标准库大量使用这种思路：只调用一次的回调接收 `FnOnce`，会对每个元素反复调用的适配器接收 `FnMut`，确实需要共享调用时才要求 `Fn`。要求过强不会带来好处，只会拒绝本来合法的闭包。

## 六、闭包作为函数参数

把闭包作为参数有三种常见写法：

```rust
// 写法一：impl Fn，最常用；本质上是泛型参数的简写
fn apply(value: i32, operation: impl Fn(i32) -> i32) -> i32 {
    operation(value)
}

// 写法二：显式泛型参数；可以在 where 里写更复杂的约束
fn apply_generic<F>(value: i32, operation: F) -> i32
where
    F: Fn(i32) -> i32,
{
    operation(value)
}

// 写法三：trait object（把实现了同一 trait 的不同具体类型统一看成一种类型）；函数只有一份，调用走动态分发
fn apply_boxed(value: i32, operation: &dyn Fn(i32) -> i32) -> i32 {
    operation(value)
}
```

调用时都可以传闭包，也可以传形状匹配的具名函数：

```rust
fn double(value: i32) -> i32 {
    value * 2
}

fn main() {
    assert_eq!(apply(3, |value| value + 1), 4);
    assert_eq!(apply(3, double), 6);
}
```

选择建议：

- 单个调用点、编译期已知具体闭包：用 `impl Fn`。编译器通常会内联（把函数调用处的代码直接展开，省去调用开销），没有堆分配。
- 需要把不同闭包放进同一个集合，或需要在运行时选择实现：用 `&dyn Fn` 或 `Box<dyn Fn>`。
- 只调用一次：把约束（bound，即“要求某个类型具备什么能力”的条件）改成 `FnOnce`。
- 要修改捕获状态：用 `FnMut`，并在参数绑定上加 `mut`。

## 七、闭包作为返回值

### 7.1 为什么返回类型写 `impl Fn`

闭包的具体类型是匿名的，源码里没有名字可写。所以函数返回闭包时，用 `impl Fn(...)` 表示“返回某个实现了这个能力的类型，但不暴露具体是哪一个”：

```rust
fn make_adder(amount: i32) -> impl Fn(i32) -> i32 {
    move |value| value + amount
}

fn main() {
    let add_five = make_adder(5);

    assert_eq!(add_five(3), 8);
    assert_eq!(add_five(10), 15);
}
```

`add_five` 是一个普通值，可以绑定、可以调用。调用方不需要知道匿名的结构体名字。

### 7.2 为什么返回闭包通常需要 `move`

`amount` 是 `make_adder` 的局部变量。函数返回后，它的栈帧就消失了。**栈帧**是函数调用时用来保存局部变量的那块内存，函数返回时会被回收。如果闭包只借用 `amount`，返回出去的闭包就会握着一个已经失效的引用。编译器会拒绝：

```rust
fn make_shower() -> impl Fn() {
    let text = String::from("local");
    || println!("{text}")
}
```

```text
error[E0373]: closure may outlive the current function, but it borrows `text`, which is owned by the current function
3 |     || println!("{text}")
  |     ^^            ---- `text` is borrowed here
  |     |
  |     may outlive borrowed value `text`
  |
help: to force the closure to take ownership of `text` (and any other referenced variables), use the `move` keyword
  |
3 |     move || println!("{text}")
  |     ++++
```

加 `move` 后，闭包把 `text` 的所有权一起带走，不再依赖已经销毁的栈帧：

```rust
fn make_shower() -> impl Fn() {
    let text = String::from("local");
    move || println!("{text}")
}

fn main() {
    let shower = make_shower();
    shower();
}
```

经验规则：**从函数返回的闭包几乎总是需要 `move`**，因为它必须拥有自己用到的一切，不能引用诞生它的那个栈帧。

### 7.3 返回 `FnMut` 与返回 `FnOnce`

返回带状态的闭包时，返回类型可以写 `impl FnMut(...)`。调用方拿到闭包后，必须用 `mut` 绑定才能反复调用：

```rust
fn make_adder(amount: i32) -> impl FnMut(i32) -> i32 {
    let mut calls = 0;
    move |value| {
        calls += 1;
        value + amount + calls
    }
}

fn main() {
    let mut add = make_adder(10);

    assert_eq!(add(1), 12); // 10 + 1 + 1
    assert_eq!(add(1), 13); // 10 + 1 + 2
}
```

返回只会执行一次的闭包时，写 `impl FnOnce(...)`：

```rust
fn make_consumer(text: String) -> impl FnOnce() -> String {
    move || text
}

fn main() {
    let consume = make_consumer(String::from("once"));
    assert_eq!(consume(), "once");
}
```

`impl FnOnce` 是三种返回类型里要求最弱的：任何闭包都至少能调用一次，所以返回 `impl FnOnce` 最容易被满足。但调用方也只能调用一次，这是有意的约束。

## 八、函数指针与闭包

普通函数，以及**不捕获任何环境**的闭包，都可以转换成函数指针：

```rust
fn double(value: i32) -> i32 {
    value * 2
}

fn apply(value: i32, function: fn(i32) -> i32) -> i32 {
    function(value)
}

fn main() {
    assert_eq!(apply(3, double), 6);
    assert_eq!(apply(3, |value| value * 2), 6);
}
```

函数指针在内存里只是一个代码地址，不带任何数据。不捕获环境的闭包是零字段的匿名结构体，没有状态可带，所以可以退化成纯代码指针。

捕获环境的闭包**不能**转成函数指针，因为指针没有地方放环境：

```rust
fn main() {
    let offset = 1;
    let shifted: fn(i32) -> i32 = |value| value + offset;
}
```

```text
error[E0308]: mismatched types
4 |     let shifted: fn(i32) -> i32 = |value| value + offset;
  |                  --------------   ^^^^^^^^^^^^^^^^^^^^^^ expected fn pointer, found closure
  |
note: closures can only be coerced to `fn` types if they do not capture any variables
```

需要环境时，用 `impl Fn` 或 `dyn Fn`，不要用函数指针。

## 九、完整示例：带状态的重试器

下面是一个完整程序，演示 `FnMut`、带状态的闭包、`Result` 和循环如何配合。它模拟“连接服务，失败就重试，最多尝试若干次”：

```rust
fn retry(
    attempts: usize,
    mut operation: impl FnMut() -> Result<String, String>,
) -> Result<String, String> {
    let mut last_error = String::new();

    for _ in 0..attempts {
        match operation() {
            Ok(text) => return Ok(text),
            Err(error) => last_error = error,
        }
    }

    Err(last_error)
}

fn main() {
    let mut calls = 0;

    let result = retry(3, || {
        calls += 1;
        if calls < 3 {
            Err(format!("attempt {calls} failed"))
        } else {
            Ok(String::from("connected"))
        }
    });

    println!("result: {result:?}");
    println!("calls: {calls}");
}
```

输出：

```text
result: Ok("connected")
calls: 3
```

逐点分析：

- **为什么参数是 `FnMut`**：`operation` 会被调用多次（重试的全部意义），并且允许它携带状态。这里的闭包修改了外部的 `calls`，属于 `FnMut`。
- **为什么不是 `Fn`**：`Fn` 要求只能读取捕获值，会拒绝修改 `calls` 的闭包。
- **为什么不是 `FnOnce`**：`FnOnce` 调用一次就消耗闭包值，`for` 循环里的第二次调用无法通过编译。
- **为什么参数绑定写 `mut operation`**：调用 `FnMut` 闭包需要可变借用它。
- **为什么循环结束后可以读 `calls`**：闭包 `operation` 只在 `retry` 调用期间存在，调用结束后它对 `calls` 的可变借用也结束。

这个例子有一个设计问题：`attempts` 为 0 时，循环一次也不执行，函数返回一个空错误字符串，调用者无法区分“没尝试过”和“尝试过但错误信息为空”。真实代码应把这种情况作为配置错误处理，或者在文档里明确要求 `attempts >= 1`。示例只聚焦闭包，没有展开完整的错误设计。

## 十、常见编译错误与排错

下面每条诊断都来自真实编译器输出（Rust 1.94，edition 2024）。先读代码和报错，再看“根因”和“修复方向”。

### 10.1 类型推断失败：`type annotations needed`（E0284）

```rust
fn main() {
    let double = |value| value * 2;
}
```

```text
error[E0284]: type annotations needed
3 |     let double = |value| value * 2;
  |                   ^^^^^        - type must be known at this point
  |
  = note: cannot satisfy `<_ as Mul<i32>>::Output == _`
```

**根因**：闭包没有被调用，上下文也没有要求某种具体类型，编译器不知道 `value` 是什么。`*` 对很多类型都成立，不能替你选。

**修复**：写参数类型 `|value: i32|`，或者先把闭包传给一个签名明确的函数。

### 10.2 调用 `FnMut` 闭包却少了 `mut`（E0596）

```rust
fn main() {
    let mut calls = 0;
    let count_call = || calls += 1;

    count_call();
}
```

```text
error[E0596]: cannot borrow `count_call` as mutable, as it is not declared as mutable
  |
3 |     let count_call = || calls += 1;
  |                         ----- calling `count_call` requires mutable binding due to mutable borrow of `calls`
4 |
5 |     count_call();
  |     ^^^^^^^^^^ cannot borrow as mutable
  |
help: consider changing this to be mutable
  |
3 |     let mut count_call = || calls += 1;
```

**根因**：这个闭包修改 `calls`，只实现 `FnMut`；调用它需要可变借用闭包值本身。

**修复**：把绑定改成 `let mut count_call`。注意这和 `let mut calls` 是两个不同的 `mut`。

### 10.3 闭包的可变借用与外部访问冲突（E0502）

```rust
fn main() {
    let mut values = vec![1, 2, 3];
    let mut append = || values.push(4);

    println!("{values:?}");
    append();
}
```

```text
error[E0502]: cannot borrow `values` as immutable because it is also borrowed as mutable
  |
3 |     let mut append = || values.push(4);
  |                      -- ------ first borrow occurs due to use of `values` in closure
  |                      |
  |                      mutable borrow occurs here
5 |     println!("{values:?}");
  |                ^^^^^^ immutable borrow occurs here
6 |     append();
  |     ------ mutable borrow later used here
```

**根因**：闭包持有 `values` 的可变借用，而 `println!` 之后还要调用闭包，所以借用一直有效。

**修复**：把 `println!` 移到 `append()` 之后；缩小闭包作用域；或者 `drop(append)`。更根本的做法是让函数接收数据参数，而不是捕获外部状态。

### 10.4 返回闭包却漏了 `move`（E0373）

```rust
fn make_shower() -> impl Fn() {
    let text = String::from("local");
    || println!("{text}")
}
```

```text
error[E0373]: closure may outlive the current function, but it borrows `text`, which is owned by the current function
  |
3 |     || println!("{text}")
  |     ^^            ---- `text` is borrowed here
  |     |
  |     may outlive borrowed value `text`
help: to force the closure to take ownership of `text` (and any other referenced variables), use the `move` keyword
  |
3 |     move || println!("{text}")
  |     ++++
```

**根因**：闭包被返回后可能比函数栈帧活得更久，但它只借用了局部变量。

**修复**：加 `move`，让闭包带走所有权。若闭包只需要数据的一小部分，也可以在函数里先算出结果再按值捕获。

### 10.5 `move` 之后又使用原变量（E0382）

```rust
fn main() {
    let text = String::from("owned");
    let print = move || println!("{text}");

    print();
    println!("{text}");
}
```

```text
error[E0382]: borrow of moved value: `text`
  |
2 |     let text = String::from("owned");
  |         ---- move occurs because `text` has type `String`, which does not implement the `Copy` trait
3 |     let print = move || println!("{text}");
  |                 -------            ---- variable moved due to use in closure
...
6 |     println!("{text}");
  |                ^^^^ value borrowed here after move
```

**根因**：`move` 对非 `Copy` 类型是真正的所有权转移。

**修复**：如果双方都需要数据，明确 `clone` 一份给闭包；如果闭包只需要短期借用，去掉 `move`。

### 10.6 把 `FnOnce` 闭包调用两次

```rust
fn main() {
    let text = String::from("once");
    let consume = move || text;

    let _first = consume();
    let _second = consume();
}
```

```text
error[E0382]: use of moved value: `consume`
  |
5 |     let _first = consume();
  |                  --------- `consume` moved due to this call
6 |     let _second = consume();
  |                   ^^^^^^^ value used here after move
  |
note: closure cannot be invoked more than once because it moves the variable `text` out of its environment
```

**根因**：闭包体把捕获的 `String` 移出环境，闭包只实现 `FnOnce`。第一次调用已经把闭包值消耗掉了。

**修复**：如果本来就只调用一次，让接收方的 bound 是 `FnOnce`；如果需要多次调用，闭包体不要移出捕获值：返回克隆、借用，或者改为读取。

### 10.7 用 `Fn` 约束接收 `FnMut` 闭包（E0525）

```rust
fn call_twice(operation: impl Fn()) {
    operation();
    operation();
}

fn main() {
    let mut count = 0;
    let mut increment = || count += 1;
    increment();
    call_twice(increment);
}
```

```text
error[E0525]: expected a closure that implements the `Fn` trait, but this closure only implements `FnMut`
  |
8 |     let mut increment = || count += 1;
  |                         ^^ ----- closure is `FnMut` because it mutates the variable `count` here
  |                         |
  |                         this closure implements `FnMut`, not `Fn`
```

**根因**：闭包会修改捕获的 `count`，只实现 `FnMut`；接收方却要求 `Fn`。

**修复**：如果接收方确实要调用多次且允许状态，把约束改成 `FnMut`，参数绑定加 `mut`。如果 `Fn` 是真实需求（例如闭包要放进只读共享结构），就让闭包停止修改捕获值，把状态改成参数或返回值传递。

### 10.8 捕获环境的闭包不能转成函数指针（E0308）

```rust
fn main() {
    let offset = 1;
    let shifted: fn(i32) -> i32 = |value| value + offset;
}
```

```text
error[E0308]: mismatched types
3 |     let shifted: fn(i32) -> i32 = |value| value + offset;
  |                  --------------   ^^^^^^^^^^^^^^^^^^^^^^ expected fn pointer, found closure
  |
note: closures can only be coerced to `fn` types if they do not capture any variables
```

**根因**：函数指针只保存代码地址，没有地方保存 `offset`。

**修复**：改用 `impl Fn(i32) -> i32` 参数、`Box<dyn Fn>`，或者把环境显式作为额外参数传入。

### 10.9 两个闭包不能放进同一个 `Vec`（E0308）

```rust
fn main() {
    let mut operations = Vec::new();
    operations.push(|value: i32| value + 1);
    operations.push(|value: i32| value * 2);
}
```

```text
error[E0308]: mismatched types
4 |     operations.push(|value: i32| value * 2);
  |                     ^^^^^^^^^^^^^^^^^^^^^^ expected closure, found a different closure
  |
  = note: no two closures, even if identical, have the same type
```

**根因**：每个闭包表达式生成独立的匿名类型，`Vec<T>` 要求元素同类型。

**修复**：不需要捕获时转成 `fn` 函数指针；需要异构时用 `Box<dyn Fn>`；操作集合固定时用枚举。具体取舍见第 3.3 节。

## 十一、代码预测

先不要运行，写出你的判断和理由，再展开答案。

### 预测 1

```rust
let text = String::from("owned");
let show = move || text.len();

assert_eq!(show(), 5);
assert_eq!(show(), 5);
```

这段能编译吗？为什么？

<details><summary>答案与解析</summary>

能编译。`move` 让闭包按值拥有 `text`，但闭包体只调用 `len()` 读取长度，没有把 `String` 移出环境。因此这个闭包实现 `Fn`，可以调用任意多次。`move` 只决定捕获方式，不决定能否多次调用。

</details>

### 预测 2

```rust
let mut count = 0;
let increment = || count += 1;

increment();
```

这段能编译吗？如果不能，编译器会要求改哪里？

<details><summary>答案与解析</summary>

不能编译。闭包修改了 `count`，只实现 `FnMut`；调用它需要可变借用闭包值本身，而 `increment` 没有用 `mut` 声明。把绑定改成 `let mut increment` 即可。

注意：`let mut count` 已经允许闭包修改 `count`，但它不允许你以可变方式调用闭包。两个 `mut` 管的是不同的事。

</details>

### 预测 3

```rust
fn make_adder(amount: i32) -> impl Fn(i32) -> i32 {
    |value| value + amount
}
```

这段能编译吗？为什么？

<details><summary>答案与解析</summary>

不能编译。`amount` 是函数的局部变量，函数返回后栈帧销毁；闭包只借用 `amount`，返回出去会变成悬空引用。编译器报 E0373，并建议在闭包前加 `move`。加上 `move` 后，闭包拥有 `amount` 的副本（`i32` 是 `Copy`），可以安全返回。

</details>

### 预测 4

```rust
let mut operations = Vec::new();
operations.push(|value: i32| value + 1);
operations.push(|value: i32| value * 2);
```

这段能编译吗？如果不能，有哪几种修法？

<details><summary>答案与解析</summary>

不能编译。两个闭包表达式生成两个不同的匿名类型，`Vec<T>` 不能同时装两种类型。修法：

- 两个闭包都不捕获环境，可以显式转成函数指针：`let mut operations: Vec<fn(i32) -> i32> = Vec::new();`，然后 `push(|value| value + 1)`。
- 用 `Vec<Box<dyn Fn(i32) -> i32>>`，每个闭包装箱；代价是堆分配和动态分发。
- 用枚举把操作编码成数据，再写一个 `apply` 方法。

</details>

## 十二、主练习：事件处理管线

### 目标

实现一个事件处理管线：先用过滤器决定“这条事件要不要处理”，再用处理器把通过过滤的事件变成一行文本。两个闭包都由调用者提供，函数本身不关心具体规则。

练习文件是 `src/exercise.rs`。你已经得到完整的类型定义、函数签名、行为注释和 15 个测试；需要完成 5 个函数体。**只替换 `todo!()`，不要修改类型定义、函数签名和测试。**

### 数据结构

```rust
pub struct Event {
    pub id: u64,
    pub value: i32,
}
```

- `id` 是事件编号，用于追踪是哪一条事件；
- `value` 是整数数值，过滤器通常根据它判断。

### 真实输入与期望输出

设输入事件为：

```text
(编号 1, 数值 5)
(编号 2, 数值 10)
(编号 3, 数值 15)
```

过滤器“数值至少为 10”，处理器把事件变成 `"编号:数值"`：

```text
输入：上述三条事件
输出：Ok(["2:10", "3:15"])
处理器实际收到的编号：2、3
```

再看错误情况：过滤器全部通过，但处理器在编号 2 时返回错误：

```text
输入：上述三条事件
处理器：编号 1 成功，编号 2 返回错误 Rejected(2)
输出：Err(Rejected(2))
处理器实际收到的编号：1、2
```

编号 3 **不会**被处理：第一个错误出现后立即停止。

### 五个函数的行为

1. **`apply_all(values: &mut [i32], operation: impl FnMut(i32) -> i32)`**
   - 从前往后处理每个整数，用 `operation(旧值)` 覆盖原位置；
   - 直接修改切片，不返回新集合；
   - 空切片什么也不做；
   - 闭包可以携带状态，例如统计调用次数。

2. **`first_match(values: &[i32], predicate: impl Fn(i32) -> bool) -> Option<(usize, i32)>`**
   - 从下标 0 开始检查（下标是从 0 开始的位置编号）；
   - 第一个让 `predicate` 返回 `true` 的元素，返回 `Some((下标, 值))`；
   - 没有匹配或输入为空时返回 `None`。

3. **`run_once(operation: impl FnOnce() -> String) -> String`**
   - 只调用闭包一次，返回它产生的文本；
   - 允许闭包消耗自己捕获的数据，例如把捕获的 `String` 移出来。

4. **`process_events<E>(events: &[Event], filter: impl FnMut(&Event) -> bool, handler: impl FnMut(&Event) -> Result<String, E>) -> Result<Vec<String>, E>`**
   - 按原顺序逐条处理；
   - 先调用 `filter`：返回 `false` 就跳过，**不调用** `handler`；
   - 返回 `true` 时调用 `handler`：`Ok(text)` 追加到结果，`Err(error)` 立即返回该错误；
   - 出错时丢弃本次已收集的文本，调用者只看到错误；
   - 空输入或全部被过滤时返回 `Ok(空 Vec)`；
   - 两个闭包都可以修改自己的捕获状态。

5. **`make_threshold_filter(threshold: i32) -> impl Fn(&Event) -> bool`**
   - 返回一个记住 `threshold` 的过滤器；
   - `event.value >= threshold` 时返回 `true`，否则 `false`；
   - 阈值相等也算通过（包含边界）；
   - 返回的闭包可以反复调用。

### 约束

- 不使用全局可变状态；所有状态都通过闭包捕获。
- 不修改测试和函数签名。
- 处理顺序必须与输入一致。
- 被过滤的事件不能调用处理器。
- `process_events` 在第一个处理器错误处停止。
- 正常输入不应 panic（panic 是程序遇到无法继续的错误时立即中止）；错误通过 `Result` 返回。
- 如果暂时不需要某个参数，脚手架已经用 `let _ = &参数;` 避免了“未使用”警告。写好实现后删除那些占位行。

### 推荐完成顺序

1. `run_once`：最短，先观察 `FnOnce` 如何允许闭包移出捕获的 `String`。
2. `first_match`：练习返回 `Option`、下标和提前返回。
3. `make_threshold_filter`：练习返回闭包，并确认 `move` 的必要性。
4. `apply_all`：练习 `FnMut` 参数和就地修改。
5. `process_events`：最后综合过滤器、处理器、顺序和 `?` 错误传播。

### 测试说明

主练习共有 **15 个测试**，覆盖：

- 正常输入：逐个替换、保留顺序、返回第一个匹配、按格式生成文本；
- 边界：空切片、无匹配、全部被过滤、阈值相等；
- 错误：处理器在中间事件失败，后续事件不再处理；
- 状态：处理闭包统计调用次数；
- 概念：`move` 闭包只读拥有型数据时仍可调用两次。

实现前运行测试，**14 个**测试会在 `todo!()` 处失败；只有 `move_closure_that_only_reads_owned_text_can_run_twice` 会通过。这不是测试写错了，而是你的实现还没有提供。

### 验收命令

```bash
# 先确认脚手架能编译（不需要实现）
cargo test -p l09_closures --no-run

# 运行全部测试；实现完成后应全部通过
cargo test -p l09_closures

# 只运行主练习的测试
cargo test -p l09_closures exercise

# 严格检查代码风格和常见问题
cargo clippy -p l09_closures --all-targets -- -D warnings
```

如果 `first_match` 使用 `for index in 0..values.len()` 这种按下标取值的写法，Clippy（Rust 自带的代码检查工具）可能提示 `needless_range_loop`；换成 `while` 循环即可。下一课学了迭代器后，也可以用 `values.iter().enumerate()`。主练习的行为测试不检查你用了哪种循环。

## 十三、课后巩固：编号、标签与一次性汇总

这个任务换一组数据和边界：不再处理事件，而是构造三个闭包并返回给调用者。文件是 `src/consolidation.rs`，同样只替换 `todo!()`，不要改签名和测试。

### 任务一：编号器

```rust
pub fn make_counter(start: u32) -> impl FnMut() -> u32
```

返回一个闭包，每次调用产出一个编号：

- 第一次返回 `start`；
- 之后每次返回上一次的编号加 1；
- 到达 `u32` 的最大值后保持不变，不再增加（避免溢出）；
- 每次调用 `make_counter` 得到独立的编号器。

真实输入输出示例：

```text
make_counter(7) 连续调用三次 → 7, 8, 9
make_counter(u32::MAX) 连续调用两次 → u32::MAX, u32::MAX
两个分别从 10 和 100 开始的编号器 → 10/100、11/101，互不影响
```

### 任务二：贴标签器

```rust
pub fn make_labeler(prefix: String) -> impl Fn(&str) -> String
```

返回一个拥有 `prefix` 的闭包：

- 对文本 `text` 返回 `"{prefix}: {text}"`；
- 空文本也保留前缀和分隔符，例如 `"note"` 对 `""` 返回 `"note: "`；
- 可以反复调用，每次都用同一份前缀，不会把前缀消耗掉。

真实输入输出示例：

```text
make_labeler(String::from("note"))("hello") → "note: hello"
make_labeler(String::from("log"))("start")   → "log: start"
make_labeler(String::from("log"))("stop")    → "log: stop"
```

### 任务三：一次性汇总

```rust
pub fn make_outbox_summary(messages: Vec<String>) -> impl FnOnce() -> String
```

返回一个接管整批消息的闭包：

- 调用一次，按原顺序用 `" | "` 连接所有消息；
- 不在开头或结尾额外添加分隔符；
- 空消息也占一个位置，例如两个空消息连接后是 `" | "`；
- 空批次返回空 `String`；
- 闭包只应调用一次，它可以把捕获的 `Vec` 消耗掉。

真实输入输出示例：

```text
["alpha", "beta", "gamma"] → "alpha | beta | gamma"
[]                         → ""
["", "middle", ""]         → " | middle | "
["solo"]                   → "solo"
```

### 测试与验收

课后巩固共有 **11 个测试**，覆盖正常调用、空输入、空消息、编号独立性、`u32::MAX` 边界和多次调用。实现前，11 个测试都会在各自的 `todo!()` 处失败。

```bash
cargo test -p l09_closures consolidation
cargo clippy -p l09_closures --all-targets -- -D warnings
```

## 十四、常见误区

1. **“`move` 闭包一定是 `FnOnce`。”**
   不对。`move` 只决定捕获方式（字段里存值还是引用）。闭包体只读取捕获值时，它仍然实现 `Fn`，可以调用多次。只有闭包体把捕获值移出环境时，才只实现 `FnOnce`。

2. **“闭包参数是泛型，所以可以传不同类型。”**
   不对。闭包的参数类型在第一次确定后固定，后续调用必须使用同一类型。泛型函数才是“对每种类型分别生成代码”。

3. **“`let mut count` 就够了，闭包调用不需要 `mut`。”**
   两个 `mut` 管不同的事：`let mut count` 允许闭包修改 `count`；`let mut closure` 允许以可变方式调用闭包本身。修改捕获值的闭包通常两个都需要。

4. **“闭包捕获的是整个结构体，所以不能再用其他字段。”**
   不一定。闭包可以精确捕获实际用到的字段。只有当闭包以“整体”方式使用结构体（例如调用 `&self` 方法、把 `&config` 传给别的函数）时，才会捕获整个变量。

5. **“返回闭包加 `move` 就一定是 `FnOnce`。”**
   不对。返回 `impl Fn` 还是 `impl FnOnce` 取决于闭包体如何使用捕获值。`move` 只是让闭包拥有数据，拥有数据的闭包仍然可以只读地实现 `Fn`。

6. **“两个闭包长得一样，所以是同一个类型。”**
   不对。每个闭包表达式生成独立的匿名类型，即使源码完全相同也不共享类型。异构集合（元素类型各不相同的集合）要用函数指针、`Box<dyn Fn>` 或枚举。

7. **“`Fn` 是一种具体类型，可以直接写 `let f: Fn(i32) -> i32`。”**
   不对。`Fn` 是 trait，不是类型。需要写 `impl Fn(...)`（静态分发，编译期确定具体类型）或 `dyn Fn(...)`（动态分发，运行时确定），并且 `dyn` 通常还要放在 `Box` 或引用后面。

8. **“闭包一定在堆上分配。”**
   不对。捕获环境的闭包是普通的匿名结构体值，通常直接放在栈上；只有你显式使用 `Box<dyn Fn>` 时才发生堆分配。

9. **“闭包可以随便修改外部变量，只要加 `mut`。”**
   不对。闭包持有外部变量的借用期间，外部不能以冲突的方式访问它。冲突是否发生，取决于闭包最后一次使用的位置。

10. **“`FnOnce` 只能调用一次，所以它没用。”**
    不对。`FnOnce` 是最弱的约束，能接收所有闭包；一次性初始化、完成回调、消费型适配器都适合用它。要求 `Fn` 反而会拒绝合法的闭包。

## 十五、本课总结

1. 闭包是**能捕获环境的一段逻辑**。它可以存进变量、传给函数、从函数返回，行为和其他值一致。
2. 编译器把每个闭包表达式变成一个匿名结构体；捕获的变量是字段。闭包的大小、所有权、生命周期和释放行为都由这个模型推出。
3. 捕获方式有三种：只读共享借用 `&T`、可变借用 `&mut T`、按值拥有 `T`。没有 `move` 时，编译器选择满足需要的最弱方式；`move` 强制按值。
4. `move` 与 `FnOnce` 无关。`move` 决定字段里存值还是引用；`Fn`/`FnMut`/`FnOnce` 由闭包体如何使用捕获值推导。
5. 三个 trait 的接收者分别是 `self`、`&mut self`、`&self`，对应“移出字段”“修改字段”“只读字段”。能力层级是 `Fn ⊂ FnMut ⊂ FnOnce`。
6. API 应接受最弱的必要约束：只调用一次用 `FnOnce`，多次调用且允许状态用 `FnMut`，只读共享调用用 `Fn`。
7. 闭包的参数类型由第一次调用或上下文固定，不是泛型；从未确定类型的闭包需要显式标注。
8. 不捕获环境的闭包可以转成函数指针 `fn(...)`；捕获环境的不能，因为函数指针没有地方保存环境。
9. 每个闭包表达式的类型都不同。异构集合按需求选择 `fn` 指针、`Box<dyn Fn>` 或枚举，并考虑堆分配和动态分发的成本。
10. 返回闭包通常需要 `move`，因为闭包不能引用已经销毁的栈帧；但 `move` 不会自动把闭包变成 `FnOnce`。

## 十六、课后复盘

用自己的话回答，不需要写代码：

1. 为什么普通具名函数不能保存“这一次任务的阈值”和“已经调用了几次”？
2. 闭包捕获一个只读的 `String` 时，闭包的字段里实际存了什么？如果加了 `move` 呢？
3. 为什么修改捕获变量的闭包调用时，闭包变量本身常常需要 `mut`？
4. `move` 和 `FnOnce` 分别决定什么？举一个“用了 `move` 但仍能调用多次”的例子。
5. 为什么只调用一次的回调应该接收 `FnOnce`，而不是 `Fn`？
6. 返回闭包时，为什么返回类型要写 `impl Fn(...)`，而不能直接写出闭包的具体类型？
7. 同一种签名的两个闭包为什么不能直接放进同一个 `Vec`？三种解决方案各自的成本是什么？

## 下一课

下一课[第十课：Iterator、适配器与所有权模式](../10-iterators/README.md)会把闭包放进迭代器管线：用 `map` 转换每个元素，用 `filter` 筛选，用 `fold` 汇总，并观察这些适配器为什么接收 `FnMut`、为什么是惰性求值（先把处理步骤记录下来，直到真正需要结果时才执行）。你在本课学到的“闭包是值”“捕获方式由使用方式推导”“API 选择最弱约束”会在那里反复出现。
