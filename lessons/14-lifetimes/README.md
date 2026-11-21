# 第十四课：生命周期、借用关系与零复制文本 API

假设程序把一份配置文件整体读进内存：一个 `String` 里有很多行，每行形如 `language = rust`。程序想建立一个目录，之后可以用 `language` 查到 `rust`。

最直接的做法是给每个键和值都新建一个 `String` 存起来。这样做能用，但一份 10 万行的配置会额外复制 20 万段文本。另一种做法是只记录“键和值在原文本中的位置”——键和值都是指向原文本的片段，不复制任何字符。

这种“只借用原数据、不复制文本”的做法叫**零复制（zero-copy，只保存指向原数据的引用，不复制内容）**。

第二种做法会遇到一个新问题：这些片段指向的文本什么时候会被销毁？如果目录还在，原文本却已经被清理，片段就会指向一块不再属于它的内存，这叫**悬垂引用（dangling reference，指向已经失效数据的引用）**。Rust 用**生命周期（lifetime）**在编译期（compile time，把源代码翻译成可执行程序的阶段）证明这种错误不会发生。

本课就从这个具体问题出发，学习生命周期标注、生命周期省略规则，以及如何设计一个安全、好用的借用式 API（API，application programming interface，程序对外提供的调用接口）。

## 本课定位

### 你将学会

- 用自己的话解释生命周期是什么，以及它为什么不是“让值活得更久”的指令。
- 读懂 `fn longest<'a>(left: &'a str, right: &'a str) -> &'a str` 这类签名。
- 判断什么时候必须写生命周期参数，什么时候可以省略。
- 定义并实现保存引用的 `struct`，例如 `Record<'a>` 和 `Catalog<'a>`。
- 区分 `&'static str`、`T: 'static` 和“这个值永远不会被销毁”。
- 识别“返回局部变量引用”这种无论怎么加标注都无法成立的接口。
- 在“返回借用”和“返回所有权”之间做有依据的选择。

### 前置知识

- **所有权与借用**：见[第三课](../03-ownership-borrowing/README.md)。简单复习：每份数据有一个所有者，所有者负责在离开作用域（scope，变量有效的那段程序范围）时清理数据；`&T` 是“借来看”，可以有多个；`&mut T` 是“借来改”，同一时间只能有一个；引用不能比它指向的数据活得更久。
- **struct 与方法**：见[第二课](../02-data-modeling/README.md)。简单复习：结构体（struct）把几项相关数据组合成一个类型，每项数据叫字段；`impl` 块为类型定义方法，`&self` 表示方法只借用这个值来读取。
- **`Vec<T>` 与切片**：见[第五课](../05-vec-slices/README.md)。简单复习：`Vec<T>` 拥有它装着的元素，长度可以在运行中变化；`&[T]` 是对一段连续元素的只读视图，不拥有元素。
- **泛型参数**：见[第十一课](../11-generics/README.md)。简单复习：`fn identity<T>(value: T) -> T` 中的 `<T>` 是一个“类型占位符”，调用时才确定具体类型。生命周期参数也用同一对尖括号声明，只是占位的是“有效范围”而不是类型。
- **`String` 与 `&str`**：第三课和第五课都出现过。本课第一节会再完整解释一次，忘记也没关系。

### 本课边界

本课不涉及以下内容：

- 泛型生命周期中的协变、逆变等高级规则（描述生命周期之间的子类型关系如何传递）；
- `for<'a>` 这样的高阶生命周期约束（higher-ranked trait bound，简写 HRTB）；
- 异步代码中的生命周期（第十九课讲异步任务时再处理）；
- 自引用结构体、`Pin`（一种保证值在内存中不会被移动的包装类型）、`unsafe`（需要程序员额外保证安全规则的代码）等需要额外保证的机制。
- 第三方依赖。本课所有示例和练习只使用 Rust 标准库，`Cargo.toml` 的 `[dependencies]` 保持为空。

如果后面的例子必须提到上面这些高级机制，会给出当前够用的临时解释，并指出深入学习的课程。

## 如何学习本课

1. 先读第一到第九节，重点理解“生命周期是约束，不是指令”。
2. 每段带输出的代码先自己预测结果，再对照解释。
3. 第十一节的编译器诊断不要跳过。能读懂报错，比背规则更能解决实际问题。
4. 完成第十二节的代码预测，再做第十三节的主练习。
5. 主练习通过后，完成第十四节的课后巩固。
6. 卡住时按 `HINTS.md` 的三级提示逐级查看，不要直接看第三级。

预计用时：概念部分约 1.5 小时，主练习约 1 小时，课后巩固约 40 分钟。

## 一、场景：一份文本，很多片段

### 1.1 `String` 和 `&str` 分别是什么

Rust 里表示文本最常用的两个类型是 `String` 和 `&str`。

- `String` 是**拥有型**的、长度可以增长和缩短的文本。它在堆（heap，程序执行时可以动态申请的一大块内存）上保存一串 UTF-8 字节（byte，字节；UTF-8 是一种把字符编码成字节的规则），并记录长度和容量。`String` 离开作用域时，会释放自己拥有的那块内存。
- `&str` 是**借用型**的文本切片。它本身通常由两部分组成：一个指向文本开头的地址（指针，pointer）和一个字节长度。它不拥有文本，只是“借来看”。`str` 本身长度不固定，所以几乎总是出现在引用后面，写成 `&str`。

```rust
fn main() {
    let owned: String = String::from("language = rust");
    let borrowed: &str = &owned[..8];

    println!("owned: {owned}");
    println!("borrowed: {borrowed}");
}
```

输出：

```text
owned: language = rust
borrowed: language
```

`borrowed` 没有复制 `language` 这 8 个字节，它只是指向 `owned` 的缓冲区（buffer，连续存放数据的一块内存）的前 8 个字节。`owned` 被销毁后，`borrowed` 就不能再使用。第三课的借用检查器（borrow checker，编译器里负责检查引用是否安全的部分）会保证这一点。

### 1.2 第一次尝试：结构体里保存切片

假设我们想让一条配置记录保存两个切片：

```rust,compile_fail
struct Record {
    key: &str,
    value: &str,
}

fn main() {}
```

这段代码无法编译。真实的诊断是：

```text
error[E0106]: missing lifetime specifier
  |
  |     key: &str,
  |          ^ expected named lifetime parameter
  |
help: consider introducing a named lifetime parameter
  |
  | struct Record<'a> {
  |     key: &'a str,
  |
```

上面的输出省略了行号；`value` 字段会报同一条错误，处理方式相同。

编译器在问：`Record` 里的 `key` 借用的是哪份数据？它能活多久？函数参数有调用者提供的值，结构体没有。一个 `Record` 可以被创建、移动、放进 `Vec`、传给别的函数；编译器必须知道它内部引用受谁约束，才能拒绝“原文本已经销毁，`Record` 还在用”的程序。

生命周期参数（写在类型里的有效期名字）就是用来补上这条信息的。改写成：

```rust
struct Record<'a> {
    key: &'a str,
    value: &'a str,
}

fn main() {}
```

读作：存在某个有效范围 `'a`，`key` 和 `value` 都指向至少在 `'a` 内有效的文本；`Record` 本身也只能在 `'a` 内使用。

这不是让文本活得更久，而是给 `Record` 加了一条使用条件。

## 二、生命周期到底是什么

### 2.1 定义

**生命周期（lifetime）**是编译器追踪的“一个引用可以安全使用的时间范围”。它不是运行时的计时器（run time，程序真正执行的阶段），不占内存，也不会生成任何“检查还剩多久”的机器指令。它只存在于编译期的分析中。

