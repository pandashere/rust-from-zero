# 第十课：迭代器——把“逐个处理”写成可组合的流水线

程序里经常出现这样的任务：手里有一列数据，需要逐个查看、挑出符合条件的、把每个值改一改，最后汇总成一个结果。比如一篮商品的价格、一局游戏的得分、一份日志文件的每一行。

这一课讲 Rust 的**迭代器**（iterator，一种按顺序一次取出一个元素的工具）。学完后你会看到：同样的任务可以写成一条由几个小步骤组成的“流水线”，每一步只做一件事，而 Rust 会在编译时检查每一步的数据类型和所有权（哪段代码负责拥有并释放一份数据）。

先说明本课的组织方式：每一节都先给出一个具体问题，再说明只用已有知识会遇到的困难，然后才介绍新语法。代码示例尽量写成可以单独运行的小程序；不能单独运行的片段会明确标注。

## 本课定位

### 你将学会

- 解释迭代器是什么，理解 `Iterator` 这个行为约定里的 `next` 方法。
- 区分**适配器**（adapter，只描述“怎么变换”，不立即执行）和**消费者**（consumer，真正驱动计算、产生结果）。
- 理解**惰性**（lazy，没有消费者就不计算），并能看懂“迭代器未被使用”的编译警告。
- 区分 `iter()`、`iter_mut()`、`into_iter()` 三种所有权模式：只读借用、可变借用、把元素搬走。
- 使用 `map`、`filter`、`filter_map`、`enumerate`、`zip`、`take`、`skip`、`fold`、`collect`、`find` 等常用方法。
- 理解 `filter` 的闭包为什么常常收到双重引用，并能写出每一步的元素类型。
- 把一串可能失败的结果收集成 `Result<Vec<_>, _>`，并说明它与“跳过坏数据”的区别。
- 为自定义类型实现 `Iterator`，并写出准确的 `size_hint`。
- 阅读迭代器相关的常见编译错误，知道错误在说什么、该往哪个方向修。

### 前置知识

本课默认你学完了前九课。下面每一项都会在用到时用一两句话复习，忘记时可以点链接回看：

- 变量、函数、表达式、`for` 循环和基本类型，见[第一课](../01-basics/README.md)。
- `struct`（把几个字段组合成一种新数据）、`enum`（从几个固定选项中取一个）、`match` 和模式匹配，见[第二课](../02-data-modeling/README.md)。
- 所有权、移动（move）、借用（borrow，临时使用而不拿走所有权）、`String` 与 `&str` 的区别，见[第三课](../03-ownership-borrowing/README.md)。
- `Option<T>`（可能有值也可能没有）、`Result<T, E>`（可能成功也可能失败）和 `?` 运算符，见[第四课](../04-option-result/README.md)。
- `Vec<T>`（可增长的列表）和 slice（对一段连续数据的借用视图），见[第五课](../05-vec-slices/README.md)。
- `HashMap`（按键查找值的表）和 `entry` 计数写法，见[第六课](../06-hash-collections/README.md)。
- 闭包（closure，写在调用现场的匿名小函数，可以记住并使用外面的变量），见[第九课](../09-closures/README.md)。

### 本课暂不深入

- 泛型（generic，用类型占位符写一份适用于多种类型的代码）的完整规则在[第十一课](../11-generics/README.md)。本课看到 `fn map<B, F>(...)` 这类签名时，只把它理解为“`B` 和 `F` 是待填的类型占位符”就够了。
- trait（特征，一组行为的约定）的系统讲解在[第十二课](../12-traits/README.md)。本课只需要知道：实现了 `Iterator` 的类型，就“会逐个产出元素”。
- 显式生命周期（lifetime，标注引用能活多久的语法）在[第十四课](../14-lifetimes/README.md)。本课只依赖编译器已有的借用规则，不写生命周期标注。
- 并行迭代（例如 rayon 库）、异步数据流（`Stream`）、手写 `unsafe` 迭代器都不在本课范围。
- 标准库迭代器有几十个方法，本课只讲最常用的一批，并教你按“适配器/消费者”分类去查其余方法。
- 本课不需要任何第三方依赖；`Cargo.toml` 的依赖列表为空，所有功能都来自 Rust 标准库（随语言一起发布、任何程序都能直接使用的一组现成代码）。

## 如何学习本课

1. 先读第一到第三节，把“迭代器是会走路的光标”和“惰性”这两个模型想清楚。
2. 第四到第八节每个方法都动手运行示例，并在纸上写出每一步的元素类型。
3. 第十一节是真实编译器诊断，不要跳过：看懂报错才能独立修代码。
4. 完成第十三节的代码预测，再看答案。
5. 按第十四节做主练习；卡住时按 `HINTS.md` 的级别逐步查看提示，不要一次看到底。
6. 主练习通过后，完成第十五节的课后巩固。

预计需要 3 到 5 小时，具体取决于你是否把每个示例都亲手运行一遍。

## 一、从一个具体问题说起

假设你在写一个收银程序，手里有一列商品价格：

```rust
fn main() {
    let prices = vec![12, 5, 30, 8];
    println!("{prices:?}");
}
```

需求是：**只考虑价格大于 10 元的商品，给它们打九折，然后求折后总价**。

只用前几课的知识，可以这样写：

```rust
fn main() {
    let prices = vec![12, 5, 30, 8];

    let mut discounted = Vec::new();
    for index in 0..prices.len() {
        let price = prices[index];
        if price > 10 {
            discounted.push(price * 9 / 10);
        }
    }

    let mut total = 0;
    for price in &discounted {
        total += price;
    }

    println!("折后价格：{discounted:?}，总价：{total}");
}
```

逐行读一遍：

- `0..prices.len()` 是一个**范围**（range），表示从 `0` 开始、到 `prices.len()` 之前结束的整数序列。`prices.len()` 是元素个数，所以最后一个下标是 `len() - 1`；范围写法正好不包含右端点。
- `prices[index]` 用**下标**（index，元素在列表中的位置编号，从 0 开始）取出一个价格。这里取出的 `i32` 是复制出来的值，因为整数实现了 `Copy`（一种标记：该类型的值可以直接复制，复制后两份都能独立使用）。
- `discounted.push(...)` 往新列表末尾追加一个折后价格。
- 第二个循环用 `&discounted` 借用列表，逐个把价格加到 `total` 上。

这段代码能工作，也没有错误。问题不在“能不能跑”，而在**它把好几件事混在了一起**，并且随着需求变化会越来越难改：

1. **下标要自己管理**。从 0 开始、右端点不包含、空列表、最后一个元素，这些细节每写一次就要想一次。写错一个边界，程序可能越界崩溃（数组或列表访问了不存在的位置），也可能漏掉最后一个元素。
2. **中间结果被迫存在**。为了求总价，先创建一个 `discounted` 列表；如果数据有一百万条，这个临时列表就占一百万份空间，而它其实只被用来求一个和。
3. **意图被语法盖住**。读者要先看懂循环怎么转，才能看出“筛选、打折、求和”这三个业务步骤。
4. **不容易组合**。今天要“筛选后求和”，明天要“筛选后取前三个”，后天要“筛选后按价格排序”，每次都得重写循环骨架。
5. **所有权选择是隐含的**。`prices[index]` 复制了整数；如果元素是 `String`，这种写法根本不能复制，得改成借用或移动，而循环代码不会提醒你这件事。

### 1.1 用迭代器重写

同样的需求，用本课的工具可以写成：

```rust
fn main() {
    let prices = vec![12, 5, 30, 8];

    let total: i32 = prices
        .iter()
        .filter(|price| **price > 10)
        .map(|price| *price * 9 / 10)
        .sum();

    println!("总价：{total}");
}
```

输出同样是 `37`：`12` 打九折是 `10`，`30` 打九折是 `27`，`5` 和 `8` 被筛掉。

这段代码现在还有几个符号你可能看不懂，尤其是 `**price` 和 `*price`。这很正常。它表达的是一条流水线：

