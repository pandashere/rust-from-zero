# 第十五课：`Rc`、`Weak`、`RefCell` 与内部可变性

假设你在写一个公司组织结构程序。每个部门要记录自己的名称、直接下属部门，还要能向上找到自己的上级部门。程序的其他部分可能正拿着某个部门的“句柄”（handle，指一个可以访问该部门的值）在做别的事，比如显示详情、发通知。于是同时出现三个需求：

1. 一个部门可以被多个位置共同使用，而不是被某一个父部门独占；
2. 子部门要能指回父部门，但这条反向链接不能阻止父部门被清理；
3. 父部门要通过一个共享的句柄添加子部门，也就是在只有“只读访问权”的情况下修改内部数据。

第十三课的 `Box<T>` 只能提供**独占**所有权：一个 `Box` 只有一个主人。普通引用又受生命周期限制，无法方便地存进结构体形成双向关系。Rust 为这类单线程共享场景准备了三个工具：

- `Rc<T>`：引用计数（reference counting，用一个整数记录“有多少个主人”）的共享所有权指针；
- `Weak<T>`：弱引用（weak reference，能观察目标，但不负责让目标继续存活）；
- `RefCell<T>`：把“共享借用或独占借用”的检查从编译期移到运行时的容器。

这里的“编译期”指编译器（compiler，把源码翻译成可执行程序的工具）检查代码的阶段，“运行时”指程序真正执行的时候。`RefCell<T>` 提供的能力叫**内部可变性**，第五节会展开解释。

本课从具体困难出发，逐个解释它们，再说明为什么组合使用必须克制。

## 本课定位

### 你将学会

- 说明 `Box<T>`、`Rc<T>`、`Weak<T>`、`RefCell<T>` 各自负责哪一层问题。
- 用 `Rc::new`、`Rc::clone`、`Rc::strong_count` 管理单线程共享所有权。
- 区分 `Rc::clone(&value)` 与 `(*value).clone()`，知道前者共享、后者可能深拷贝（把底层数据也复制一份）。
- 用 `Rc::get_mut` 在“只有一个主人”时取得可变访问。
- 用 `RefCell::borrow`、`borrow_mut`、`try_borrow_mut` 在运行时借用数据，并让借用凭证及时离开作用域。
- 解释内部可变性（interior mutability，通过共享引用修改内部状态的机制）适合什么场景。
- 用 `Rc<RefCell<T>>` 表示单线程共享可变状态，并说清两层的成本。
- 用 `Weak::downgrade`、`Weak::upgrade`、`Rc::weak_count` 建立父指针、缓存和观察者关系。
- 画出强引用边和弱引用边，判断哪里会形成引用计数环（cycle，互相保活的一圈引用）。
- 判断什么时候应该改用普通所有权、索引或集中存储，而不是 `Rc<RefCell<T>>`。

### 前置知识

本课会反复用到下面几课的内容。每一条都先用一两句话复习，读不懂时再点链接回看。

- **结构体、方法与 `#[derive(...)]`**：见[第二课](../02-data-modeling/README.md)。复习：`struct` 把若干字段组合成一个新类型；`impl` 块里的函数叫方法；`&self` 表示方法只借用结构体来读，`&mut self` 表示借来修改；`#[derive(Debug)]` 让值可以用 `{:?}` 打印，`#[derive(PartialEq, Eq)]` 让值可以用 `==` 比较。
- **所有权、移动与借用**：见[第三课](../03-ownership-borrowing/README.md)。复习：每份数据有一个所有者，所有者离开作用域时数据被清理；把值赋给别处或传进函数可能发生**移动（move）**，原变量不能再使用；`&T` 是共享借用，可以有多个，只能读；`&mut T` 是独占借用，同一时间只能有一个，可以改；共享借用与独占借用不能重叠。
- **`Option<T>` 与 `Result<T, E>`**：见[第四课](../04-option-result/README.md)。复习：`Option` 表示“可能有值，也可能没有”，用 `Some(x)`/`None` 表示；`Result` 表示“成功或失败”，用 `Ok(x)`/`Err(e)` 表示；要使用里面的值，必须先处理外层状态。
- **`Vec<T>`**：见[第五课](../05-vec-slices/README.md)。复习：`Vec` 是长度可变的列表，拥有它的元素；`push` 追加，`len` 取长度，索引从 0 开始。
- **`Box<T>` 与堆**：见[第十三课](../13-box-recursion/README.md)。复习：局部变量通常在**栈（stack）**上，函数返回时自动回收；大小在编译期不确定或需要长期存在的数据可以放到**堆（heap）**上，堆内存由所有者负责释放。`Box<T>` 是“独占拥有堆上一个 `T`”的指针。
- **生命周期**：见[第十四课](../14-lifetimes/README.md)。复习：生命周期描述引用能合法使用多久；标注不会延长任何值的生命；把引用存进结构体时，结构体不能比被引用的数据活得更久。本课会看到：有些关系用引用很难表达，于是改用 `Rc`/`Weak`。
- **`String` 与 `&str`**：见[第三课](../03-ownership-borrowing/README.md)。复习：`String` 是拥有文本的、可以增长的字符串；`&str` 是借用的一段文本。`str` 和 `String` 的关系类似切片和 `Vec`。

### 本课边界

- 本课只讨论**单线程**。`Rc` 和 `RefCell` 不能安全地跨线程共享；线程、`Arc`、`Mutex` 在[第十六课](../16-threads-channels/README.md)和[第十七课](../17-shared-concurrency/README.md)学习。本课提到“线程安全”时只会给出够用的临时解释。
- 本课不展开 `Cell<T>`（另一种内部可变性容器，适合存放可以整体复制的值）。学完 `RefCell` 后，你可以按需查阅它的文档。
- `Rc` 内部的计数更新用到了 `unsafe`（程序员额外保证安全、编译器不再全面检查的代码）。本课只使用安全接口，不要求你写 `unsafe`；细节留到[第二十一课](../21-unsafe-ffi/README.md)。
- 本课会提到“用 `Vec` 加索引表示图”这一替代方案，但不会实现完整的索引图或内存池（arena）。目标是让你能判断何时不该使用 `Rc<RefCell<T>>`。
- `impl Into<String>`、`impl Trait` 等写法的完整规则在[第十一课](../11-generics/README.md)和[第十二课](../12-traits/README.md)。本课只把它当作“任何能转换成拥有型 `String` 的文本”来用。

## 如何学习本课

1. 先读第一节，把三个需求分别对应到“共享所有权”“弱引用”“内部可变性”，不要急着记 API。
2. 第二到第八节每节都有一个完整示例。先预测输出，再运行或对照讲解。
3. 第十节的诊断不要跳过。`Rc` 和 `RefCell` 的错误常出现在编译期和运行期的不同位置，能读懂提示比背规则更省时间。
4. 做完第十一节的代码预测，再打开 `src/exercise.rs` 做主练习。
5. 卡住时按 `HINTS.md` 的一级、二级、三级顺序查看，不要一开始就看第三级。
6. 主练习通过后，再做 `src/consolidation.rs` 的课后巩固。

预计用时：概念与示例约 2 小时，主练习约 1 小时，课后巩固约 40 分钟。

## 一、先看具体问题：为什么 `Box` 不够

### 1.1 用 `Vec<Unit>` 直接嵌套

最直接的写法是让父部门用一个 `Vec` 装子部门：

```rust,ignore
// 片段：这段代码能表达“父子”，但满足不了本节的需求。
struct Unit {
    name: String,
    children: Vec<Unit>,
}
```

它有两个困难：

- 子部门被**移动**进了父部门的 `Vec`。外部如果还想单独持有并操作某个子部门，就拿不到它了；`Vec` 的所有者是父部门。
- 子部门没有指回父部门的字段。就算加一个引用字段，也会遇到生命周期问题：子部门不能保证父部门比自己活得久，而且父部门移动时地址可能变化。

### 1.2 用 `Box<Unit>` 也只是换了独占者

```rust,ignore
// 片段：仍然只有一个所有者。
struct Unit {
    name: String,
    children: Vec<Box<Unit>>,
}
```

`Box` 把子部门放到堆上，解决了“结构体大小必须固定”的问题，但**所有权仍然唯一**：`Vec<Box<Unit>>` 拥有每个子部门。外部想把某个部门交给另一个模块长期保存，就必须转移所有权；转移之后父部门就不能再用它。

