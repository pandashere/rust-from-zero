# 第十一课：泛型、trait bound 与单态化

假设你要写一个函数，返回一组整数里的第一个；过几天，又需要返回一组文本里的第一个。两段逻辑一模一样，只有处理的**类型**不同。各写一份当然能工作，但每增加一种类型就要再复制一遍；将来要改“空集合返回什么”，还得记得改所有副本。

本课介绍**泛型（generics）**：把“类型”本身也变成参数，让同一份代码适用于多种类型。泛型不是“什么都能做”的魔法；它用 **trait bound**（能力约束）明确声明代码需要类型具备哪些行为。最后我们会看到编译器如何用**单态化（monomorphization）**把泛型代码变成具体类型的代码，以及这份便利的代价。

## 本课定位

### 你将学会

- 写泛型函数、泛型 `struct`、泛型 `enum` 和泛型 `impl`。
- 用 trait bound 和 `where` 子句声明“这个算法需要类型能做什么”。
- 读懂“缺少 `Ord`”“缺少 `Clone`”这类编译错误，并知道该加什么、不该加什么。
- 理解单态化：为什么泛型调用通常没有运行时查表成本，以及代码体积的代价。
- 使用类型推断和 turbofish（`::<...>`）在必要时补上类型信息。
- 使用 const 泛型参数，让数组长度成为类型的一部分。
- 理解每个泛型参数隐含的 `Sized` 约束，以及 `?Sized` 的用途。

### 前置知识

- **函数、`struct`、`impl` 与方法**：见[第二课](../02-data-modeling/README.md)。简单复习：`struct` 把多个字段组合成一个整体；`impl` 块给某个类型添加方法；`&self` 表示方法只借用数据查看，`self` 表示方法拿走整个值。
- **所有权、移动、`Copy` 与借用**：见[第三课](../03-ownership-borrowing/README.md)。简单复习：一个值通常只有一个所有者；把一个值交给另一个变量叫**移动（move）**；`&T` 是只读借用，`&mut T` 是可修改借用。
- **`Option`、`Result` 与 `match`**：见[第四课](../04-option-result/README.md)。简单复习：`Option<T>` 表示“可能有值（`Some`），也可能没有（`None`）”；`Result<T, E>` 表示“成功（`Ok`）或失败（`Err`）”；`match` 按情况分别处理。
- **切片 `&[T]` 与 `Vec<T>`**：见[第五课](../05-vec-slices/README.md)。简单复习：切片是对一段连续数据的只读借用视图，长度不必在类型里写死；`&[T]` 表示“一串 `T` 的借用”。
- **闭包与 `FnMut`/`FnOnce`**：见[第九课](../09-closures/README.md)。简单复习：闭包是能捕获周围变量的匿名函数；`FnMut` 表示可以反复调用并修改捕获的状态；`FnOnce` 表示最多调用一次，可以消费捕获的值。
- **迭代器**：见[第十课](../10-iterators/README.md)。本课会用到 `values.iter().max()`；只要知道 `iter()` 逐个借出元素、`max()` 取最大元素即可。

### 本课边界

本课把 trait 当作“能力清单”使用，不展开如何定义 trait、关联类型、`impl Trait`（在参数或返回值位置直接写 trait 的简写）和 `dyn Trait`（运行时才确定具体类型的 trait 对象）——这些是[第十二课](../12-traits/README.md)的内容。显式生命周期参数只作为阅读泛型签名所需的工具简单介绍，系统讲解在[第十四课](../14-lifetimes/README.md)。const 泛型只讲基础整数参数，不讲 `N + 1` 这类高级 const 表达式，也不讲特化（给某个具体类型替换整套实现）。

## 如何学习本课

1. 先读第一到第十一节，边读边回答“这个函数对哪些类型可用，为什么”。
2. 读编译错误一节时，不要只看修改后的代码；先自己判断错误里的关键词对应哪条规则。
3. 完成“代码预测”，再打开折叠答案。
4. 做主练习：先运行一次测试，确认只有概念测试通过，然后按推荐顺序实现。
5. 最后做课后巩固。卡住时按 `HINTS.md` 的三级提示逐级查看。

预计用时：概念阅读约 2 小时，主练习约 1 小时，课后巩固约 40 分钟。

## 一、问题：同一逻辑写两遍

读这一段前，先约定两个最基础的说法。**函数（function）**是一段有名字、可以反复调用的代码：它接收输入（**参数**），执行一段操作，产生一个结果（**返回值**）。**类型（type）**描述一类值以及能对它们做的操作，例如 `i32` 描述整数，`String` 描述一段可增长的所有权文本。编译器（compiler）是把源代码翻译成可执行程序的工具，类型检查发生在翻译阶段。

先看没有泛型时的写法。下面的程序分别返回整数切片和文本切片的第一个元素：

```rust
fn first_number(values: &[i32]) -> Option<&i32> {
    values.first()
}

fn first_text(values: &[String]) -> Option<&String> {
    values.first()
}

fn main() {
    let numbers = [10, 20, 30];
    let texts = [String::from("alpha"), String::from("beta")];

    println!("整数第一个: {:?}", first_number(&numbers));
    println!("文本第一个: {:?}", first_text(&texts));

    let empty: [i32; 0] = [];
    println!("空切片: {:?}", first_number(&empty));
}
```

输出：

```text
整数第一个: Some(10)
文本第一个: Some("alpha")
空切片: None
```

逐项解释新出现的写法：

- `i32` 是 Rust 的 32 位有符号整数类型。`String` 是拥有自己文本内容、长度可以增长的字符串类型；`String::from("alpha")` 从一段文本字面量创建一个这样的拥有型字符串。
- `&[i32]` 读作“整数切片的只读借用”。函数不拥有整串数据，只借来看，所以调用方在调用后仍能使用原数组。
- `.first()` 是切片自带的方法，返回 `Option<&i32>`：有元素时给出第一个元素的引用，没有元素时给出 `None`。它不需要比较、复制或修改元素。方法（method）是绑定在某个类型上的函数，用点号调用。
- `println!` 是宏（macro）：名字后面带 `!`，在编译时展开成打印代码。它把 `{}` 或 `{:?}` 的位置替换成后面参数的内容。
- `[10, 20, 30]` 是数组（array），长度固定；`let empty: [i32; 0] = [];` 是长度为 0 的整数数组。
- `{:?}` 是“调试格式”，适合开发时查看值的结构；它要求类型实现 `Debug`。第四课已经讲过 `Option` 的 `Some`/`None`。

两个函数体一字不差，只有类型不同。这带来三个具体问题：

1. **类型一多就要复制。** 以后还要处理 `f64`、自定义的 `User`、`Token`，每加一种都要再写一份。
2. **逻辑修改要同步多处。** 如果以后决定“空集合时打印警告”，或者加入日志，必须记得改每一份副本；漏掉一份就产生行为不一致。
3. **编译器无法确认它们真的是同一算法。** 两份独立函数可以逐渐改得不一样，测试也要分别覆盖。

**泛型代码**就是带**类型参数（type parameter）**的代码。类型参数是一个占位符，写代码时先不决定它具体是什么类型；调用时才由调用方传入的具体类型填充。一个函数可以因此同时服务多种类型，而函数体只写一次。

## 二、泛型函数：`T` 是编译期变量

