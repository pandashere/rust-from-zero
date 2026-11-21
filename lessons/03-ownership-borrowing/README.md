# 第三课：所有权、借用、`String` 与 slice

前两课我们学会了把数据装进 struct（结构体）和 enum（枚举），并给它们写方法。但有一个问题一直被悄悄绕开了：程序里的数据存放在内存中，内存是有限的，**用完的数据谁来清理、什么时候清理？**

Rust 给出的答案叫**所有权（ownership）**：一套在编译期就检查完毕的规则，决定了每份数据由哪段代码负责清理。所有权以及配套的**借用（borrowing）**机制是 Rust 最核心的设计，本课也是整个课程最重要的一课。

本课内容较多，建议分成两次完成：第一次学所有权、move、`Copy` 与函数传参；第二次学引用、借用、`String`、`&str` 与 slice。目标不是背下所有规则，而是能用规则预测一段代码能否编译、为什么。

## 本课定位

### 你将学会

- 理解所有权的三条核心规则，以及它要解决的问题。
- 区分语义上的 move（所有权转移）与真正的数据复制。
- 判断常见类型是否实现 `Copy`（复制语义）。
- 理解显式 `clone()` 的含义和成本。
- 分析按值传参和返回值时所有权如何流动。
- 使用 `&T` 进行共享借用，使用 `&mut T` 进行独占的可变借用。
- 运用“多个读者或一个写者”的规则预判借用冲突。
- 理解借用在最后一次使用后结束的行为（非词法生命周期）。
- 区分 `String`、`str`、`&str` 与字符串字面量。
- 使用字符串 slice（切片）与普通 slice，理解它们由“指针加长度”构成。
- 为函数参数和返回值选择“拥有型”或“借用型”的签名。
- 阅读 use-after-move、借用冲突、悬垂引用等常见编译器诊断。

### 前置知识

- 变量绑定、作用域、函数、tuple（元组）：见[第一课](../01-basics/README.md)。简单复习：`let x = 1;` 把值 `1` 绑定到名字 `x`，这个名字从声明处活到所在大括号结束，这段范围叫作用域。
- struct、enum、方法、`self`、`&self`、`&mut self`：见[第二课](../02-data-modeling/README.md)。简单复习：方法是定义在 `impl` 块里、第一个参数为某种 `self` 的函数；`&self` 表示方法只借来看，`&mut self` 表示方法要借来改。
- 基础类型（整数、`bool`、`char`）和 `#[derive(...)]` 的初步使用。

### 本课边界

本课不系统讲显式生命周期标注（lifetime annotation，第 14 课）、`Rc`/`RefCell` 等智能指针（第 15 课）、trait object 或并发。本课只需要接受一条直觉：**引用必须始终指向还活着的数据**，编译器会强制执行这一点，复杂写法留到后面。

## 如何学习本课

1. 先掌握 move 与 `Copy`，再去看借用报错，不要颠倒顺序。
2. 每读一段代码，就在纸上标出“此刻谁拥有这个值”。
3. 学借用时，标出每个引用从创建到**最后一次使用**的范围。
4. 对 `String` 始终区分“栈上的描述信息”和“堆上的字符内容”。
5. 先完成「代码预测」的自测，再动手做 `Note` 主练习。

## 一、从一个具体问题说起：谁来清理内存

程序运行时，数据都放在**内存**（memory，程序临时存放数据的地方，断电即失）里。比如一个程序不断把用户输入拼进一段文本，这段文本占用的内存就会不断增长。如果程序只管申请、从不归还，内存迟早被耗尽。

先看内存的两个区域（粗略模型，足够本课使用）：

- **栈（stack）**：每个函数调用得到的一小块空间，存放局部变量。大小在编译期就已知，函数结束时整块自动回收，速度极快。比如 `let x = 10_i32;` 中的 `x` 就在栈上。
- **堆（heap）**：一大块按需申请的区域，用于大小会变化、或需要活得比一次函数调用更久的数据。比如一段会不断变长的文本。堆上的区域不会自动回收，必须有人明确说“这块不要了”，这个动
作叫**释放（free/deallocate）**。

问题来了：堆上的数据，由谁、在什么时刻释放？

### 已有的两种办法和它们的困难

**办法一：程序员手动释放。** 程序里每申请一块堆内存，就由程序员在合适的位置写一句释放（C 语言就是这样，`malloc` 申请、`free` 释放；这只是对照，不需要你了解 C）。困难在于人总会出错：

- 忘了释放 → 内存越用越多（内存泄漏，memory leak）；
- 释放了两次 → **重复释放（double free）**，可能破坏其他数据；
- 释放后还在用 → **悬垂引用（dangling reference）**：一个引用指向的内存已经被释放，读到的是无意义或被他人占用的数据。

**办法二：垃圾回收。** 程序在运行过程中由一个专门的组件（垃圾回收器，garbage collector，简称 GC）周期性检查哪些数据再也用不到了，自动释放。Python、Java 等语言采用这种方式（同样只是对照）。代价是运行时要额外消耗时间和内存来做检查，而且释放发生的时机不可预测。

### Rust 的办法：把“谁负责释放”变成编译期规则