可以把生命周期想成程序执行路径上的一段区间：

```text
程序开始 --------------------------------------------------> 程序结束
                 [==== 'a ====]
         创建 text          最后一次使用 text
```

引用 `&'a str` 的意思是：这个引用指向的数据，在 `'a` 这段区间内保证有效。

### 2.2 每个引用都有生命周期

Rust 中每个引用都有生命周期，但大多数时候不需要写出来：

```rust
fn main() {
    let text = String::from("rust");
    let view: &str = &text;
    println!("{view}");
}
```

`view` 的类型是 `&str`，编译器自动为它推导出一个生命周期：从创建 `view` 开始，到最后一次使用 `view` 为止。这里没有写 `'a`，但生命周期分析照常进行。

只有当你需要**表达多个引用之间的关系**时，才需要把生命周期写出来。

### 2.3 生命周期不是作用域

初学者常把生命周期等同于“花括号围起来的范围”。它们相关，但不完全一样。

现代 Rust 的借用通常在某次使用的**最后一次使用**之后结束，而不是等到整个花括号结束。这叫**非词法生命周期（non-lexical lifetimes，简称 NLL）**。第九节会用一个完整例子说明。

因此，更准确的说法是：生命周期是编译器推导出的、引用需要保持有效的程序区间；它可能比变量的词法作用域短。

### 2.4 术语速查

| 术语 | 中文解释 |
| --- | --- |
| 引用（reference） | 指向另一份数据的值，写作 `&T` 或 `&mut T`，不拥有数据 |
| 借用（borrow） | 通过引用暂时访问数据的行为 |
| 生命周期（lifetime） | 引用可以安全使用的编译期区间 |
| 生命周期参数（lifetime parameter） | 写在尖括号里的名字，如 `'a`，表示某个有效范围 |
| 生命周期标注（lifetime annotation） | 在类型或签名中写出 `'a` 这类名字 |
| 借用检查器（borrow checker） | 编译器里检查借用和生命周期是否合法的部分 |
| 悬垂引用（dangling reference） | 指向已经失效数据的引用 |
| 省略（elision） | 满足固定规则时，编译器自动补出生命周期 |

## 三、函数签名里的生命周期

### 3.1 一个必须标注的函数

下面这个函数接收两个字符串切片，返回其中较长的一个：

```rust
fn longest<'a>(left: &'a str, right: &'a str) -> &'a str {
    if left.len() >= right.len() {
        left
    } else {
        right
    }
}

fn main() {
    let a = String::from("rust");
    let b = String::from("borrow checker");
    println!("{}", longest(&a, &b));
}
```

输出：

```text
borrow checker
```

逐部分读签名：

- `<'a>`：声明一个生命周期参数，名字叫 `'a`。名字本身没有特殊含义，写成 `'input`、`'left` 也可以；`'a` 只是惯例。
- `left: &'a str`：`left` 这个引用至少在 `'a` 内有效。
- `right: &'a str`：`right` 这个引用至少在 `'a` 内有效。
- `-> &'a str`：返回的引用也只在 `'a` 内有效。

整体读作：对某个有效范围 `'a`，只要两个输入都至少活到 `'a`，返回的引用就保证在 `'a` 内有效。

一次具体调用时，编译器会选一个同时满足两个输入的范围。直觉上，返回值不能比两个候选输入中**较短**的那个活得更久。

### 3.2 为什么必须写

如果省略标注，写成：

```rust,compile_fail
fn longest(left: &str, right: &str) -> &str {
    if left.len() >= right.len() {
        left
    } else {
        right
    }
}

fn main() {}
```

编译器拒绝编译：

```text
error[E0106]: missing lifetime specifier
  |
  | fn longest(left: &str, right: &str) -> &str {
  |                  ----         ----     ^ expected named lifetime parameter
  |
  = help: this function's return type contains a borrowed value, but the signature does not say whether it is borrowed from `left` or `right`
```

关键在最后一句：返回值借自 `left` 还是 `right`？只看签名，编译器不知道。它不会去读函数体再猜，因为签名是调用者和实现之间的契约，调用者可能只看到签名。

生命周期标注补上的正是这条契约。

### 3.3 标注不会延长任何东西

常见误解是“加上 `'a`，返回的引用就能活得更久”。不会。看这个程序：

```rust,compile_fail
fn longest<'a>(left: &'a str, right: &'a str) -> &'a str {
    if left.len() >= right.len() {
        left
    } else {
        right
    }
}

fn main() {
    let result;
    {
        let inner = String::from("inner");
        result = longest("outer value", &inner);
    }
    println!("{result}");
}
```

它无法编译：

```text
error[E0597]: `inner` does not live long enough
  |
  |         let inner = String::from("inner");
  |             ----- binding `inner` declared here
  |         result = longest("outer value", &inner);
  |                                         ^^^^^^ borrowed value does not live long enough
  |     }
  |     - `inner` dropped here while still borrowed
  |     println!("{result}");
  |                ------ borrow later used here
```

`longest` 可能在运行时返回 `inner`，所以返回值的有效期不能超过 `inner`。`inner` 在内层花括号结束时被销毁，而 `result` 在之后还要使用，冲突。标注只是把这个限制写清楚，不能改变 `inner` 的销毁时机。

### 3.4 两个输入可以有不同的生命周期

如果返回值只可能来自第一个参数，就不必要求两个参数共享同一个生命周期：

```rust
fn first<'a, 'b>(left: &'a str, _right: &'b str) -> &'a str {
    left
}

fn main() {
    let outer = String::from("outer");
    let result;
    {
        let inner = String::from("inner");
        result = first(&outer, &inner);
    }
    println!("{result}");
}
```

这段代码可以编译，输出 `outer`。`result` 只与 `left` 的 `'a` 有关；`inner` 的 `'b` 可以更短，函数返回后 `inner` 被销毁也不影响 `result`。

`_right` 前面的下划线表示“这个参数故意不使用”，避免“未使用变量”的警告。它仍然会被借用，只是函数体不读它。

### 3.5 生命周期参数和类型参数的写法

生命周期参数和类型参数可以放在同一对尖括号里：

```rust
fn describe<'a, T>(text: &'a str, value: T) -> &'a str
where
    T: std::fmt::Display,
{
    println!("{value}");
    text
}

fn main() {
    let name = String::from("rust");
    println!("{}", describe(&name, 42));
}
```

这里 `T` 是值的类型，`'a` 是文本引用的有效范围。`value` 的生命周期与返回值无关，所以不需要和 `'a` 绑定。

惯例上，生命周期参数写在类型参数前面：`<'a, T>`。

### 3.6 可变引用

`&'a mut T` 表示“在 `'a` 内有效的、独占的可变借用”。它的生命周期规则和 `&'a T` 一样，只是额外要求同一时间没有其他借用：

```rust
fn push_mark<'a>(text: &'a mut String) -> &'a mut String {
    text.push('!');
    text
}

fn main() {
    let mut value = String::from("hello");
    let marked = push_mark(&mut value);
    println!("{marked}");
}
```

输出：

```text
hello!
```

`push_mark` 返回的引用仍然借用 `value`，所以在 `marked` 最后一次使用之前，不能再以别的方式访问 `value`。这是第三课已经学过的独占借用规则，生命周期只是把“借用持续多久”写进签名。

## 四、生命周期省略的三条规则

如果每个引用都要手写生命周期，常见代码会变得很啰嗦。Rust 定义了三条例省略规则，让编译器在能确定关系时自动补全。

这些规则只适用于**函数和方法签名**，不适用于结构体定义。结构体里出现引用时，仍然要写生命周期参数。