```text
prices.iter()                 逐个借出价格
    .filter(...)              只留下大于 10 的
    .map(...)                 把留下的价格打九折
    .sum()                    把它们加起来
```

每一行只做一件事，没有下标，没有临时列表。接下来几节会把这条流水线拆开，逐个解释这些符号。

### 1.2 术语：迭代、元素、集合

- **迭代**（iterate）：按某种顺序，一次处理一个数据项。
- **元素**（element，也叫 item）：被逐个处理的数据项。上面的每个价格就是一个元素。
- **集合**（collection）：装着多个元素的数据结构，例如 `Vec`、数组、`HashMap`。
- **迭代器**（iterator）：一个**自己保存“走到哪了”的状态**的值，每次向它要下一个元素，它交出一个，直到没有为止。

关键区别：`Vec` 是“装着一堆元素”的容器，它本身不知道你读到哪了；迭代器是“正在走路的读取位置”。同一个 `Vec` 可以创建多个互不影响的迭代器。

## 二、迭代器的核心：`Iterator` 与 `next`

Rust 用标准库里的 `Iterator` 来描述“会逐个产出元素”这件事。它是**trait**（特征）：一组行为的约定。一个类型实现了某个 trait，就表示它承诺提供这个 trait 规定的方法。

`Iterator` 的简化签名如下（省略了标准库中的其他默认方法和约束，这段不能单独编译）：

```rust,ignore
trait Iterator {
    type Item;
    fn next(&mut self) -> Option<Self::Item>;
}
```

三个部分各自回答一个问题：

### 2.1 `type Item`：产出什么类型的元素

`Item` 是**关联类型**（associated type）：trait 里留的一个类型位置，由每个实现者填上具体类型。

例如：

- `Vec<i32>` 的只读迭代器，`Item` 是 `&i32`；
- `Vec<String>` 的 `into_iter()` 迭代器，`Item` 是 `String`；
- 整数范围 `1..=3` 的迭代器，`Item` 是 `i32`。

一个迭代器只产出一种类型的元素，所以这里用关联类型，而不是让调用者每次再指定一个泛型参数。

### 2.2 `fn next(&mut self) -> Option<Self::Item>`：怎么取下一个

- `next` 是方法（method，属于某个类型的函数，用 `值.方法名(...)` 调用）。
- 返回值是 `Option<Self::Item>`：还有元素时给 `Some(元素)`，结束时给 `None`。第四课讲过，`Option<T>` 是一个“可能有值、也可能没有”的 enum（第二课学过的“从几个固定选项中取一个”的类型）；这样“有没有下一个”和“下一个是什么”合并成一次调用，不会出现“忘了先检查还有没有”的情况。
- 参数是 `&mut self`：`next` 要修改迭代器内部记录的当前位置，所以需要**可变借用**（mutable borrow，独占的临时访问权，期间不能有别的访问）。这也是为什么保存迭代器的变量必须声明成 `mut`。

### 2.3 一个会走路的光标

把迭代器想成一个光标：创建时站在第一个元素前面；每次调用 `next`，它向前走一步并交出一个元素；走到尽头就交 `None`。

```rust
fn main() {
    let mut cursor = [10, 20, 30].into_iter();

    assert_eq!(cursor.next(), Some(10));
    assert_eq!(cursor.next(), Some(20));
    assert_eq!(cursor.next(), Some(30));
    assert_eq!(cursor.next(), None);
}
```

这里 `[10, 20, 30]` 是一个数组（长度固定、元素类型相同的一组值）。数组的 `into_iter()` 会按值产出每个 `i32`。`cursor` 前面的 `mut` 不能省：`next` 需要可变借用，编译器会检查变量是否有权被修改。

注意 `assert_eq!` 只是“断言相等，不相等就 panic（程序立即中止并打印原因）”的宏；这段代码运行没有输出，说明四个断言都成立。

### 2.4 `for` 循环只是“反复调用 `next`”的简写

下面两段意思相同。第二段把 `for` 替我们做的事写了出来；它也能直接编译，只是日常不这样写：

```rust,ignore
// 写法一
for value in [10, 20, 30] {
    println!("{value}");
}

// 写法二：概念上等价
let mut iterator = IntoIterator::into_iter([10, 20, 30]);
while let Some(value) = iterator.next() {
    println!("{value}");
}
```

`while let Some(value) = iterator.next()` 的意思是：只要 `next()` 给出 `Some(value)`，就执行循环体；给出 `None` 就结束。`for` 循环替我们做了“创建迭代器、反复取、判断结束”这三件事，所以日常优先用 `for`。

### 2.5 一个容易忽略的边界：`None` 之后还会不会有值

`Iterator` 的约定只保证“这一次没有值”，并不保证以后永远没有值。一个返回过 `None` 之后永远返回 `None` 的迭代器，叫 **fused**（已融合、已终止）的迭代器。标准库的大多数迭代器都是 fused 的；自己实现迭代器时也应当做到这一点，第十五节的巩固任务会专门检查。

如果一个迭代器不是 fused，某些组合操作（例如把多个迭代器摊平）可能得到奇怪结果。拿不准时，可以给迭代器加上 `.fuse()` 适配器兜底。

## 三、惰性：先搭流水线，等消费者开工

看下面这个完整程序：

```rust
fn main() {
    let values = vec![1, 2, 3];

    let mapped = values.iter().map(|value| {
        println!("正在变换 {value}");
        value * 2
    });

    println!("流水线创建完成");

    let doubled: Vec<i32> = mapped.collect();
    println!("{doubled:?}");
}
```

实际输出是：

```text
流水线创建完成
正在变换 1
正在变换 2
正在变换 3
[2, 4, 6]
```

也就是说，执行到 `let mapped = ...` 时，闭包 `|value| { ... }` **一次都没有被调用**。`map` 返回的不是结果列表，而是另一个迭代器：它内部存着“原来的迭代器”和“要执行的闭包”，像一张流水线图纸。直到 `collect()` 开始向它要元素，闭包才逐个执行。

这种“只描述、不执行，等别人来取才算”的性质叫**惰性**（lazy）。

### 3.1 适配器与消费者

迭代器的方法可以分成两类，看返回值就能分辨：

| 类别 | 返回值 | 是否立即执行 | 例子 |
|---|---|---|---|
| 适配器（adapter） | 一个新的迭代器 | 否，只搭流水线 | `map`、`filter`、`take`、`skip`、`enumerate`、`zip`、`chain` |
| 消费者（consumer） | 普通值，或产生副作用 | 是，驱动整条流水线 | `collect`、`sum`、`fold`、`count`、`any`、`find`、`next`、`for_each` |

这里的**副作用**（side effect）指除了产生返回值之外，还改变外部状态，例如打印到屏幕、写文件、修改共享计数。

`next` 是最小的消费者：调用一次，整条流水线只为产出一个元素运转一次。

**只写适配器、不写消费者**，是初学者最常见的错误：

```rust,ignore
let values = vec![1, 2, 3];
values.iter().map(|value| value * 2); // 片段：这一行没有接住结果
```

编译器给出警告（不是错误，程序仍能编译）：

```text
warning: unused `Map` that must be used
  = note: iterators are lazy and do nothing unless consumed
```

`Map` 是 `map` 返回的迭代器类型。这条警告在说：你创建了一个迭代器，然后把它丢掉了；迭代器是惰性的，不消费就什么都不做。看到它就问自己：我是想收集结果（加 `collect`）、还是想产生副作用（改成普通 `for` 循环）？

### 3.2 为什么设计成惰性

惰性不是风格偏好，它带来三个实际好处：

1. **没有中间集合**。`values.iter().map(f).filter(g).take(3)` 不会在中途创建任何临时 `Vec`；元素一个接一个流过整条链，处理完就丢。
2. **只计算被消费的部分**。`take(3)` 之后的元素根本不会被处理。因此“无限长的序列”也能安全参与流水线，只要有人负责在某个地方停下来：

   ```rust
   fn main() {
       let first_three: Vec<u32> = (0..)
           .filter(|value| *value % 7 == 0)
           .take(3)
           .collect();
       println!("{first_three:?}");
   }
   ```

   输出 `[0, 7, 14]`。`0..` 表示从 0 开始、没有结束的整数范围；如果没有 `take(3)`，`collect` 会一直取下去，直到内存耗尽。