更关键的是反向链接。用普通引用写：

```rust,ignore
// 片段：这段代码无法通过编译，原因见下面的文字。
struct Unit {
    name: String,
    parent: Option<&Unit>,
    children: Vec<Box<Unit>>,
}
```

编译器会要求你给 `Unit` 加上生命周期参数，并且要求父部门在所有子部门使用期间一直存活。可是父部门本身又拥有子部门，二者互相引用，生命周期关系会变得难以满足。此外，`Vec` 增长搬家时，子部门在内存中的位置可能改变，指向它的引用也会失效。第十三课和第十四课分别讲过“移动”和“生命周期”，这里正是两者的交界。

### 1.3 三个需求，三种工具

把需求拆开看：

| 需求 | 普通所有权的问题 | 本课工具 |
| --- | --- | --- |
| 多个位置共同使用一个部门 | 一份数据只有一个所有者 | `Rc<T>` 共享所有权 |
| 子部门指回父部门，但不阻止父部门销毁 | 强引用会让父部门一直活着 | `Weak<T>` 弱引用 |
| 通过共享句柄修改父子列表 | 共享借用 `&T` 不能修改 | `RefCell<T>` 运行时借用检查 |

**智能指针（smart pointer）**是一个普通的结构体，它保存一个指向数据的指针，同时附带额外行为，例如计数、借用检查或自动释放。`Box<T>`、`Rc<T>`、`Weak<T>` 都属于智能指针；`RefCell<T>` 更准确地说是一个“提供内部可变性的容器”，但它常和智能指针一起使用。它们都不是编译器关键字，而是标准库（随 Rust 一起安装、可以直接使用的一组代码）里的类型。

## 二、`Rc<T>`：让多个位置共同拥有

### 2.1 引用计数在解决什么

`Rc` 是 “reference counted” 的缩写，意思是“带引用计数的”。`Rc::new(value)` 把 `value` 放到堆上，并额外保存两个计数：

- **强引用计数（strong count）**：有多少个 `Rc` 正在共同拥有这个值。强计数降到 0 时，堆上的值被销毁。
- **弱引用计数（weak count）**：有多少个 `Weak` 正在观察这个值。弱引用不参与“是否销毁”的决定，本课第七节再讲。

可以粗略画成：

```text
栈上的 Rc 变量            堆上的控制块                堆上的值
+-------------+          +------------------+       +-------------+
| pointer ----|--------->| strong_count = 2 |       |  String     |
+-------------+          | weak_count   = 0 |       |  "Rust"     |
                         +------------------+       +-------------+
```

每次 `Rc::clone` 只把强计数加 1，**不复制**堆上的值；每个 `Rc` 离开作用域时把计数减 1，减到 0 才销毁值。

### 2.2 完整示例：三个名字共享同一段文本

```rust
use std::rc::Rc;

fn main() {
    let shared = Rc::new(String::from("Rust"));

    let first = Rc::clone(&shared);
    let second = Rc::clone(&shared);

    println!("强引用数量: {}", Rc::strong_count(&shared));
    println!("first 看到: {first}");
    println!("second 看到: {second}");

    drop(second);
    println!("丢弃 second 后: {}", Rc::strong_count(&shared));

    drop(first);
    println!("丢弃 first 后: {}", Rc::strong_count(&shared));
}
```

输出：

```text
强引用数量: 3
first 看到: Rust
second 看到: Rust
丢弃 second 后: 2
丢弃 first 后: 1
```

逐行解释：

- `fn main()` 是程序入口；`println!` 是向终端打印一行文本的宏（编译时生成代码的工具，第一课介绍过）。
- `use std::rc::Rc;` 把 `Rc` 这个名字引入当前文件。`std` 是标准库，`rc` 是其中存放引用计数类型的模块。
- `Rc::new(String::from("Rust"))` 在堆上创建 `String`，并返回一个强计数为 1 的 `Rc<String>`。
- `Rc::clone(&shared)` 接收一个 `&Rc<String>`，返回新的 `Rc<String>`，强计数加 1。注意它的参数是借用，不是把 `shared` 移动走。
- `Rc::strong_count(&shared)` 读取当前强计数。三个变量 `shared`、`first`、`second` 各持有一个强引用，所以是 3。
- `drop(second)` 是标准库函数，作用是立刻销毁这个变量（而不是等它离开作用域）。销毁一个 `Rc` 会把计数减 1；因为计数还不是 0，堆上的 `String` 不受影响。
- 最后 `shared` 和 `first` 仍然存在，计数为 1。等 `main` 结束，最后一个强引用销毁，`String` 才被释放。

### 2.3 为什么推荐写 `Rc::clone(&value)`

对一个 `Rc<String>` 写 `value.clone()` 也能工作，因为 `Rc` 实现了 `Clone`（标准库的 trait，表示“可以显式复制一个值”）。但 `.clone()` 这个名字有歧义：它可能是在克隆 `Rc`（只加计数），也可能是在克隆底层的 `String`（复制整段文本）。看下面两行：

片段（省略了 `fn main` 包装；假设 `shared` 已经创建）：

```rust
use std::rc::Rc;

let shared = Rc::new(String::from("Rust"));
let shared_again = Rc::clone(&shared); // 类型 Rc<String>，共享同一段文本
let owned_copy = (*shared).clone();    // 类型 String，复制出一段新文本
```

- `Rc::clone(&shared)` 的类型是 `Rc<String>`，`Rc::ptr_eq(&shared, &shared_again)` 为 `true`，二者指向同一块堆内存。
- `(*shared).clone()` 先解引用得到 `String`，再克隆 `String`，类型是 `String`。这是一次真正的深拷贝，之后修改副本不会影响原值。

代码审阅工具 Clippy（Rust 自带的静态检查器，见[第八课](../08-testing-quality/README.md)）通常会建议把 `shared.clone()` 写成 `Rc::clone(&shared)`。这不只是风格问题：读到 `Rc::clone` 时，维护者立刻知道“没有复制大数据”。

### 2.4 身份与相等不是一回事

`Rc::ptr_eq(&a, &b)` 判断两个 `Rc` 是否指向**同一块分配**（同一个堆对象），也就是身份相同。`a == b` 比较的是两个 `Rc` 里**值的内容**，要求底层类型实现了 `PartialEq`。两个不同部门都叫 `"Ada"`，内容相等，但身份不同；本课主练习会用到这个区别。

### 2.5 `Rc` 的限制

- **默认只能共享读。** `Rc<T>` 只实现了解引用到 `&T`，没有提供 `&mut T`。原因和第三课的借用规则一致：多个主人同时修改同一份数据，会破坏“独占借用时没有别人读”的保证。下一节会看到具体报错。
- **计数不是原子操作。** “原子”指多个线程同时执行也不会互相打断。`Rc` 的计数更新没有做线程同步，所以两个线程同时克隆同一个 `Rc` 可能把计数改坏。因此 `Rc<T>` 不能跨线程共享；跨线程的对应物是 `Arc<T>`（atomic reference counted，原子引用计数），见第十七课。
- **有运行时成本。** 每次 clone/drop 都要改计数；每个 `Rc` 还会带来一次间接访问。它不是“零成本抽象”，只是成本通常可以接受。
- **可能形成环。** 两个 `Rc` 互相强引用时，计数永远不归零，值不会被释放。第八节专门处理。

## 三、共享不等于可变

`Rc<T>` 允许很多个主人，但每个主人只能读。尝试通过 `Rc` 修改内部值：

```rust,compile_fail
use std::rc::Rc;

fn main() {
    let shared = Rc::new(String::from("rust"));
    shared.push('!');
}
```

真实报错（当前稳定版 Rust，错误码 E0596）：

```text
error[E0596]: cannot borrow data in an `Rc` as mutable
 --> src/main.rs:5:5
  |
5 |     shared.push('!');
  |     ^^^^^^ cannot borrow as mutable
  |
  = help: trait `DerefMut` is required to modify through a dereference, but it is not implemented for `Rc<String>`
```

读法：

- `cannot borrow data in an Rc as mutable` 的意思是“不能把 `Rc` 里的数据借成可变的”。
- `trait DerefMut is required` 提到一个 trait（特征，一组行为的约定，见第十二课）。`push` 需要 `&mut String`，而 `Rc<String>` 只能给出 `&String`。
- 这不是编译器“忘了”提供功能，而是共享所有权与自由修改本来就冲突。