Rust 的思路是：释放本应是**确定无疑**的事——一份数据只有一个明确的负责人，负责人生效范围结束时立即释放。既然如此，就可以让编译器在编译期（而不是运行时）检查这些规则，不符合规则的代码直接拒绝编译。

这套规则就是**所有权**。它既不需要程序员手写释放，也不需要运行时垃圾回收。配合**借用**规则，它还在编译期挡住了重复释放、悬垂引用，以及后课会讲到的并发数据竞争。

### 1.1 所有权不等于“数据在堆上”

所有权是一套关于**责任**的规则，和数据放在哪里无关：

- 栈上的整数同样有拥有者；
- 一个 `String` 值拥有它在堆上的字符缓冲区；
- 一个引用**不**拥有它指向的数据；
- 文件对象拥有操作系统分配的文件句柄（操作系统给打开文件的编号），即使那不是普通堆内存。

所以不要把所有权理解成“Rust 管堆内存”，它是“每个值都有明确的清理负责人”。

## 二、所有权三条核心规则

先记住三条，整节课都是对它们的展开：

1. Rust 中每个值都有一个 **owner（拥有者）**：负责清理这个值的那个绑定或数据。
2. 同一时刻，一个值只有一个 owner。（少数共享所有权的智能指针，例如第 15 课的 `Rc`，会单独解释；本课其余内容都按这条规则理解。）
3. owner 离开作用域时，值会被 **drop（丢弃）**：编译器在这个位置自动插入清理逻辑——对 `String` 就是释放堆缓冲区，对文件就是关闭句柄。

```rust
{
    let text = String::from("rust");
    println!("{text}");
} // text 离开作用域，编译器在这里插入清理代码，堆上的 "rust" 被释放
```

这里 owner 是绑定 `text`。你不需要写任何释放代码，大括号结束就是释放时刻。

### 2.1 为什么“只有一个 owner”很重要

如果允许两个普通变量都声称拥有同一块堆缓冲区，它们各自离开作用域时都会去释放它——这就是重复释放。Rust 的做法是：对没有 `Copy` 语义的类型，赋值时把所有权**移交**给新绑定，旧绑定当场作废（下面第四节的 move），从根上保证只有一个人负责释放。

## 三、`String` 的内存模型

`String` 是理解 move 的典型类型，它是一份**拥有堆缓冲区、可以增长的文本**。

```rust
let text = String::from("rust");
```

概念上，一个 `String` 值由两部分组成：

```text
栈上（String 值本身，三项描述信息）      堆上（字符缓冲区）
  pointer（指向缓冲区的指针）  ───────►   r u s t
  length   = 4（已用字节数）
  capacity >= 4（缓冲区总容量，避免每次追加都重新申请）
```

**指针（pointer）**就是“记录某块内存地址的数值”。描述信息在栈上（或包含它的 struct 里），真正的字符在堆上。

这个模型是用来理解语义的，具体字段顺序和布局不是语言承诺，不要依赖它写代码。

## 四、move：转移所有权

### 4.1 `String` 赋值发生 move

```rust
let first = String::from("rust");
let second = first;

println!("{second}");
// println!("{first}"); // 取消注释会编译失败，错误输出见「错误实验」一节
```

`let second = first;` 之后，所有权从 `first` **转移（move）**到 `second`：

```text
move 前：first  ──owns──►  堆缓冲区 "rust"
move 后：second ──owns──►  堆缓冲区 "rust"
         first   失效，再用就编译错误
```

注意堆上的字符**没有被复制**。被移交的只是栈上那三项描述信息（指针、长度、容量），以及“谁负责释放”的责任。

### 4.2 为什么不能让 `first` 继续用

假如 `first`、`second` 都保存着同一个指针，又都在离开作用域时执行释放，同一块堆内存就被释放两次——正是第一节说的 double free。Rust 干脆让旧绑定失效：编译器记得 `first` 已经“被搬走”，之后任何对 `first` 的使用都是编译错误。

### 4.3 move 是语义，不要猜测机器指令

编译器可能把数据放在寄存器里、可能优化掉实际的字节复制。学习所有权时只需要问两个问题：

- 哪个绑定现在有权使用这个值？
- 谁最终负责 drop 它？

至于 CPU 是否真的复制了几个机器字（machine word，CPU 一次处理的数据单位），交给编译器。

## 五、`Copy`：简单值的隐式复制

整数的行为和 `String` 不同：

```rust
let first = 10_i32;
let second = first;

println!("{first}, {second}"); // 两个都能用
```

`i32` 实现了 **`Copy`**：一个标记，表示“这个类型的值复制起来就是逐位照抄，复制出的副本和原值完全独立”。实现了 `Copy` 的类型，赋值时自动复制，原绑定继续有效。

为什么 `String` 不能是 `Copy`？逐位照抄会复制出第二个指向同一堆缓冲区的指针，于是出现两个 owner，离开作用域时 double free。所以拥有资源的类型一律不是 `Copy`。

### 5.1 常见 `Copy` 类型