3. **组合不改变数据流方式**。链变长，只是多了几层“取一个元素、处理、交给下一个”的包装；编译器能看到完整链条并整体优化，通常与手写循环性能相当（原理见第十节末尾）。

代价是：写完一条链不代表它执行了。一定要有消费者。

## 四、三种所有权模式：读、改、搬走

第三课讲过，Rust 的每个值都有唯一的所有者；使用一个值有两种基本方式：

- **借用**（borrow）：临时使用，不拿走所有权。共享借用 `&T` 允许多个读者；可变借用 `&mut T` 只允许一个写者，且期间不能有读者。
- **移动**（move）：把所有权交给别人，原变量之后不能再用。

迭代器把这两种方式显式地做成了三个入口。以 `Vec<String>` 为例：

```rust
fn main() {
    let mut words = vec![String::from("rust"), String::from("go")];

    // 1. 只读借用：Item 是 &String
    let lengths: Vec<usize> = words.iter().map(|word| word.len()).collect();
    assert_eq!(lengths, [4, 2]);

    // 2. 可变借用：Item 是 &mut String
    for word in words.iter_mut() {
        word.push('!');
    }
    assert_eq!(words, ["rust!", "go!"]);

    // 3. 按值消费：Item 是 String
    let owned: Vec<String> = words.into_iter().collect();
    assert_eq!(owned, ["rust!", "go!"]);
}
```

`Vec<usize>` 里的 `usize` 是用来表示长度和位置的非负整数类型，大小随平台变化；`len()` 返回的就是它。

三个入口的对比：

| 写法 | 元素类型 | 对原集合的影响 | 典型场景 |
|---|---|---|---|
| `values.iter()` | `&T` | 共享借用，集合仍可用 | 只读查看、统计、查找 |
| `values.iter_mut()` | `&mut T` | 独占借用，期间不能以其他方式访问集合 | 原地修改每个元素 |
| `values.into_iter()` | `T` | 所有权移交，原变量失效 | 元素要转成别的拥有型值，或原集合不再需要 |

选择时问自己一个问题：**流水线只需要读元素、需要原地改元素，还是需要把元素搬走？**

### 4.1 `for` 循环也在做同样的选择

`for value in ...` 会对右边的值调用 `IntoIterator::into_iter`。`Vec` 有三种实现，所以下面三种写法含义不同：

```rust,ignore
// 片段：假设 words 是 Vec<String>
for word in &words {        // Item = &String，只借用
    println!("{word}");
}

for word in &mut words {    // Item = &mut String，可变借用
    word.push('!');
}

for word in words {         // Item = String，把 Vec 消费掉
    println!("{word}");
}
// 这里以后不能再使用 words
```

`&words` 的类型是 `&Vec<String>`，标准库为它实现了 `IntoIterator`，产出 `&String`；`&mut words` 产出 `&mut String`；直接写 `words` 则按值消费。

### 4.2 用了 `into_iter` 就不能再用原集合

```rust,compile_fail
fn main() {
    let values = vec![String::from("a")];
    let owned: Vec<String> = values.into_iter().collect();
    println!("{values:?}"); // 编译错误：borrow of moved value
}
```

原因很直接：`into_iter` 已经把 `Vec` 的所有权拿走了，元素也逐个交给了新列表。原变量 `values` 不再拥有任何东西，当然不能读。第十一节会展示完整报错。

如果之后还要用原集合，就该用 `iter()`；如果确实需要一份独立的拥有型副本，可以用 `.cloned()`（后面会讲），但要清楚那是复制。

## 五、`map` 与 `filter`：变换与筛选

流水线里最常用的两个适配器是 `map`（映射：把每个元素变成另一个值）和 `filter`（筛选：只留下满足条件的元素）。

### 5.1 `map`：每个元素都换成一个新值

概念上的签名如下（`<B, F>` 是泛型参数，下一课系统讲；这里只需知道它们代表“结果类型”和“闭包类型”）：

```rust,ignore
fn map<B, F>(self, f: F) -> Map<Self, F>
where
    F: FnMut(Self::Item) -> B;
```

读法：

- `self` 表示这个方法会拿走当前迭代器（不是拿走原集合），返回一个新的迭代器；
- `f` 是一个闭包，接收一个元素，返回变换后的值；
- `FnMut` 是闭包的一种能力约定，表示“可以被反复调用，并且允许修改自己记住的状态”。第九课讲过，这里不展开。

完整例子：

```rust
fn main() {
    let numbers = [1, 2, 3, 4];

    let doubled: Vec<i32> = numbers.into_iter().map(|value| value * 2).collect();

    assert_eq!(doubled, [2, 4, 6, 8]);
}
```

- `numbers.into_iter()` 的 `Item` 是 `i32`；
- `map` 的闭包收到一个 `i32`，返回 `i32`；
- `map` 之后的 `Item` 仍是 `i32`；
- `collect()` 把它收进 `Vec<i32>`。

`map` 的闭包拿到元素的**所有权**（当 `Item` 是拥有型值时）。因为它负责把元素变成别的东西，原元素不需要再往下传。

### 5.2 `filter`：只留下满足条件的元素

概念签名：

```rust,ignore
fn filter<P>(self, predicate: P) -> Filter<Self, P>
where
    P: FnMut(&Self::Item) -> bool;
```

关键在 `&Self::Item`：筛选取到的是元素的**引用**。它只负责判断，不负责改造；无论通过还是不通过，元素本身都还要继续存在——通过的交给下游，不通过的被丢弃。如果闭包拿走所有权，那“通过”的元素就还不回去了。

完整例子：

```rust
fn main() {
    let numbers = [1, 2, 3, 4, 5, 6];

    let even_tens: Vec<i32> = numbers
        .into_iter()
        .filter(|value| *value % 2 == 0)
        .map(|value| value * 10)
        .collect();

    assert_eq!(even_tens, [20, 40, 60]);
}
```

注意 `filter` 闭包里的 `*value`：`Item` 是 `i32`，`filter` 借出 `&i32`，所以 `value` 的类型是 `&i32`；取余运算需要 `i32`，用 `*` 解引用。`map` 在 `filter` 之后，此时元素已经通过筛选，`map` 拿到的是 `i32` 本身。

### 5.3 顺序会改变执行成本

下面的两条链结果相同，成本不同：

```rust
fn main() {
    let numbers = [1, 2, 3, 4];

    let a: Vec<i32> = numbers
        .iter()
        .filter(|value| **value % 2 == 0)
        .map(|value| *value * 10)
        .collect();

    let b: Vec<i32> = numbers
        .into_iter()
        .map(|value| value * 10)
        .filter(|value| *value % 20 == 0)
        .collect();

    assert_eq!(a, [20, 40]);
    assert_eq!(b, [20, 40]);
}
```

第一条先筛后映射，`map` 只对两个元素执行；第二条先映射后筛选，`map` 对四个元素都执行。元素多、`map` 里的计算昂贵时，把便宜的筛选放前面能省下实际工作。

### 5.4 为什么 `filter` 的闭包有时收到双重引用

这是本课最容易卡住的地方，值得单独讲清楚。

前面用 `into_iter()` 时，`Item` 是 `i32`，`filter` 的闭包收到 `&i32`，所以写 `|value| *value > 0`。

但如果你用的是 `iter()`，`Item` 已经是 `&i32`，`filter` 再借一次，闭包收到的是 `&&i32`。看一个 `String` 的例子：

```rust
fn main() {
    let words = vec![String::from("rust"), String::from("go"), String::from("zig")];

    let long: Vec<&String> = words
        .iter()
        .filter(|word| word.len() > 2)
        .collect();

    assert_eq!(long.len(), 2);
    assert_eq!(words.len(), 3); // words 只是被借用，仍然可用
}
```

逐步写出类型：