如果确定当前只有一个强引用，可以用 `Rc::get_mut` 请求独占访问：

```rust
use std::rc::Rc;

fn main() {
    let mut shared = Rc::new(String::from("rust"));

    // 此时只有一个强引用，get_mut 返回 Some(&mut String)。
    let exclusive = Rc::get_mut(&mut shared).expect("只有一个主人时应该成功");
    exclusive.push('!');
    println!("{shared}");

    let second = Rc::clone(&shared);
    // 现在有两个强引用，不能再取得独占访问。
    assert!(Rc::get_mut(&mut shared).is_none());

    drop(second);
    // 又只剩一个强引用，可以再次修改。
    Rc::get_mut(&mut shared).unwrap().push('?');
    println!("{shared}");
}
```

输出：

```text
rust!
rust!?
```

要点：

- `Rc::get_mut(&mut rc)` 返回 `Option<&mut T>`。只有**恰好一个强引用且没有弱引用**时才返回 `Some`，否则返回 `None`。
- 为了调用它，`shared` 这个变量本身要声明为 `mut`，因为要借出 `&mut Rc<String>`。但“变量可写”不等于“里面的 `String` 随时可写”，仍然要经过 `get_mut` 的检查。
- 这个办法只适合“短暂地确认独占”。只要还有别的 `Rc` 活着，它就失败。共享可变状态需要下一节的 `RefCell`。

## 四、`RefCell<T>`：把借用检查移到运行时

### 4.1 先复习借用规则

第三课的核心规则是：

- 同一时间，可以有任意多个共享借用 `&T`，**或者**恰好一个可变借用 `&mut T`；
- 两者不能重叠。

普通引用由编译器在编译期检查，所以违反规则的代码根本编译不过。`RefCell<T>` 保留同样的规则，但把检查推迟到程序运行时：

- `borrow()` 申请共享借用，返回一个借用凭证；
- `borrow_mut()` 申请独占借用，返回另一个借用凭证；
- 如果申请时违反规则，程序会 **panic**（立即中止当前执行，打印错误信息）；
- 凭证销毁时，借用自动结束。

`RefCell` 是 “reference cell” 的意思，可以理解为“一个装着值、并记录当前借用状态的小格子”。

### 4.2 完整示例：在运行时借来修改

```rust
use std::cell::RefCell;

fn main() {
    let value = RefCell::new(String::from("rust"));

    {
        let mut borrowed = value.borrow_mut();
        borrowed.push('!');
        println!("修改中: {borrowed}");
    } // borrowed 在这里销毁，独占借用结束

    println!("修改后: {}", value.borrow());
    println!("长度: {}", value.borrow().len());
}
```

输出：

```text
修改中: rust!
修改后: rust!
长度: 5
```

逐点解释：

- `RefCell::new(...)` 把值放进格子。`value` 本身不需要声明为 `mut`，因为修改能力来自 `RefCell` 的内部状态，而不是来自 `&mut value`。
- `value.borrow_mut()` 返回 `RefMut<'_, String>`。这是一个**借用凭证**：只要它还活着，`RefCell` 就认为独占借用尚未结束。
- `borrowed.push('!')` 能调用 `String` 的方法，因为 `RefMut` 实现了 `DerefMut`，可以像 `&mut String` 一样使用。
- 花括号让 `borrowed` 在右花括号处销毁，借用随之结束。删除花括号会让 `borrowed` 活到 `main` 结尾，后面的 `borrow()` 就会 panic。
- `value.borrow()` 返回 `Ref<'_, String>`，可以像 `&String` 一样读。最后一个 `value.borrow().len()` 中，临时 `Ref` 在整条语句结束后销毁。

`Ref` 和 `RefMut` 里的 `'_` 是第十四课讲过的生命周期占位符，表示“这个凭证借用了 `value`，不能比 `value` 活得更久”。编译器仍然会检查凭证不逃出 `value` 的生命周期；`RefCell` 只是把“同时有几个借用”这件事改到运行时判断。

### 4.3 违反规则：能编译，但运行时 panic

```rust,should_panic
use std::cell::RefCell;

fn main() {
    let value = RefCell::new(vec![1, 2, 3]);
    let shared = value.borrow();
    let mut exclusive = value.borrow_mut();
    exclusive.push(4);
    println!("{}", shared.len());
}
```

编译能通过。运行输出（线程编号和行号会随程序不同）：

```text
thread 'main' (126075) panicked at src/main.rs:6:31:
RefCell already borrowed
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

`RefCell already borrowed` 的意思是“这个格子已经被借用了”。这里是共享借用 `shared` 仍然活着时申请独占借用，违反规则。注意 panic 发生在 `borrow_mut()` 那一行，而不是后面的 `push`。

**panic 不是普通错误处理。** 它会让当前执行路径立刻停止，默认还会展开调用栈（逐层清理局部变量）。库代码不应把用户输入错误变成 panic；需要可恢复处理时，用下一节的 `try_borrow_mut`。

### 4.4 `try_borrow` 与 `try_borrow_mut`：把失败变成 `Result`

```rust
use std::cell::RefCell;

fn main() {
    let cell = RefCell::new(10);

    let first = cell.borrow_mut();
    match cell.try_borrow_mut() {
        Ok(_) => println!("意外地拿到了独占借用"),
        Err(error) => println!("申请失败: {error}"),
    }

    drop(first);
    assert!(cell.try_borrow_mut().is_ok());
    println!("释放后可以再次借用");
}
```

输出：

```text
申请失败: RefCell already borrowed
释放后可以再次借用
```

- `try_borrow()` 返回 `Result<Ref<'_, T>, BorrowError>`；
- `try_borrow_mut()` 返回 `Result<RefMut<'_, T>, BorrowMutError>`；
- 失败时返回 `Err`，不会 panic，调用者可以打印提示、跳过本次操作或换一条路径。

`BorrowMutError` 这个名字可以拆成 “borrow mut error”，即“申请可变借用失败的错误”。错误类型本身不携带“谁还拿着旧借用”的信息；排查时仍然要自己检查凭证的作用域。

### 4.5 为什么 Rust 不总在编译期检查

有些修改发生在“编译器看不到全貌”的地方：

- 一个方法只拿到 `&self`，但内部需要记录调用日志；
- 一个图结构通过共享句柄增删边，编译器无法静态证明没有重叠访问；
- 回调、观察者等关系在运行时才建立。

`RefCell` 让这些代码可以写出来，代价是把一份责任交给运行时：**你必须自己保证不会重叠借用，否则程序会 panic**。它不是“关闭借用检查”，而是“换一个时间检查”。

## 五、内部可变性：通过 `&self` 修改内部状态

普通方法如果要修改结构体，签名通常写 `&mut self`：

片段（只定义类型与方法，省略了 `fn main`；这段代码本身可以编译）：

```rust
struct Counter {
    value: u32,
}

impl Counter {
    fn increment(&mut self) {
        self.value += 1;
    }
}
```

调用者必须拥有一个可写的 `Counter`，并且在同一时间不能有别人读它。这在很多场景是正确的：修改是公开行为，调用者应该知道。

但有些状态属于“实现细节”，不适合要求调用者提供 `&mut`。例如一个记录调用日志的对象：

```rust
use std::cell::RefCell;

struct CallLog {
    entries: RefCell<Vec<String>>,
}

impl CallLog {
    fn new() -> Self {
        Self {
            entries: RefCell::new(Vec::new()),
        }
    }

    // 注意：这里只借用 &self，但仍然修改了 entries。
    fn record(&self, message: &str) {
        self.entries.borrow_mut().push(message.to_owned());
    }

    fn entries(&self) -> Vec<String> {
        self.entries.borrow().clone()
    }
}

fn main() {
    let log = CallLog::new();

    // log 不是 mut，也能记录。
    log.record("启动");
    log.record("加载配置");

    println!("{:?}", log.entries());
}
```

输出：

```text
["启动", "加载配置"]
```

这种“外部看到的是只读操作，内部却改变了状态”的能力就叫**内部可变性（interior mutability）**。`RefCell` 是最常用的单线程实现；标准库内部用 `UnsafeCell` 和 `unsafe` 实现它，但使用者只需要安全接口。你在本课不会写任何 `unsafe`。