- 所有整数类型（`i32`、`u8`、`usize` 等）；
- `bool`、`char`；
- 浮点类型（`f32`、`f64`）；
- 只包含 `Copy` 元素的 tuple 和数组，例如 `(i32, bool)`、`[u8; 4]`；
- 共享引用 `&T`（对，引用本身也是值，`&T` 这个值可以随便复制）；
- 所有字段都是 `Copy`、并显式 `#[derive(Copy, Clone)]` 的 struct/enum（`Copy` 要求类型同时实现 `Clone`，所以两者要一起派生，下一小节会再说明）。

注意：可变引用 `&mut T` **不是** `Copy`。它是“独占访问”的承诺，随便复制就出两个写者了。

### 5.2 常见非 `Copy` 类型

- `String`、`Vec<T>`（动态数组，第 5 课主角）、`Box<T>`（第 13 课）；
- 文件、网络连接等资源对象；
- 含有非 `Copy` 字段的 struct/enum。

### 5.3 `Copy` 与 `Drop` 不能同时实现

需要自定义清理逻辑（实现 `Drop`）的类型不能是 `Copy`：否则每个副本都以为自己要负责清理，同一份资源会被清理多次。

### 5.4 自定义类型的 `Copy`

```rust
#[derive(Debug, Clone, Copy)]
struct Point {
    x: i32,
    y: i32,
}

let first = Point { x: 1, y: 2 };
let second = first;
println!("{first:?}, {second:?}"); // 两个都能用
```

两个字段都是 `Copy`，所以 `Point` 可以整体派生 `Copy`。（`derive` 要求同时写上 `Clone`，这是标准库的约定：`Copy` 的类型必须也能 `Clone`。）

## 六、`clone()`：显式复制拥有型数据

确实需要两份独立的 `String` 时，可以显式克隆：

```rust
let first = String::from("rust");
let second = first.clone();

println!("{first}, {second}"); // 两份互不影响
```

`clone()` 会新申请一块堆缓冲区并把字符逐个复制过去（**深复制（deep copy）**：连指向的数据一起复制，而不只是复制指针）。

### 6.1 为什么 clone 必须显式写出来

深复制要花时间和内存。要求写 `.clone()` 是为了让读代码的人一眼看到“这里发生了真实的资源复制”。

### 6.2 不要把 clone 当作报错消除器

遇到所有权或借用报错就无脑 `.clone()`，代码也许能编译，但往往掩盖了设计问题，还增加无谓的内存分配。先问三个问题：

- 函数真的需要**拥有**这个值吗？
- 只是读一读的话，能不能收一个引用？
- 调用者之后还要不要用原值？

## 七、作用域与 drop

同一个作用域里，值按声明的**逆序** drop（先声明的后清理）：

```rust
{
    let first = String::from("first");
    let second = String::from("second");
} // 先 drop second，再 drop first
```

也可以显式提前 drop：

```rust
let text = String::from("一大段文本");
drop(text);
// text 的所有权已经移进 drop()，这里再用就是编译错误
```

`drop` 不是什么特殊语法，就是一个普通函数：它收走值的所有权，函数结束时值自然被清理。

## 八、函数参数与返回值的所有权流动

### 8.1 按值参数：所有权移进函数

```rust
fn consume(text: String) {
    println!("{text}");
} // 参数 text 在这里被 drop

let text = String::from("rust");
consume(text);
// println!("{text}"); // 编译错误：text 已经移进函数了
```

`consume(text)` 把 `String` move 进参数。函数结束，参数被 drop，调用者手里的 `text` 早已失效。

### 8.2 `Copy` 参数：复制一份

```rust
fn print_number(value: i32) {
    println!("{value}");
}

let number = 10;
print_number(number);
println!("{number}"); // 仍可用，i32 是 Copy
```

### 8.3 通过返回值交还所有权

```rust
fn take_and_return(text: String) -> String {
    println!("length = {}", text.len());
    text // 把所有权交还给调用者
}

let text = String::from("rust");
let text = take_and_return(text); // 重新绑定返回值
println!("{text}");
```

这能工作，但为了读一个长度就把所有权送来送去，非常笨重。这正是**借用**要解决的问题。

## 九、引用与借用

**引用（reference）**是一个指向某个值的指针，它只提供**临时访问**，不取得所有权。**创建引用的动作叫借用（borrow）**。

```rust
fn print_length(text: &String) {
    println!("{}", text.len());
}

let text = String::from("rust");
print_length(&text); // 把 &text（text 的借用）传进去
println!("{text}");  // text 仍归这里所有，继续可用
```

- `&text` 创建一个指向 `text` 的引用；
- 参数 `text: &String` 接收这个引用；
- 函数从头到尾没碰所有权，调用结束后 `text` 照样有效。

### 9.1 引用不负责清理

引用离开作用域，只是借用关系结束，不会释放被指向的数据。真正的 owner 离开作用域时才释放。

### 9.2 解引用

`*引用` 表示“访问引用指向的那个值”，这个动作叫**解引用（dereference）**：

```rust
let value = 10;
let reference = &value;

assert_eq!(*reference, 10);
```

方法调用和字段访问会自动解引用，所以写 `reference.len()` 即可，不用写 `(*reference).len()`。

## 十、共享借用：`&T`

`&T` 叫**共享引用（shared reference）**：同一时间可以存在很多个，都能读：