把上面的两个函数合成一个：

```rust
fn first<T>(values: &[T]) -> Option<&T> {
    values.first()
}

fn main() {
    let numbers = [10, 20, 30];
    let texts = [String::from("alpha"), String::from("beta")];

    println!("整数第一个: {:?}", first(&numbers));
    println!("文本第一个: {:?}", first(&texts));

    let empty: [i32; 0] = [];
    println!("空切片: {:?}", first(&empty));
}
```

输出与上一节相同。签名 `fn first<T>(values: &[T]) -> Option<&T>` 要这样读：

- `fn first`：函数名是 `first`。
- `<T>`：**声明**一个类型参数，名字叫 `T`。它像值参数一样是“变量”，只不过它的值是类型，而且只在编译期存在。`T` 是社区常用名（type 的首字母），换成 `Element` 也可以。
- `values: &[T]`：参数 `values` 是“一串 `T` 的只读借用”。
- `-> Option<&T>`：返回“可能有、也可能没有的 `T` 的引用”。

签名里第一次出现的 `<T>` 是声明，后面参数和返回值里的 `T` 都是**使用**这个类型参数。同一次调用中，所有 `T` 必须指同一个具体类型，所以不会出现“传入整数切片却返回文本引用”的情况；这种错误在类型检查阶段就不成立。

调用时不需要写 `first::<i32>(&numbers)`。编译器从实参 `&numbers` 的类型 `&[i32; 3]` 看出元素是 `i32`，于是这次调用的 `T` 就是 `i32`；另一次调用传入文本数组，那个调用的 `T` 就是 `String`。两次调用各自决定自己的 `T`，互不影响。

### 为什么这个 `T` 不需要 bound

函数体只调用了 `values.first()`。这是切片对**所有** `T` 都提供的方法：它只是查看有没有第一个位置，不需要元素能比较、能复制、能打印。因此 `first` 对任意类型都成立，不额外声明要求。

这一点很重要：**泛型参数本身的含义是“某个类型”，不是“任意类型都能做任何事”**。能对 `T` 做什么，由函数体里的操作和签名里的 bound 共同决定；没有 bound 时，几乎只能移动或丢弃值。

## 三、trait bound：声明算法需要的能力

把需求从“第一个”改成“最大的那个”，直接写：

```rust,ignore
fn largest<T>(values: &[T]) -> Option<&T> {
    values.iter().max()
}
```

编译器拒绝这段代码，真实诊断（省略了标准库源码位置）如下：

```text
error[E0277]: the trait bound `T: Ord` is not satisfied
  --> src/main.rs:2:19
   |
 2 |     values.iter().max()
   |                   ^^^ the trait `Ord` is not implemented for `T`
   |
   = note: required for `&T` to implement `Ord`
help: consider restricting type parameter `T` with trait `Ord`
   |
 1 | fn largest<T: std::cmp::Ord>(values: &[T]) -> Option<&T> {
   |             +++++++++++++++
```

逐行读这份诊断：

- `E0277` 是错误编号，表示“某个 trait bound 没有满足”。
- 诊断第一行里的 `the trait bound` 和 `is not satisfied`：`T` 没有满足 `Ord` 这个要求。
- 箭头指向 `.max()`：出问题的是这个调用，不是 `values` 参数本身。
- 诊断里的 `required for` 后面说明：`iter()` 产生的元素类型是 `&T`，而 `max` 要求元素能比较大小；`&T` 只有在 `T` 本身能比较时才满足这个要求。
- 最后的 `help` 给出修复方向：给 `T` 加上 `Ord` 约束。

**trait**（中文常译“特征”）在这里可以先理解成一份**能力约定**：它列出一组行为，声称“实现了我这个 trait 的类型，保证能做这些事”。`Ord`（ordering，全序）表示“值可以按大小排列”，支持 `<`、`>`、`max` 这类操作。整数和 `String` 都实现了 `Ord`；一个只有名字和编号、没有定义大小顺序的自定义 `struct` 则不一定实现。

修复就是把这个能力写进签名：

```rust
fn largest<T: Ord>(values: &[T]) -> Option<&T> {
    values.iter().max()
}

fn main() {
    let numbers = [3, 1, 4, 1, 5];
    println!("最大整数: {:?}", largest(&numbers));

    let words = [String::from("pear"), String::from("apple")];
    println!("最大文本: {:?}", largest(&words));

    let empty: [i32; 0] = [];
    println!("空切片: {:?}", largest(&empty));
}
```

输出：

```text
最大整数: Some(5)
最大文本: Some("pear")
空切片: None
```

`T: Ord` 读作“`T` 是某个实现了 `Ord` 的类型”。这个声明同时约束两边：

- **函数体只能使用 `Ord` 承诺的操作。** 如果函数体里写 `values[0].clone()`，编译器仍然会报错，因为 `Ord` 没有承诺 `Clone`。要复制就得再加 `Clone`。
- **调用方只能传满足 `Ord` 的类型。** 传一个没有实现 `Ord` 的 `struct`，错误会在调用处出现，而不是等运行到比较时才发现。

Rust 会在**定义函数时**检查函数体一次：它假设 `T` 满足签名里的 bound，检查所有操作是否有依据。之后每个调用点只负责确认实参类型满足 bound，不需要重新推理整个函数体。这也是为什么读一个泛型函数时，**只看签名里的 bound 就能知道它能对值做什么**，不必先读完整实现。

由此得到本课最重要的一条读签名方法：

> **每多一个 bound，函数体就多一批可用操作，同时调用方可选的具体类型就少一批。**

所以 bound 不是越多越好。它应该精确等于算法真正需要的能力。

### `Display`：另一个常见能力

```rust
fn show<T: std::fmt::Display>(value: &T) {
    println!("{value}");
}

fn main() {
    show(&42);
    show(&"hello");
}
```

`Display` 表示“能格式化成用户可读文本”，所以函数体可以用 `{}` 打印。`Debug` 则是“能格式化成开发者调试用的形式”，对应 `{:?}`。两者都是 trait；本课只把它们当作能力使用，第十二课会讲如何定义自己的 trait。

## 四、泛型 struct 与 enum

函数可以泛型化之后，自己定义的容器如果只写具体类型，仍然只能装一种元素；要保存两个同类型值，每换一种元素类型就得定义一个新结构体。

**结构体（struct）**是一种自定义类型：它把若干个**字段**（有名字的数据位置）组合成一个整体。类型也可以带类型参数。下面是一个装有两个同类型值的容器：

```rust
#[derive(Debug)]
struct Pair<T> {
    left: T,
    right: T,
}

fn main() {
    let numbers = Pair { left: 1, right: 2 };
    let words = Pair {
        left: String::from("a"),
        right: String::from("b"),
    };

    println!("{numbers:?}");
    println!("{words:?}");
}
```

`#[derive(Debug)]` 让编译器自动生成 `Debug` 实现，于是这个类型可以用 `{:?}` 打印；它只是省去手写重复代码，不改变字段或方法。

输出：

```text
Pair { left: 1, right: 2 }
Pair { left: "a", right: "b" }
```

`Pair<i32>` 和 `Pair<String>` 是**两个不同的具体类型**，概念上相当于你手写了：

```rust
struct PairI32 { left: i32, right: i32 }
struct PairString { left: String, right: String }
```