适合使用内部可变性的场景：

- trait 方法只提供 `&self`，但实现需要记录调用、计数或缓存；
- 缓存不属于对象对外的可见语义，调用者不关心它何时更新；
- 单线程图结构需要在共享节点下增删边；
- 测试替身（mock）需要记录被调用了多少次。

不适合的场景：

- 修改是对象的主要行为，调用者应当明确知道要传 `&mut`；
- 多线程共享：`RefCell` 不是线程安全的；
- 为了绕过一次编译错误，把整个结构体所有字段都包进 `RefCell`。先问：能不能缩短借用、拆分字段、调整所有权？

## 六、`Rc<RefCell<T>>`：共享可变状态

把两层组合起来：

- `Rc<...>` 让多个位置共同拥有同一个格子；
- `RefCell<...>` 允许任何一个主人通过共享句柄申请运行时可变借用；
- 最里层才是真正的数据。

```rust
use std::cell::RefCell;
use std::rc::Rc;

fn main() {
    let shared = Rc::new(RefCell::new(vec![1, 2]));

    let another_owner = Rc::clone(&shared);

    shared.borrow_mut().push(3);
    another_owner.borrow_mut().push(4);

    println!("{:?}", shared.borrow());
    println!("强引用数量: {}", Rc::strong_count(&shared));
}
```

输出：

```text
[1, 2, 3, 4]
强引用数量: 2
```

逐层读类型 `Rc<RefCell<Vec<i32>>>`：

1. `Vec<i32>`：真正的数据，一个整数列表。
2. `RefCell<Vec<i32>>`：装着列表的格子，负责记录当前有没有借用。
3. `Rc<RefCell<Vec<i32>>>`：指向这个格子的共享句柄，负责记录有多少个主人。

每一层都带来一种失败模式：

- `Rc` 可能形成强引用环，导致泄漏；
- `RefCell` 可能在运行时重叠借用，导致 panic；
- 多层包装让类型签名变长，也让“谁在什么时候借用了什么”更难追踪。

### 6.1 借用凭证的作用域是常见坑

下面这段代码在运行时 panic：

```rust,should_panic
use std::cell::RefCell;

fn main() {
    let values = RefCell::new(vec![1, 2, 3]);
    let first = values.borrow();
    let total = first.len();
    values.borrow_mut().push(total);
}
```

因为 `first` 一直活到最后一行。正确写法是让读借用尽早结束：

```rust
use std::cell::RefCell;

fn main() {
    let values = RefCell::new(vec![1, 2, 3]);

    // 方案一：临时 Ref 在语句结束时销毁。
    let total = values.borrow().len();
    values.borrow_mut().push(total);

    // 方案二：用块明确限制作用域。
    let first = {
        let borrowed = values.borrow();
        borrowed.first().copied()
    };
    values.borrow_mut().push(first.unwrap_or_default());

    println!("{:?}", values.borrow());
}
```

输出：

```text
[1, 2, 3, 3, 1]
```

`values.borrow().len()` 中的临时 `Ref` 在分号处销毁，所以下一行可以申请独占借用。块中的 `borrowed` 在右花括号处销毁，返回的 `Option<i32>` 是复制出来的整数，不携带借用。

不要试图返回指向 `RefCell` 内部的普通引用：

```rust,compile_fail
use std::cell::RefCell;

fn first(values: &RefCell<Vec<i32>>) -> &i32 {
    &values.borrow()[0]
}
```

`borrow()` 产生的 `Ref` 在函数返回时销毁，借用随之结束，不能把里面的引用带出去。需要把借用带出函数时，可以返回 `Ref<'_, T>` 本身（让调用者持有凭证），或者更常见地返回复制/克隆后的值。

### 6.2 多种合法写法，优先选容易读的

`shared.borrow_mut().push(3)` 是一条短表达式：临时 `RefMut` 在语句结束时销毁。它很清爽，但如果同一行里还要读同一个格子，就可能重叠借用。初学阶段可以把步骤拆开：

```rust
use std::cell::RefCell;
use std::rc::Rc;

fn main() {
    let shared = Rc::new(RefCell::new(Vec::<i32>::new()));
    {
        let mut values = shared.borrow_mut();
        values.push(10);
        values.push(20);
    }
    println!("{:?}", shared.borrow());
}
```

清晰比短更重要。主练习的注释也要求每个 `borrow`/`borrow_mut` 都有明确作用域。

## 七、`Weak<T>`：看得见，但不保活

### 7.1 为什么需要弱引用

`Rc<T>` 的强引用会让目标一直活着。但有些关系天然是“单向拥有”：

- 子部门需要知道父部门，但父部门不应该因为子部门还活着而不能销毁；
- 缓存想记住对象，但缓存不应该阻止对象被清理；
- 观察者想引用被观察者，但被观察者不应被观察者保活。

如果这些反向关系也用 `Rc`，就会形成强引用环。`Weak<T>` 提供另一条边：它指向同一块分配，但**不增加强引用计数**，因此不参与“目标是否销毁”的决定。

```text
Rc<T>    ──强引用──▶ 目标；强计数 +1，目标因此保活
Weak<T>  ──弱引用──▶ 目标；只增加弱计数，目标仍可销毁
```

### 7.2 完整示例：缓存不阻止对象销毁

```rust
use std::rc::Rc;

fn main() {
    let strong = Rc::new(String::from("data"));
    let weak = Rc::downgrade(&strong);

    println!("强计数: {}", Rc::strong_count(&strong));
    println!("弱计数: {}", Rc::weak_count(&strong));
    println!("upgrade 结果: {:?}", weak.upgrade());

    drop(strong);

    println!("销毁后 upgrade: {:?}", weak.upgrade());
    println!("weak 看到的强计数: {}", weak.strong_count());
}
```

输出：

```text
强计数: 1
弱计数: 1
upgrade 结果: Some("data")
销毁后 upgrade: None
weak 看到的强计数: 0
```

解释：

- `Rc::downgrade(&strong)` 从强引用创建弱引用。它不复制 `String`，也不增加强计数。
- `weak.upgrade()` 返回 `Option<Rc<T>>`：目标还活着时返回 `Some(新的强引用)`，否则返回 `None`。为什么是 `Option`？因为弱引用可以比目标活得更久，调用时目标可能已经销毁。
- `upgrade()` 一旦成功，会临时创建一个强引用。只要这个 `Rc` 还在，目标就不会被销毁；这就是“观察者可以安全使用目标”的保证。
- `drop(strong)` 后，强计数降到 0，堆上的 `String` 被销毁。弱引用句柄本身仍然可以存在，再次 `upgrade` 得到 `None`；对已经失效的目标，`weak.strong_count()` 也返回 0。
- 当最后一个 `Weak` 也销毁后，控制块（保存计数的内存）才会被释放。

### 7.3 空弱引用与结构体初始化

`Weak::new()` 创建一个“不指向任何目标”的弱引用，`upgrade()` 永远返回 `None`。它常用于初始化结构体字段：

```rust
use std::cell::RefCell;
use std::rc::Weak;

struct Node {
    parent: RefCell<Weak<Node>>,
}

fn main() {
    let node = Node {
        parent: RefCell::new(Weak::new()),
    };
    assert!(node.parent.borrow().upgrade().is_none());
    println!("还没有父节点");
}
```

为什么字段类型是 `RefCell<Weak<Node>>`，而不是直接 `Weak<Node>`？因为建立父子关系时要修改这个字段，而节点本身是通过共享句柄使用的，拿不到 `&mut Node`；`RefCell` 提供了运行时可变借用。

### 7.4 `Weak` 不能直接当 `Rc` 用

`Weak<T>` 没有实现 `Deref`，不能直接调用 `T` 的方法。必须先 `upgrade()`，检查 `Option`，再进行操作：

片段（假设 `owner` 已经创建）：

```rust
use std::rc::Rc;

let owner = Rc::new(vec![1, 2, 3]);
let weak = Rc::downgrade(&owner);

if let Some(values) = weak.upgrade() {
    println!("长度: {}", values.len());
} else {
    println!("目标已经销毁");
}
```

不要把 `weak.upgrade().unwrap()` 当成默认写法。弱引用本来就允许目标消失；除非你的设计确实保证目标存在，否则应该处理 `None`。主练习的 `parent()` 方法明确要求处理失效情况。