```text
words.iter()            Item = &String
filter(...)             闭包收到 &&String；Item 仍然是 &String
collect()               Vec<&String>
```

`word` 是 `&&String`，为什么 `word.len()` 能直接工作？因为方法调用会自动解引用：`&&String` 上没有 `len` 方法，编译器会沿着引用逐层找到 `String` 上的 `len`。这叫**自动解引用**（auto-deref），是方法调用的便利，不是你手动加了一堆 `*`。

但一旦涉及比较、模式匹配，或者把值传给不做自动解引用的地方，引用层数就要自己处理：

```rust
fn main() {
    let numbers = [1, -2, 3];

    let positive: Vec<i32> = numbers
        .iter()
        .filter(|&&value| value > 0) // 在模式里一次剥掉两层引用
        .map(|value| *value)
        .collect();

    assert_eq!(positive, [1, 3]);
}
```

`|&&value|` 是一个模式：第一个 `&` 匹配 `filter` 传进来的引用，第二个 `&` 匹配 `iter` 元素本身的引用，最后 `value` 绑定到真正的 `i32`。等价的写法是先写 `|value| **value > 0`，再用 `.map(|value| *value)` 取出值。

**不要靠不断加 `*` 猜类型。** 先写出每一步的 `Item` 类型，再决定用模式解构还是显式解引用。猜出来的代码即使碰巧编译，也常常掩盖了真实的数据流。

### 5.5 从 `&T` 得到 `T`：`copied` 与 `cloned`

`iter()` 产出的是引用。如果下游需要拥有型值，有两个适配器做这层转换：

```rust
fn main() {
    let numbers = [1, 2, 3];
    let owned: Vec<i32> = numbers.iter().copied().collect();
    assert_eq!(owned, [1, 2, 3]);

    let words = vec![String::from("a"), String::from("bb")];
    let copies: Vec<String> = words.iter().cloned().collect();
    assert_eq!(copies, words);
    assert_eq!(words.len(), 2);
}
```

- `copied()` 把 `Iterator<Item = &T>` 变成 `Iterator<Item = T>`，要求 `T: Copy`；它直接复制值本身，成本固定且很小。
- `cloned()` 也做同样的类型转换，但要求 `T: Clone`，对每个元素调用 `clone()`；`String`、`Vec` 的 `clone` 是深拷贝（把内部数据也完整复制一份，而不是只复制一个指向它的引用），n 个元素通常就是 n 次内存分配（向系统申请新内存）。

选择规则：下游只读到流水线末尾，就保留 `&T`；`T` 是 `Copy` 且需要值，用 `copied()`；确实需要互不相关的独立副本，才用 `cloned()`，并清楚复制成本。

## 六、位置、配对与截取：`enumerate`、`zip`、`take`、`skip`

### 6.1 `enumerate`：同时拿到位置和元素

`enumerate` 把每个元素包装成 `(位置, 元素)` 的元组（tuple，把几个值放在一起的复合值），位置从 `0` 开始。

```rust
fn main() {
    let lines = ["INFO: started", "WARN: low disk", "ERROR: stopped"];

    for (index, line) in lines.iter().enumerate() {
        println!("第 {} 行：{line}", index + 1);
    }
}
```

输出：

```text
第 1 行：INFO: started
第 2 行：WARN: low disk
第 3 行：ERROR: stopped
```

注意两点：

- `enumerate` 给的是从 0 开始的**迭代位置**，不是“第几行”。想显示行号就自己 `+ 1`。
- 位置只代表“当前这个元素是第几个被处理的”，不一定是原集合的下标。如果先 `filter` 再 `enumerate`，位置会重新从 0 开始连续编号，不再对应原下标：

```rust
fn main() {
    let labeled: Vec<(usize, i32)> = [10, 15, 20, 25]
        .into_iter()
        .filter(|value| *value % 10 == 0)
        .enumerate()
        .collect();

    assert_eq!(labeled, [(0, 10), (1, 20)]);
}
```

`20` 在原数组里的下标是 2，但筛选后它是第二个被处理的元素，所以位置是 1。如果业务需要原下标，要先 `enumerate` 再筛选，让编号跟着元素一起走。

### 6.2 `zip`：把两条迭代器逐项配对

`zip` 从两条迭代器各取一个元素，组成 `(左, 右)`，直到**任意一边结束**：

```rust
fn main() {
    let pairs: Vec<(i32, &str)> = [1, 2, 3]
        .into_iter()
        .zip(["a", "b"])
        .collect();

    assert_eq!(pairs, [(1, "a"), (2, "b")]);
}
```

左边有三个元素，右边只有两个，结果只有两对；左边多出来的 `3` 被**静默丢弃**，不会有警告。如果“两边长度必须一致”是业务前提，先比较长度再 `zip`，不要让截断行为悄悄掩盖数据问题。

### 6.3 `take` 与 `skip`：截取前一段、跳过前一段

```rust
fn main() {
    let first_two: Vec<i32> = (1..=5).take(2).collect();
    let after_two: Vec<i32> = (1..=5).skip(2).collect();

    assert_eq!(first_two, [1, 2]);
    assert_eq!(after_two, [3, 4, 5]);
}
```

`take(n)` 最多取前 `n` 个，取够就停止；`skip(n)` 丢掉前 `n` 个，再从后面继续。它们都是适配器，仍然是惰性的。`take` 经常用来给无限序列加上边界。

## 七、`collect`：把流水线结果收进集合

`collect` 是最常用的消费者：它把迭代器里的元素逐个取出，放进某种集合。

### 7.1 目标类型决定收进什么

`collect` 不是“生成 `Vec` 的方法”，而是“把元素交给任何能从迭代器构建自己的类型”。目标类型不同，结果就不同：

```rust
use std::collections::{HashMap, HashSet};

fn main() {
    let list: Vec<i32> = [1, 2, 2, 3].into_iter().collect();
    let set: HashSet<i32> = [1, 2, 2, 3].into_iter().collect();
    let map: HashMap<&str, i32> = [("a", 1), ("b", 2)].into_iter().collect();
    let text: String = ['R', 'u', 's', 't'].into_iter().collect();

    assert_eq!(list, [1, 2, 2, 3]);
    assert_eq!(set.len(), 3); // 重复元素被静默去重
    assert_eq!(map["a"], 1);
    assert_eq!(text, "Rust");
}
```

每种目标有自己的规则：

- `Vec`：照单全收，保持顺序；
- `HashSet`：重复元素只保留一个，不保证顺序；
- `HashMap`：元素必须是 `(键, 值)` 元组；重复的键，后面的值覆盖前面的值；
- `String`：元素是 `char` 时逐个拼接成文本。

这些规则是类型自己的实现，`collect` 只负责把元素递过去。像“键必须唯一”这样的业务约束，`collect` 不会替你检查；需要时先检查，或收集后比较长度。

### 7.2 目标类型必须能推断

编译器必须知道你往哪种集合里收。上下文没有线索时：

```rust,compile_fail
fn main() {
    let values = (1..=3).collect(); // 错误：type annotations needed
    println!("{values:?}");
}
```

`Vec<i32>` 和 `HashSet<i32>` 的语义不同，编译器不会替你猜。两种等价修法：

```rust
fn main() {
    let values: Vec<_> = (1..=3).collect();
    let also_values = (1..=3).collect::<Vec<_>>();

    assert_eq!(values, also_values);
}
```

- 左侧标注 `let values: Vec<_>`：`_` 表示“元素类型你来推断”。
- 右侧 **turbofish**（涡轮鱼，`::<...>` 这个形状的写法）：`collect::<Vec<_>>()`。它适合把类型说明嵌在表达式中间。

### 7.3 收集可能失败的结果

如果每个元素本身是 `Result`，可以把整条流水线收集成 `Result<Vec<_>, _>`：

```rust
fn main() {
    let parsed: Result<Vec<i32>, _> = ["1", "2", "3"]
        .into_iter()
        .map(str::parse::<i32>)
        .collect();

    assert_eq!(parsed, Ok(vec![1, 2, 3]));
}
```

规则是：