只是编译器替你生成了这些类型。`Pair<i32>` 的两个字段都必须是 `i32`；如果想让两个字段各自是不同类型，就声明两个类型参数：

```rust
#[derive(Debug)]
struct Entry<K, V> {
    key: K,
    value: V,
}

fn main() {
    let entry = Entry {
        key: String::from("score"),
        value: 95,
    };
    println!("{entry:?}");
}
```

这里 `K` 和 `V` 相互独立：`Entry<String, i32>` 的 `key` 是 `String`，`value` 是 `i32`。

**枚举（enum）**也是一种自定义类型，但它的值只能是列出的几种**变体**之一。`Outcome` 有两个变体：`Success` 装一个 `T`，`Failure` 装一个 `E`。你从第四课起就在使用泛型 enum：

```rust
enum Outcome<T, E> {
    Success(T),
    Failure(E),
}
```

`Option<T>` 可以看成“只有成功值的 `Outcome`”，`Result<T, E>` 则是“成功值或错误值”的泛型 enum。类型参数写在类型名后面，成为类型的一部分：`Option<i32>` 里的 `i32` 不是注释，它决定了 `Some` 里到底装什么、这个值占多少内存、能调用哪些方法。

## 五、泛型 impl：两个 `T` 各是什么意思

给泛型类型实现方法时，语法里会出现两个位置上的 `T`：

```rust
struct Reading<T> {
    value: T,
}

impl<T> Reading<T> {
    fn new(value: T) -> Self {
        Self { value }
    }

    fn value(&self) -> &T {
        &self.value
    }
}

fn main() {
    let reading = Reading::new(42);
    println!("{}", reading.value());
}
```

- `impl<T>` 里的 `T` 是**声明**：“下面这个 impl 块对任意类型 `T` 成立。”
- `Reading<T>` 里的 `T` 是**使用**：“这个块实现的是 `Reading<T>` 这个类型的方法。”

合起来读：“对任意 `T`，`Reading<T>` 都有 `new` 和 `value` 方法。”`Reading::new(42)` 让编译器推断出 `T` 是 `i32`。

如果某些方法需要额外能力，可以把 bound 加在整个 impl 上：

```rust
impl<T: Ord> Reading<T> {
    fn at_least(&self, other: &T) -> bool {
        self.value >= *other
    }
}
```

这样一来，只有 `T: Ord` 的 `Reading<T>` 才有 `at_least`。`Reading<i32>` 能调用，`Reading<SomeTypeWithoutOrd>` 不能；编译器会在方法调用处拒绝，而不是等运行到 `>=` 时才出错。

也可以只给一个方法加 bound：

```rust
impl<T> Reading<T> {
    fn copied_value(&self) -> T
    where
        T: Clone,
    {
        self.value.clone()
    }
}
```

此时同一个 impl 块里的 `value()` 对所有 `T` 可用，只有 `copied_value()` 要求 `T: Clone`。**能放在方法上就不要放在整个 impl 上**，否则一个方法的需求会无辜地限制块里所有其他方法。

还可以给某个具体类型追加专有方法：

```rust
impl Reading<i32> {
    fn doubled(&self) -> i32 {
        self.value * 2
    }
}
```

`Reading<i32>` 同时拥有通用块里的方法（`new`、`value`、`copied_value`）和专有块里的 `doubled`；`Reading<String>` 只有通用方法。

### 多个 impl 块是合并，不是覆盖

上面的写法看起来像“给 `Reading<i32>` 换一份专门实现”，但 Rust 的固有 impl（inherent impl，直接写在类型上的 impl）不是这样工作的：**同一个类型的多个 impl 块是合并关系，不是竞争关系**。`Reading<i32>` 最终拥有的方法等于各块方法的并集，不会因为某个块更具体就替换掉另一个块的方法。

因此不允许同名方法。下面这段代码会失败：

```rust,ignore
struct Boxed<T> {
    first: T,
    second: T,
}

impl<T> Boxed<T> {
    fn head(&self) -> &T {
        &self.first
    }
}

impl Boxed<String> {
    fn head(&self) -> &String {
        &self.first
    }
}
```

真实诊断：

```text
error[E0592]: duplicate definitions with name `head`
  --> src/main.rs:7:5
   |
 7 |     fn head(&self) -> &T {
   |     ^^^^^^^^^^^^^^^^^^^^ duplicate definitions for `head`
...
13 |     fn head(&self) -> &String {
   |     ------------------------- other definition for `head`
```

编译器不会挑“更具体的那个”，也不会“就近覆盖”；同名就是错误。Rust 稳定版没有给某个具体类型替换整套实现的稳定特化机制。

trait impl（为某个 trait 写实现）的规则更严格：同一个 trait 对同一个类型只允许一份实现，而且不能有范围重叠；这属于第十二课。本课只记住固有 impl 的合并规则即可。

## 六、`where` 子句与多个 bound

一个类型可以同时要求多种能力，用 `+` 连接：

```rust
fn duplicate_and_print<T>(value: T)
where
    T: Clone + std::fmt::Debug,
{
    let copy = value.clone();
    println!("{copy:?}");
}

fn main() {
    duplicate_and_print(String::from("hello"));
    duplicate_and_print(42);
}
```

`T: Clone + Debug` 读作“`T` 同时实现了 `Clone` 和 `Debug`”。函数体里 `value.clone()` 用了 `Clone`，`println!("{copy:?}")` 用了 `Debug`，两个 bound 都有实际依据。

`where` 子句与写在尖括号里的 `<T: Clone + Debug>` **语义完全相同**，只是排版方式不同：

```rust
fn describe<T, U>(left: T, right: U) -> String
where
    T: std::fmt::Display,
    U: std::fmt::Debug,
{
    format!("{left} / {right:?}")
}
```

当 bound 很多、签名一行放不下时，`where` 让每个要求单独占一行，更容易读。选择哪一种只影响可读性，不影响类型检查。

### bound 是对外调用契约

写在公共函数签名里的每个 bound 都是对调用方的要求，也是 API（别人调用你代码时看到的接口）的一部分。删掉一个 bound 可能让函数体编译不过；增加一个 bound 则可能让原本能调用这个函数的类型突然不能用了。因此：

- 不要为了“看起来保险”堆 `Clone + Copy + Debug + Default`；
- 判断一个 bound 是否必要，可以临时删掉它，看编译器是否报缺少操作。若不报，它本来就不该在签名里；
- 如果某个 impl 级 bound 只有一两个方法用到，把它下移到方法上。

## 七、单态化：泛型为什么通常没有运行时成本

写出 `largest<T>` 之后，编译器并不是在运行时保存一个“通用函数”，再根据传入的类型查表。它使用**单态化（monomorphization，单一形态化）**：收集整个程序中每个泛型函数、泛型类型实际用到的具体类型，然后为每一种具体类型生成一份专门的代码。

概念上，调用：

```rust,ignore
largest::<i32>(&numbers)
largest::<String>(&words)
```

会让编译器生成类似两份函数：

```text
largest_i32     // 函数体里所有 T 都换成 i32
largest_String  // 函数体里所有 T 都换成 String
```

运行时程序里没有 `largest<T>` 这个“通用版本”，只有两份各管一种类型的具体函数。由此有三个性质：