```rust
let text = String::from("rust");

let first = &text;
let second = &text;

println!("{first}, {second}"); // 两个读者和平共处
```

但通过共享引用不能修改数据：

```rust
let text = String::from("rust");
let reference = &text;
// reference.push('!'); // 编译错误：共享引用背后不允许修改
```

多个读者之所以安全，是因为没有任何一方能在别人不知情的情况下改动数据。

## 十一、可变借用：`&mut T`

要修改又不转移所有权，用**可变引用（mutable reference）**：

```rust
fn append_mark(text: &mut String) {
    text.push('!');
}

let mut text = String::from("rust");
append_mark(&mut text);

println!("{text}"); // rust!
```

三处配合缺一不可：原绑定声明为 `mut`；用 `&mut text` 创建可变借用；参数类型写成 `&mut String`。

### 11.1 同一时间只能有一个可变引用

```rust
let mut text = String::from("rust");

let first = &mut text;
let second = &mut text; // 编译错误

// 如果允许 first 和 second 交替写，任何一方做的假设都可能被另一方悄悄推翻
```

### 11.2 可变引用与共享引用不能重叠

```rust
let mut text = String::from("rust");

let reader = &text;
let writer = &mut text; // 编译错误：reader 还要用呢

println!("{reader}");
writer.push('!');
```

如果允许这段代码，`writer` 的修改可能让 `reader` 读到自相矛盾的数据；对 `String` 这类内部有指针的类型，修改甚至可能让缓冲区搬家，使 `reader` 里的旧指针直接失效。

规则一句话总结：

```text
同一段时间内：要么任意多个 &T（只读），要么恰好一个 &mut T（可写），二者不可兼得
```

## 十二、借用在哪里结束：非词法生命周期

直觉上容易以为借用持续到所在大括号结束，但现代 Rust 更精确：借用持续到该引用的**最后一次使用**为止。这个行为叫**非词法生命周期（non-lexical lifetimes，NLL）**——“生命周期”指引用有效的区间，“非词法”表示它不再简单地等于大括号范围，而是由实际使用情况决定。

```rust
let mut text = String::from("rust");

let reader = &text;
println!("{reader}"); // reader 的最后一次使用，借用到这里就结束了

let writer = &mut text; // 所以可以建立可变借用
writer.push('!');
```

`reader` 这个绑定本身还活着，但它的借用已经结束，因此不冲突。分析借用冲突时，请标出每个引用的**最后一次使用**，而不是只看大括号。

## 十三、引用不能比被引用的值活得更久

```rust,compile_fail
fn dangling(input: &str) -> &str {
    let text = String::from(input);
    &text
} // text 在这里被 drop，返回的引用指向已释放的内存——悬垂引用
```

Rust 拒绝这种代码（真实诊断见「错误实验」19.4 节）。正确做法通常是返回**拥有型**的值：

```rust
fn make_text() -> String {
    String::from("rust") // 所有权移动给调用者
}
```

返回 `String` 是 move：缓冲区直接交给调用者，不会“复制一份再释放原件”。

## 十四、`String`、`str` 与 `&str`

### 14.1 `String`：拥有型文本

```rust
let mut text = String::from("rust");
text.push('!');          // 追加一个字符
text.push_str(" language"); // 追加一段文本
```

它拥有并负责释放自己的堆缓冲区，可以增长。

### 14.2 `str`：一段字符串数据本身

`str` 表示一段 UTF-8 编码的字符序列**本身**。它的长度运行时才知道，编译期无法确定大小（这类类型叫动态大小类型，dynamically sized type，DST），所以不能直接作为局部变量的类型：

```rust
// let text: str = *"rust"; // 编译错误：str 大小不固定，不能这样裸放
```

实际代码总是通过某种指针使用它，最常见的就是 `&str`。

### 14.3 `&str`：借用的字符串切片

`&str` 是对一段字符串数据的借用，概念上就是：

```text
pointer（指向第一个字节）+ length（字节数）
```

它不拥有字符，只是一扇“看进去”的窗口：

```rust
let owned = String::from("rust language");
let whole: &str = &owned;        // 借用整个字符串
let part: &str = &owned[0..4];   // 只借用前 4 个字节："rust"
```

### 14.4 字符串字面量

```rust
let literal = "rust";
```

写在代码里的 `"rust"` 叫**字符串字面量（string literal）**，类型是 `&'static str`：字符数据存放在程序的只读数据区，随程序一起加载，整个程序运行期间都有效。`'static` 读作“静态生命周期”，当前阶段把它理解成“这段数据不依赖任何局部变量，永远活着”即可，显式生命周期语法在第 14 课讲。

## 十五、为什么字符串参数通常写 `&str`

一个只读文本的函数：

```rust
fn print_length(text: &str) {
    println!("{}", text.len());
}
```

两种调用者都能用：

```rust
print_length("rust"); // 字面量，本身就是 &str

let owned = String::from("rust");
print_length(&owned); // &String 自动转换成 &str
```

`&String` 能自动变成 `&str`，靠的是 **deref coercion（解引用强制转换）**：`String` 声明了“解引用我，就得到一个 `str`”，所以 `&String` 在需要 `&str` 的场合会自动转换。反过来不行：`&str` 不能变成 `&String`，因为那段文本可能根本不在 `String` 里。