### 4.1 规则一：每个输入引用获得独立生命周期

概念上：

```rust
fn compare(left: &str, right: &str) {}

fn main() {}
```

被看作：

```rust
fn compare<'a, 'b>(left: &'a str, right: &'b str) {}

fn main() {}
```

两个输入各自有一个生命周期，互不绑定。因为函数不返回引用，这两个名字实际上不会被用到，所以平时完全不用写。

### 4.2 规则二：恰好一个输入引用时，输出使用它的生命周期

```rust
fn trim(text: &str) -> &str {
    text.trim()
}

fn main() {}
```

等价于：

```rust
fn trim<'a>(text: &'a str) -> &'a str {
    text.trim()
}

fn main() {}
```

只有一个输入引用，返回值只可能借自它，没有歧义。第三课写的 `first_word(text: &str) -> &str` 能工作，就是因为这条规则。

注意“恰好一个输入引用”。如果输入有两个引用，即使其中一个明显和返回值无关，规则二也不适用，需要你明确说明。

### 4.3 规则三：方法有 `&self` 或 `&mut self` 时，输出使用 `self` 的生命周期

```rust
struct Label(String);

impl Label {
    fn as_str(&self) -> &str {
        &self.0
    }
}

fn main() {
    let label = Label(String::from("rust"));
    println!("{}", label.as_str());
}
```

`as_str` 的返回引用借自 `self`。方法签名的第一个参数是 `&self`，编译器自动把输出的生命周期与 `self` 的这次借用绑定。等价的完整写法是：

```rust
struct Label(String);

impl Label {
    fn as_str<'a>(&'a self) -> &'a str {
        &self.0
    }
}

fn main() {}
```

通常不需要写这么长。

### 4.4 三条规则不够时

如果应用规则后仍不能确定输出来自哪里，编译器要求显式标注。例如两个输入引用、返回一个引用，且没有 `self`：

```rust,compile_fail
fn choose(first: &str, second: &str) -> &str {
    if first.is_empty() { second } else { first }
}

fn main() {}
```

规则一给两个输入各自的生命周期，规则二不适用（不是恰好一个输入），规则三不适用（不是方法），于是编译器报 `missing lifetime specifier`。

### 4.5 `'_` 占位符

有时你希望明确写出“这里有一个生命周期，但具体是什么由编译器推断”，可以写 `'_`：

```rust
struct Record<'a> {
    key: &'a str,
    value: &'a str,
}

fn parse(line: &str) -> Option<Record<'_>> {
    let (key, value) = line.split_once('=')?;
    Some(Record {
        key: key.trim(),
        value: value.trim(),
    })
}

fn main() {
    let line = String::from("language=rust");
    let record = parse(&line).unwrap();
    println!("{} -> {}", record.key, record.value);
}
```

输出：

```text
language -> rust
```

`Record<'_>` 的意思是“这里有一个 `Record`，它的生命周期参数由编译器根据输入推断”。在返回位置，`'_` 通常等价于省略写法。

`'_` 不是“任意长的生命周期”，也不是 `'static`。它只是把推断工作交给编译器。

## 五、保存引用的结构体

### 5.1 一个完整例子

```rust
#[derive(Debug)]
struct Excerpt<'a> {
    source: &'a str,
}

impl<'a> Excerpt<'a> {
    fn new(source: &'a str) -> Self {
        Self { source }
    }

    fn text(&self) -> &str {
        self.source
    }

    fn contains(&self, needle: &str) -> bool {
        self.source.contains(needle)
    }
}

fn main() {
    let document = String::from("Rust prevents dangling references.");
    let excerpt = Excerpt::new(&document);

    println!("{:?}", excerpt);
    println!("{}", excerpt.text());
    println!("{}", excerpt.contains("dangling"));
}
```

输出：

```text
Excerpt { source: "Rust prevents dangling references." }
Rust prevents dangling references.
true
```

逐部分解释：

- `struct Excerpt<'a>`：声明一个生命周期参数。
- `source: &'a str`：`Excerpt` 里保存一个文本切片，这个切片至少在 `'a` 内有效。
- `impl<'a> Excerpt<'a>`：为所有 `Excerpt<'a>` 实现方法，所以 `impl` 后面也要声明 `'a`。
- `fn new(source: &'a str) -> Self`：传入的文本必须至少和 `Excerpt` 的使用范围一样长。
- `fn text(&self) -> &str`：按规则三，返回值绑定到 `&self`，而不是原始 `'a`。这已经够用，也更灵活。
- `fn contains(&self, needle: &str) -> bool`：`needle` 只在调用期间读取，不保存，也不返回，所以不需要写成 `&'a str`。

### 5.2 `Excerpt<'a>` 的含义

`Excerpt<'a>` 不是“一个活 `'a` 那么久的对象”，而是“一个内部保存了 `&'a str` 的对象”。它带来两条约束：

1. 创建 `Excerpt` 时，被引用的文本必须至少活到 `'a`。
2. `Excerpt` 本身不能在 `'a` 结束后继续使用。

如果违反第二条，编译器会报错：

```rust,compile_fail
#[derive(Debug)]
struct Excerpt<'a> {
    source: &'a str,
}

fn main() {
    let excerpt;
    {
        let document = String::from("short lived");
        excerpt = Excerpt { source: &document };
    }
    println!("{:?}", excerpt);
}
```

真实诊断：

```text
error[E0597]: `document` does not live long enough
  |
  |         excerpt = Excerpt { source: &document };
  |                                   ^^^^^^^^^ borrowed value does not live long enough
  |     }
  |     - `document` dropped here while still borrowed
  |     println!("{:?}", excerpt);
  |                     ------- borrow later used here
```

标注不能让 `document` 自动延长到外层。正确做法是让 `document` 活得和 `excerpt` 一样久，或者让 `Excerpt` 拥有自己的 `String`。

### 5.3 拥有型版本

如果希望摘录独立于原始文本存在，就把字段改成 `String`：

```rust
#[derive(Debug)]
struct OwnedExcerpt {
    source: String,
}

fn main() {
    let excerpt = {
        let document = String::from("short lived");
        OwnedExcerpt {
            source: document,
        }
    };

    println!("{:?}", excerpt);
}
```

输出：

```text
OwnedExcerpt { source: "short lived" }
```

`OwnedExcerpt` 没有生命周期参数，因为它不借用外部文本；它拥有自己的缓冲区。创建时把 `document` 的所有权移进结构体，没有复制文本，换来的是独立生命周期。这是本课后面会反复讨论的 API 取舍。

### 5.4 两个字段共享一个 `'a` 意味着什么

```rust
struct Pair<'a> {
    left: &'a str,
    right: &'a str,
}

fn main() {}
```

这表示 `left` 和 `right` 都至少在 `'a` 内有效，`Pair` 可以在 `'a` 内使用。它**不**表示两个文本必须同时创建、同时销毁。编译器只需要找到一个两者都有效的共同范围。

例如：

```rust
struct Pair<'a> {
    left: &'a str,
    right: &'a str,
}

fn main() {
    let long = String::from("long lived");
    let pair;
    {
        let short = String::from("short");
        pair = Pair {
            left: &long,
            right: &short,
        };
        println!("{} {}", pair.left, pair.right);
    }
    let _ = &long;
}
```

`pair` 只在 `short` 还活着的内层使用，所以 `'a` 被推断为内层那段共同范围。`long` 本身可以活得更久，不需要被缩短。

## 六、多个生命周期让接口更精确

### 6.1 返回值只来自结构体内部

假设一个文档对象持有一段长期文本，某个方法接收一个短期前缀，但要返回文档内部的文本：