1. **静态分发（static dispatch）**：调用哪个函数、用哪条比较指令，在编译期就确定，不需要运行时查表，也没有额外的间接调用。
2. **可内联、可优化**：每份代码的类型完全确定，优化器可以把泛型函数体内联进调用处（内联：把函数体直接放到调用处），再做后续优化。这就是人们说泛型抽象通常“零成本”的原因：抽象本身不引入运行时开销。
3. **代码膨胀**：代价也很直接——用了多少种具体类型，就生成多少份机器码。一个泛型函数被 50 种类型实例化，二进制（编译出的可执行文件）里就有 50 份。这会增加编译时间和程序体积；公共库里被广泛使用的泛型函数改动，还可能导致下游大量重新编译。

这一节说的“零成本”是通常情况，不是绝对承诺。代码体积、编译时间和指令缓存都可能受影响；真正关心性能时应测量，而不是只凭“泛型一定快”下结论。

第十二课会介绍另一条路：`dyn Trait` 动态分发，整个程序只生成一份代码，运行时再确定具体类型。它适合“运行时才知道类型”或“需要把不同类型放进同一个集合”的场景，但有间接调用成本。两条路各有位置，不能不经测量就互相替换。

## 八、类型推断与 turbofish

大多数时候不需要写出类型参数，编译器会从上下文推断：

```rust
fn first<T>(values: &[T]) -> Option<&T> {
    values.first()
}

fn main() {
    let numbers = [10, 20, 30];
    let first_number = first(&numbers); // T 被推断为 i32
    println!("{first_number:?}");
}
```

推断的“燃料”是使用方式：实参的类型、赋值目标上的类型标注、后续方法调用，都能提供线索。燃料不足时，编译器不会替你猜：

```rust,ignore
fn main() {
    let parsed = "42".parse().unwrap();
    println!("{parsed}");
}
```

真实诊断：

```text
error[E0284]: type annotations needed
 --> src/main.rs:2:9
  |
2 |     let parsed = "42".parse().unwrap();
  |         ^^^^^^        ----- type must be known at this point
  |
  = note: cannot satisfy `<_ as FromStr>::Err == _`
help: consider giving `parsed` an explicit type
  |
2 |     let parsed: /* Type */ = "42".parse().unwrap();
  |               ++++++++++++
```

解释：

- `parse` 是泛型方法，能把文本解析成多种目标类型（`u32`、`i32`、`f64` 等）。
- `unwrap()` 从 `Result` 里取出成功值；成功值的类型仍然未知，错误类型也未知。
- `_` 是编译器用来表示“这里有个类型，但我还不知道”的占位符。诊断里的 `cannot satisfy` 说明目标类型和错误类型都无法确定。
- `type annotations needed` 不是语法错误，而是“缺少类型线索”。

两种补线索的方式：

```rust
fn main() {
    let parsed: u32 = "42".parse().unwrap();
    println!("{parsed}");

    let other = "7".parse::<i32>().unwrap();
    println!("{other}");
}
```

第一种把标注放在变量左边；第二种在方法调用处写 `::<i32>`。这个 `::<...>` 写法常被社区称为 **turbofish**（因为 `::<>` 看起来像一条小鱼）。它解决的是一个纯语法问题：在表达式内部标注泛型参数需要一个不和“小于号”混淆的位置。直接写 `parse<i32>()` 会被解析器看成“`parse` 小于 `i32`”，所以方法调用位置用 `::` 前缀消歧。它不是另一套泛型机制，只是在调用点补充类型参数。

风格上，**把标注放在数据边界**：一个变量后面还会被多处使用、类型影响面大时，标在左边 `let values: Vec<u64> = ...`；只是某个表达式一步之内的临时需要，用 turbofish。不要为了“明确”给每个调用都写满类型参数——推断本来就是为了少写。

## 九、借用泛型值，避免无意义约束

泛型参数决定“值是什么类型”，它和“需要什么能力”是两件事。函数只需要比较、不需要复制时，参数应该用借用：

```rust
fn equal<T: PartialEq>(left: &T, right: &T) -> bool {
    left == right
}

fn main() {
    println!("{}", equal(&10, &10));
    println!("{}", equal(&String::from("a"), &String::from("b")));
}
```

`PartialEq` 表示“支持 `==` 和 `!=` 比较”。函数接收两个只读引用，不拿走所有权，也不要求 `Clone`。

如果写成：

```rust,ignore
fn equal<T: PartialEq + Clone>(left: T, right: T) -> bool {
    left == right
}
```

就多出两个不必要的代价：要求 `T: Clone`，并且把两个实参**移动**进函数，调用方之后不能再使用它们。为一个按引用就能完成的比较付出所有权和复制成本，是不必要的约束。

如果确实需要把某一个值交还给调用方，可以直接按值传入、返回其中一侧，同样不需要 `Clone`：

```rust
fn choose_owned<T>(left: T, right: T, pick_left: bool) -> T {
    if pick_left { left } else { right }
}

fn main() {
    let value = choose_owned(String::from("left"), String::from("right"), true);
    println!("{value}");
}
```

两个值进入函数，没被选中的那个在函数结束时被清理，选中的那个被返回。全程只有移动，没有复制。

### 返回引用时的生命周期参数

如果函数要返回某个输入的引用，签名里会出现生命周期参数：

```rust
fn choose_ref<'a, T>(left: &'a T, right: &'a T, pick_left: bool) -> &'a T {
    if pick_left { left } else { right }
}

fn main() {
    let left = String::from("left");
    let right = String::from("right");
    let chosen = choose_ref(&left, &right, true);
    println!("{chosen}");
}
```

这个签名里有两类参数，各管一件事：

- **类型参数 `T`** 决定“值的类型是什么”；
- **生命周期参数 `'a`** 决定“引用能合法使用多久”。`'a` 声明两个输入引用和返回值共享同一段有效期，编译器据此保证返回的引用不会比输入活得更久。

这里只需要会读 `'a`，知道它不是类型参数即可；第十四课会系统讲生命周期省略、结构体里的引用和借用 API 设计。

经验规则：**需要 bound 往往是因为按值操作；先问“借用能不能满足需求”，再决定是否加 `Clone`、`Copy` 之类的约束。**

### 转换类 bound：`AsRef` 与 `Into` 的取舍

有时函数不想只接受一个固定类型，而是想接受“能当作路径使用的东西”或“能变成 `String` 的东西”。标准库有两个常用的转换 trait，选择标准是：只需要借用视图，还是要拥有结果？

```rust
fn print_path(path: impl AsRef<std::path::Path>) {
    println!("{}", path.as_ref().display());
}

fn set_name(name: impl Into<String>) -> String {
    name.into()
}

fn main() {
    print_path("notes/today.txt");
    print_path(String::from("notes/tomorrow.txt"));
    println!("{}", set_name("Ada"));
}
```

输出：

```text
notes/today.txt
notes/tomorrow.txt
Ada
```

- `Path` 表示文件系统路径的借用形式。`AsRef<Path>` 表示“能廉价地借出 `&Path`”。`print_path` 全程只借用，不分配新字符串；`&str` 和 `String` 都能满足这个要求。
- `Into<String>` 表示“能转换成拥有型 `String`”。`&str` 转换时会发生一次堆分配（堆是程序运行时动态申请的内存区域），得到一个由函数拥有的新 `String`。
- `impl AsRef<Path>` 和 `impl Into<String>` 写在参数位置，是“泛型参数 + bound”的简写。例如 `fn set_name(name: impl Into<String>)` 等价于 `fn set_name<N: Into<String>>(name: N)`；`impl Trait` 的完整规则在第十二课。