经验法则：

- 只读任意文本：参数写 `&str`；
- 函数要长期保存这份文本：收 `String`（取得所有权）；
- 函数要造一份新文本：返回 `String`；
- 返回输入中的一段：返回 `&str`，但引用的来源必须是传入的数据，不能是函数里新建的局部值。

## 十六、字符串 slice 与 UTF-8

**slice（切片）**是对一段连续数据的借用视图。字符串的切片用**字节区间**取：

```rust
let text = String::from("hello");
let slice = &text[0..2]; // 字节 0 到 2（不含 2）

assert_eq!(slice, "he");
```

**UTF-8** 是 Rust 字符串的编码方式：一种用 1～4 个字节表示一个字符的国际标准。英文各 1 字节，汉字各 3 字节：

```rust,ignore
fn main() {
    let text = String::from("你好");
    println!("{}", text.len()); // 6：len() 数的是字节，不是字符
    let slice = &text[0..1]; // panic：字节 1 落在“你”的内部，不是字符边界
    println!("{slice}");
}
```

真实输出：

```text
6

thread 'main' (12345) panicked at src/main.rs:4:22:
byte index 1 is not a char boundary; it is inside '你' (bytes 0..3) of `你好`
```

（括号里的数字是本次运行的线程编号，每次运行都不同。出错位置是第 4 行的 `text[0..1]`：切片下标 1 落在汉字“你”的中间。）

同理，Rust 不提供 `text[0]` 这样的单字符索引：一个“字符”可能占不同数量的字节，下标语义说不清，也无法保证常数时间完成。

## 十七、普通 slice：`&[T]`

slice 不止服务于字符串。数组和 `Vec<T>` 都能借出一段 `&[T]`：

```rust
fn sum(values: &[i32]) -> i32 {
    let mut total = 0;
    for value in values { // 遍历 slice 时，value 是每个元素的引用 &i32
        total += value;   // &i32 可直接参与加法
    }
    total
}

let array = [1, 2, 3, 4];
let part = &array[1..3]; // 借用中间两个元素

assert_eq!(sum(&array), 10);
assert_eq!(sum(part), 5);
```

`&[i32]` 表示“借用的一段连续 `i32`”，不拥有元素。参数写 `&[i32]` 比写 `&Vec<i32>` 通用：数组、Vec、它们的子区间都能传进来。

## 十八、从 struct 字段返回值或引用

```rust
struct User {
    name: String,
    age: u32,
}
```

**返回 `Copy` 字段**——复制一份，对象不受影响：

```rust
impl User {
    fn age(&self) -> u32 {
        self.age
    }
}
```

**借用非 `Copy` 字段**——返回 `&str`，窗口开在 `User` 内部：

```rust
impl User {
    fn name(&self) -> &str {
        &self.name
    }
}
```

调用者不能让这个引用活得比 `User` 本身更久（第十三节的规则）。

**消费对象、移出字段**——方法名习惯以 `into_` 开头：

```rust
impl User {
    fn into_name(self) -> String {
        self.name
    }
}
```

`self`（不带 `&`）表示方法拿走整个 `User` 的所有权，所以可以把 `name` move 出来。调用后原 `User` 不再可用。

反过来，在 `&self` 方法里直接返回 `self.name`（`String`）是**不行**的：你手里只有借来的对象，不能把人家的字段搬走、留个空洞。真实诊断见下一节的 E0507。

## 十九、错误实验：阅读真实编译器诊断

下面每一条都是把代码交给 `rustc`（Rust 编译器）后得到的真实输出（节选，删除了与理解无关的提示行）。学排错的第一步是**读懂编译器在指哪一行、说什么事实**。

### 19.1 使用已被 move 的值：E0382

```rust
fn main() {
    let first = String::from("rust");
    let second = first;
    println!("{second}");
    println!("{first}");
}
```

```text
error[E0382]: borrow of moved value: `first`
 --> src/main.rs:5:16
  |
2 |     let first = String::from("rust");
  |         ----- move occurs because `first` has type `String`, which does not implement the `Copy` trait
3 |     let second = first;
  |                  ----- value moved here
5 |     println!("{first}");
  |                ^^^^^ value borrowed here after move
```

读法：编译器按时间顺序列出了三个事实——`first` 是 `String`（没有 `Copy`）、在第 3 行被 move、第 5 行又被使用。错误码 E0382 可以用 `rustc --explain E0382` 查看官方解释。

你可能疑惑：为什么叫 “borrow of moved value”？因为 `println!` 打印参数时内部用的是引用，即借用——它借了一个已经失效的绑定。

修复方向：不再使用旧绑定（最常见）、把 `println!("{first}")` 挪到 move 之前、确实需要两份时改用 `first.clone()`。

### 19.2 共享借用与可变借用重叠：E0502

```rust
fn main() {
    let mut text = String::from("rust");
    let reader = &text;
    let writer = &mut text;
    println!("{reader}");
    writer.push('!');
}
```