```rust
struct Document<'a> {
    body: &'a str,
}

impl<'a> Document<'a> {
    fn after_prefix<'b>(&self, prefix: &'b str) -> Option<&'a str> {
        self.body.strip_prefix(prefix)
    }
}

fn main() {
    let body = String::from("chapter: rust lifetimes");
    let document = Document { body: &body };

    let result = {
        let prefix = String::from("chapter: ");
        document.after_prefix(&prefix)
    };

    println!("{:?}", result);
}
```

输出：

```text
Some("rust lifetimes")
```

`after_prefix` 的签名说：

- `&self`：调用期间借用文档；
- `prefix: &'b str`：前缀只需要在调用期间有效，`'b` 与文档的生命周期无关；
- `-> Option<&'a str>`：返回的文本来自 `self.body`，有效范围和原始 `body` 一样长。

`prefix` 在内层花括号结束时被销毁，但 `result` 仍然可以使用，因为 `result` 指向的是 `body`，不是 `prefix`。

如果按规则三省略成 `fn after_prefix(&self, prefix: &str) -> Option<&str>`，返回值会被绑定到 `&self` 的这次借用。这通常也够用，但不如 `&'a str` 精确：调用者拿到结果后，不能把文档的借用结束得太早。两种写法都是安全的，区别是接口承诺的宽松程度。

### 6.2 把无关参数绑到 `'a` 会限制调用者

下面是一个过度约束的目录：

```rust,compile_fail
struct Catalog<'a> {
    records: Vec<(&'a str, &'a str)>,
}

impl<'a> Catalog<'a> {
    fn get(&self, key: &'a str) -> Option<&'a str> {
        self.records
            .iter()
            .find(|(stored, _)| *stored == key)
            .map(|(_, value)| *value)
    }
}

fn main() {
    let source = String::from("language=rust");
    let catalog = Catalog {
        records: vec![("language", &source[9..])],
    };
    let value = {
        let query = String::from("language");
        catalog.get(&query)
    };
    println!("{:?}", value);
}
```

这里 `get` 把 `key` 也写成了 `&'a str`。编译器要求查询字符串至少和目录里的原始文本活得一样久，于是短命的 `query` 无法调用：

```text
error[E0597]: `query` does not live long enough
  |
  |     let value = {
  |         ----- borrow later stored here
  |         let query = String::from("language");
  |             ----- binding `query` declared here
  |         catalog.get(&query)
  |                     ^^^^^^ borrowed value does not live long enough
  |     };
  |     - `query` dropped here while still borrowed
```

修正方向不是延长 `query`，而是把 `key` 的生命周期与 `'a` 解耦：

```rust
struct Catalog<'a> {
    records: Vec<(&'a str, &'a str)>,
}

impl<'a> Catalog<'a> {
    fn get(&self, key: &str) -> Option<&str> {
        self.records
            .iter()
            .find(|(stored, _)| *stored == key)
            .map(|(_, value)| *value)
    }
}

fn main() {
    let source = String::from("language=rust");
    let catalog = Catalog {
        records: vec![("language", &source[9..])],
    };
    let value = {
        let query = String::from("language");
        catalog.get(&query)
    };
    println!("{:?}", value);
}
```

输出：

```text
Some("rust")
```

`key: &str` 是规则一推导出的独立生命周期；返回的 `&str` 按规则三绑定到 `&self`，所以结果仍然来自目录，而不是查询参数。

### 6.3 设计原则

只给真正存在关系的引用共享生命周期：

- 返回值来自哪个输入，就把返回值的生命周期与那个输入关联；
- 只是调用期间读一下的参数，不要绑定到结构体的 `'a`；
- 保存进结构体的引用才需要结构的生命周期参数；
- 结构体方法的返回值如果来自 `self`，通常让省略规则处理即可，确实需要暴露原始 `'a` 时再显式写。

过度约束不会造成内存错误，但会让本来能编译的调用者无法调用。这是借用式 API 最常见的设计问题之一。

## 七、`'static` 不是万能延长术

`'static` 是一个特殊的生命周期名字，表示“整个程序运行期间”。它有三个常见语境，含义并不相同。

### 7.1 `&'static str`：程序全程有效的引用

字符串字面量直接嵌在程序二进制里：

```rust
fn main() {
    let name: &'static str = "rust";
    println!("{name}");
}
```

输出：

```text
rust
```

这个文本不属于任何局部变量，不会被某个作用域销毁，所以引用可以是 `'static`。

### 7.2 `T: 'static`：类型里没有短期借用

`T: 'static` 读作“类型 `T` 满足 `'static` 约束”。它的意思不是“这个值永远不会被销毁”，而是“`T` 里面不包含生命周期短于 `'static` 的借用数据”。

```rust
fn require_static<T: 'static>(value: T) {
    drop(value);
}

fn main() {
    let owned = String::from("owned locally");
    require_static(owned);
    println!("done");
}
```

输出：

```text
done
```

局部 `String` 完全满足 `T: 'static`，因为它拥有自己的缓冲区，不借用任何局部变量。`require_static` 仍然可以在函数里立刻 `drop` 它。

对比之下，`&'a str` 只有在 `'a` 是 `'static` 时才满足 `T: 'static`。一个借用局部 `String` 的 `&str` 不满足。

### 7.3 `Box::leak`：故意泄漏得到的 `'static`

`Box::leak` 可以把堆上的值“漏”掉，永远不释放，从而得到一个 `'static` 引用：

```rust
fn main() {
    let leaked: &'static mut str = Box::leak(String::from("leaked").into_boxed_str());
    println!("{leaked}");
}
```

输出：

```text
leaked
```

这段程序能运行，但内存直到进程结束都不会回收。`Box::leak` 有合法用途，例如进程级只初始化一次的数据；它不是解决普通生命周期错误的办法。每次遇到“活得不够久”就泄漏，会把一个编译期问题变成内存泄漏（memory leak，程序不再使用、也没有释放的内存）。

### 7.4 不要把 `'static` 当快捷键

给所有引用都加 `'static`，通常只是把错误推到调用处：调用者现在必须提供能活到程序结束的数据，而大多数局部数据做不到。生命周期错误应该通过调整数据所有权、缩短借用范围或返回拥有型值来解决，而不是升级成 `'static`。

## 八、不能返回局部值的引用

### 8.1 一个无论怎么标注都不成立的函数

```rust,compile_fail
fn build_name<'a>() -> &'a str {
    let name = String::from("local");
    &name
}

fn main() {
    println!("{}", build_name());
}
```

真实诊断：

```text
error[E0515]: cannot return reference to local variable `name`
  |
  |     &name
  |     ^^^^^ returns a reference to data owned by the current function
```

`name` 是函数里的局部变量，函数返回时它就被销毁。返回的引用会立刻悬垂。签名里的 `'a` 由调用者选择，调用者完全可以选择一个很长的范围，但函数没有任何输入能保证 `name` 活那么久。

生命周期标注描述已有数据的有效范围，不能创造存储。

### 8.2 三种正确设计

**方案一：返回拥有型值。** 函数创建了新文本，就让调用者拥有它：

```rust
fn build_name() -> String {
    String::from("local")
}

fn main() {
    let name = build_name();
    println!("{name}");
}
```

输出：

```text
local
```

**方案二：让调用者提供文本，返回其中的切片。** 这样返回值的来源是调用者的数据，生命周期可以正确关联：

```rust
fn first_word(text: &str) -> &str {
    text.split_whitespace().next().unwrap_or("")
}

fn main() {
    let text = String::from("rust lifetimes");
    let word = first_word(&text);
    println!("{word}");
}
```