- 所有元素都是 `Ok`：得到 `Ok(收集好的集合)`；
- 遇到**第一个** `Err`：立即停止，返回这个 `Err`；已经收集的临时结果被正常销毁。

也就是说，`collect::<Result<Vec<_>, _>>()` 是**遇错即停**（fail-fast）的。它不会返回“部分成功的列表”。

对比三种处理坏数据的策略：

| 策略 | 错误处理 | 适用场景 |
|---|---|---|
| `filter_map(|x| x.ok())` | 静默丢掉所有 `Err` | 坏数据无关紧要、调用方不需要知道 |
| `collect::<Result<Vec<_>, _>>()` | 第一个错误就整体失败 | 坏数据必须上报，且只需知道第一个 |
| 手动 `for` 循环分成两堆 | 成功和失败全部保留 | 表单校验，需要一次告诉用户所有问题 |

`filter_map` 同时做“映射”和“筛选”：闭包返回 `Option`，`Some` 留下，`None` 丢掉。它很方便，但如果 `Err` 代表真实问题，用它就等于把问题藏起来。写之前问自己：**调用方需不需要知道有坏数据？**

## 八、`fold` 与常见消费者

### 8.1 `fold`：带累积值的单遍遍历

`fold`（折叠）把一个初始累积值和每个元素依次结合，最后得到一个结果。概念签名：

```rust,ignore
fn fold<B, F>(self, init: B, f: F) -> B
where
    F: FnMut(B, Self::Item) -> B;
```

`init` 是第一份累积值；闭包每轮收到“当前累积值”和“下一个元素”，返回新的累积值。

```rust
fn main() {
    let total = [1, 2, 3]
        .into_iter()
        .fold(0, |accumulator, value| accumulator + value);

    assert_eq!(total, 6);
}
```

追踪过程：

```text
开始：accumulator = 0
元素 1：accumulator = 0 + 1 = 1
元素 2：accumulator = 1 + 2 = 3
元素 3：accumulator = 3 + 3 = 6
结束：返回 6
```

累积值是按所有权传递的：每轮闭包消费旧的累积值，交出一个新的。因此累积 `String`、`Vec` 这类拥有型数据时，不需要克隆：

```rust
fn main() {
    let joined = ["a", "b", "c"]
        .into_iter()
        .fold(String::new(), |mut accumulator, part| {
            accumulator.push_str(part);
            accumulator
        });

    assert_eq!(joined, "abc");
}
```

### 8.2 有专门名字的消费者优先用专门方法

概念上，很多消费者都是 `fold` 的特例：`sum` 是累加的 fold，`collect` 是往集合里追加的 fold，`count` 是“每轮加 1”的 fold。Rust 为常见需求提供了专门方法，读起来比手写 `fold` 更清楚：

```rust
fn main() {
    let values = [3, 1, 4, 1, 5];

    assert!(values.into_iter().any(|value| value > 4)); // 有任意一个满足
    assert!(values.into_iter().all(|value| value > 0)); // 全部满足
    assert_eq!(values.into_iter().max(), Some(5));
    assert_eq!(values.into_iter().position(|value| value == 4), Some(2));
    assert_eq!(values.into_iter().find(|value| *value == 1), Some(1));
    assert_eq!(values.into_iter().count(), 5);
    assert_eq!(values.into_iter().sum::<i32>(), 14);
}
```

这些方法的共同点：它们都是消费者，调用时立刻开始取元素。

- `any`：任意一个满足就返回 `true`；`all`：全部满足才返回 `true`。
- `max` / `min`：返回最大/最小元素的引用（对 `iter()` 而言）或值（对 `into_iter()` 而言）。
- `position`：返回第一个满足条件的元素的位置，类型是 `Option<usize>`。
- `find`：返回第一个满足条件的元素本身，类型是 `Option<Item>`。
- `count`、`sum`、`product`：计数、求和、求积。

还有一个容易忽略的差别：`position` 的闭包按值接收元素（示例里写 `value == 4`），而 `find` 按引用接收（示例里写 `*value == 1`）。原因在签名：`position` 只需判断并返回位置；`find` 要把元素本身交还给调用者，所以先借用，找到后再交出所有权。

### 8.3 短路：找到答案就停

`any`、`all`、`find`、`position` 都会**短路**（short-circuit）：一旦结果已经确定，就不再检查后面的元素。

```rust
fn main() {
    let first_large = (1..).find(|value| {
        let value = *value;
        value * value > 100
    });

    assert_eq!(first_large, Some(11));
}
```

`(1..)` 是无限范围，但 `find` 找到 `11` 就停止，整条流水线只跑了 11 步。这也是惰性的实际价值：无限序列只要有人负责停下来，就可以安全使用。

短路也意味着：**闭包有副作用时，后续元素不会执行**。如果每个元素都必须处理（例如每个都要写进文件），不能用 `any`/`find` 代替完整遍历。

## 九、完整示例：学生成绩报告

下面是一个完整的程序，综合使用 `iter`、`filter`、`map`、`collect`、`sum` 和 `max_by_key`。它和后面的主练习数据不同，可以先运行一遍，观察输出。

```rust
struct Student {
    name: String,
    score: u32,
}

fn build_report(students: &[Student]) -> (Vec<&str>, u32, Option<&str>) {
    let passed: Vec<&str> = students
        .iter()
        .filter(|student| student.score >= 60)
        .map(|student| student.name.as_str())
        .collect();

    let total: u32 = students.iter().map(|student| student.score).sum();

    let best: Option<&str> = students
        .iter()
        .max_by_key(|student| student.score)
        .map(|student| student.name.as_str());

    (passed, total, best)
}

fn main() {
    let students = vec![
        Student { name: String::from("Ada"), score: 92 },
        Student { name: String::from("Lin"), score: 58 },
        Student { name: String::from("Grace"), score: 77 },
    ];

    let (passed, total, best) = build_report(&students);

    println!("及格：{passed:?}");
    println!("总分：{total}");
    println!("最高分：{best:?}");
}
```

输出：

```text
及格：["Ada", "Grace"]
总分：227
最高分：Some("Ada")
```

逐步看 `build_report` 的类型变化：

```text
students.iter()                         Item = &Student
.filter(|student| ...)                  闭包收到 &&Student；Item 仍是 &Student
.map(|student| student.name.as_str())   Item = &str
.collect()                              Vec<&str>

students.iter()                         Item = &Student
.map(|student| student.score)           Item = u32
.sum()                                  u32

students.iter()                         Item = &Student
.max_by_key(|student| student.score)    Option<&Student>
.map(|student| student.name.as_str())   Option<&str>
```

几个值得注意的点：

- `student.score` 在闭包里看起来像直接访问字段，但 `student` 是 `&&Student`；字段访问会自动解引用，和 `len()` 的方法调用一样。
- 返回的 `Vec<&str>` 和 `Option<&str>` 都借用 `students` 里的名字，没有复制文本。因此只要报告还在使用，`students` 就不能被修改或销毁。这是借用检查器（borrow checker，负责验证借用规则的编译器组件）在保证安全，不是麻烦。
- `max_by_key` 在分数相同时返回**最后一个**最大值；如果业务要求同分取先出现的人，需要自己写更明确的规则，不能依赖未说明的顺序。
- 总分用 `u32` 累加，学生人数很多时可能溢出。真实程序要评估数据范围，必要时用 `u64` 或带检查的加法。

## 十、自定义迭代器：实现一个方法，获得整套工具

迭代器最划算的地方：只要实现 `next`，就能免费获得 `map`、`filter`、`fold`、`collect` 等全部适配器和消费者——它们都是 `Iterator` trait 上的默认方法，只依赖 `next`。

下面实现一个“小于 `limit` 的偶数”迭代器：