```text
error[E0502]: cannot borrow `text` as mutable because it is also borrowed as immutable
 --> src/main.rs:4:18
  |
3 |     let reader = &text;
  |                  ----- immutable borrow occurs here
4 |     let writer = &mut text;
  |                  ^^^^^^^^^ mutable borrow occurs here
5 |     println!("{reader}");
  |                ------ immutable borrow later used here
```

读法：三个位置分别是“共享借用开始”“可变借用试图开始”“共享借用后来又被使用”——正因为第 5 行还要用 `reader`，两个借用发生了重叠。修复方向：先用完 `reader`（比如把打印提前），再创建 `writer`；回忆第十二节，借用到**最后一次使用**为止。

### 19.3 两个可变借用：E0499

```rust
fn main() {
    let mut text = String::from("rust");
    let first = &mut text;
    let second = &mut text;
    first.push('!');
    second.push('?');
}
```

```text
error[E0499]: cannot borrow `text` as mutable more than once at a time
 --> src/main.rs:4:18
  |
3 |     let first = &mut text;
  |                 --------- first mutable borrow occurs here
4 |     let second = &mut text;
  |                  ^^^^^^^^^ second mutable borrow occurs here
5 |     first.push('!');
  |     ----- first borrow later used here
```

修复方向：让两个可变借用的使用区间错开，或者重新设计，把两次修改合并进同一段可变借用里完成。

### 19.4 返回局部值的引用：E0515

```rust
fn invalid(input: &str) -> &str {
    let text = String::from(input);
    &text
}
```

```text
error[E0515]: cannot return reference to local variable `text`
 --> src/main.rs:3:5
  |
3 |     &text
  |     ^^^^^ returns a reference to data owned by the current function
```

读法：“返回了一个引用，它指向当前函数拥有的数据”。函数一结束 `text` 就被 drop，引用必然悬空。修复方向：返回拥有型的 `String`，或者让返回的引用来自参数（比如直接返回 `input` 的某一段）。

### 19.5 从共享引用中移出字段：E0507

```rust
struct User {
    name: String,
}

impl User {
    fn name(&self) -> String {
        self.name
    }
}
```

```text
error[E0507]: cannot move out of `self.name` which is behind a shared reference
 --> src/main.rs:7:9
  |
7 |         self.name
  |         ^^^^^^^^^ move occurs because `self.name` has type `String`, which does not implement the `Copy` trait
```

读法：`&self` 只是借来的对象，不能把非 `Copy` 的字段从中搬走（搬走会让原对象留下一个空洞）。修复方向：返回 `&str`（`&self.name`）、确实需要副本时 `self.name.clone()`、或者让方法接收 `self` 以消费整个对象。

## 二十、完整示例：修饰一段文本

这个例子可以从输入到输出完整运行（保存为某个 `src/main.rs` 后 `cargo run`，或在练习项目里新建临时 bin）。请跟着注释追踪“谁拥有 `word`”：

```rust
/// 只借用 text 和 wrapper，返回一个全新的、属于自己的 String
fn surround(text: &str, wrapper: &str) -> String {
    let mut result = String::new();
    result.push_str(wrapper);
    result.push_str(text);
    result.push_str(wrapper);
    result
}

/// 按值接收：text 的所有权移进函数，函数结束时被 drop
fn consume_and_report(text: String) -> usize {
    let length = text.len();
    println!("consume_and_report 拿到了「{text}」，长度 {length}");
    length
}

fn main() {
    let word = String::from("rust");

    let decorated = surround(&word, "*"); // 只借用 word
    println!("装饰结果：{decorated}");
    println!("原值仍可用：{word}");

    let length = consume_and_report(word); // word 的所有权移入函数
    println!("报告的长度：{length}");
    // println!("{word}"); // 取消注释会得到 E0382：word 已被 move

    let numbers = [10, 20, 30, 40];
    let middle: &[i32] = &numbers[1..3]; // 借用数组中间一段
    println!("切片 {middle:?} 有 {} 个元素", middle.len());
}
```

运行输出：

```text
装饰结果：*rust*
原值仍可用：rust
consume_and_report 拿到了「rust」，长度 4
报告的长度：4
切片 [20, 30] 有 2 个元素
```

所有权流逐步拆解：

1. `word` 由 `main` 拥有；
2. `surround(&word, "*")` 只借走 `word` 看了一眼，返回的 `decorated` 是一份**全新**的 `String`，归 `main` 所有，与 `word` 互不影响；
3. `consume_and_report(word)` 把 `word` 的所有权移进参数；函数结束时参数被 drop，堆缓冲区随之释放；函数返回的 `usize` 是 `Copy` 类型，照常复制出来；
4. `middle` 借用数组 `numbers` 的两个元素，不复制、不拥有。

## 二十一、代码预测与解析

先写下你的判断和理由，再展开核对。

### 题目 1：move

```rust
let first = String::from("rust");
let second = first;
println!("{second}");
```

能否使用 `second`？能否在此之后使用 `first`？

<details>
<summary>查看解析</summary>

`second` 可以用，它是新的 owner。`first` 已失效：所有权在 `let second = first;` 时移交，再使用会得到 E0382。

</details>

### 题目 2：Copy

```rust
let first = 10_i32;
let second = first;
println!("{first}, {second}");
```

<details>
<summary>查看解析</summary>