输出：

```text
rust
```

**方案三：让调用者传入可修改的缓冲区。** 如果函数需要把新内容写进已有存储，可以接收 `&mut String`，写完后返回一个借用：

```rust
fn fill_name(buffer: &mut String) -> &str {
    buffer.push_str("local");
    buffer.as_str()
}

fn main() {
    let mut buffer = String::new();
    let name = fill_name(&mut buffer);
    println!("{name}");
}
```

输出：

```text
local
```

具体选哪种，取决于“谁应该拥有结果”。函数内部新建的数据，通常应该返回所有权；结果本来就是输入的一部分，才适合返回借用。

## 九、借用什么时候结束：NLL 与重新借用

### 9.1 最后一次使用后借用结束

```rust
fn main() {
    let mut text = String::from("rust");
    let view = &text;
    println!("{view}");

    text.push_str(" language");
    println!("{text}");
}
```

输出：

```text
rust
rust language
```

`view` 是共享借用。按旧式的“作用域结束才释放借用”理解，`text.push_str` 会与 `view` 冲突；但 `view` 的最后一次使用是 `println!`，之后借用就结束了，所以后面可以可变借用 `text`。这就是 NLL。

### 9.2 可变引用的重新借用

```rust
fn append_mark(text: &mut String) {
    text.push('!');
}

fn main() {
    let mut value = String::from("hello");
    let borrowed = &mut value;

    append_mark(borrowed);
    append_mark(borrowed);

    println!("{value}");
}
```

输出：

```text
hello!!
```

每次把 `borrowed` 传给 `append_mark`，编译器都会临时重新借用 `*borrowed`，而不是永久移动这个 `&mut String`。因此同一个可变引用可以连续传给函数多次。

### 9.3 对排错的启示

很多“借活得不够久”的错误，并不是因为缺少生命周期标注，而是因为借用开始得太早、结束得太晚。常见调整方式：

- 把引用的创建推迟到真正需要使用之前；
- 缩小引用变量的作用域；
- 在最后一次使用后不再持有引用；
- 让函数返回拥有型值，而不是借用局部数据；
- 重新考虑数据应该由谁拥有。

先判断“哪个值活得太短”或“哪个借用持续得太久”，再决定是否真的需要写生命周期参数。

## 十、完整示例：借用式摘录卡片

下面是一个完整、可运行的程序。它不依赖本课练习中的任何类型。

```rust
#[derive(Debug)]
struct Excerpt<'a> {
    source: &'a str,
}

impl<'a> Excerpt<'a> {
    fn new(source: &'a str) -> Self {
        Self { source }
    }

    fn first_sentence(&self) -> &str {
        match self.source.split_once('.') {
            Some((sentence, _rest)) => sentence,
            None => self.source,
        }
    }

    fn starts_with(&self, prefix: &str) -> bool {
        self.source.starts_with(prefix)
    }

    fn source(&self) -> &'a str {
        self.source
    }
}

fn longest<'a>(left: &'a str, right: &'a str) -> &'a str {
    if left.len() >= right.len() {
        left
    } else {
        right
    }
}

fn main() {
    let document =
        String::from("Rust checks borrows at compile time. It adds no runtime cost.");
    let excerpt = Excerpt::new(&document);

    println!("first sentence: {}", excerpt.first_sentence());
    println!("starts with Rust: {}", excerpt.starts_with("Rust"));
    println!(
        "word count: {}",
        excerpt.source().split_whitespace().count()
    );

    let other = String::from("short");
    println!("longer: {}", longest(excerpt.source(), &other));
}
```

输出：

```text
first sentence: Rust checks borrows at compile time
starts with Rust: true
word count: 11
longer: Rust checks borrows at compile time. It adds no runtime cost.
```

数据和类型的变化过程：

1. `document` 是一个局部 `String`，拥有整段文本的缓冲区。
2. `Excerpt::new(&document)` 接收一个借用，`'a` 被推断为 `document` 有效的范围。`excerpt` 内部只保存指针和长度，不复制文本。
3. `first_sentence` 按规则三返回 `&self` 绑定的切片，实际指向 `document` 中第一个 `.` 之前的部分。
4. `starts_with` 只返回 `bool`；`prefix` 只在调用期间读取，不需要生命周期参数。
5. `source` 返回 `&'a str`，而不是 `&self` 绑定的更短生命周期。所以即使 `excerpt` 先被销毁，只要 `document` 还在，返回值仍然有效。
6. `longest` 接收两个 `&'a str`，返回其中一个。调用时两个参数都还活着，所以结果可以立即打印。

这个例子展示了三种常见输出：

- 输出借用 `self`（`first_sentence`）；
- 输出不借用任何东西（`starts_with`）；
- 输出借用原始文本、可以比结构体活得更久（`source`）。

主练习会把这些模式组合成一个更完整的数据结构。

## 十一、编译器诊断：五条真实错误

以下诊断来自 `rustc 1.94.0`（edition 2024）。为了便于阅读，省略了临时文件路径和行号，保留了错误码、关键源码行和帮助信息。

### 11.1 结构体缺少生命周期参数

```rust,compile_fail
struct Record {
    key: &str,
    value: &str,
}

fn main() {}
```

```text
error[E0106]: missing lifetime specifier
  |
  |     key: &str,
  |          ^ expected named lifetime parameter
  |
help: consider introducing a named lifetime parameter
  |
  | struct Record<'a> {
  |     key: &'a str,
  |
```

编译器还会对 `value` 字段报同一条错误，处理方式相同。

**根因**：结构体保存引用，但没说明引用能活多久。

**方向**：给结构体加生命周期参数，并给每个保存引用的字段标注同一个或不同的生命周期。

### 11.2 函数返回值来源不明确

```rust,compile_fail
fn longest(left: &str, right: &str) -> &str {
    if left.len() >= right.len() {
        left
    } else {
        right
    }
}

fn main() {}
```

```text
error[E0106]: missing lifetime specifier
  |
  | fn longest(left: &str, right: &str) -> &str {
  |                  ----         ----     ^ expected named lifetime parameter
  |
  = help: this function's return type contains a borrowed value, but the signature does not say whether it is borrowed from `left` or `right`
```

**根因**：两个输入引用，返回值可能来自任意一个，省略规则无法确定。

**方向**：声明生命周期参数，把返回值和正确的输入关联起来。

### 11.3 返回局部变量的引用

```rust,compile_fail
fn build_name<'a>() -> &'a str {
    let name = String::from("local");
    &name
}

fn main() {}
```

```text
error[E0515]: cannot return reference to local variable `name`
  |
  |     &name
  |     ^^^^^ returns a reference to data owned by the current function
```

**根因**：`name` 在函数返回时销毁，引用会悬垂。

**方向**：返回 `String`，或让调用者提供存储，或从输入参数中返回切片。加生命周期标注无法解决。

### 11.4 结构体比被引用文本活得久

```rust,compile_fail
#[derive(Debug)]
struct Excerpt<'a> {
    source: &'a str,
}

fn main() {
    let excerpt;
    {
        let document = String::from("short lived");
        excerpt = Excerpt { source: &document };
    }
    println!("{:?}", excerpt);
}
```

```text
error[E0597]: `document` does not live long enough
  |
  |         excerpt = Excerpt { source: &document };
  |                                   ^^^^^^^^^ borrowed value does not live long enough
  |     }
  |     - `document` dropped here while still borrowed
  |     println!("{:?}", excerpt);
  |                     ------- borrow later used here
```

**根因**：`excerpt` 保存了指向 `document` 的切片，但 `document` 先被销毁。

**方向**：让 `document` 活得更久，或让 `Excerpt` 拥有 `String`。