## 八、引用计数环与所有权方向

### 8.1 一个能被观察到的泄漏

下面两个节点互相强引用：

```rust
use std::cell::RefCell;
use std::rc::{Rc, Weak};

struct Node {
    name: &'static str,
    next: RefCell<Option<Rc<Node>>>,
}

fn main() {
    let first = Rc::new(Node {
        name: "A",
        next: RefCell::new(None),
    });
    let second = Rc::new(Node {
        name: "B",
        next: RefCell::new(None),
    });

    *first.next.borrow_mut() = Some(Rc::clone(&second));
    *second.next.borrow_mut() = Some(Rc::clone(&first));

    let weak_first = Rc::downgrade(&first);

    println!("A 强计数: {}", Rc::strong_count(&first));
    println!("B 强计数: {}", Rc::strong_count(&second));

    drop(first);
    drop(second);

    // 外部强引用都没有了，但两个节点互相保活。
    println!("环里的 A 还能访问吗: {}", weak_first.upgrade().is_some());
}
```

输出：

```text
A 强计数: 2
B 强计数: 2
环里的 A 还能访问吗: true
```

为什么强计数是 2？每个节点有：

1. 外部变量 `first`/`second` 持有的一个强引用；
2. 对方节点 `next` 字段里的一个强引用。

`drop(first)` 只销毁外部变量，A 仍被 B 的 `next` 拥有；`drop(second)` 同理。两个节点的强计数都降到 1，永远不会到 0，所以堆上的值不会释放。`weak_first.upgrade()` 还能成功，直接证明了泄漏。

Rust 保证内存安全：你不会访问已经释放的内存。但 Rust **不保证**所有引用计数图都无泄漏。环是逻辑设计问题，编译器不会替你发现。

### 8.2 用 `Weak` 打断环

把其中一条边改成弱引用：

```rust
use std::cell::RefCell;
use std::rc::{Rc, Weak};

struct Node {
    name: &'static str,
    next: RefCell<Weak<Node>>,
}

fn main() {
    let first = Rc::new(Node {
        name: "A",
        next: RefCell::new(Weak::new()),
    });
    let second = Rc::new(Node {
        name: "B",
        next: RefCell::new(Weak::new()),
    });

    *first.next.borrow_mut() = Rc::downgrade(&second);
    *second.next.borrow_mut() = Rc::downgrade(&first);

    let weak_first = Rc::downgrade(&first);

    drop(first);
    drop(second);

    println!("环里的 A 还能访问吗: {}", weak_first.upgrade().is_some());
}
```

输出：

```text
环里的 A 还能访问吗: false
```

两个 `next` 都不保活，外部强引用销毁后强计数归零，节点被正常释放。注意弱引用不是“随便挑一条边换掉”：方向要符合领域里的所有权。

### 8.3 父子树的所有权方向

组织树、文件目录、界面控件树通常这样设计：

```text
父节点 ──Rc（强，拥有）──▶ 子节点
父节点 ◀──Weak（弱，观察）── 子节点
```

- 父节点强拥有子节点：父节点活着，子节点就应该活着；父节点销毁时，子节点也随之销毁（除非外部还持有子节点的 `Rc`）。
- 子节点弱引用父节点：子节点需要向上查找，但它的存在不应该让父节点无法销毁。

如果反过来（父弱子强），父节点可能在子节点还需要它时就被销毁；如果两条都强，就形成环。判断标准不是“哪个能跑通”，而是“谁在语义上拥有谁”。

### 8.4 环不一定只出现在两个节点之间

三个、四个甚至一个节点指向自己，都能形成环。主练习要求拒绝“把祖先挂到后代下面”和“把节点挂到自己下面”，正是因为这两种操作会让结构从树变成环。检测时不能只看直接父子关系，要沿父链一路向上比较身份。

## 九、完整示例：多窗口共享文档

下面这个程序把本课的主要工具串起来，场景和主练习不同：一个文档可以被多个“窗口”共享，一个“最近文档”列表只弱引用文档，因此不会阻止文档关闭。

```rust
use std::cell::RefCell;
use std::rc::{Rc, Weak};

struct Document {
    text: RefCell<String>,
}

impl Document {
    fn new(text: &str) -> Rc<Self> {
        Rc::new(Self {
            text: RefCell::new(text.to_owned()),
        })
    }

    fn append(&self, extra: &str) {
        self.text.borrow_mut().push_str(extra);
    }

    fn text(&self) -> String {
        self.text.borrow().clone()
    }
}

struct RecentList {
    documents: RefCell<Vec<Weak<Document>>>,
}

impl RecentList {
    fn new() -> Self {
        Self {
            documents: RefCell::new(Vec::new()),
        }
    }

    fn remember(&self, document: &Rc<Document>) {
        self.documents.borrow_mut().push(Rc::downgrade(document));
    }

    fn live_texts(&self) -> Vec<String> {
        let mut texts = Vec::new();
        for weak in self.documents.borrow().iter() {
            if let Some(document) = weak.upgrade() {
                texts.push(document.text());
            }
        }
        texts
    }
}

fn main() {
    let document = Document::new("第一版");

    // 两个窗口共享同一个文档。
    let editor = Rc::clone(&document);
    let viewer = Rc::clone(&document);

    editor.append("；第二版");
    println!("viewer 看到: {}", viewer.text());
    println!("强引用数量: {}", Rc::strong_count(&document));

    // 最近文档列表只保存弱引用。
    let recent = RecentList::new();
    recent.remember(&document);
    println!("最近文档: {:?}", recent.live_texts());

    // 关闭所有窗口和文档本体。
    drop(editor);
    drop(viewer);
    drop(document);

    println!("全部关闭后最近文档: {:?}", recent.live_texts());
}
```

输出：

```text
viewer 看到: 第一版；第二版
强引用数量: 3
最近文档: ["第一版；第二版"]
全部关闭后最近文档: []
```

这段代码用到几个前面课程的工具：`iter()` 逐个借用列表元素，`for ... in ...` 逐个处理，`if let Some(...) = ...` 只在有值时执行，`to_owned()` 把借来的 `&str` 复制成拥有型 `String`。它们分别在第三、四、五、十课讲解；不熟悉语法时可以先扫一眼，不影响理解本课的所有权主线。

按数据变化逐步看：

1. `Document::new` 返回一个 `Rc<Document>`，强计数为 1。`Document` 内部的 `text` 是 `RefCell<String>`，允许通过 `&self` 修改。
2. 两次 `Rc::clone(&document)` 让强计数变成 3：`document`、`editor`、`viewer` 各一个。
3. `editor.append(...)` 通过共享句柄修改文本。`borrow_mut()` 产生的 `RefMut` 在 `push_str` 语句结束时销毁。
4. `viewer.text()` 新建一个 `Ref`，读取文本后 `clone` 成拥有型 `String` 返回。返回的 `String` 不借用文档，所以之后可以继续修改文档。
5. `recent.remember(&document)` 只保存 `Weak<Document>`，强计数不变，仍是 3。
6. `recent.live_texts()` 依次 `upgrade` 每个弱引用；文档还活着，所以能取到文本。
7. 三个强引用都 drop 后，文档被销毁。`live_texts()` 中的 `upgrade()` 全部返回 `None`，结果为空 `Vec`。

这里每一层都有明确职责：

- `Rc<Document>`：窗口共同拥有文档；
- `Document.text: RefCell<String>`：通过共享句柄修改文本；
- `Weak<Document>`：最近列表观察文档，但不保活。

如果 `RecentList` 使用 `Rc<Document>`，那么即使所有窗口都关闭，最近列表仍会让文档活着，用户会觉得“关闭了却还占着内存”。这就是弱引用的实际价值。

## 十、编译错误、运行时错误与排错

本节展示的输出来自真实编译和运行。错误码（如 `E0596`）可以用 `rustc --explain E0596` 查看官方解释。

报错中的文件路径、行号和列号取决于你把代码放在文件中的哪个位置；真正要读的是错误码、错误消息和帮助信息。

### 10.1 E0596：不能通过 `Rc` 修改

```rust,compile_fail
use std::rc::Rc;

fn main() {
    let shared = Rc::new(String::from("rust"));
    shared.push('!');
}
```