粗略的选择规则：只需要看/借用时用 `AsRef`，需要拥有结果时用 `Into`；如果根本不需要接受多种类型，直接写 `&str` 或 `&Path` 最清楚。不要为了显得通用，给每个参数都套一层转换 bound——每个 bound 都是调用方要满足的义务。

## 十、const generics：让编译期常量参与类型

普通泛型参数是**类型**；const 泛型参数是**编译期就已知的值**，目前最常用的是整数。最简单的例子是让数组长度成为类型的一部分：

如果只处理长度为 3 的整数数组，直接写 `[i32; 3]` 就够了；但长度 4、5 的数组又需要另一份代码，或者退化成运行时长度可变的 `Vec`。const 泛型参数让长度本身成为类型的一部分。

```rust
fn first_or_zero<const N: usize>(values: [i32; N]) -> i32 {
    values.first().copied().unwrap_or(0)
}

fn main() {
    let three = [10, 20, 30];
    let empty: [i32; 0] = [];

    println!("{}", first_or_zero(three));
    println!("{}", first_or_zero(empty));
}
```

输出：

```text
10
0
```

逐项解释：

- `usize` 是 Rust 用来表示“大小、长度、索引”的无符号整数类型，不可能为负。
- `[i32; N]` 是长度为 `N` 的整数数组。普通数组的长度必须写成编译期常量，这里用 const 参数 `N` 代替具体数字。
- `<const N: usize>` 声明“`N` 是一个 `usize` 类型的编译期常量参数”。调用 `first_or_zero(three)` 时，编译器从数组类型 `[i32; 3]` 推出 `N = 3`。
- `.copied()` 把 `Option<&i32>` 变成 `Option<i32>`（复制出整数）；`.unwrap_or(0)` 在没有元素时给出 0。这里 `i32` 是 `Copy` 类型（复制后原值仍然可用），所以复制很便宜。

类型定义也可以带 const 参数：

```rust
#[derive(Debug, PartialEq, Eq)]
struct FixedWindow<T, const N: usize> {
    values: [T; N],
}

fn main() {
    let bytes = FixedWindow {
        values: [1_u8, 2, 3],
    };
    let larger = FixedWindow {
        values: [1_u8, 2, 3, 4],
    };

    println!("{bytes:?}");
    println!("{larger:?}");
    println!(
        "u8 窗口大小: {} 和 {}",
        std::mem::size_of::<FixedWindow<u8, 3>>(),
        std::mem::size_of::<FixedWindow<u8, 4>>()
    );
}
```

输出（最后一行数值来自数组元素个数）：

```text
FixedWindow { values: [1, 2, 3] }
FixedWindow { values: [1, 2, 3, 4] }
u8 窗口大小: 3 和 4
```

`FixedWindow<u8, 3>` 和 `FixedWindow<u8, 4>` 是两个不同的类型。数组直接内联在结构体里，不经过堆分配，所以 `size_of` 分别是 3 和 4 字节。主练习会用 `size_of::<FixedWindow<u8, 16>>() == 16` 检查这一点。

const 泛型适合“长度是协议或设计的一部分”的场景：固定宽度的数据包、矩阵维度、固定大小的缓冲区。**长度来自用户输入或运行时数据时，必须用 `Vec`/切片**，因为 const 参数在编译期就要定死。`N + 1` 这类更复杂的 const 表达式仍有较多限制，本课不使用。

## 十一、`Sized` 默认 bound 与 `?Sized`

Rust 里有些类型没有编译期可知的大小：`str`（任意长度的 UTF-8 文本；UTF-8 是一种把文字编码成字节的方式）、切片 `[T]`，以及后面的 `dyn Trait`（运行时才确定具体类型的 trait 对象）。这些类型不能直接按值放在栈上（栈是程序自动管理、存放局部变量的一块内存），只能待在引用或指针后面（指针是保存内存地址的值）。

问题来了：`fn consume<T>(value: T)` 这样的泛型函数，能不能传一个 `str`？不能——按值传参需要知道大小。Rust 因此做了一个默认决定：**每个泛型参数都隐含 `T: Sized` bound**，也就是“默认只接受编译期大小确定的类型”。你之前写的每个 `<T>` 都自带这个默认要求，只是没有写出来。

少数函数确实想接收不定长类型，前提是它只通过引用接触值。这时用 `?Sized` **取消**默认要求：

```rust
fn size_of_value<T: ?Sized>(value: &T) -> usize {
    std::mem::size_of_val(value)
}

fn main() {
    println!("{}", size_of_value("hello"));
    println!("{}", size_of_value(&42));
}
```

输出：

```text
5
4
```

`?Sized` 里 `?` 是 bound 语法中唯一的“减法”：普通 bound 是“必须额外具备某能力”，`?Sized` 是“允许不具备 `Sized`”。因为值可能没有固定大小，参数必须是 `&T`——引用本身有固定大小；函数体里也只能做不依赖大小的操作。`size_of_val` 正是根据指针在运行时查看值占多少字节。

第十三课会结合 `str`、切片和 `dyn Trait` 深入不定长类型。现在只需记住：**`?Sized` 不是“让类型变小”，而是“放宽默认的大小要求”**。

## 十二、常见编译错误与诊断

泛型报错看起来长，但根因通常只有几种。下面按“诊断原文 → 术语解释 → 根因 → 修复方向”阅读。

### 1. 对没有 bound 的 `T` 使用操作（E0599）

```rust,ignore
fn duplicate<T>(value: T) -> (T, T) {
    (value.clone(), value)
}
```

```text
error[E0599]: no method named `clone` found for type parameter `T` in the current scope
 --> src/main.rs:2:12
  |
1 | fn duplicate<T>(value: T) -> (T, T) {
  |              - method `clone` not found for this type parameter
2 |     (value.clone(), value)
  |            ^^^^^ method not found in `T`
  |
  = help: items from traits can only be used if the type parameter is bounded by the trait
help: the following trait defines an item `clone`, perhaps you need to restrict type parameter `T` with it:
  |
1 | fn duplicate<T: Clone>(value: T) -> (T, T) {
  |               +++++++
```

`no method named ... found for type parameter` 的意思是“类型参数 `T` 没有被承诺拥有这个方法”。根因不是 `clone` 拼错，而是 `T` 可以是任意类型，其中很多类型不能复制。修复方向是加算法真正需要的最小 bound，例如 `T: Clone`；不要顺手堆上 `Copy + Debug + Default`。

### 2. 类型推断不足（E0284）

```rust,ignore
let parsed = "42".parse().unwrap();
```

`parse` 的目标类型无法从上下文确定。修复方向是在数据边界加类型标注，或用 turbofish（第八节）。不要为了“帮助”编译器而先转成字符串再解析——那通常是在掩盖类型设计问题。

### 3. 泛型参数没有被字段使用（E0392）

```rust,ignore
struct Wrapper<T> {
    value: i32,
}
```