### 11.5 把查询参数错误地绑到结构体生命周期

```rust,ignore
impl<'a> Catalog<'a> {
    fn get(&self, key: &'a str) -> Option<&'a str> { /* ... */ }
}
```

这是签名片段：省略了 `Catalog` 的定义和函数体，完整可编译程序见 6.2 节。下面的诊断来自那个完整程序中用内层 `query` 调用的版本。

用内层作用域里的短命 `query` 调用：

```text
error[E0597]: `query` does not live long enough
  |
  |     let value = {
  |         ----- borrow later stored here
  |         let query = String::from("language");
  |             ----- binding `query` declared here
  |         catalog.get(&query)
  |                     ^^^^^^ borrowed value does not live long enough
  |     };
  |     - `query` dropped here while still borrowed
```

**根因**：`key: &'a str` 要求查询字符串也满足结构体的 `'a`，但查询只在本次调用中需要。

**方向**：把 `key` 改成独立生命周期的 `&str`。返回值按规则三绑定到 `&self`，仍然来自目录。

## 十二、代码预测

先自己判断每段代码能否编译、输出是什么，再展开解析。

### 12.1 返回第一个参数的引用

```rust
fn first<'a, 'b>(left: &'a str, _right: &'b str) -> &'a str {
    left
}

fn main() {
    let outer = String::from("outer");
    let result;
    {
        let inner = String::from("inner");
        result = first(&outer, &inner);
    }
    println!("{result}");
}
```

这段代码能编译吗？

<details><summary>解析</summary>

能。返回值只与 `left` 的 `'a` 有关，`inner` 的 `'b` 可以在内层结束。输出 `outer`。

如果把两个参数都写成 `&'a str`，`'a` 就必须同时覆盖 `outer` 和 `inner`，`result` 会被限制在内层范围，在内层结束后使用就会报错。

</details>

### 12.2 返回两个参数中较长者的引用

```rust,ignore
fn longest<'a>(left: &'a str, right: &'a str) -> &'a str {
    if left.len() >= right.len() { left } else { right }
}

fn main() {
    let result;
    {
        let inner = String::from("inner");
        result = longest("outer value", &inner);
    }
    println!("{result}");
}
```

这段代码能编译吗？

<details><summary>解析</summary>

不能。编译器不知道运行时到底返回哪个参数，只能按签名要求返回值在 `'a` 内有效；而 `'a` 必须同时覆盖两个输入，所以不能超过 `inner` 的生命周期。`inner` 在内层结束时销毁，`result` 之后仍被使用，报 `E0597`。

这不是标注写法的问题，而是接口本身承诺了“返回值可能来自任意一个输入”。如果调用者需要结果比 `inner` 活得更久，就不能把可能指向 `inner` 的引用带出去。

</details>

### 12.3 局部 `String` 满足 `T: 'static`

```rust
fn require_static<T: 'static>(value: T) {
    drop(value);
}

fn main() {
    let owned = String::from("owned locally");
    require_static(owned);
}
```

这段代码能编译吗？`owned` 会永远不销毁吗？

<details><summary>解析</summary>

能编译。`T: 'static` 约束的是类型内部不能含有短期借用；`String` 拥有自己的缓冲区，满足这个约束。`owned` 被传入后立即 `drop`，没有任何泄漏。`'static` 不等于“值永远不销毁”。

</details>

### 12.4 非词法生命周期

```rust
fn main() {
    let mut text = String::from("rust");
    let view = &text;
    println!("{view}");

    text.push_str(" language");
    println!("{text}");
}
```

这段代码能编译吗？

<details><summary>解析</summary>

能。`view` 的最后一次使用是第一个 `println!`，共享借用在那之后结束。后面的 `push_str` 需要可变借用，此时已经没有冲突。借用不必持续到整个 `main` 函数结束。

</details>

## 十三、主练习：零复制配置目录

### 13.1 目标

实现一个从单段配置文本借用内容的目录：

```rust
#[derive(Debug, PartialEq, Eq)]
pub struct Record<'a> {
    pub key: &'a str,
    pub value: &'a str,
}

pub struct Catalog<'a> {
    records: Vec<Record<'a>>,
}

fn main() {}
```

目录不复制键和值。所有记录的字段都必须指向原始 `source` 内部的字节范围。

需要实现（以下只列签名，函数体由你完成）：

```rust,ignore
pub fn parse_record(line: &str) -> Option<Record<'_>>;

impl<'a> Catalog<'a> {
    pub fn parse(source: &'a str) -> Self;
    pub fn get(&self, key: &str) -> Option<&str>;
    pub fn records(&self) -> &[Record<'a>];
    pub fn longest_value(&self) -> Option<&str>;
}
```

要修改的文件是 `lessons/14-lifetimes/src/exercise.rs`。类型定义、函数签名、文档注释和文件底部的测试已经准备好，不要修改它们，只替换 `todo!()`。

### 13.2 输入与期望输出

给定输入：

```text
  # settings
language = rust

invalid line
 = no key
editor=helix
language = oxidized
empty =
token = a=b
```

`records()` 按原顺序返回五项，重复键不去重：

```text
[
    Record { key: "language", value: "rust" },
    Record { key: "editor", value: "helix" },
    Record { key: "language", value: "oxidized" },
    Record { key: "empty", value: "" },
    Record { key: "token", value: "a=b" },
]
```

查询结果：

```text
get("language")       -> Some("rust")
get("LANGUAGE")       -> None
get("missing")        -> None
longest_value()       -> Some("oxidized")
```

### 13.3 单行解析规则

`parse_record(line)` 的行为：

1. 先去掉整行两端的空白（空格、制表符、换行等）。
2. 去掉后为空，或以 `#` 开头，返回 `None`。
3. 用第一个 `=` 把行分成键和值；没有 `=` 返回 `None`。
4. 键和值各自去掉两端空白。
5. 键为空返回 `None`；值允许为空。
6. 第二个及之后的 `=` 都属于值，不再切分。
7. 成功时返回的 `key` 和 `value` 都是 `line` 内部的连续片段，不复制文本。

示例：

| 输入 | 结果 |
| --- | --- |
| `"  language = rust  "` | `Some(Record { key: "language", value: "rust" })` |
| `"empty ="` | `Some(Record { key: "empty", value: "" })` |
| `"token = a=b"` | `Some(Record { key: "token", value: "a=b" })` |
| `" 语言 = 生锈 "` | `Some(Record { key: "语言", value: "生锈" })` |
| `""` / `"   "` | `None` |
| `"# comment"` / `"  # comment"` | `None` |
| `"missing separator"` | `None` |
| `" = no key"` | `None` |

### 13.4 目录规则

- `Catalog::parse(source)` 按行拆开 `source`，对每一行使用上面的规则；无效行跳过。
- 有效记录保持输入顺序，重复键全部保留。
- `source` 为空或只有无效行时，目录为空。
- `records()` 返回全部记录的只读切片，包括重复键。
- `get(key)` 返回第一条键完全相等的记录的值；比较区分大小写；没有匹配返回 `None`。
- `get` 的查询参数只在调用期间有效，返回的值来自目录保存的原始文本，不来自查询参数。
- `longest_value()` 返回值中 UTF-8 字节数最大的那个；并列时返回先出现的值；目录为空返回 `None`。

### 13.5 边界与错误行为