两个都能打印。`i32` 实现了 `Copy`，赋值时复制数值，两个绑定各自独立。

</details>

### 题目 3：函数参数

```rust
fn consume(text: String) {
    println!("{text}");
}

let text = String::from("rust");
consume(text);
println!("{text}");
```

<details>
<summary>查看解析</summary>

最后一行编译失败。`text` 在调用时 move 进参数，并在 `consume` 结束时被 drop。

</details>

### 题目 4：借用的最后一次使用

```rust
let mut text = String::from("rust");
let shared = &text;
println!("{shared}");
text.push('!');
```

<details>
<summary>查看解析</summary>

可以编译。`shared` 的借用在 `println!` 处最后一次使用后结束，之后通过 owner 直接修改不再冲突（第十二节，NLL）。

</details>

### 题目 5：返回字段

```rust
impl Diary {
    fn title(&self) -> String {
        self.title
    }
}
```

若 `title` 字段是 `String`，为什么编译失败？改成什么签名最自然？

<details>
<summary>查看解析</summary>

`&self` 只是借用，不能把非 `Copy` 的 `String` 字段从对象里搬走（E0507）。最自然的改法是返回 `&str`（`fn title(&self) -> &str { &self.title }`）；确实要副本就 `clone`；确实要消费对象就改成 `self`。

</details>

## 二十二、主练习：拥有型笔记 `Note`

你将实现一条“拥有自己标题与正文”的笔记，把本课五种所有权/借用形态各用一遍。

### 22.1 在哪里写代码

本课是一个独立的 Cargo package（包名 `l03_ownership_borrowing`），主练习的脚手架在 `src/lib.rs`，五个 `todo!()` 是你要实现的部分。测试已经写好，在 `tests/note_tests.rs`。

### 22.2 数据结构与签名（已给出，不要修改）

```rust
pub struct Note {
    title: String,
    body: String,
}

impl Note {
    pub fn new(title: String, body: String) -> Self;
    pub fn title(&self) -> &str;
    pub fn body(&self) -> &str;
    pub fn append_line(&mut self, line: &str);
    pub fn into_parts(self) -> (String, String);
}
```

### 22.3 行为要求与示例

- `new`：取得两个 `String` 的所有权，分别存为标题与正文。
- `title` / `body`：返回对应字段的借用视图，**不复制**。
- `append_line`：正文为空时，正文直接变成 `line`；正文非空时，先追加一个换行符 `'\n'` 再追加 `line`。标题不受影响。
- `into_parts`：消费整条笔记，返回 `(标题, 正文)` 两个拥有型字符串。

| 输入操作 | 期望结果 |
| --- | --- |
| `Note::new("购物清单", "牛奶")` | `title()` → `"购物清单"`；`body()` → `"牛奶"` |
| 空正文 `""` 上 `append_line("first line")` | `body()` → `"first line"`（前面没有换行） |
| 正文 `"first line"` 上依次追加 `"second line"`、`"third line"` | `body()` → `"first line\nsecond line\nthird line"` |
| 空正文上 `append_line("")`（边界） | `body()` → `""` |
| 正文 `"only line"` 上 `append_line("")`（边界） | `body()` → `"only line\n"` |
| `Note::new("Rust", "所有权\n借用")` 调用 `into_parts()` | 返回 `("Rust", "所有权\n借用")`，两个返回值可各自独立修改 |
| 标题正文均为空的笔记调用 `into_parts()`（边界） | 返回 `("", "")` |

### 22.4 所有权要求（测试会间接检查这些性质）

- `new` 调用后，传入的两个 `String` 归笔记所有；
- `title()`、`body()` 可以反复调用，笔记始终可用；
- `append_line()` 结束后，笔记仍归调用者所有；
- `into_parts()` 调用后，笔记不再可用，返回的两个字符串成为独立的新 owner。

### 22.5 实现约束

- 不修改 struct 定义和方法签名；
- `title`、`body` 不得 `clone`；
- `into_parts` 不得 `clone` 字段；
- 不使用 `Rc`、`RefCell`、`unsafe` 或外部 crate；
- 不删除、不弱化 `tests/note_tests.rs` 中的测试。

### 22.6 推荐完成顺序

1. 实现 `new`，体会参数的所有权移入字段；
2. 实现 `title`、`body`，返回字段的借用；
3. 实现 `append_line`，分清 `&mut self`（可改的借用）与 `line: &str`（只读的借用）；
4. 实现 `into_parts`，直接移动两个字段构造 tuple；
5. 运行验收命令。

### 22.7 验收

在**仓库根目录**运行（也可以先 `cd lessons/03-ownership-borrowing`，再去掉 `-p` 参数运行）：

```bash
cargo run -p l03_ownership_borrowing          # 可运行演示（脚手架状态下也能跑）
cargo check -p l03_ownership_borrowing --all-targets
cargo test -p l03_ownership_borrowing
```

实现之前，`tests/note_tests.rs` 里的 10 个测试会全部失败（`todo!()` 触发 panic），这是正常的起点；`cargo check` 和 `cargo test --no-run` 在脚手架状态下就应该通过。实现完成后 10 个测试应全部变绿。