```text
error[E0596]: cannot borrow data in an `Rc` as mutable
 --> src/main.rs:5:5
  |
5 |     shared.push('!');
  |     ^^^^^^ cannot borrow as mutable
  |
  = help: trait `DerefMut` is required to modify through a dereference, but it is not implemented for `Rc<String>`
```

`DerefMut` 是让一个类型可以通过 `*` 被当作可变引用使用的 trait；`Rc<String>` 只提供了只读的 `Deref`。

排查顺序：

1. 我真的需要多个主人吗？只有一个主人时，用 `Box` 或普通值，配 `&mut`。
2. 需要共享**读**，还是共享**写**？只读用 `Rc`；共享写才考虑 `Rc<RefCell<T>>`。
3. 如果确实共享写，把字段包进 `RefCell`，并在每次借用时控制凭证作用域。

### 10.2 E0507：不能把值从 `Rc` 里搬走

```rust,compile_fail
use std::rc::Rc;

fn main() {
    let shared = Rc::new(String::from("rust"));
    let owned = *shared;
}
```

```text
error[E0507]: cannot move out of an `Rc`
 --> src/main.rs:5:17
  |
5 |     let owned = *shared;
  |                 ^^^^^^^ move occurs because value has type `String`, which does not implement the `Copy` trait
  |
help: consider removing the dereference here
  |
5 -     let owned = *shared;
5 +     let owned = shared;
  |
help: consider cloning the value if the performance cost is acceptable
  |
5 -     let owned = *shared;
5 +     let owned = shared.clone();
```

原因：`Rc` 是共享所有权，不能允许某一个主人把里面的值移走，否则其他主人就看到一个空壳。编译器给出的两个建议含义不同：

- `let owned = shared;` 只是把 `Rc` 句柄移动给 `owned`，类型仍是 `Rc<String>`，不是 `String`；
- `shared.clone()` 克隆的是 `Rc`（加计数）。如果你确实想要独立的 `String`，应写 `(*shared).clone()`，并接受深拷贝成本。

### 10.3 运行时 panic：`RefCell already borrowed`

```rust,should_panic
use std::cell::RefCell;

fn main() {
    let value = RefCell::new(vec![1, 2, 3]);
    let shared = value.borrow();
    let mut exclusive = value.borrow_mut();
    exclusive.push(4);
}
```

```text
thread 'main' (126075) panicked at src/main.rs:6:31:
RefCell already borrowed
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

排查顺序：

1. 找到 panic 行上的 `borrow`/`borrow_mut`。
2. 向上找最近一个同格子的借用凭证：`let shared = ...`、`let mut exclusive = ...`，或者藏在临时表达式里的 `Ref`。
3. 看这个凭证最后一次使用在哪一行；把读取结果复制出来，或加花括号让凭证提前销毁。
4. 如果失败是正常业务流程（例如“正在被占用，稍后再试”），改用 `try_borrow_mut` 返回 `Result`。

### 10.4 E0277：`Rc` 不能跨线程

```rust,compile_fail
use std::cell::RefCell;
use std::rc::Rc;

let shared = Rc::new(RefCell::new(0));
let handle = std::thread::spawn(move || {
    *shared.borrow_mut() += 1;
});
handle.join().unwrap();
```

```text
error[E0277]: `Rc<RefCell<i32>>` cannot be sent between threads safely
   |
   = help: within `{closure@src/main.rs:6:37: 6:44}`, the trait `Send` is not implemented for `Rc<RefCell<i32>>`
```

`Send` 是一个标记 trait（没有方法的约定），表示“这个类型的值可以安全地转移到另一个线程”。`Rc` 的计数不是原子操作，所以没有实现 `Send`。第十七课会用 `Arc<Mutex<T>>`（原子引用计数 + 互斥锁，mutual exclusion，同一时间只允许一个线程进入临界区）解决同类问题；本课不要试图用 `unsafe` 绕过这条检查。

### 10.5 测试输出里的 `not yet implemented`

在脚手架上运行测试，行为测试会失败：

```text
thread 'exercise::tests::new_unit_has_no_parent_and_no_children' panicked at
lessons/15-smart-pointers/src/exercise.rs:52:9:
not yet implemented
```

`todo!()` 是 Rust 的宏（编译时生成代码的工具，见第二十课），它的运行时行为就是 panic 并打印 `not yet implemented`。这说明测试本身已经编译成功，只是你还没有替换对应的 `todo!()`。如果测试**编译失败**，那才是需要先检查签名或类型的问题。

### 10.6 遇到借用 panic 时的通用检查表

1. 这个 `RefCell` 是不是在同一个表达式里被借了两次？
2. 读借用是不是活到了写借用之后？把读的结果复制出来。
3. 有没有在循环里保存凭证，却在循环里再次借用？
4. 有没有把 `Ref`/`RefMut` 存进结构体，导致它长期不释放？
5. 这个共享可变状态是否真的必要？能不能改成传递 `&mut`、拆分字段或使用索引？

## 十一、代码预测

下面每道题都是片段（省略了 `fn main` 包装和部分前置代码），只用来判断行为。先写下你的判断和理由，再展开核对。

### 题目 1

```rust
use std::rc::Rc;

let owner = Rc::new(String::from("rust"));
let weak = Rc::downgrade(&owner);
let second = weak.upgrade().unwrap();
drop(owner);
println!("{second}");
```

`drop(owner)` 之后，字符串是否已经销毁？这段代码能正常打印吗？

<details><summary>解析</summary>

没有销毁，能正常打印 `rust`。`upgrade()` 成功时创建了一个新的强引用 `second`。`drop(owner)` 只把强计数从 2 降到 1，字符串仍由 `second` 保活。等 `second` 也销毁，字符串才释放。

</details>

### 题目 2

```rust
use std::cell::RefCell;

let cell = RefCell::new(vec![1]);
let first = cell.borrow_mut();
let second = cell.borrow_mut();
```

这段代码能编译吗？运行结果是什么？

<details><summary>解析</summary>

能编译，运行时会 panic，错误信息是 `RefCell already borrowed`。`first` 仍然持有独占借用，第二次 `borrow_mut()` 违反“同一时间恰好一个可变借用”的规则。改法：让 `first` 先销毁，或使用 `try_borrow_mut()` 得到 `Err`。

</details>

### 题目 3

```rust
use std::rc::Rc;

let original = Rc::new(String::from("data"));
let copy = Rc::clone(&original);
println!("{}", Rc::strong_count(&original));
drop(copy);
println!("{}", Rc::strong_count(&original));
```

两次打印分别是什么？`original` 和 `copy` 指向同一段文本吗？

<details><summary>解析</summary>

分别打印 `2` 和 `1`。`Rc::clone` 只增加强计数，不复制 `String`，两个句柄指向同一块堆分配。`Rc::ptr_eq(&original, &copy)` 为 `true`。

</details>

### 题目 4

```rust
use std::rc::Rc;

let mut value = Rc::new(10);
let other = Rc::clone(&value);
println!("{:?}", Rc::get_mut(&mut value));
```

打印什么？如果把 `let other = ...` 删掉呢？

<details><summary>解析</summary>

保留 `other` 时打印 `None`，因为有两个强引用，不能取得独占访问。删掉 `other` 后只剩一个强引用，`Rc::get_mut` 返回 `Some(&mut 10)`，`{:?}` 会打印类似 `Some(10)`。

</details>

### 题目 5

```rust
use std::cell::RefCell;
use std::rc::{Rc, Weak};

let parent = Rc::new(RefCell::new(Vec::<Rc<String>>::new()));
let child = Rc::new(String::from("child"));
parent.borrow_mut().push(Rc::clone(&child));
let weak_child: Weak<String> = Rc::downgrade(&child);
drop(child);
drop(parent);
println!("{}", weak_child.upgrade().is_some());
```

打印 `true` 还是 `false`？

<details><summary>解析</summary>

打印 `false`。`parent` 的 `Vec` 里有一个强引用，但 `drop(parent)` 会销毁 `Vec`，其中的 `Rc<String>` 也随之销毁。外部 `child` 已经先 drop，强计数归零，字符串释放，弱引用 `upgrade()` 返回 `None`。

如果把 `parent` 的 `Vec` 换成 `Vec<Weak<String>>`（只存弱引用），结果也一样是 `false`，因为弱边本来就不保活。

</details>

### 题目 6

```rust
use std::rc::Rc;