- **空值**：`"empty ="` 是有效记录，值为空字符串。
- **额外的 `=`**：`"token = a=b"` 的值是 `a=b`，不是 `a`。
- **Unicode**：`"语言 = 生锈"` 必须得到完整的 `语言` 和 `生锈`，不能按字节位置截断字符。
- **重复键**：`language=rust` 和 `language=oxidized` 都保留，`get("language")` 返回 `rust`。
- **空目录**：只有注释、空行和无效行时，`records()` 是空切片，`get` 和 `longest_value` 都返回 `None`。
- **最长值并列**：`first=aa`、`second=bb`、`third=c` 返回 `aa`。
- **字节数比较**：`accent=é` 和 `letters=zz` 中，`é` 和 `zz` 都是 2 个 UTF-8 字节，返回先出现的 `é`；如果按字符个数比较会错误地选 `zz`。
- **零复制**：测试会检查字段切片的地址确实落在原输入缓冲区中，而不只是文本内容相同。

### 13.6 约束

- `Record` 的字段必须借用原始 `source`，不能改成 `String`。
- 不得为了绕过生命周期问题调用 `Box::leak`。
- 重复键全部保留，`get` 返回第一项。
- 缺少 `=` 或空键的行忽略；空值允许。
- `longest_value` 按 UTF-8 字节数比较，长度相同时返回先出现的值。
- `get` 的查询字符串只需在调用期间有效，返回的 value 不能依赖查询字符串继续存在。

### 13.7 推荐完成顺序

1. `parse_record`：先用测试里的单行输入验证。
2. `Catalog::parse` 和 `records`：验证顺序、过滤和空目录。
3. `get`：验证重复键、大小写和查询字符串的独立生命周期。
4. `longest_value`：验证字节长度和并列规则。
5. 运行完整测试和 Clippy，再运行演示程序看输出。

### 13.8 测试说明与验收

主练习共有 13 个测试：

- 2 个概念测试不依赖待实现代码，初始即应通过：
  - `record_literal_borrows_source_without_copying`
  - `owned_string_satisfies_static_bound`
- 11 个行为测试在对应 `todo!()` 处失败，完成实现后应全部通过：
  - `parse_record_trims_key_and_value`
  - `parse_record_rejects_blank_comment_and_invalid_lines`
  - `parse_record_allows_empty_value_and_later_equals_signs`
  - `parse_record_preserves_unicode_content`
  - `catalog_filters_invalid_lines_and_preserves_order`
  - `catalog_keeps_duplicate_keys_and_get_returns_the_first`
  - `get_query_does_not_need_to_outlive_the_catalog`
  - `empty_catalog_has_no_records_or_values`
  - `longest_value_compares_utf8_byte_length`
  - `longest_value_returns_first_on_tie`
  - `record_fields_are_slices_of_original_source`

实现前运行行为测试时，失败来自 `todo!()` 的 panic，不是测试自身写错。脚手架必须能通过编译：

```bash
cargo test -p l14_lifetimes --no-run
```

完成后的验收命令：

```bash
cargo test -p l14_lifetimes exercise
cargo test -p l14_lifetimes
cargo run -p l14_lifetimes
cargo clippy -p l14_lifetimes --all-targets -- -D warnings
```

完成主练习后，`cargo run -p l14_lifetimes` 的输出应为：

```text
records: [Record { key: "language", value: "rust" }, Record { key: "editor", value: "helix" }, Record { key: "language", value: "oxidized" }, Record { key: "empty", value: "" }, Record { key: "token", value: "a=b" }]
language: Some("rust")
missing: None
longest value: Some("oxidized")
token -> a=b
```

### 13.9 编译期验收

下面的代码必须无法编译，因为 `source` 销毁后，`catalog` 仍保存指向它的切片。这是人工检查，不要为了让它通过而泄漏内存：

```rust,compile_fail
use l14_lifetimes::exercise::Catalog;

fn main() {
    let catalog;
    {
        let source = String::from("language=rust");
        catalog = Catalog::parse(&source);
    }
    println!("{:?}", catalog.records());
}
```

它应该报 `E0597`，关键信息是 `source does not live long enough`。如果你的实现把 `Record` 改成拥有型 `String`，这段代码会通过编译，但主练习的零复制要求就没有达到。

### 13.10 完成后应能解释

- 为什么 `Catalog<'a>` 不能比 `source` 活得更久？
- `get` 的 `key` 参数为什么不应绑定到 `'a`？当前签名让返回值借自哪里？
- `records` 和 `longest_value` 的返回引用分别来自哪里？为什么不需要借用调用方的其他临时参数？
- 如果希望目录独立于源文本存在，`Record` 和 `Catalog` 应如何改成拥有型设计？

## 十四、课后巩固：借用的单词索引

### 14.1 目标

主练习处理“键 = 值”的配置行；这个任务处理一段普通文本，建立借用的单词索引。数据不同，边界也不同：标点、Unicode 字符、没有任何单词的输入。

要修改的文件是 `lessons/14-lifetimes/src/consolidation.rs`。同样只替换 `todo!()`，不要修改类型定义、方法签名或测试。

需要实现（以下只列签名，函数体由你完成）：

```rust,ignore
pub struct WordIndex<'a> {
    text: &'a str,
}

impl<'a> WordIndex<'a> {
    pub fn new(text: &'a str) -> Self;
    pub fn source(&self) -> &'a str;
    pub fn word_count(&self) -> usize;
    pub fn first_word(&self) -> Option<&'a str>;
    pub fn last_word(&self) -> Option<&'a str>;
    pub fn longest_word(&self) -> Option<&'a str>;
    pub fn count(&self, word: &str) -> usize;
    pub fn first_with_prefix<'b>(&self, prefix: &'b str) -> Option<&'a str>;
}
```

### 14.2 单词的定义

把文本按空白字符（空格、换行、制表符等）切开，再删除每个片段两端属于下面集合的字符：

```text
. , ; : ! ? " ' ( ) [ ] ， 。 ； ： ！ ？ “ ” ‘ ’
```

删除后为空的片段不算单词。单词内部出现的标点保留，例如 `don't` 中间的撇号不会删除。

### 14.3 输入与期望输出

输入：

```text
Rust, rust! 生锈。 Rust?
```

切分并去标点后的单词：

```text
Rust
rust
生锈
Rust
```

结果：

```text
word_count()                   -> 4
first_word()                   -> Some("Rust")
last_word()                    -> Some("Rust")
count("Rust")                  -> 2
count("rust")                  -> 1
count("rust,")                 -> 0
longest_word()                 -> Some("Rust")
first_with_prefix("ru")        -> Some("rust")
first_with_prefix("Ru")        -> Some("Rust")
first_with_prefix("x")         -> None
```

`longest_word` 按字符个数比较：`Rust` 和 `rust` 都是 4 个字符，`生锈` 是 2 个字符，所以并列时返回先出现的 `Rust`。

这里的“字符”指 Rust 的 `char`，也就是一个 Unicode 标量值：一个汉字通常是 1 个 `char`，但在 UTF-8 中占 3 个字节。按字符数比较更接近“人看到的长度”，按字节数比较更接近“实际占用的存储空间”。两种规则没有绝对优劣，关键是接口把规则写清楚。

### 14.4 方法规则

- `new(text)`：保存对 `text` 的借用，不复制文本。
- `source()`：返回完整的原始文本。返回值的有效期跟随原始文本，而不是索引本身；索引销毁后，只要原文本还在，返回值仍可使用。
- `word_count()`：单词个数；空文本、只有空白或只有标点的文本返回 `0`。
- `first_word()` / `last_word()`：按文本顺序的第一个/最后一个单词；没有单词返回 `None`。
- `longest_word()`：字符个数最多的单词；并列返回先出现的；没有单词返回 `None`。
- `count(word)`：统计与 `word` 完全相等（区分大小写）的单词个数。`word` 本身不清洗标点，查询 `"rust,"` 不会匹配文本中的 `rust`。
- `first_with_prefix(prefix)`：第一个以 `prefix` 开头（区分大小写）的单词；空前缀匹配第一个单词；没有单词或没有匹配返回 `None`。
- `prefix` 只在调用期间有效，返回的单词来自原始文本，不来自 `prefix`。