```text
error[E0392]: type parameter `T` is never used
 --> src/main.rs:1:16
  |
1 | struct Wrapper<T> {
  |                ^ unused type parameter
  |
  = help: consider removing `T`, referring to it in a field, or using a marker such as `PhantomData`
  = help: if you intended `T` to be a const parameter, use `const T: /* Type */` instead
```

`T` 没有出现在任何字段里，编译器无法用它参与字段布局或方法签名，不同的 `T` 也不会带来可观察的差别。通常是设计写错了：删掉无用的 `T` 最直接。若你确实想用类型参数携带“编译期标记”而不占空间，标准库的工具是 `PhantomData<T>`（零大小、只携带类型信息的标记）；它常见于 unsafe 代码（需要程序员额外保证内存安全、编译器检查更少的代码）和所有权标记，本课程不展开它的用法。**不要为了消警告随手加 `PhantomData`**：参数没有真实语义时，正确做法是删除它。

### 4. 多个 impl 里出现同名方法（E0592）

```text
error[E0592]: duplicate definitions with name `swap`
```

固有 impl 块是合并关系，不是覆盖关系；同一个类型不能有两个同名方法。修复方向是给方法换一个能表达差异的名字，或者合并成一个方法，用参数区分行为。不要期待“更具体的 impl 会赢”。

### 5. bound 过强导致调用方被拒（E0277）

调用方看到：

```text
error[E0277]: the trait bound `MyType: Clone` is not satisfied
```

而你确认 `MyType` 本来应该能用这个 API。常见原因是：

- 整个 impl 上放了 `T: Clone`，但其实只有一个方法需要它——把 bound 下移到那个方法；
- 参数按值传入，导致必须复制；改成 `&T` 借用可能就只需要 `PartialEq` 或 `Ord`；
- 多加了一个“以后可能会用到”的 bound，而函数体根本没用到。

先删掉多余的 bound，让编译器告诉你缺什么，再补回真正必要的那一个。

## 十三、完整示例：有界值

下面把本课的几个概念组合起来：一个泛型 `Bounded<T>` 保存当前值以及允许的最小值、最大值；创建时把越界的值夹到边界内，并拒绝“最小值大于最大值”的非法范围。

```rust
#[derive(Debug, PartialEq, Eq)]
struct Bounded<T> {
    value: T,
    min: T,
    max: T,
}

impl<T: Ord + Clone> Bounded<T> {
    fn new(value: T, min: T, max: T) -> Result<Self, &'static str> {
        if min > max {
            return Err("最小值不能大于最大值");
        }

        let bounded = if value < min {
            min.clone()
        } else if value > max {
            max.clone()
        } else {
            value
        };

        Ok(Self {
            value: bounded,
            min,
            max,
        })
    }
}

fn main() {
    println!("{:?}", Bounded::new(5, 0, 10));
    println!("{:?}", Bounded::new(12, 0, 10));
    println!("{:?}", Bounded::new(-3, 0, 10));
    println!("{:?}", Bounded::new(5, 10, 0));
}
```

输出：

```text
Ok(Bounded { value: 5, min: 0, max: 10 })
Ok(Bounded { value: 10, min: 0, max: 10 })
Ok(Bounded { value: 0, min: 0, max: 10 })
Err("最小值不能大于最大值")
```

按本课的读法检查这个签名：

- `T: Ord`：函数体用 `min > max`、`value < min`、`value > max` 比较，必须有 `Ord`。
- `T: Clone`：值被夹到边界时，同一个边界值既要放进 `value`，又要保留在 `min` 或 `max` 字段里，所以需要复制一份。
- `Result<Self, &'static str>`：成功时返回构造好的 `Bounded`，失败时返回错误说明。`&'static str` 表示一个在整个程序运行期间都有效的字符串字面量；第十四课会解释 `'static` 的生命周期含义。
- `Self` 在这里指 `Bounded<T>`，是 impl 块所实现类型的简写。

数据变化过程（以 `Bounded::new(12, 0, 10)` 为例）：

1. `min > max` 吗？`0 > 10` 为假，范围合法。
2. `12 < 0` 吗？为假。
3. `12 > 10` 吗？为真，于是 `bounded = max.clone()`，得到 10。
4. 原来的 `min` 和 `max` 被移动进结构体，返回值是 `Ok`。

也可以换一种数据模型消掉 `Clone`：只保存当前值，边界改成借用，或者让多个值共享边界数据（引用计数：记录同一份数据被多少个所有者共用，会带来运行时维护成本）。但前一种会引入生命周期参数，后一种有运行时开销。设计泛型 API 时，**bound、字段所有权、生命周期三者总是连动的**；没有“只加一个 bound 就解决一切”的免费方案。

## 十四、代码预测

先不要运行，逐题写出答案，再展开对照。

### 1. 需要 `Clone` 吗？

```rust
fn reorder<T>(left: T, right: T) -> (T, T) {
    (right, left)
}
```

`reorder` 需不需要给 `T` 加 `Clone`？

<details><summary>答案与推理</summary>

不需要。两个参数按值传入，函数只是把它们**移动**到元组的另一个位置，全程没有复制值。移动是所有权交接，对任何 `T` 都合法，包括不能 `Clone` 的类型。如果签名改成接收 `&T` 并返回 `(&T, &T)`，也只借用、不复制；只有要在借用的情况下返回两个独立的拥有值，才需要 `Clone`。

</details>

### 2. 闭包会被调用几次？

```rust
let mut calls = 0;
let result = Pair::new(1, 2).map(|value| {
    calls += 1;
    value * 10
});

println!("{result:?}");
println!("{calls}");
```

假设 `map` 按主练习的约定实现，输出是什么？

<details><summary>答案与推理</summary>

闭包被调用 2 次：一次处理 `left`，一次处理 `right`。结果是 `Pair { left: 10, right: 20 }`，`calls` 为 2。这也是 `map` 参数用 `FnMut` 而不是 `Fn` 的原因：闭包需要修改捕获的 `calls`。

</details>

### 3. 空窗口会 panic 吗？

```rust
let window = FixedWindow::<i32, 0>::new([]);
println!("{:?}", window.first());
```

<details><summary>答案与推理</summary>

输出 `None`，不会 panic（panic 是程序遇到无法继续的情况时立即中止）。`N = 0` 表示数组长度为 0，是合法的类型参数；`first` 的约定是空窗口返回 `None`。这正是主练习要求 `first`、`last`、`get` 在空窗口和越界时返回 `None`、不能直接索引 `values[0]` 的原因。

</details>

### 4. 这段代码能编译吗？

```rust
let parsed = "42".parse().unwrap();
```

<details><summary>答案与推理</summary>

不能。`parse` 的目标类型没有被任何上下文确定，`unwrap` 的错误类型也无法确定，编译器报 E0284 `type annotations needed`。加上 `let parsed: u32 = ...` 或 `"42".parse::<u32>()` 即可。

</details>

### 5. 大小是多少？

```rust
println!("{}", std::mem::size_of::<FixedWindow<u8, 16>>());
```

<details><summary>答案与推理</summary>

16。`FixedWindow<u8, 16>` 内联保存 16 个 `u8`，每个 1 字节，没有堆分配，也没有额外的长度字段需要运行时保存；长度 `16` 已经是类型的一部分。

</details>

## 十五、主练习：泛型双元素容器与定长窗口

### 目标与行为