```rust
struct Evens {
    next: u32,
    limit: u32,
}

impl Evens {
    fn new(limit: u32) -> Self {
        Self { next: 0, limit }
    }
}

impl Iterator for Evens {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next >= self.limit {
            return None;
        }

        let value = self.next;
        self.next = self.next.saturating_add(2);
        Some(value)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = if self.next >= self.limit {
            0
        } else {
            (self.limit - self.next).div_ceil(2) as usize
        };
        (remaining, Some(remaining))
    }
}

fn main() {
    let values: Vec<u32> = Evens::new(10).map(|value| value + 1).collect();

    assert_eq!(values, [1, 3, 5, 7, 9]);
}
```

逐项解释：

- `Evens::new(10)` 会依次产出 `0, 2, 4, 6, 8`，因为 `limit` 是**不包含**的上界。
- `next` 先检查是否已经到达上界；到达就返回 `None`，不再修改状态。因此返回过一次 `None` 之后，后面仍然返回 `None`，满足 fused 要求。
- `self.next = self.next.saturating_add(2)` 用**饱和加法**：如果再加 2 会超出 `u32` 能表示的最大值，就停在最大值，而不是在调试模式（开发时默认的编译方式，会检查整数溢出）下 panic。自定义迭代器要考虑这种边界。
- `size_hint` 返回“还能产出多少个”的提示，类型是 `(下界, Option<上界>)`：下界表示“至少还有多少个”，上界表示“至多还有多少个”。这里数量可以精确算出，所以下界和上界都给真实值。`div_ceil(2)` 是“除以 2 后向上取整”，用来数 `next, next+2, ...` 这样的等差数列还剩几项。

`size_hint` 的作用是给消费者优化提示：`collect` 可以用它预先分配空间，`ExactSizeIterator` 的 `len()` 默认实现会读取它并要求上下界一致。默认实现返回 `(0, None)`，永远正确但帮不上忙。

**上界必须真实。** 夸大上界会让依赖它的代码分配过多空间，某些底层代码甚至可能因此越界，这是正确性问题，不是性能问题。拿不准时，返回更保守的上界或 `None`。

### 10.1 为什么链式写法通常不慢

每个适配器返回的都是一个泛型结构体，链式调用把它们嵌套成一个具体的类型，例如 `Map<Filter<std::vec::IntoIter<i32>, ...>, ...>`。编译器为实际用到的具体类型生成专门代码，并把每一层很小的 `next` 调用内联（把被调用函数的代码直接展开到调用处），最终通常与手写循环相同，也不会产生中间集合。

真正可能花时间的是别处：`collect` 的内存分配、`cloned()` 的深拷贝、排序、哈希计算，以及闭包里的昂贵工作。不要因为链长就猜它慢，也不要因为链短就猜它快；需要判断时先测量。

实现自定义迭代器时要先明确四件事：

1. 当前要产出什么值；
2. 结束边界是否包含；
3. 计数或加法会不会溢出；
4. 返回 `None` 后是否永远结束。

第十五节的课后巩固会让你实现一个方向相反、边界不同的倒计时迭代器。

## 十一、真实编译错误与诊断

这一节展示五种常见报错。报错信息来自实际编译，换行和路径做了删减；重点是学会读“错误在说哪一行、为什么、往哪修”。

### 11.1 忘记消费者：`unused Map that must be used`

```rust,ignore
fn main() {
    let values = vec![1, 2, 3];
    values.iter().map(|value| value * 2);
}
```

```text
warning: unused `Map` that must be used
 --> lazy.rs:3:5
  |
3 |     values.iter().map(|value| value * 2);
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  |
  = note: iterators are lazy and do nothing unless consumed
```

- `warning` 是警告，不是错误，程序仍会编译。
- `unused Map`：`map` 返回的迭代器类型叫 `Map`，它被创建后没有被使用。
- `iterators are lazy...`：迭代器是惰性的，不消费就什么都不做。

修复方向：要结果就在链尾加 `collect`；要副作用就改成普通 `for` 循环。

### 11.2 `into_iter` 之后又使用原集合：E0382

```rust,compile_fail
fn main() {
    let values = vec![String::from("a")];
    let owned: Vec<String> = values.into_iter().collect();
    println!("{values:?}");
}
```

```text
error[E0382]: borrow of moved value: `values`
   |
 2 |     let values = vec![String::from("a")];
   |         ------ move occurs because `values` has type `Vec<String>`,
   |                which does not implement the `Copy` trait
 3 |     let owned: Vec<String> = values.into_iter().collect();
   |                                     ----------- `values` moved due to this method call
 4 |     println!("{values:?}");
   |                ^^^^^^ value borrowed here after move
```

- `E0382` 是“使用了已经被移走的值”。
- 诊断直接指出移动发生在 `into_iter` 调用处，因为 `into_iter` 按值接收 `self`。
- `Vec<String>` 没有实现 `Copy`，所以不能像整数那样自动复制。

修复方向：之后还要用原集合就改用 `iter()`，需要拥有型副本就用 `cloned()`，或者干脆接受原集合已经不再需要。

### 11.3 `collect` 缺目标类型：E0283

```rust,compile_fail
fn main() {
    let values = (1..=3).collect();
    println!("{values:?}");
}
```

```text
error[E0283]: type annotations needed
   |
 2 |     let values = (1..=3).collect();
   |         ^^^^^^           ------- type must be known at this point
   |
   = note: cannot satisfy `_: FromIterator<i32>`
```

- `E0283` 是“编译器需要更多类型信息才能选出一个实现”。
- `FromIterator` 是“能从迭代器构建自身”的 trait；`Vec`、`HashSet`、`String` 等都实现了它，所以编译器不知道选哪个。
- `_: FromIterator<i32>` 里的 `_` 就是“还没推断出来的类型”。

修复方向：`let values: Vec<_> = ...` 或 `...collect::<Vec<_>>()`。

### 11.4 迭代器还活着时修改集合：E0502

```rust,compile_fail
fn main() {
    let mut values = vec![1, 2, 3];
    let mut iterator = values.iter();

    values.push(4);
    println!("{:?}", iterator.next());
}
```

```text
error[E0502]: cannot borrow `values` as mutable because it is also borrowed as immutable
   |
 3 |     let mut iterator = values.iter();
   |                        ------ immutable borrow occurs here
4 |
5 |     values.push(4);
   |     ^^^^^^^^^^^^^^ mutable borrow occurs here
6 |     println!("{:?}", iterator.next());
   |                      -------- immutable borrow later used here
```

- 迭代器内部保存着对 `Vec` 元素的共享借用，它不是一个副本，而是“趴在集合上的光标”。
- 只要迭代器之后还会被使用，这个借用就还活着。此时 `push` 可能需要重新分配内存，让迭代器里的指针失效，所以借用检查器在编译期直接拒绝。
- 诊断的最后一行 `immutable borrow later used here` 很关键：借用持续到迭代器的**最后一次使用**，不是到大括号结束。

修复方向：缩小迭代器的作用域；或者先把迭代器消费完、`drop(iterator)` 之后再修改集合。

### 11.5 从共享引用里把值移出来：E0507

```rust,compile_fail
fn main() {
    let words = vec![String::from("a")];
    let owned: Vec<String> = words.iter().map(|word| *word).collect();
    println!("{owned:?}");
}
```

```text
error[E0507]: cannot move out of `*word` which is behind a shared reference
 --> moveout.rs:3:54
  |
3 |     let owned: Vec<String> = words.iter().map(|word| *word).collect();
  |                                                      ^^^^^ move occurs because `*word` has type `String`, which does not implement the `Copy` trait
  |
help: consider cloning the value if the performance cost is acceptable
  |
3 -     let owned: Vec<String> = words.iter().map(|word| *word).collect();
3 +     let owned: Vec<String> = words.iter().map(|word| word.clone()).collect();
```

关键信息：

- `word` 是 `&String`，`*word` 是它指向的 `String`；`String` 不是 `Copy` 类型，不能直接复制出第二份。
- 从共享引用后面把值“搬走”，会在原 `Vec` 里留下一个空洞，破坏“每个值只有一个所有者”的规则，所以编译器禁止。

修复方向按需求选：

- 只需要长度等派生信息：`map(|word| word.len())`；
- 确实需要一份独立副本：`.cloned()`（见 5.5 节的成本说明）；
- 确实要把元素搬出来：改用 `into_iter()`，把集合消费掉。