let root = Rc::new(String::from("root"));
let child = Rc::new(String::from("child"));
let grandchild = Rc::new(String::from("grandchild"));
let _links = (Rc::clone(&root), Rc::clone(&child), Rc::clone(&grandchild));
drop(root);
drop(child);
drop(grandchild);
```

最后还有几个强引用？字符串什么时候释放？

<details><summary>解析</summary>

`_links` 里还有三个强引用，所以三个字符串都还活着。`_links` 在 `main` 结束时销毁，三个强计数同时归零，字符串才释放。这个例子说明：变量离开作用域的顺序和“最后一个主人”是谁，都可以通过 `drop` 和句柄数量来推理。

</details>

## 十二、主练习：共享组织树

### 12.1 目标与场景

练习代码在 `src/exercise.rs`。你要实现一个组织树 `Unit`：

- 每个部门有名称、父部门和直接子部门；
- 父部门通过 `Rc` 强拥有子部门；
- 子部门通过 `Weak` 指向父部门；
- 外部变量可以同时持有任意部门的 `Rc`；
- 通过 `&Rc<Unit>` 这样的共享句柄添加子部门，因此子列表和父指针都必须支持内部可变性。

类型已经定义好：

```rust
use std::cell::RefCell;
use std::rc::{Rc, Weak};