实现一个泛型双元素容器 `Pair<T>` 和一个 const generic 定长窗口 `FixedWindow<T, N>`。脚手架在 `lessons/11-generics/src/exercise.rs`，你需要替换其中所有 `todo!()`。

行为示例（`Pair` 的字段是私有的，外部通过方法观察）：

```text
Pair::new(3, 9).as_tuple()                     → (&3, &9)，原 Pair 仍可使用
Pair::new(1, 2).swap()                         → Pair { left: 2, right: 1 }
Pair::new(1, 2).map(|v| v * 10)                → Pair { left: 10, right: 20 }，闭包恰好调用 2 次
Pair::new(1, 2).map_each(|v| v + 100, |v| v + 200)
                                               → Pair { left: 101, right: 202 }
Pair::new(3, 9).max_ref()                      → &9
Pair::new(7, 7).max_ref()                      → &7（相等时取 right 的引用）
Pair::new(String::from("a"), String::from("b")).duplicated_left()
                                               → Pair { left: "a", right: "a" }，原 Pair 不受影响
Pair::new(String::from("ab"), String::from("cd")).joined("-")
                                               → "ab-cd"
FixedWindow::new([10, 20, 30]).last()          → Some(&30)
FixedWindow::new([10, 20, 30]).get(3)          → None（越界不 panic）
FixedWindow::<i32, 0>::new([]).first()         → None
```

### 输入、输出与边界

**正常输入：**

- `Pair::new(3, 9)`：`as_tuple` 返回 `(&3, &9)`，`max_ref` 返回 `&9`。
- `Pair::new(1, 2).map(|value| value * 10)`：得到 `Pair { left: 10, right: 20 }`，同一个闭包被调用两次。
- `FixedWindow::new([10, 20, 30])`：`first` 是 `Some(&10)`，`last` 是 `Some(&30)`，`as_slice` 是 `&[10, 20, 30]`。

**边界输入：**

- `Pair::new(7, 7).max_ref()`：两个字段相等，按约定返回 `right` 的引用。
- `joined("")`：分隔符为空，结果是 `"ab" + "" + "cd" = "abcd"`。
- `FixedWindow::new([7])`：长度为 1，`first` 和 `last` 返回同一个元素。
- `FixedWindow::<i32, 0>::new([])`：空数组，`first`、`last`、`get(0)` 都返回 `None`。

**错误/失败行为：**

- `FixedWindow::new([10, 20, 30]).get(3)` 和 `get(usize::MAX)`：位置越界，返回 `None`，不允许 panic。
- `FixedWindow::<i32, 0>::get(0)`：空窗口没有位置 0，同样返回 `None`。
- 这些失败是**可恢复的**：调用者拿到 `None` 后可以继续执行，不需要捕获异常。

### 脚手架与约束

- package 名：`l11_generics`；主练习文件：`src/exercise.rs`。
- 只修改 `todo!()` 所在的方法体。不要修改类型定义、方法签名，也不要修改文件底部的测试。
- `Pair::new`、`Pair::as_tuple`、`Pair::map`、`Pair::map_each`、`Pair::swap`、`Pair::max_ref`、`Pair::duplicated_left`、`Pair::joined`，以及 `FixedWindow::first`/`last`/`as_slice`/`get` 由你实现。
- `FixedWindow::new` 已经提供，不需要修改。
- `swap` 和 `map` 必须能作用于不可 `Clone` 的类型（测试用没有 `Clone` 的 `Token` 验证）。不要为了省事给整个 `Pair<T>` 加 `Clone` 或 `Ord`。
- bound 放在最小范围：能放方法上就不放整个 impl；`max_ref` 和 `duplicated_left` 已经在各自需要的 impl 块里。
- `map` 消费 `self`，两个字段直接移动，不允许先 `clone` 再转换。
- `FixedWindow` 的 `N = 0` 是合法边界；用安全的切片操作实现，不要直接索引 `values[0]`。
- 不使用 `dyn Trait`，也不使用 `unsafe`。本练习的所有类型在编译期已知。

### 推荐完成顺序

1. `new` 和 `as_tuple`：体会最普通的泛型 struct 和两个只读借用。
2. `swap`：消费 `self` 移动字段，确认不需要任何 bound（对应代码预测第 1 题）。
3. `map` 和 `map_each`：对照第九课的 `FnMut`/`FnOnce`，注意闭包分别被调用几次。
4. `max_ref`、`duplicated_left`、`joined`：三种不同粒度的 bound——整个 impl、方法级 bound、具体类型专有 impl。
5. `FixedWindow` 的 `first`、`last`、`as_slice`、`get`，最后专门处理 `N = 0` 和越界。
6. 运行完整测试和质量检查，再回答下面的书面题。

### 测试说明与验收命令

脚手架提供 21 个测试。其中 2 个是概念测试，不调用任何待实现方法，初始就会通过：

- `pair_fields_support_equality_directly`
- `fixed_window_size_includes_the_array_inline`

另外 19 个行为测试会在你实现的 `todo!()` 处失败，直到对应方法完成。失败信息是 `not yet implemented`，这是预期现象，不是测试写错。

想看演示输出，可以运行 `cargo run -p l11_generics`；完成前它会在第一个待实现方法处 panic。

先确认脚手架状态：

```bash
cargo test -p l11_generics --no-run
```

然后逐个实现并运行：

```bash
cargo test -p l11_generics
```

提交前运行本课的全部质量门禁：

```bash
cargo fmt -p l11_generics -- --check
cargo check -p l11_generics --all-targets
cargo test -p l11_generics --no-run
cargo clippy -p l11_generics --all-targets -- -D warnings
cargo test -p l11_generics
```

最后一条应在所有 `todo!()` 被替换后通过。完成后回答两道书面题：

1. 解释 `map` 为什么消费 `self`：这对“`T` 被移动而不是被克隆”意味着什么？如果改成 `&self`，对不可 `Clone` 的 `T` 会发生什么？
2. 从单态化角度说明 `Pair<i32>` 和 `Pair<String>` 各自会生成哪些代码，`joined` 为什么只存在于后者。

## 十六、课后巩固：固定容量的栈

主练习处理的是固定两个字段和一个只读窗口；这个任务换成**会变化的后进先出栈**，数据和边界都不同。这里的“栈（stack）”是一种容器：最后放入的值最先取出；它和第十一节提到的“内存栈”不是同一个概念。脚手架在 `src/consolidation.rs`。

### 目标与行为

实现 `Stack<T, N>`：最多保存 `N` 个值，后放入的先取出。

```text
Stack::<i32, 3>::new()                  → 空栈，capacity() == 3，len() == 0
push(10)                                → Ok(())，10 成为栈顶
push(20)                                → Ok(())，20 成为栈顶
push(30)                                → Ok(())，30 成为栈顶，len() == 3
peek()                                  → Some(&30)，不取出
pop()                                   → Some(30)，30 被移出，len() 变回 2
push(40)                                → Ok(())，40 成为栈顶，len() 回到 3
push(50)                                → Err(50)，栈已满，栈内原有内容不变
Stack::<i32, 0>::new().push(7)          → Err(7)，容量为 0 时永远放不进去
空栈的 pop() / peek()                   → None
```

### 边界与错误行为