## 十二、常见误区

1. **只写适配器，忘了消费者**。链尾没有 `collect`/`sum`/`for` 等消费者，整条链不会执行。编译器的 `unused ... that must be used` 警告就是在提醒这件事。
2. **`collect` 不写目标类型**。上下文推断不出来时，用 `let v: Vec<_>` 或 `collect::<Vec<_>>()`。
3. **用 `filter_map` 吞掉错误**。闭包里的 `.ok()` 会把 `Err` 变成 `None` 再丢掉；如果错误重要，应该用 `collect::<Result<Vec<_>, _>>()` 或手动分区。
4. **对 `Result` 使用 `flatten`**。`flatten` 会把嵌套的可迭代值摊平，而 `Result` 也被视为“0 或 1 个元素”，于是 `Err` 会被静默丢弃。错误处理语义不应该藏在一个看起来无害的 `flatten` 里。
5. **`into_iter` 之后继续用原集合**。所有权已经移走，编译器会报 E0382。先想清楚是借用还是搬走。
6. **迭代器还活着时修改集合**。共享借用未结束，`push`/`clear` 等操作会报 E0502。缩小作用域或先消费完。
7. **在 `filter` 闭包里引用层数搞错**。`iter()` 的 `Item` 是 `&T`，`filter` 又借一次，闭包看到 `&&T`。先写类型流，再决定 `*` 或模式解构。
8. **为了拿到拥有值无脑 `cloned()`**。`cloned()` 对每个元素调用 `clone()`；`String`、`Vec` 是深拷贝，n 个元素就是 n 次内存分配。先确认真的需要独立副本。
9. **`zip` 长度不一致时以为会报错**。`zip` 在较短一边结束时停止，较长一边的剩余元素被静默丢弃。业务要求等长时，先检查。
10. **对无限迭代器直接 `collect`**。没有 `take`、`find` 之类的边界，程序会一直取下去直到内存耗尽。
11. **把复杂状态机硬塞进一个 `fold`**。`fold` 适合“累积一个值”的清晰场景；如果闭包要维护多个互相影响的状态，普通 `for` 循环往往更容易读、也更容易处理错误。
12. **自定义迭代器返回 `None` 后又“复活”**。标准库的许多组合操作假设迭代器是 fused 的；自己实现时，让结束状态保持不变。

## 十三、代码预测

先写下答案和理由，再展开解析。

### 预测 1：`zip` 的结果

```rust
fn main() {
    let pairs: Vec<(i32, &str)> = [1, 2, 3]
        .into_iter()
        .zip(["a", "b"])
        .collect();
    println!("{pairs:?}");
}
```

<details><summary>解析</summary>

输出 `[(1, "a"), (2, "b")]`。`zip` 在较短的一边（右边的两个元素）结束时停止，`3` 被静默丢弃。不会报错，也不会有警告。

</details>

### 预测 2：`filter` 之后的 `enumerate`

```rust
fn main() {
    let labeled: Vec<(usize, i32)> = [10, 15, 20, 25]
        .into_iter()
        .filter(|value| *value % 10 == 0)
        .enumerate()
        .collect();
    println!("{labeled:?}");
}
```

<details><summary>解析</summary>

输出 `[(0, 10), (1, 20)]`。`enumerate` 在 `filter` 之后，位置重新从 0 开始，只数通过筛选的元素。`20` 在原数组中的下标是 2，但在这里是第二个被处理的元素，所以位置是 1。若需要原下标，应先 `enumerate` 再 `filter`。

</details>

### 预测 3：收集 `Result`

```rust
fn main() {
    let parsed: Result<Vec<i32>, _> = ["1", "bad", "3"]
        .into_iter()
        .map(str::parse::<i32>)
        .collect();
    println!("{parsed:?}");
}
```

<details><summary>解析</summary>

输出一个 `Err`，其中是解析 `"bad"` 失败的错误。`collect` 遇到第一个 `Err` 就停止，不会返回 `Ok(vec![1])` 这样的部分结果。第三个元素 `"3"` 不会被处理。

</details>

### 预测 4：借用之后原集合还能不能用

```rust
fn main() {
    let values = vec![String::from("a"), String::from("bb")];
    let lengths: Vec<usize> = values.iter().map(|word| word.len()).collect();
    println!("{lengths:?} {values:?}");
}
```

<details><summary>解析</summary>

输出 `[1, 2] ["a", "bb"]`。`iter()` 只共享借用元素，`map` 产生独立的 `usize`；`collect` 完成后迭代器的借用结束，原 `Vec<String>` 没有被消费，仍然可以打印。

</details>

### 预测 5：这个程序能编译吗

```rust,compile_fail
fn main() {
    let mut values = vec![1, 2, 3];
    let mut iterator = values.iter();
    values.push(4);
    println!("{:?}", iterator.next());
}
```

<details><summary>解析</summary>

不能编译，报 E0502。`iterator` 在 `push` 之后还会被使用（`iterator.next()`），所以它对 `values` 的共享借用仍然活着；`push` 需要可变借用，两者冲突。删除最后的 `println!` 或缩小 `iterator` 的作用域后，借用提前结束，就能编译。

</details>

## 十四、主练习：日志转换管线

现在把本课的工具用在一个完整任务上。请打开 `src/exercise.rs`，只替换函数体里的 `todo!()`；类型、函数签名和测试都已准备好，不要修改它们。

### 14.1 任务背景与真实输入

服务器日志的每一行形如 `LEVEL: message`，其中级别只能是 `INFO`、`WARN`、`ERROR` 三种。例如：

```text
INFO: service started
WARN: disk usage above 80%
ERROR: connection refused
```

程序需要把文本变成结构化的 `LogEntry`，并支持几种查询。输入用 `&[&str]` 表示：一个切片（借用一段连续数据的视图），其中每个元素都是一个 `&str` 文本引用，代表一行文本。

### 14.2 数据定义

脚手架已经给出：

```rust,ignore
pub enum Level {
    Info,
    Warn,
    Error,
}

pub struct LogEntry {
    pub line: usize,
    pub level: Level,
    pub message: String,
}

pub struct ParseLogError {
    pub line: usize,
    pub text: String,
}
```

- `line` 从 1 开始，表示原始输入中的行号。
- `text` 保存失败行的**原始内容**，包括两端空白，方便向用户展示。
- `message` 是去掉两端空白后的正文，拥有自己的 `String`。

### 14.3 需要实现的函数

| 函数 | 作用 |
|---|---|
| `parse_line(line, content)` | 解析一行；失败时返回带行号和原文的错误 |
| `parse_all(lines)` | 解析全部行；遇到第一条坏行就返回错误 |
| `partition_logs(lines)` | 同时返回所有成功项和所有错误项 |
| `error_messages(entries)` | 借用并返回所有 `ERROR` 正文 |
| `summarize(entries)` | 统计每种级别出现多少次 |
| `first_error(entries)` | 返回第一条 `ERROR` 级别的日志 |

每个函数上方的文档注释是完整的行为约定，包含分隔规则、空白处理、行号规则和错误内容。实现前先读一遍。

### 14.4 输入、输出与边界示例

解析单行：

| 调用 | 结果 |
|---|---|
| `parse_line(3, "WARN:  disk full ")` | `Ok(LogEntry { line: 3, level: Warn, message: "disk full" })` |
| `parse_line(4, "ERROR: failed at 12:30")` | `Ok(... message: "failed at 12:30")`（只在第一个冒号处切分） |
| `parse_line(5, "just noise")` | `Err(ParseLogError { line: 5, text: "just noise" })` |
| `parse_line(6, "DEBUG: chatty")` | `Err(... text: "DEBUG: chatty")`（级别不认识） |
| `parse_line(7, "INFO:   ")` | `Err(... text: "INFO:   ")`（正文为空） |
| `parse_line(8, " INFO: x")` | `Err(...)`（级别前有空格，不匹配 `"INFO"`） |

批量解析：