pub struct Unit {
    name: String,
    parent: RefCell<Weak<Unit>>,
    children: RefCell<Vec<Rc<Unit>>>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum LinkError {
    AlreadyHasParent,
    DuplicateChild,
    WouldCreateCycle,
}
```

字段保持私有，外部只能通过方法访问。你需要补完五个方法：

| 方法 | 作用 |
| --- | --- |
| `Unit::new(name) -> Rc<Self>` | 创建独立部门，返回唯一强引用 |
| `Unit::add_child(&Rc<Self>, Rc<Self>) -> Result<(), LinkError>` | 建立父子链接，失败时保持原图不变 |
| `parent(&self) -> Option<Rc<Unit>>` | 返回仍存活的父部门 |
| `child_names(&self) -> Vec<String>` | 按加入顺序返回直接子部门名称 |
| `path_from_root(&self) -> Vec<String>` | 返回从最高可访问祖先到自己的名称路径 |

`Unit::name(&self) -> &str` 已经提供，不需要你实现；它只是返回名称的只读借用。

### 12.2 真实输入与期望输出

创建三个部门：

```text
root     = Unit::new("Engineering")
platform = Unit::new("Platform")
ada      = Unit::new("Ada")

Unit::add_child(&root, Rc::clone(&platform))     -> Ok(())
Unit::add_child(&platform, Rc::clone(&ada))      -> Ok(())

root.child_names()        -> ["Platform"]
platform.child_names()    -> ["Ada"]
ada.parent() 的名称        -> "Platform"
ada.path_from_root()      -> ["Engineering", "Platform", "Ada"]
```

此时计数关系：

```text
Rc::strong_count(&platform) -> 2    // 外部变量 platform + root 的子列表
Rc::weak_count(&root)       -> 1    // platform 的父指针
Rc::weak_count(&platform)   -> 1    // ada 的父指针
Rc::strong_count(&ada)      -> 2    // 外部变量 ada + platform 的子列表
```

继续使用同一张图，错误操作必须返回精确错误，并且不能修改已有链接：

```text
Unit::add_child(&root, Rc::clone(&platform))     -> Err(DuplicateChild)

research = Unit::new("Research")
Unit::add_child(&research, Rc::clone(&platform)) -> Err(AlreadyHasParent)

Unit::add_child(&ada, Rc::clone(&root))          -> Err(WouldCreateCycle)
Unit::add_child(&ada, Rc::clone(&ada))           -> Err(WouldCreateCycle)
```

最后一个“自己挂自己”返回 `WouldCreateCycle`，因为成环检查优先于“已有存活父节点”检查。完整优先级：

1. 已经是同一父部门的直接子部门：`DuplicateChild`；
2. 自身挂载，或把祖先挂到后代下面：`WouldCreateCycle`；
3. 子部门仍有其他存活的父部门：`AlreadyHasParent`。

销毁外部根句柄时，外部仍持有的节点继续存在，弱引用链在失效处停止：

```text
drop(root)
platform.parent()          -> None
platform.path_from_root()  -> ["Platform"]
ada.path_from_root()       -> ["Platform", "Ada"]

drop(platform)
ada.parent()               -> None
ada.path_from_root()       -> ["Ada"]
```

旧父部门销毁后，允许把该子部门重新挂到另一个父部门。单独节点的路径只包含自己的名称；没有子部门时 `child_names()` 返回空 `Vec`。

### 12.3 约束

- 父部门通过 `Rc` 强拥有子部门，子部门通过 `Weak` 指向父部门；不能反转这两条边。
- 节点身份由 `Rc` 是否指向同一分配决定（`Rc::ptr_eq`），不能按名称判重；同名但不同的节点是合法节点。
- `parent()` 必须处理弱引用已失效的情况，不能直接 `upgrade().unwrap()`。
- `add_child` 失败时不得新增、删除或改写任何已有父子链接。
- 不允许使用 `unsafe`、全局变量或 `Box::leak`。
- 所有 `borrow`/`borrow_mut` 凭证都要有清晰作用域，正常操作和错误分支都不能发生运行时借用 panic。
- 只替换 `src/exercise.rs` 各方法体中的 `todo!()`；不要修改类型定义、方法签名和文件底部的测试。

### 12.4 推荐完成顺序

1. `Unit::new`：让 `new_unit_has_no_parent_and_no_children` 先能运行。
2. `parent` 和 `child_names`：读路径先打通。
3. `path_from_root`：先处理“没有父节点”的情况，再沿父链向上收集。
4. `add_child` 的重复检查与成功写入：先让“第一次挂载”和“重复挂载”通过。
5. `add_child` 的成环检查：沿准备接收子节点的父部门向上走，比较身份。
6. `add_child` 的存活父节点检查：放在最后，确保优先级正确。
7. 重新运行父部门销毁、断链路径和重新挂载相关测试。

### 12.5 测试说明与验收命令

`src/exercise.rs` 底部已有 16 个测试：

- 2 个是独立概念测试（`rc_clone_shares_one_allocation`、`weak_reference_does_not_keep_value_alive`），在脚手架上就会通过；
- 14 个是行为测试，实现前会在对应的 `todo!()` 处以 `not yet implemented` 失败。

`src/consolidation.rs` 另有 12 个行为测试，实现前也会因 `todo!()` 失败。因此完整测试套件共 28 个测试：初始状态 2 个通过，26 个失败。失败来自未实现，不是测试写错。

按顺序运行：

```bash
cargo fmt -p l15_smart_pointers -- --check
cargo check -p l15_smart_pointers --all-targets
cargo test -p l15_smart_pointers
cargo clippy -p l15_smart_pointers --all-targets -- -D warnings
```

只想先跑主练习时可以过滤测试名：

```bash
cargo test -p l15_smart_pointers exercise
```

完成标准：上述四条命令全部通过；`cargo run -p l15_smart_pointers` 能打印组织树演示；你能解释为什么 `children` 与 `parent` 都需要内部可变性，以及 `Rc<RefCell<T>>` 相比“`Vec` 加索引”的代价。

卡住时看 `HINTS.md` 的主练习三级提示。

## 十三、课后巩固：只弱引用订阅者的消息频道

### 13.1 目标与场景

主练习管理父子树，这个任务换成“一个频道 + 多个订阅者”。代码在 `src/consolidation.rs`，数据和边界都不同：

- 频道只保存订阅者的**弱引用**，所以订阅者不会因为“还在频道里”而无法销毁；
- 订阅检查要区分“同一个 `Rc` 重复登记”和“同名但不同身份”；
- 已销毁订阅者留下的空登记不能阻止同名新订阅者加入；
- 发布消息时只统计并投递给仍存活的订阅者。

已有类型：

```rust
use std::cell::RefCell;
use std::rc::{Rc, Weak};

pub struct Subscriber {
    name: String,
    inbox: RefCell<Vec<String>>,
}

pub struct Channel {
    name: String,
    subscribers: RefCell<Vec<Weak<Subscriber>>>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum SubscribeError {
    AlreadySubscribed,
    DuplicateName,
}
```

你需要补完：

| 函数 | 作用 |
| --- | --- |
| `Subscriber::new(name) -> Rc<Self>` | 创建订阅者，收件箱为空 |
| `Subscriber::inbox(&self) -> Vec<String>` | 按收到顺序返回消息副本 |
| `Channel::new(name) -> Rc<Self>` | 创建空频道 |
| `subscribe(&Rc<Channel>, &Rc<Subscriber>) -> Result<(), SubscribeError>` | 登记订阅，检查身份和名称 |
| `publish(&Rc<Channel>, &str) -> usize` | 给所有存活订阅者投递，返回人数 |
| `subscriber_count(&Rc<Channel>) -> usize` | 存活订阅者数量 |
| `subscriber_names(&Rc<Channel>) -> Vec<String>` | 存活订阅者名称，按登记顺序 |

`Subscriber::name` 和 `Channel::name` 已经提供，不需要你实现。

### 13.2 真实输入与期望输出

正常流程：

```text
channel = Channel::new("releases")
ada     = Subscriber::new("Ada")
lin     = Subscriber::new("Lin")

subscribe(&channel, &ada)        -> Ok(())
subscribe(&channel, &lin)        -> Ok(())
subscriber_count(&channel)       -> 2
subscriber_names(&channel)       -> ["Ada", "Lin"]

publish(&channel, "v1")          -> 2
ada.inbox()                      -> ["v1"]
lin.inbox()                      -> ["v1"]

publish(&channel, "v2")          -> 2
ada.inbox()                      -> ["v1", "v2"]
lin.inbox()                      -> ["v1", "v2"]
```

重复登记与同名冲突：

```text
subscribe(&channel, &ada)        -> Err(AlreadySubscribed)   // 同一个 Rc
other = Subscriber::new("Ada")
subscribe(&channel, &other)      -> Err(DuplicateName)      // 不同 Rc，同名
subscriber_count(&channel)       -> 2                       // 失败不改变频道
```

销毁与重新加入：

```text
drop(lin)
subscriber_count(&channel)       -> 1
subscriber_names(&channel)       -> ["Ada"]
publish(&channel, "v3")          -> 1
ada.inbox()                      -> ["v1", "v2", "v3"]

drop(ada)
new_ada = Subscriber::new("Ada")
subscribe(&channel, &new_ada)    -> Ok(())   // 旧登记已失效，不构成同名冲突
subscriber_count(&channel)       -> 1
publish(&channel, "v4")          -> 1
new_ada.inbox()                  -> ["v4"]
```

空频道：

```text
empty = Channel::new("empty")
subscriber_count(&empty)         -> 0
subscriber_names(&empty)         -> []
publish(&empty, "hello")         -> 0
```

关键边界：

- `subscribe` 的检查优先级：先看同一个 `Rc`，再看存活同名者。
- 已销毁订阅者的弱引用登记不算“已登记”，也不能阻止同名新人。
- `publish` 的返回值是**实际收到消息的人数**，不是登记条数。
- 每个订阅者收到独立的 `String` 副本；之后修改传入的文本不会影响已投递消息。
- 频道不保活订阅者：订阅后 `Rc::strong_count(&subscriber)` 仍为 1，`Rc::weak_count` 为 1。

### 13.3 约束、顺序与验收

- 只替换 `src/consolidation.rs` 各函数体中的 `todo!()`；不要修改类型、签名和测试。
- 频道必须保存 `Weak`，不能改成 `Rc`；否则 `channel_does_not_keep_subscriber_alive` 会失败，也违背任务目标。
- `publish` 和两个查询函数跳过失效弱引用，不 panic。
- 失败时不能修改频道内容。
- 建议顺序：先 `Subscriber::new`/`inbox` 和 `Channel::new`，再 `subscribe` 的两项检查，再 `publish`，最后 `subscriber_count`/`subscriber_names`。
- 验收命令：

```bash
cargo test -p l15_smart_pointers consolidation
cargo clippy -p l15_smart_pointers --all-targets -- -D warnings
```

巩固任务卡住时看 `HINTS.md` 的“课后巩固”三级提示。

## 十四、常见误区

- **把 `Rc::clone` 当成深拷贝。** `Rc::clone(&x)` 只加计数，两个句柄指向同一份数据；要独立副本必须克隆底层值，例如 `(*x).clone()`。
- **以为 `Rc` 可以随便改。** `Rc` 只给共享读。共享写需要 `RefCell` 或其他内部可变性容器，并接受运行时检查的成本。
- **把 `RefCell` 当成“关闭借用检查”。** 规则没有消失，只是从编译期移到运行期；重叠借用仍然会 panic。
- **让 `Ref`/`RefMut` 活得太久。** 常见于把读借用存进变量、跨循环使用，或在同一个长表达式里再次借用。让凭证在语句或小块内结束。
- **对 `Weak` 直接 `unwrap`。** 弱引用允许目标消失；除非设计保证目标存在，否则要处理 `None`。
- **只把一条边改成 `Weak` 来“解决环”。** 先确定领域里的所有权方向。父强子、子弱父是常见方向；反过来可能让父节点提前销毁。
- **按名称判断节点身份。** 两个不同部门可以同名；身份应比较 `Rc::ptr_eq`。
- **为了绕过借用检查而到处包 `Rc<RefCell<T>>`。** 先尝试缩短借用、拆分字段、传递 `&mut`；集中存储加索引往往更简单。
- **把 `Rc<RefCell<T>>` 用到多线程。** 它没有实现 `Send`，编译器会阻止；跨线程应使用第十七课的 `Arc` 和锁。
- **忘记 `drop(root)` 的真实含义。** 销毁外部强引用不等于销毁节点：父部门或别的主人还可能持有它；强计数归零才销毁值。

## 十五、本课总结

1. `Box<T>` 是独占所有权；`Rc<T>` 是单线程共享所有权，clone 只增加强计数。
2. 强计数降到 0 时值被销毁；`Weak<T>` 不参与保活，使用前必须 `upgrade()` 并处理 `None`。
3. `Rc<T>` 默认只允许共享读；共享写需要 `RefCell<T>`，它把借用规则移到运行时。
4. `RefCell` 的 `borrow`/`borrow_mut` 返回借用凭证；凭证销毁时借用结束，重叠借用会 panic，需要可恢复处理时用 `try_borrow*`。
5. 内部可变性适合缓存、日志、单线程图和 `&self` 接口；它不该成为绕过所有借用错误的默认手段。
6. `Rc<RefCell<T>>` 能表达共享可变状态，但每一层都增加失败模式和推理成本。
7. 强引用环会导致泄漏。用领域所有权方向决定哪条边用 `Rc`、哪条边用 `Weak`。
8. `Rc`/`RefCell` 不是线程安全工具；跨线程需要 `Arc` 和同步原语。
9. 能用普通所有权、`&mut`、`Vec` 加索引表达的关系，通常比指针图更易测试和维护。

## 十六、课后复盘

用自己的话回答，答不出就回对应小节重读：

1. 为什么一个部门不能同时用 `Box` 被父部门拥有、又被外部变量长期持有？
2. `Rc::strong_count` 和 `Rc::weak_count` 分别在哪一刻变化？强计数归零时发生什么？
3. `Rc::clone(&x)` 与 `(*x).clone()` 的结果类型和复制成本有什么不同？
4. `RefCell` 和普通 `&mut` 都要求“独占”，它们检查的时机和失败方式有什么不同？
5. 什么情况下适合通过 `&self` 修改内部状态？举一个本课之外的例子。
6. 为什么父子树通常让父强拥有子、子弱引用父？反过来会怎样？
7. 你怎样用 `Weak` 观察一个引用计数环是否泄漏？
8. 主练习中 `add_child` 的三个错误为什么有固定优先级？失败时为什么不能先写入再回滚？
9. 如果组织树有十万个节点，`Rc<RefCell<Unit>>` 可能带来哪些成本？`Vec<Unit>` 加索引会怎样改变设计？

## 下一课预告

本课的所有工具都限定在单线程：`Rc` 的计数不是原子操作，`RefCell` 的借用状态也没有线程同步。下一课[第十六课：线程、`move` 闭包与 channel](../16-threads-channels/README.md)开始处理并发：用 `std::thread::spawn` 创建线程，用 `move` 闭包把数据所有权交给新线程，再用 channel 在所有者之间传递消息。你会看到“共享所有权”在跨线程时为什么需要另一套工具。