- **容量为 0**：`push` 总是失败并返回 `Err(value)`；`pop` 和 `peek` 返回 `None`。这是合法构造，不是 panic 条件。
- **栈满**：`push` 不覆盖旧值，也不丢弃新值；它把新值放进 `Err` 交还调用者，栈的长度和内容保持原样。
- **栈空**：`pop` 和 `peek` 返回 `None`。
- **非 `Copy` 类型**：栈必须能保存 `String` 这类拥有型数据；`pop` 时把所有权交还给调用者。

### 约束与推荐顺序

- 只修改 `push`、`pop`、`peek` 的 `todo!()`。`new`、`capacity`、`len`、`is_empty` 已提供，不要修改；类型定义、签名和测试也不要改。
- 不要给 `T` 加 `Clone` 或 `Copy`：`push` 和 `pop` 只需要移动值。
- 容量判断使用 `N`（const 参数），不要从 `Vec` 的实际容量猜。
- 推荐顺序：先 `push`（先处理未满，再处理已满和 `N = 0`），再 `pop`，最后 `peek`；每步都运行对应测试。

### 测试与验收命令

脚手架提供 8 个巩固测试。`new_stack_is_empty_and_has_its_declared_capacity` 只使用已提供的方法，初始通过；其余 7 个在 `todo!()` 处失败。

```bash
cargo test -p l11_generics consolidation
```

完成后用本课完整命令再验证一次：

```bash
cargo test -p l11_generics
cargo clippy -p l11_generics --all-targets -- -D warnings
```

## 十七、常见误区

### 误区 1：“泛型就是任何类型都能做任何事”

不是。`T` 默认只保证“有确定大小”，没有 bound 时函数体几乎什么操作都不能做。能比较是因为加了 `Ord`，能复制是因为加了 `Clone`。泛型把“需要什么能力”变成签名的一部分，而不是绕过类型检查。

### 误区 2：“bound 越多越安全，反正编译器会帮我”

多余的 bound 不会让代码更安全，只会让 API 能接受的类型更少。一个只需要比较的函数要求 `Clone + Debug + Default`，会让本来能用的类型全部被挡在门外。按“删掉它，看编译器是否报缺操作”的方法最小化 bound。

### 误区 3：“把 bound 都放在 impl 上更整齐”

impl 级 bound 限制块里**所有**方法，包括根本不用该能力的方法。只有整个块都需要时才放 impl 上；只有个别方法需要时，用方法级 `where`。

### 误区 4：“泛型一定慢”或“泛型一定零成本”

单态化通常消除运行时查表和间接调用，并允许内联，所以抽象本身往往没有额外运行时开销；但为每种具体类型生成代码会增加二进制体积和编译时间，也可能影响指令缓存。“零成本”是通常性质，不是无条件承诺；需要结论时测量。

### 误区 5：“const 泛型的长度可以来自用户输入”

不能。`N` 必须在编译期确定。用户输入的长度、文件里读到的行数、网络包的实际大小，都要用 `Vec` 或切片在运行时表示。

### 误区 6：“需要复制时就加 `Clone`，反正能编译”

先问是否真的需要拥有两份值。按引用比较、移动字段、返回其中一侧，常常不需要 `Clone`。不必要的 `Clone` 既增加分配，又扩大 API 对调用方的要求。

### 误区 7：“多个 impl 块会挑更具体的那个执行”

固有 impl 是合并关系，同名方法直接报 E0592。Rust 稳定版没有“给某个具体类型换一套实现”的特化。需要不同行为时，换方法名或把差异做成参数。

### 误区 8：“`?Sized` 表示类型没有大小，所以什么都能传”

`?Sized` 只是取消“必须有编译期大小”的默认要求，让 `str`、`[T]` 这类类型能通过引用参与泛型。值仍然不能按大小未知的形式直接放在栈上，参数通常必须是 `&T`。

## 十八、自测

先用自己的话回答，再展开对照。

1. 为什么 `fn largest<T>(values: &[T])` 不能直接调用 `max`？
2. `where T: Clone + Debug` 与 `<T: Clone + Debug>` 有什么区别？
3. 单态化带来什么好处、什么代价？
4. 为什么把 `Clone` 放在整个 `impl<T>` 上可能过强？
5. `Into<String>` 和 `AsRef<Path>` 分别对应“拥有”还是“借用”的需求？
6. `T: ?Sized` 里的 `?` 是什么意思？为什么参数通常必须是引用？
7. const 泛型参数 `N` 和普通类型参数 `T` 的区别是什么？

<details><summary>参考答案</summary>

1. `T` 表示“某个类型”，没有声明它具备比较能力；`max` 要求元素实现 `Ord`。Rust 在定义处按 bound 检查函数体，不会假设调用方一定传可比较的类型。
2. 语义完全相同；`where` 只是把多个约束移到签名下方，便于排版和阅读。
3. 好处是每种具体类型生成专门代码，调用静态确定、可内联，通常没有运行时查表开销；代价是每种实例化一份机器码，增加编译时间和二进制体积。
4. impl 级 bound 限制块内所有方法，包括不使用 `clone` 的方法；不可 `Clone` 的类型会无辜失去这些方法。
5. `Into<String>` 把值转换成拥有型 `String`（可能发生分配）；`AsRef<Path>` 只在调用期间借出 `&Path` 视图。前者交出所有权，后者只借用。
6. `?` 取消泛型参数默认的 `Sized` 要求，允许 `str`、`[T]` 等不定长类型参与；这些值不能按值存放，所以通常通过 `&T` 这样的引用接触。
7. `T` 是类型参数，调用时由具体类型填充；`N` 是编译期常量参数，调用时由常量值填充。`FixedWindow<u8, 3>` 和 `FixedWindow<u8, 4>` 是不同类型，而 `N` 的值参与内存布局。

</details>

## 本课总结

1. 泛型参数是编译期变量：`<T>` 声明它，签名其余部分使用它；同一次调用里所有 `T` 指同一个具体类型。
2. trait bound 是能力声明：函数体能用什么操作由 bound 决定；Rust 在定义处检查一次，读泛型签名只看 bound 就能知道它能做什么。
3. 每多一个 bound，函数体多一批可用操作，调用方可选类型少一批；bound 应最小化。
4. bound 可以放在整个 impl 上，也可以只放在一个方法上；只有整个块都需要时才放 impl 级。
5. 多个固有 impl 块是合并关系，同名方法即冲突（E0592）；Rust 稳定版没有特化。
6. 单态化为每种具体类型生成专门代码，换来静态分发和可内联，代价是代码体积和编译时间。
7. 类型推断从实参、标注和后续使用获取线索；不足时用左侧标注或 turbofish（`::<...>`）。
8. 借用能完成的需求不要按值传，避免无谓的 `Clone`；返回引用需要生命周期参数说明有效期。
9. const 泛型让编译期常量（典型是数组长度）进入类型；运行时长度仍用 `Vec`/切片。
10. 每个 `<T>` 隐含 `Sized`；`?Sized` 是唯一“减法” bound，允许不定长类型经引用参与泛型。

## 下一课

下一课[第十二课：trait、关联类型、`impl Trait` 与 `dyn Trait`](../12-traits/README.md)会回答本课留下的问题：trait 本身如何定义？一个类型如何实现 trait？关联类型和泛型参数有什么区别？什么时候该用编译期的泛型，什么时候该用运行时的 `dyn Trait`？本课的 bound、单态化和“最小化约束”会直接成为那节课的设计工具。