### 14.5 边界与错误行为

- 空文本：所有查询返回空结果，`word_count` 为 0。
- 只有空白和标点：没有单词。
- 中文标点：`语言，` 要得到 `语言`，`“引用”` 要得到 `引用`。
- 字符数比较：`"é zz aa"` 返回 `zz`，因为 `é` 是 1 个字符、`zz` 和 `aa` 是 2 个字符，并列取先出现的 `zz`。如果按 UTF-8 字节数比较，会错误地先选 `é`（2 字节）。
- 字符数比较（中文）：`"生锈 abc"` 返回 `abc`（3 个字符），而不是 `生锈`（2 个字符，但 6 个字节）。
- 空前缀：`"alpha beta"` 的 `first_with_prefix("")` 返回 `Some("alpha")`。
- 大小写：`"Rust"` 和 `"rust"` 是不同单词；前缀 `"Ap"` 不匹配 `apple`。

### 14.6 测试说明与验收

课后巩固共有 10 个测试，全部只依赖待实现的方法，初始会因 `todo!()` 失败，完成后应全部通过：

- `empty_text_has_no_words`
- `whitespace_and_punctuation_only_text_has_no_words`
- `words_are_split_on_whitespace_and_trimmed_at_both_ends`
- `unicode_words_and_chinese_punctuation_are_handled`
- `count_matches_whole_words_case_sensitively`
- `longest_word_uses_character_count_and_first_on_tie`
- `first_with_prefix_returns_first_match`
- `empty_prefix_matches_first_word_and_missing_prefix_returns_none`
- `source_can_outlive_the_index`
- `returned_words_borrow_the_original_text`

验收命令：

```bash
cargo test -p l14_lifetimes consolidation
cargo clippy -p l14_lifetimes --all-targets -- -D warnings
```

### 14.7 完成后应能解释

- `source()` 为什么返回 `&'a str`，而不是让省略规则写成 `-> &str`？
- `first_with_prefix` 的 `prefix` 为什么可以有独立的 `'b`？
- 为什么 `longest_word` 按字符数比较，而主练习的 `longest_value` 按字节数比较？两种规则分别在什么场景更合理？
- 如果希望单词索引在原始文本销毁后仍然存在，结构体应该怎么改？会付出什么代价？

## 十五、常见误区

### 15.1 “加上 `'a` 就能让引用活得更久”

生命周期标注只是描述约束，不改变值的创建和销毁时机。`&'a str` 不会把局部 `String` 搬到堆上，也不会延迟 `drop`。返回局部变量引用时，加什么标注都无效。

### 15.2 “两个参数都写 `'a`，表示它们必须同生共死”

`left: &'a str` 和 `right: &'a str` 表示两者都至少在 `'a` 内有效。它们可以有不同的创建和销毁时间，只要存在一个共同的有效区间。真正的问题是把无关参数绑到同一个 `'a`，可能不必要地限制调用者。

### 15.3 “所有生命周期错误都要加标注”

很多错误来自借用持续得太久，而不是缺少标注。先检查能否缩小引用作用域、推迟借用创建、提前结束使用，或改成返回拥有型值。添加更多 `'a` 往往只是把错误推到别处。

### 15.4 “`'static` 是万能解”

`'static` 要求数据在整个程序运行期间有效。局部 `String` 的借用通常不满足。`Box::leak` 可以制造 `'static`，但会永久占用内存，不是普通错误的修复方案。

### 15.5 “结构体里存 `&str` 总比 `String` 好”

存 `&str` 避免复制，但会让结构体依赖外部缓冲区，并把生命周期参数传播到所有使用它的类型和函数。如果数据需要独立保存、跨线程传递或长期存在，拥有型 `String` 往往更简单。选择标准是所有权关系，不是“零复制一定更高级”。

### 15.6 “返回引用总是更高效”

返回引用避免了复制，但会增加 API 的约束。调用者可能被迫延长输入的生命周期，或者无法把结果带出当前作用域。数据很小、复制成本可接受时，返回拥有型值可能让整体设计更清晰。

### 15.7 “生命周期是运行时机制”

生命周期只存在于编译期。程序运行时没有“生命周期检查器”，也没有为每个引用保存计时器。它不会带来运行时开销。

### 15.8 “结构体可以同时保存 `String` 和指向该 `String` 的 `&str`”

这会产生自引用结构体：结构体内部的引用指向同一个结构体的另一个字段。结构体被移动时，字段地址可能改变，引用就会失效。安全 Rust 很难直接表达这种关系。常见替代方案是保存索引、字节范围，或使用专门的自引用抽象；本课不展开。

## 十六、本课总结

- 生命周期是编译器追踪的引用有效区间，不是运行时资源，也不分配内存。
- 标注描述引用之间的约束，不会延长任何值的生命。
- 多个输入引用、返回一个引用且来源不明确时，需要显式生命周期。
- 三条省略规则覆盖了大多数单输入函数和 `&self` 方法。
- 保存引用的结构体必须带生命周期参数，实例不能比被引用数据活得更久。
- 结构体方法中，只有真正与 `self` 数据相关的返回值才需要绑定生命周期；临时参数不要强行绑到 `'a`。
- `&'static str`、`T: 'static` 和“值永不销毁”是三件不同的事。
- 返回局部变量引用无法靠标注修复；应返回所有权、从输入借用，或使用调用者提供的缓冲区。
- 借用通常在最后一次使用后结束（NLL），排错时先考虑缩短借用，而不是堆叠标注。
- 借用式 API 的收益是零复制，代价是生命周期约束会传播；设计时要明确谁拥有数据、结果需要活多久。

## 十七、课后复盘

用自己的话回答，不需要写代码：

1. 为什么 `fn longest(left: &str, right: &str) -> &str` 不能编译，而 `fn first_word(text: &str) -> &str` 可以？
2. `struct Excerpt<'a>` 中的 `'a` 约束了谁？它会不会改变被引用文本的销毁时间？
3. 为什么 `T: 'static` 允许传入局部 `String`？这和 `&'static str` 有什么区别？
4. 一个方法返回 `&str`，什么情况下应该让它按省略规则绑定 `&self`，什么情况下应该显式返回 `&'a str`？
5. 遇到 `does not live long enough` 时，你会按什么顺序排查：延长数据、缩短借用、改所有权，还是加标注？

## 下一课：共享所有权与内部可变性

本课的目录和单词索引都借用外部文本：只要原文本被销毁，索引就不能再用。这在一段程序内很清楚，但当多个部分都想长期共享同一份数据时，单一所有者和借用就不够方便了。

下一课[第十五课：`Rc<T>`、`Weak<T>`、`RefCell<T>` 与内部可变性](../15-smart-pointers/README.md)会学习：

- `Rc<T>` 如何用引用计数实现共享所有权；
- `Weak<T>` 如何表达“只观察、不负责保活”的关系，并打破循环引用；
- `RefCell<T>` 如何把借用检查从编译期移动到运行时，以及它带来的 panic 风险（panic，程序遇到无法继续的错误时立即中止当前线程的机制）。

到那时你会看到：共享所有权并不是免费的，它用运行时计数和更宽松的检查换取了灵活性；而本课学到的“谁拥有数据、引用能活多久”仍然是判断设计是否合理的核心问题。