卡住时按级查看本课目录下的 `HINTS.md`。

## 二十三、课后巩固：借用与 slice 小工具

主练习练的是“拥有与转移”，巩固任务换一个角度：**全程不取得任何输入的所有权**，只用借用完成三个小工具。代码写在 `src/text.rs`，测试在 `tests/text_tests.rs`。

### 23.1 任务与行为示例

**`first_line(text: &str) -> &str`** —— 返回第一行（不含换行符）：

| 输入 | 期望返回 |
| --- | --- |
| `"hello\nworld"` | `"hello"` |
| `"a\nb\nc"` | `"a"` |
| `"no newline here"`（无换行） | `"no newline here"` |
| `""`（边界） | `""` |
| `"\nsecond"`（边界） | `""` |

**`total(values: &[i32]) -> i32`** —— 返回元素之和：

| 输入 | 期望返回 |
| --- | --- |
| `&[3, -1, 10, -4]` | `8` |
| `&[]`（边界） | `0` |
| `&[42]` | `42` |

**`byte_prefix(text: &str, max_bytes: usize) -> &str`** —— 返回开头一段，尽量接近但不超过 `max_bytes` 个字节，且不得切断字符、不得 panic：

| 输入 | 期望返回 |
| --- | --- |
| `("ownership", 4)` | `"owne"` |
| `("rust", 4)` 或 `("rust", 100)` | `"rust"` |
| `("你好", 4)` 或 `("你好", 5)`（边界：切在字符中间） | `"你"` |
| `("你好", 6)` | `"你好"` |
| `("hello", 0)`（边界） | `""` |

### 23.2 提示：你可能会遇到下一课的主角

实现 `first_line` 时，`str` 的 `find` 方法会返回一个 `Option<usize>`：`Some(下标)` 表示找到，`None` 表示没找到。`Option`（“可能有、可能没有”的枚举）是第四课的主角；本课只需要会用 `match` 分清这两种情况即可（HINTS.md 给了结构）。你也可以完全不用 `find`，想想还有什么办法。

### 23.3 验收

```bash
cargo test -p l03_ownership_borrowing --test text_tests
```

实现之前，这 12 个测试会全部因 `todo!()` 失败；完成后应全部通过。

## 二十四、常见误区

### 误区 1：move 等于复制了整个堆缓冲区

move 移交的是栈上的描述信息和“谁负责释放”的责任，堆上的字符原封不动。

### 误区 2：借用都持续到大括号结束

借用持续到引用的**最后一次使用**（NLL）。分析冲突时找最后使用点，不要只数大括号。

### 误区 3：`&String` 参数比 `&str` 更“准确”

只读文本的参数应写 `&str`：字面量、`String` 的借用、子切片都能传进来，适用面更宽。

### 误区 4：看到 move 错误就 clone

先判断函数是否真的需要所有权。只读操作几乎总是借用更合适。

### 误区 5：在 `&self` 方法里返回 `String` 字段

这是从借来的对象里搬东西（E0507）。返回 `&str`，或让方法消费 `self`。

### 误区 6：`&mut` 只是“能修改的指针”

`&mut` 同时是**独占访问**的承诺：它存在期间，同一数据不能有任何其他重叠的引用。这个“独占性”是编译器保证数据不被意外改动的基础。

## 二十五、本课总结

1. 每个值都有 owner；owner 离开作用域时，值被自动 drop。
2. 非 `Copy` 值的赋值和按值传参会 move，旧绑定当场失效。
3. `Copy` 类型赋值产生独立副本；`clone()` 是显式的深复制，写出让读者看到成本。
4. `&T` 是共享借用：多个读者可同时存在，但不能改。
5. `&mut T` 是独占的可变借用：不能与其他任何借用重叠。
6. 借用持续到引用的最后一次使用，而非大括号结束。
7. 引用必须始终指向活着的数据；返回局部值的引用会被拒绝。
8. `String` 拥有可增长的 UTF-8 缓冲区；`&str` 是借用的字符串切片；字符串字面量是 `&'static str`。
9. slice（`&[T]`）是“指针加长度”的借用视图，不拥有数据；字符串切片边界必须落在字符边界上。
10. API 按“是否需要拥有”选择 `T`、`&T`、`&mut T` 或拥有型返回值。

## 二十六、课后复盘

完成练习后，不看讲义，用自己的话回答：

1. 为什么 `String` 赋值后旧绑定失效，而 `i32` 不会？
2. 借用和所有权转移的根本区别是什么？
3. 为什么不能同时存在 `&T` 和 `&mut T` 的重叠使用？
4. `String`、`str`、`&str` 各自扮演什么角色？
5. 为什么 `fn title(&self) -> &str` 可行，而 `fn title(&self) -> String` 不能直接返回字段？
6. `into_parts(self)` 调用后，笔记和返回的两个字符串分别处于什么状态？

## 下一课预告

[第四课：Option 与 Result](../04-option-result/README.md)。本课的 `first_line` 里你可能已经遇见了 `Option`——“值可能不存在”这件事，Rust 不用空指针，而是用类型明确表达。下一课会系统学习 `Option`、`Result` 与 `?` 运算符，开始编写不靠 panic 表达普通失败的函数。