```text
输入：["INFO: ok", "bad line", "ERROR: never reached"]
parse_all 结果：Err { line: 2, text: "bad line" }
partition_logs 结果：([line 1 的 Info 条目], [line 2 的错误])
```

`parse_all` 遇到第 2 行就整体失败，不会返回部分列表；`partition_logs` 则会保留第 1 行的成功和第 2 行的错误。这就是“遇错即停”和“全量保留”两种策略的差别。

其他函数的边界：

- `error_messages` 只看 `ERROR` 级别，按原顺序返回消息的借用；没有错误时返回空列表，原 `entries` 始终可用。
- `summarize` 只统计出现过的级别；空输入得到空映射。
- `first_error` 只返回第一条 `ERROR`；没有时返回 `None`，返回的是借用。

### 14.5 约束

1. 只修改 `src/exercise.rs` 中 `todo!()` 所在的函数体，不修改类型、签名和测试。
2. 至少有一条流水线使用 `collect::<Result<Vec<_>, _>>()` 来收集可能失败的结果。
3. `first_error` 使用 `find` 或 `find_map` 这类短路查找。
4. 不允许用 `unwrap`、`expect` 或 `panic!` 处理输入；输入错误必须变成 `ParseLogError`。
5. 不允许用 `filter_map(...ok())` 或其他方式静默丢弃错误。
6. 保留行号、原文和输入顺序；错误信息不能被简化成一句“解析失败”。
7. `partition_logs` 如果普通 `for` 循环更清楚，可以用 `for`；本课不强制所有函数都写成链式。
8. 重点体会 `iter`、`enumerate`、`filter` 的引用层数，以及最终 `collect` 的目标类型。

### 14.6 推荐完成顺序

1. `parse_line`：先完成单行解析，弄清“第一个冒号”“级别精确匹配”“正文 trim 后非空”“错误保存原文”四件事。
2. `parse_all`：用 `enumerate` 得到从 1 开始的行号，把每行交给 `parse_line`，再收集成 `Result`。
3. `partition_logs`：把所有结果分成成功和失败两堆，保持顺序；可以先用普通 `fold` 或 `for`。
4. `error_messages` 与 `first_error`：练习只读借用、`filter`、`map` 和短路 `find`。
5. `summarize`：用 `fold` 或 `for` 配合 `HashMap` 的 `entry` 计数。
6. 运行测试和严格 Clippy，处理所有警告。

### 14.7 测试说明与验收

主练习共有 **22 个自动测试**，覆盖：

- 正常输入：三种级别、消息中带额外冒号、两端空白、空输入；
- 边界输入：`INFO:` 这类空正文、级别前有空格、空列表、全部成功和全部失败的分区；
- 错误输入：缺冒号、未知级别、大小写不符、错误保留原文、首条失败和中间失败。

这些测试在脚手架上已经可以编译；运行时会因为 `todo!()` 而失败，这是预期的，不是测试写错了。完成实现后应全部通过。

逐项验收：

```bash
cargo fmt -p l10_iterators -- --check
cargo check -p l10_iterators --all-targets
cargo test -p l10_iterators --no-run
cargo test -p l10_iterators
cargo clippy -p l10_iterators --all-targets -- -D warnings
```

- 前三条在实现前就应通过；
- `cargo test -p l10_iterators` 在实现前会在 `todo!()` 处失败；
- 实现完成后五条命令都应成功，且没有剩余 `todo!()`。

## 十五、课后巩固：自定义倒计时迭代器

主练习处理的是文本和错误；这个任务换成数字序列，边界也不同。请打开 `src/consolidation.rs`，只替换三处 `todo!()`：`Countdown::new`、`Countdown::next` 和 `Countdown::size_hint`。

### 15.1 目标与行为

`Countdown` 从某个起始值开始，每次减 1：

```text
Countdown::new(3) 依次产出：3, 2, 1，然后永远 None
Countdown::new(1) 依次产出：1，然后永远 None
Countdown::new(0) 一开始就是 None
```

要求：

- `new(start)` 保存起始值，第一次 `next` 返回 `start`；
- 产出 `1` 之后返回 `None`；返回过 `None` 后必须一直返回 `None`（fused）；
- `size_hint` 的下界和上界都要等于剩余值的真实数量；
- 减 1 不能溢出：只在还有值时递减；
- 不修改 `ExactSizeIterator` 的空实现和测试。

### 15.2 验收测试

课后巩固共有 **9 个自动测试**，覆盖：

- 正常序列：从 3 开始、从 1 开始；
- 边界：从 0 开始、连续多次 `None`、`u32::MAX` 起始且不溢出；
- 属性：`size_hint` 随消费更新、`len()` 与剩余数量一致；
- 组合：在 `for` 循环中使用、接 `filter`/`map`/`collect`。

验收命令：

```bash
cargo test -p l10_iterators consolidation
```

也可以运行全部测试和严格检查：

```bash
cargo test -p l10_iterators
cargo clippy -p l10_iterators --all-targets -- -D warnings
```

完成前测试会在 `todo!()` 处失败；完成后应全部通过。

## 十六、本课总结

1. 迭代器是一个保存“走到哪了”的状态的值；`Iterator` 的核心方法是 `next(&mut self) -> Option<Item>`。有元素给 `Some`，结束给 `None`。
2. `for` 循环是“反复调用 `next`”的语法糖；它通过 `IntoIterator` 从集合创建迭代器。
3. 适配器只搭流水线，消费者才真正驱动计算。链尾没有消费者时，什么都不会执行。
4. 惰性让流水线不产生中间集合、只计算被消费的部分，也让无限序列可以配合 `take`、`find` 等边界使用。
5. `iter`、`iter_mut`、`into_iter` 分别让 `&T`、`&mut T`、`T` 流过流水线；选择哪一个，取决于你只需要读、需要改，还是需要搬走。
6. `map` 的闭包取得元素本身并返回新值；`filter` 的闭包只借用元素做判断，所以 `iter()` 之后常常看到 `&&T`。先写类型流，再决定怎么解引用。
7. `collect` 的目标类型决定结果集合；类型推断不出来时用 `Vec<_>` 标注或 turbofish。收集 `Result` 是遇错即停，和 `filter_map` 静默丢弃错误有本质区别。
8. `fold` 表达“带累积值的单遍遍历”；有专门名字的消费者（`sum`、`count`、`any`、`find` 等）通常更清楚。短路消费者找到答案就停止。
9. 自定义迭代器只需要实现 `next`，就能接入全部适配器；还要考虑结束边界、溢出和 fused 语义，`size_hint` 的上界必须真实。
10. 借用型迭代器活着时不能修改集合（E0502）；`into_iter` 消费集合后原变量失效（E0382）；`collect` 缺类型信息报 E0283；从共享引用移出值报 E0507。看懂这些诊断，比背方法名更重要。

## 十七、课后复盘

用自己的话回答，不要求写代码：

1. 为什么 `map` 是惰性的？如果改成“每调用一次就立即生成整个结果列表”，会失去哪些能力？
2. `iter()` 和 `into_iter()` 对 `Vec<String>` 的元素类型分别是什么？各自适合什么场景？
3. 为什么 `filter` 的闭包收到的是元素的引用，而不是元素本身？
4. `collect::<Result<Vec<_>, _>>()` 遇到第二个元素失败时会怎样？已经处理的第一个元素去哪了？
5. `zip` 两边长度不同时会发生什么？如果你的业务不能接受这种结果，应该先做什么？
6. 自定义迭代器返回 `None` 之后又返回 `Some`，可能给哪些组合操作带来问题？`size_hint` 的上界说谎又会怎样？
7. 如果你看到 E0502，如何判断是哪个借用还没有结束？

## 下一课

下一课是[第十一课：泛型](../11-generics/README.md)。本课已经多次看到 `map<B, F>`、`Item`、`FnMut` 这样的写法，但一直把它们当作“待填的占位符”。下一课会解释：泛型参数到底是什么、`trait bound`（对类型能力的约束）如何声明、编译器如何为每种具体类型生成代码，以及为什么迭代器链通常没有运行时开销。
