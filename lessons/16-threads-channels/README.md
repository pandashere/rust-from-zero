# 第十六课：线程、`move` 闭包与 channel

## 本课定位

前十五课的程序都只有一条执行路线：从 `main` 开始，按顺序调用函数，遇到循环就重复，遇到 `return` 就结束。这一课让程序同时拥有多条执行路线，并学习如何在这些路线之间安全地传递数据。

具体来说，本课解决三个问题：

1. 怎样启动一条新的执行路线，并等它结束、取回结果；
2. 新路线需要的数据怎样交给它，才不会出现“数据已经销毁，线程还在使用”的错误；
3. 多条路线之间怎样传递消息，而不是同时改同一块内存。

本课只使用标准库（Rust 安装时自带的代码库）的 `std::thread` 和 `std::sync::mpsc`，不添加第三方依赖（别人写的、需要额外下载的库）。共享可变状态（多个线程同时修改同一个值）留到[下一课](../17-shared-concurrency/README.md)。

## 前置知识

你不需要学过任何其他编程语言。下面这些概念在之前的课里出现过，本课会反复用到，先各用一句话复习：

- **所有权（ownership）**：Rust 用一套规则决定“哪段代码负责释放一份数据”。一个值通常只有一个所有者；所有者离开作用域时，值被释放。详见[第三课](../03-ownership-borrowing/README.md)。
- **移动（move）**：把一个值的所有权从一个位置交给另一个位置。移动之后，原来的变量不能继续使用。详见[第三课](../03-ownership-borrowing/README.md)。
- **借用（borrow）**：不取得所有权，只临时读取（`&T`）或修改（`&mut T`）一个值。详见[第三课](../03-ownership-borrowing/README.md)。
- **闭包（closure）**：能记住并使用周围变量的匿名函数，例如 `|x| x + 1`。详见[第九课](../09-closures/README.md)。
- **`Option<T>`**：表示“可能有值，也可能没有值”，要么是 `Some(值)`，要么是 `None`。详见[第四课](../04-option-result/README.md)。
- **`Result<T, E>`**：表示“操作可能成功，也可能失败”，要么是 `Ok(值)`，要么是 `Err(错误)`。详见[第四课](../04-option-result/README.md)。
- **生命周期（lifetime）**：引用能合法使用的时间范围。详见[第十四课](../14-lifetimes/README.md)。
- **`panic!`**：程序遇到无法继续的错误时主动中止当前执行路线的机制。测试里的 `todo!()` 也会以 panic 的形式报告“还没实现”。

如果上面有概念已经完全想不起来，先回到对应课复习；本课会在用到时再简短解释一次。

---

## 一、程序为什么需要“多条执行路线”

### 1.1 线程是什么

**线程（thread）**是程序内部一条独立的执行路线。可以把程序想象成一条正在读指令的队伍：单线程程序只有一条队伍，读完一条指令再读下一条；多线程程序有多条队伍，操作系统（管理硬件和程序运行的系统软件）决定某一刻让哪条队伍使用 CPU（中央处理器，真正执行指令的硬件）。

一个**进程（process）**是一次程序运行所占的整体资源，例如打开的文件、申请到的内存（程序运行时存放数据的区域）。同一进程内的多个线程共享大部分内存，但每个线程有自己的**栈（stack）**。栈是存放函数调用现场和局部变量的区域；函数返回后，它的栈空间就被回收。

“共享大部分内存”是线程强大的原因，也是危险的来源：两个线程可能同时读写同一个位置。**数据竞争（data race）**指两个线程同时访问同一块内存、至少一个在写、并且没有同步措施。数据竞争会让程序读到一半新一半旧的数据，甚至崩溃。

Rust 的一个重要目标是：**安全的 Rust 代码不允许数据竞争**。Rust 编译器（把源码翻译成可执行程序的工具）会检查跨线程传递的类型是否满足 `Send`（下一节解释），也会检查借用是否可能比数据活得更久。但编译器不能阻止所有并发错误：死锁（线程互相等待，谁也不前进）、逻辑顺序错误、线程太多导致变慢，仍然需要程序员自己判断。

### 1.2 并发与并行不是同一个词

- **并发（concurrency）**：多个任务在时间上交错推进。一个 CPU 核心（CPU 内部真正执行指令的单元）也能并发：操作系统快速切换线程，让每个线程都前进一点。
- **并行（parallelism）**：多个任务真的在同一时刻由多个 CPU 核心执行。

本课说“并行处理”时，重点是让多个线程分别处理独立的数据块；具体能不能同时在多个 CPU 核心上跑，由操作系统和硬件决定。写代码时不要依赖“哪个线程先跑”。

### 1.3 线程适合做什么

适合交给线程的任务通常有这些特点：

- 任务之间依赖少，可以独立计算；
- 单个任务的计算量足够大，创建线程的成本可以被摊薄；
- 数据可以清晰地分块，每块只需要自己的那一份；
- 需要等待外部资源（例如网络、磁盘）时，可以让等待不堵住其他工作。

反例：把“给每个整数加一”拆成一千个线程。创建和切换线程本身有成本，这样通常比一个普通循环更慢。本课练习会让固定数量的工作线程反复处理多个任务，而不是“一个任务一个线程”。

---

## 二、创建并等待线程

标准库用 `std::thread::spawn` 启动新线程。`spawn` 的英文意思是“产卵、催生”，这里指“催生一条新线程”。它接收一个闭包，闭包里是这条线程要执行的代码。

最小的完整程序：

```rust
use std::thread;

fn main() {
    let handle = thread::spawn(|| {
        println!("工作线程开始运行");
        2 + 3
    });

    println!("主线程可以继续做别的事");
    let result = handle.join().unwrap();
    println!("工作线程返回 {result}");
}
```

逐行解释：

- `use std::thread;` 把标准库的线程模块引入当前文件，之后可以写 `thread::spawn`。
- `thread::spawn(|| { ... })` 启动新线程。`||` 是“没有参数”的闭包写法；闭包最后一行 `2 + 3` 是它的返回值。
- `spawn` 返回一个 `JoinHandle<T>`。`JoinHandle` 可以理解为“线程的取票凭证”：拿着它就能等待那条线程结束，并取回它返回的 `T`。这里的 `T` 是 `i32`，因为 `2 + 3` 的类型被推断为 `i32`。
- `handle.join()` 让当前线程**阻塞**（暂停），直到目标线程结束。正常结束时返回 `Ok(线程返回值)`；目标线程 panic 时返回 `Err(panic 携带的值)`。
- `unwrap()` 是“如果是 `Ok` 就取出里面的值，如果是 `Err` 就 panic”。示例为了简短这样写；真实程序应根据情况处理错误。

`i32` 是 32 位有符号整数（可以为负）。本课后面还会用到 `u8`（`0` 到 `255` 的字节）、`u32`、`u64`、`usize`（表示长度或索引的整数）、`i64` 和 `i128`；第一次出现时都会说明用途。

输出顺序不固定：主线程可能先打印“主线程可以继续做别的事”，也可能工作线程先打印。操作系统调度（决定哪个线程何时运行）不保证顺序。**不要用 `println!` 的先后顺序推断线程完成顺序**。

一个容易忽略的规则：如果 `main` 直接结束，整个进程会退出，尚未结束的其他线程不保证完成。需要结果或必须完成的工作，要保存 `JoinHandle` 并 `join`。

### 2.1 `join` 的错误不是业务错误

`join` 返回的 `Result` 有两层含义：

```rust
use std::thread;

fn main() {
    let handle = thread::spawn(|| {
        panic!("工作线程遇到了无法继续的问题");
    });

    match handle.join() {
        Ok(()) => println!("工作线程正常结束"),
        Err(payload) => println!("工作线程 panic 了，携带值：{payload:?}"),
    }

    println!("主线程仍然活着");
}
```

`Err` 里的 `payload` 是 panic 时携带的值，常见类型是 `&str` 或 `String`，被装在一个可以跨线程传递的盒子里。主线程不会因为工作线程 panic 就自动跟着 panic；它只是从 `join` 得到一个 `Err`。

处理策略要按场景选择：

- 工作线程失败意味着整个程序状态不可信：重新 panic，让问题显式暴露；
- 失败可以恢复：把 `join` 的 `Err` 转换成自定义错误，返回给调用者；
- 还有其他独立任务：记录失败，继续处理其他任务；
- 服务型程序：可能需要重启工作线程。

本课练习采用第二种思路：把“线程 panic”转换成结构化的 `PipelineError::WorkerPanicked`，而不是在每个 `join()` 上盲目 `unwrap()`。

---

## 三、线程闭包为什么通常要写 `move`

### 3.1 先看不写 `move` 会发生什么

下面的程序看起来“马上就 join，应该没问题”，但编译不通过：

```rust,compile_fail
use std::thread;

fn main() {
    let numbers = vec![1, 2, 3];
    let handle = thread::spawn(|| {
        println!("{numbers:?}");
    });
    handle.join().unwrap();
}
```

真实诊断（省略了部分重复的路径信息）：

```text
error[E0373]: closure may outlive the current function, but it borrows `numbers`,
              which is owned by the current function
 --> src/main.rs:5:32
  |
5 |     let handle = thread::spawn(|| {
  |                                ^^ may outlive borrowed value `numbers`
6 |         println!("{numbers:?}");
  |                    ------- `numbers` is borrowed here
  |
note: function requires argument type to outlive `'static`
 --> src/main.rs:5:18
  |
5 |       let handle = thread::spawn(|| {
  |  __________________^
...
help: to force the closure to take ownership of `numbers` (and any other
      referenced variables), use the `move` keyword
  |
5 |     let handle = thread::spawn(move || {
  |                                ++++
```

关键句是 `closure may outlive the current function`：闭包可能比当前函数活得更久。`thread::spawn` 的接口必须对**任意**调用都安全，它不能假设调用者一定会在函数返回前 join。因此闭包不能借用当前函数栈上的局部变量，因为函数返回后那块栈空间就失效了。

### 3.2 `move` 强制按值捕获

在闭包前面加 `move`：

```rust
use std::thread;

fn main() {
    let numbers = vec![1, 2, 3];
    let handle = thread::spawn(move || numbers.iter().sum::<i32>());
    let total = handle.join().unwrap();
    println!("总和是 {total}");
}
```

**捕获（capture）**指闭包使用外部变量。`move || ...` 的意思是：闭包捕获外部变量时，**按值**取得它们，而不是借用。`numbers` 的所有权进入闭包，再随闭包进入新线程。线程结束时，`numbers` 在线程里被释放。

代价是主线程不能再使用 `numbers`：

```rust,compile_fail
use std::thread;

fn main() {
    let numbers = vec![1, 2, 3];
    let handle = thread::spawn(move || numbers.len());
    println!("{numbers:?}"); // 错误：numbers 已经移动进闭包
    let _ = handle.join();
}
```

真实诊断：

```text
error[E0382]: borrow of moved value: `numbers`
 --> src/main.rs:6:16
  |
4 |     let numbers = vec![1, 2, 3];
  |         ------- move occurs because `numbers` has type `Vec<i32>`,
  |                 which does not implement the `Copy` trait
5 |     let handle = thread::spawn(move || numbers.len());
  |                                ------- ------- variable moved due to use in closure
  |                                |
  |                                value moved into closure here
6 |     println!("{numbers:?}");
  |                ^^^^^^^ value borrowed here after move
  |
help: consider cloning the value before moving it into the closure
  |
5 ~     let value = numbers.clone();
6 ~     let handle = thread::spawn(move || value.len());
```

编译器建议 `clone()`（显式复制一份的方法）是一种选择，但不是默认答案。先问：主线程真的还需要这份数据吗？

- 不需要：让它移动，减少复制和借用上的复杂度；
- 只需要一小部分信息：先取出可复制的元数据（例如长度、编号），再移动原数据；
- 确实需要两边都保留：考虑 `clone`，或者让线程把数据还回来；
- 只是想在函数返回前并行处理借用数据：用下一节的 `thread::scope`，通常比克隆更合适。

### 3.3 `move` 不是“把所有东西都复制一份”

`move` 只改变**捕获方式**：按值捕获。按值捕获一个 `String` 会移动它；按值捕获一个 `u32` 会复制它，因为 `u32` 实现了 `Copy`（赋值时自动复制的标记 trait）。

```rust
use std::thread;

fn main() {
    let name = String::from("worker");
    let count = 3_u32;

    let handle = thread::spawn(move || format!("{name}-{count}"));
    println!("{}", handle.join().unwrap());

    // count 是 Copy 类型，移动进闭包的是副本，所以这里仍然可以使用。
    println!("count 仍然是 {count}");
}
```

`name` 被移动，`count` 被复制。`move` 闭包本身仍然可能是 `Fn`、`FnMut` 或 `FnOnce`，取决于闭包内部怎样使用捕获的值；`move` 不决定这一点。

### 3.4 `'static` 不等于“线程永远运行”

`thread::spawn` 的要求可以简写为：

```text
闭包：Send + 'static
返回值：Send + 'static
```

`'static` 是一个生命周期（引用能合法使用的时间范围）名称，表示“引用指向的数据在整个程序运行期间都有效”。对 `thread::spawn` 来说，它实际排除的是：闭包携带指向当前函数栈帧的短期引用。

一个拥有所有权的 `String`、`Vec<T>` 通常满足 `'static`，即使它在线程结束时马上被销毁。“满足 `'static`”说的是它内部没有短期借用，不是说它永远不会被释放。

不要为了让编译通过而把局部数据泄漏成“永远有效”的引用。绝大多数情况下，正确做法是移动拥有型数据，或者使用下面的作用域线程。

### 3.5 `Send`：能不能跨线程转移

`Send` 是一个 **trait**（行为接口，表示某类类型共同具备的能力）。实现了 `Send` 的类型可以安全地从一个线程转移到另一个线程。整数、`String`、`Vec<T>`（当 `T: Send` 时）通常都实现了 `Send`。

`Rc<T>`（引用计数智能指针，第十五课）**没有**实现 `Send`。它的计数更新不是线程安全的；如果两个线程同时克隆或销毁同一个 `Rc`，计数可能出错。把 `Rc` 移进线程会得到真实诊断：

```rust,compile_fail
use std::rc::Rc;
use std::thread;

fn main() {
    let shared = Rc::new(5);
    let copy = Rc::clone(&shared);
    thread::spawn(move || {
        println!("{copy}");
    })
    .join()
    .unwrap();
}
```

```text
error[E0277]: `Rc<i32>` cannot be sent between threads safely
 --> src/main.rs:7:19
  |
7 |       thread::spawn(move || {
  |       ------------- ^------
  |       |             |
  |  _____|_____________within this closure
  | |     |
  | |     required by a bound introduced by this call
...
  | |_____^ `Rc<i32>` cannot be sent between threads safely
  |
  = help: within this closure, the trait `Send` is not implemented for `Rc<i32>`
note: required because it's used within this closure
note: required by a bound in `spawn`
  |
128 |     F: Send + 'static,
  |        ^^^^ required by this bound in `spawn`
```

下一课会学习 `Arc<T>`：它是可以在多线程间共享所有权的引用计数指针，但共享修改仍然需要锁。本课先走“每个线程拥有自己的数据，通过消息传递结果”的路线。

---

## 四、作用域线程：借用当前函数的数据

### 4.1 问题：只想并行读一段数据，不想复制

假设有一个大 `Vec`，想把它切成两半，两个线程分别求和。用 `thread::spawn` 会遇到所有权问题：要么把整个 `Vec` 移进一个线程（另一个线程就拿不到另一半），要么为每个线程克隆一份数据（浪费内存）。

Rust 标准库提供 `std::thread::scope`：它创建一个**作用域**（一段有明确开始和结束的范围），并保证作用域结束前，其中创建的所有线程都已经结束。因为编译器知道线程不会活过作用域，所以这些线程可以安全地借用作用域外、但仍活着的局部数据。

完整示例：

```rust
use std::thread;

fn main() {
    let numbers = vec![1, 2, 3, 4, 5, 6];
    let (left, right) = numbers.split_at(3);

    let total = thread::scope(|scope| {
        let left_handle = scope.spawn(|| left.iter().sum::<i32>());
        let right_handle = scope.spawn(|| right.iter().sum::<i32>());

        left_handle.join().unwrap() + right_handle.join().unwrap()
    });

    assert_eq!(total, 21);
    println!("总和是 {total}");
    println!("原数据仍然可用：{numbers:?}");
}
```

逐项解释：

- `thread::scope(|scope| { ... })` 接收一个闭包，闭包的参数 `scope` 是“作用域凭证”。在闭包内部，用 `scope.spawn(...)` 启动线程。
- `scope.spawn` 返回 `ScopedJoinHandle<T>`，用法和 `JoinHandle` 类似，也有 `join()`。
- 两个线程分别借用 `left` 和 `right` 这两段切片。`split_at` 返回的是原 `Vec` 的两个只读借用，没有复制数据。
- 闭包最后一行是两个 `join` 结果相加，这个值成为 `thread::scope` 的返回值。
- 作用域结束前会等待所有线程结束，所以借用不会悬垂（指向已经失效的数据）。`numbers` 在作用域之后仍然可用。

### 4.2 什么时候用 `spawn`，什么时候用 `scope`

选择时看任务的职责和结束边界：

- 线程的职责是“接手一份数据，独立活下去”，主线程不需要在函数返回前等它：用 `thread::spawn`，把数据 `move` 进去。
- 线程只是“在当前函数返回前，帮忙并行处理一段借用数据”：用 `thread::scope`，不要克隆整份数据。

`thread::scope` 不是“更高级所以更好”的写法。它适合有明确结束边界的并行计算；如果任务要长期运行、脱离当前函数，仍然应该用 `spawn`。

### 4.3 作用域线程中的 panic

`scope.spawn` 返回的 handle 调用 `join()` 时，同样会得到 `Result`：线程正常结束是 `Ok`，panic 是 `Err`。如果你不处理某个作用域线程的 panic，作用域结束时也可能把 panic 继续传播给调用者。初学阶段最清楚的做法是：显式保存并 `join` 每个 handle，按需要处理 `Err`。

---

## 五、channel：通过传递消息共享数据

### 5.1 为什么需要 channel

多个线程协作时，一种办法是让它们同时读写同一块内存。但这需要锁（一种同一时刻只允许一个线程进入某段代码的机制）、原子操作（不可被其他线程打断的读改写）等同步（让多个线程按确定的先后关系协作）工具，否则就是数据竞争。另一种办法是**消息传递**：一个线程把数据打包成消息，放进一个队列；另一个线程从队列取出。数据的所有权随消息转移，同一时刻只有一个线程拥有它，因此不需要共享可变内存。

**channel（通道）**就是这样的队列。标准库的 `std::sync::mpsc` 提供 **mpsc** 通道：

- **m**ulti-**p**roducer：多个发送者可以往同一个通道发消息；
- **s**ingle-**c**onsumer：只有一个接收者从通道取消息。

发送消息的一方叫**生产者（producer）**，接收消息的一方叫**消费者（consumer）**。

### 5.2 第一个 channel 程序

```rust
use std::sync::mpsc;
use std::thread;

fn main() {
    let (sender, receiver) = mpsc::channel::<String>();

    let producer = thread::spawn(move || {
        sender.send(String::from("第一块数据")).unwrap();
        sender.send(String::from("第二块数据")).unwrap();
    });

    for message in receiver {
        println!("收到：{message}");
    }

    producer.join().unwrap();
}
```

逐项解释：

- `mpsc::channel::<String>()` 创建一个通道，消息类型是 `String`。它返回一个元组：发送端 `Sender<String>` 和接收端 `Receiver<String>`。
- `sender.send(value)` 把 `value` 的所有权放进通道。成功返回 `Ok(())`；如果接收端已经销毁，返回 `Err(SendError(value))`，错误里带着没送出去的值。
- `receiver` 用 `for` 循环逐个接收。`Receiver` 可以直接迭代；迭代会在**所有发送端都销毁**、并且队列里的消息都取完之后结束。
- `sender` 被 `move` 进工作线程。线程结束时，这个发送端被销毁。主线程没有保留其他发送端，所以接收循环最终会结束。
- `producer.join().unwrap()` 等待工作线程结束，确认它没有 panic。

### 5.3 发送之后，原变量不能再用

`send` 会移动值。下面的程序编译失败：

```rust,compile_fail
use std::sync::mpsc;

fn main() {
    let (sender, receiver) = mpsc::channel::<String>();
    let message = String::from("hi");
    sender.send(message).unwrap();
    println!("{message}"); // 错误：message 已经移动进通道
    drop(receiver);
}
```

真实诊断：

```text
error[E0382]: borrow of moved value: `message`
 --> src/main.rs:7:16
  |
5 |     let message = String::from("hi");
  |         ------- move occurs because `message` has type `String`,
  |                 which does not implement the `Copy` trait
6 |     sender.send(message).unwrap();
  |                 ------- value moved here
7 |     println!("{message}");
  |                ^^^^^^^ value borrowed here after move
  |
help: consider cloning the value if the performance cost is acceptable
  |
6 |     sender.send(message.clone()).unwrap();
  |                        ++++++++
```

这不是限制，而是消息传递安全性的基础：发送者把值交出去之后，不能再无同步地修改同一个值。接收者拿到的是唯一的所有权。

补充一个后面会反复出现的函数：`drop(value)` 会立即销毁一个值，释放它占用的资源；对 `Sender` 来说，销毁意味着“这个发送端不会再发送消息”。

### 5.4 接收端的四种常用方式

| 方法 | 行为 | 什么时候用 |
|---|---|---|
| `recv()` | 阻塞等待下一条消息；所有发送端销毁且队列为空时返回 `Err(RecvError)` | 确定还有消息，愿意等 |
| `try_recv()` | 立即返回 `Ok(消息)`、`Err(TryRecvError::Empty)`（暂时没有消息）或 `Err(TryRecvError::Disconnected)`（所有发送端都已销毁） | 不想阻塞，先做别的事 |
| `recv_timeout(时长)` | 最多等指定时间；超时返回 `Err(RecvTimeoutError::Timeout)`（等待时间用完） | 需要防止永久等待 |
| `iter()` / `for 消息 in receiver` | 持续接收，直到通道断开 | 消费者要处理完整个消息流 |

一个常见错误是：

```rust
use std::sync::mpsc;

fn main() {
    let (sender, receiver) = mpsc::channel::<i32>();
    drop(sender);
    let value = receiver.recv().unwrap();
    println!("{value}");
}
```

这里发送端已经销毁，通道断开，`recv()` 立刻返回 `Err(RecvError)`，`unwrap()` 会 panic。`recv()` 的 `Err` 不一定表示“程序坏了”，它也可能只是“没有更多消息了”。要根据协议决定这是正常结束还是错误。

---

## 六、多生产者与关闭协议

### 6.1 克隆发送端

`Sender<T>` 可以克隆。每个工作线程拿一个克隆，向同一个接收端发消息：

```rust
use std::sync::mpsc;
use std::thread;

fn main() {
    let (sender, receiver) = mpsc::channel::<(usize, &'static str)>();

    let mut handles = Vec::new();
    for worker_id in 0..3 {
        let worker_sender = sender.clone();
        handles.push(thread::spawn(move || {
            worker_sender.send((worker_id, "完成")).unwrap();
        }));
    }

    drop(sender);

    let mut messages: Vec<_> = receiver.iter().collect();
    messages.sort_by_key(|(worker_id, _)| *worker_id);

    for handle in handles {
        handle.join().unwrap();
    }

    for (worker_id, text) in messages {
        println!("worker {worker_id}: {text}");
    }
}
```

关键点：

- `sender.clone()` 创建一个新的发送端，原 `sender` 仍然存在。
- 每个线程 `move` 自己那个克隆；线程结束时克隆被销毁。
- 主线程在启动完所有线程后执行 `drop(sender)`。这一步表示“主线程不会再发送消息了”。
- `receiver.iter()` 只有在**所有**发送端都销毁后才结束。如果主线程忘记 `drop(sender)`，接收循环会在收完消息后继续等待，程序看起来“卡住”。

### 6.2 消息到达顺序不是任务顺序

多个线程同时发送时，消息进入通道的先后顺序由调度决定。上例中，worker 2 的消息完全可能比 worker 0 先到。因此：

- 不要用接收顺序代表业务顺序；
- 每条消息携带任务编号（例如 `worker_id` 或任务 `id`）；
- 收集后按编号排序，或者放进按编号索引的结构。

本课主练习会要求结果按任务 `id` 升序返回，而不是按消息到达顺序返回。

### 6.3 关闭 channel 是协议的一部分

“最后一个发送端销毁”是接收端判断“不会再有消息”的唯一依据。因此，关闭发送端不是可选的清理细节，而是并发协议的一部分：

- 每个工作线程结束时会销毁自己的发送端克隆；
- 主线程如果也持有发送端，必须在合适的时刻 `drop` 它；
- 接收端的 `iter()` 或 `recv()` 只有在所有发送端都销毁后才会报告结束。

如果程序在接收循环里永久等待，第一件事就是列出所有 `Sender` 的拥有者，确认该销毁的是不是都销毁了。

---

## 七、有界 channel 与背压

`mpsc::channel()` 创建的通道是**无界**的：发送不会因为队列满而等待。如果生产者比消费者快很多，消息会在内存里越积越多，程序可能耗尽内存。

`mpsc::sync_channel(capacity)` 创建**有界**通道，容量由 `capacity` 指定：

```rust
use std::sync::mpsc;
use std::thread;

fn main() {
    let (sender, receiver) = mpsc::sync_channel::<u32>(2);

    let producer = thread::spawn(move || {
        for value in 1..=5 {
            sender.send(value).unwrap();
            println!("发送 {value}");
        }
    });

    for value in receiver {
        println!("接收 {value}");
    }

    producer.join().unwrap();
}
```

当通道里的消息达到容量上限时，`send` 会阻塞，直到消费者取走一条消息腾出空间。这形成**背压（backpressure）**：消费者的处理速度会反过来限制生产者的速度。对于“生产很快、消费很慢”的流水线，有界通道能防止内存无限增长。

容量为 `0` 时，通道没有缓冲区（通道里暂时存放消息的空间），每次 `send` 都必须和一次 `recv` 直接会合；发送者会等到接收者准备好。这种“手递手”模型适合严格同步，但如果同一个线程既发送又接收，很容易死锁。

选择建议：

- 消息量小、生产消费速度接近：普通 `channel()` 足够简单；
- 生产可能远快于消费：用 `sync_channel` 限制积压；
- 需要多个消费者：标准库 `mpsc` 的接收端不能克隆，需要额外的共享或分片设计；本课不展开。

---

## 八、完整示例：多生产者日志汇总

下面是一个完整程序：三批单词分别由三个线程发送到同一个通道，主线程统计每个单词出现的次数，并用 `BTreeMap` 按单词排序输出。

`BTreeMap` 是按键排序的映射（键值表）；第六课讲过 `HashMap`，它不保证遍历顺序。这里用 `BTreeMap` 是为了让输出稳定、方便比较。

```rust
use std::collections::BTreeMap;
use std::sync::mpsc;
use std::thread;

fn main() {
    let batches = vec![
        vec!["rust", "thread"],
        vec!["channel", "rust"],
        vec!["move"],
    ];

    let (sender, receiver) = mpsc::channel::<&'static str>();

    let mut handles = Vec::new();
    for batch in batches {
        let batch_sender = sender.clone();
        handles.push(thread::spawn(move || {
            for word in batch {
                batch_sender.send(word).unwrap();
            }
        }));
    }

    drop(sender);

    let mut counts = BTreeMap::new();
    for word in receiver {
        *counts.entry(word).or_insert(0) += 1;
    }

    for handle in handles {
        handle.join().unwrap();
    }

    for (word, count) in counts {
        println!("{word}: {count}");
    }
}
```

输出：

```text
channel: 1
move: 1
rust: 2
thread: 1
```

数据流：

1. `batches` 是一个 `Vec<Vec<&'static str>>`，三批单词。
2. 每次循环把一批 `batch` 移动进新线程；`batch_sender` 是该线程专属的发送端克隆。
3. 主线程在循环后 `drop(sender)`，表示自己不再发送。
4. 三个工作线程各自发送完单词后结束，它们的发送端克隆随之销毁。
5. 所有发送端都销毁后，`for word in receiver` 结束。
6. `counts.entry(word).or_insert(0)` 在单词第一次出现时插入 `0`，然后 `+= 1` 计数。
7. `BTreeMap` 的遍历按单词升序，所以输出顺序稳定。

这个例子和主练习的结构不同：主练习是“主线程分发任务，工作线程把结果送回”，本例子是“多个生产者主动发送，主线程汇总”。两者都依赖同一个原则：**发送端的所有者全部销毁，接收端才知道流结束**。

---

## 九、编译错误与诊断

并发代码的编译错误通常指向所有权或线程安全约束。下面列出本课最常见的几类，并说明怎样读。

### 9.1 `closure may outlive the current function`

完整诊断见 3.1 节。读法：

- 第一行告诉你闭包借用了 `numbers`，而 `numbers` 属于当前函数；
- `note: function requires argument type to outlive 'static` 说明 `spawn` 要求闭包满足 `'static`；
- `help` 建议加 `move`。

修复方向：如果线程应该拥有数据，加 `move`；如果只是想借用当前函数的数据并在返回前完成，改用 `thread::scope`。

### 9.2 `borrow of moved value`

诊断见 3.2 和 5.3 节。读法：

- `move occurs because ... does not implement the Copy trait`：这个类型移动后原变量失效；
- `value moved into closure here` / `value moved here`：所有权在哪一行被交出去；
- `value borrowed here after move`：哪一行还在使用已经交出的变量。

修复方向：先确认是否真的需要两边都使用。需要就克隆、提取元数据，或让线程把值还回来；不需要就删除旧变量后面的使用。

### 9.3 `Rc<i32> cannot be sent between threads safely`

诊断见 3.5 节。读法：

- `the trait Send is not implemented for Rc<i32>`：这个类型不允许跨线程转移；
- `required by a bound in spawn`：`spawn` 的泛型参数要求 `Send`。

修复方向：单线程共享用 `Rc`，多线程共享用 `Arc`（下一课）；如果只是想把数据交给一个线程，确认是否可以用拥有型数据代替共享指针。

### 9.4 接收循环永不结束

这不是编译错误，而是运行时“卡住”。排查顺序：

1. 列出所有 `Sender` 的拥有者：主线程、每个工作线程、是否还有克隆藏在某个结构里？
2. 确认主线程的原始 `Sender` 是否在接收前 `drop`；
3. 确认每个工作线程都会结束，而不是在等另一条永远不会来的消息；
4. 用 `try_recv()` 或 `recv_timeout()` 做诊断，观察通道是“空”还是“断开”。

不要用 `sleep` 来“等它完成”。`sleep` 只让当前线程暂停一段时间，不建立任何“另一条线程已经完成”的保证。

### 9.5 测试失败但代码“看起来能跑”

并发测试失败常见原因是依赖了不确定的顺序：

- 假设 worker 0 的消息一定先到；
- 假设 `join` 的顺序等于线程完成顺序；
- 用固定的 `sleep` 等待任务完成；
- 只运行一次，碰巧通过。

正确做法是依赖明确同步：`join()`、通道关闭、任务编号、结果数量、排序后的集合比较。超时可以用来防止测试永久挂起，但不能代替正确同步。

---

## 十、代码预测

先自己在纸上或注释里写出答案，再展开对照。

### 预测 1：移动之后还能用吗

```rust
use std::thread;

fn main() {
    let data = vec![1, 2, 3];
    let handle = thread::spawn(move || data.len());
    let len = handle.join().unwrap();
    println!("{len}");
}
```

线程结束后，主线程能否重新使用原来的 `data`？

<details><summary>解析</summary>

不能。`data` 已经移动进闭包，并随闭包进入工作线程；线程结束时它在线程里被释放。`join` 只返回闭包的结果 `usize`，不会自动把捕获的值还回来。若确实需要归还，可以让闭包返回 `(data, len)`，由主线程接收这个元组。

</details>

### 预测 2：这个循环会结束吗

```rust,ignore
use std::sync::mpsc;
use std::thread;

fn main() {
    let (sender, receiver) = mpsc::channel();
    let worker_sender = sender.clone();
    thread::spawn(move || worker_sender.send(1).unwrap());

    for value in receiver {
        println!("{value}");
    }
}
```

<details><summary>解析</summary>

不会正常结束。收到 `1` 之后，主线程仍然持有原始 `sender`，所以通道没有断开，`for` 循环继续等待下一条消息。应在接收循环前 `drop(sender)`，并在完整程序中保存、`join` 工作线程。

</details>

### 预测 3：容量为 0 的通道

```rust
use std::sync::mpsc;
use std::thread;

fn main() {
    let (sender, receiver) = mpsc::sync_channel::<i32>(0);

    let producer = thread::spawn(move || {
        sender.send(7).unwrap();
        println!("发送完成");
    });

    let value = receiver.recv().unwrap();
    println!("收到 {value}");
    producer.join().unwrap();
}
```

`send(7)` 会立刻返回吗？

<details><summary>解析</summary>

不会。容量为 `0` 表示没有缓冲区，`send` 必须等到有一次 `recv` 与它会合。主线程执行 `recv()` 后，`send(7)` 才能完成，生产者随后打印“发送完成”。如果生产者在同一个线程里先 `send` 再 `recv`，就会永远等不到会合，形成死锁。

</details>

### 预测 4：作用域结束后数据还在吗

```rust
use std::thread;

fn main() {
    let values = vec![1, 2, 3];
    let total = thread::scope(|scope| {
        scope.spawn(|| values.iter().sum::<i32>()).join().unwrap()
    });
    println!("{total} {}", values.len());
}
```

<details><summary>解析</summary>

可以。`thread::scope` 在返回前等待作用域内所有线程结束，所以线程对 `values` 的借用不会超过 `values` 的生命周期。作用域结束后 `values` 仍然可用。这里没有复制 `Vec`。

</details>

### 预测 5：工作线程 panic 后主线程会怎样

```rust
use std::thread;

fn main() {
    let handle = thread::spawn(|| panic!("boom"));
    let result = handle.join();
    println!("join 是否成功：{}", result.is_ok());
    println!("主线程继续运行");
}
```

<details><summary>解析</summary>

工作线程 panic，`join()` 返回 `Err`，`result.is_ok()` 为 `false`。主线程不会自动 panic，它会继续执行后面的打印。注意：panic 的详细信息默认会打印到终端；如果你在 `join` 后 `unwrap()`，主线程才会跟着 panic。

</details>

---

## 十一、常见误区

1. **“`join` 的顺序就是线程完成的顺序。”** 不是。`join` 只是“等这一个 handle”；你先 join 哪个，就先等哪个。线程实际完成的先后由调度决定。
2. **“`move` 会把所有变量都复制一份。”** 不是。`move` 只改变捕获方式；`Copy` 类型被复制，非 `Copy` 类型被移动。
3. **“`'static` 表示数据永远不会释放。”** 不是。它表示数据内部没有短期借用；拥有型数据可以在线程结束时正常释放。
4. **“channel 会保证所有发送者的消息按某个全局顺序到达。”** 不会。跨发送者的到达顺序不确定；需要稳定顺序就携带编号并在接收后排序。
5. **“`drop(sender)` 只是清理，不影响逻辑。”** 它直接影响接收端何时结束。少 drop 一个发送端，接收循环就可能永久等待。
6. **“`sleep` 可以代替同步。”** 不能。`sleep` 只是延迟，不证明另一条线程已经完成。用 `join`、通道关闭或明确的同步原语。
7. **“线程越多越快。”** 创建和切换线程有成本；任务太小时，多线程可能比串行更慢。
8. **“无界 channel 没有代价。”** 生产快、消费慢时，消息会在内存里堆积。需要限制积压时用 `sync_channel`。
9. **“`Rc` 加个 `move` 就能跨线程。”** 不能。`Rc` 没有实现 `Send`，编译器会拒绝。多线程共享所有权需要 `Arc`（下一课）。
10. **“工作线程 panic 会让整个进程立刻退出。”** 通常不会。panic 先结束那条线程；主线程通过 `join` 得到 `Err`。是否让整个程序失败，由你决定。

---

## 十二、主练习：多工作线程校验流水线

### 12.1 场景

一个程序收到若干块字节数据。每块数据有一个调用者指定的任务编号 `id`，需要独立计算校验值。数据块之间没有依赖，可以把它们分给固定数量的工作线程同时处理；主线程负责分发任务、收集结果，并按任务编号恢复稳定顺序。

代码在 `src/exercise.rs`。你需要完成：

- `checksum`：校验算法；
- `run_pipeline_with`：线程、channel、分发、收集、清理和错误处理。

`run_pipeline` 已经写好，它只是把默认的 `checksum` 包装成计算步骤再调用 `run_pipeline_with`，不需要修改。

### 12.2 数据类型

```rust
pub struct Task {
    pub id: usize,
    pub bytes: Vec<u8>,
}

pub struct Checksum {
    pub id: usize,
    pub value: u64,
    pub worker_id: usize,
}

pub enum PipelineError {
    ZeroWorkers,
    WorkerPanicked { worker_id: usize },
    ResultCountMismatch { expected: usize, actual: usize },
}
```

- `Task::id`：任务编号，用来在结果乱序到达时恢复顺序。
- `Task::bytes`：原始字节。所有权随任务一起交给工作线程，不能为了“省事”对每个任务做深拷贝。
- `Checksum::id`：原样来自对应任务的 `id`。
- `Checksum::value`：对 `Task::bytes` 计算出的校验值。
- `Checksum::worker_id`：实际处理该任务的工作线程编号，从 `0` 开始，按工作线程创建顺序分配。

类型小抄：`usize` 用来表示长度和索引；`u64` 是 64 位无符号整数（不能为负，最大值约 `1.8×10^19`）；`u8` 是一个字节（`0` 到 `255`）；`i64` 和 `i128` 是 64 位、128 位有符号整数（可以为负）。`Vec<u8>` 就是“一串字节”。

### 12.3 校验算法

从 `0_u64` 开始，按输入顺序处理每个字节：

```text
acc = acc.wrapping_mul(31).wrapping_add(byte as u64)
```

`wrapping` 的意思是“回绕”：结果超出 `u64` 能表示的范围时，丢弃高位，从另一端继续，而不是 panic。空块返回 `0`。

真实输入与期望输出：

```text
checksum([])          -> 0
checksum([1, 2, 3])   -> 1026
checksum([255])       -> 255
checksum([255; 13])   -> 4634958717477794239
checksum(b"learn rust") -> 2944394269393984
```

`[255; 13]` 表示“13 个 255”；`b"learn rust"` 是字节字符串，每个字符按 ASCII（把常见英文字符编号为 0 到 127 的标准）写成 `u8`；`0_u64` 表示类型为 `u64` 的 `0`。`[255; 13]` 的中间结果已经超过普通 `u64` 乘加能表示的范围。如果在 debug 构建（开发时默认的编译模式，保留整数溢出检查）里使用普通 `*` 和 `+`，这里会 panic；必须使用回绕运算。

### 12.4 流水线输入与输出

创建四个任务，使用两个 worker：

```text
tasks = [
    Task { id: 8,  bytes: [1, 2, 3] },
    Task { id: 3,  bytes: [] },
    Task { id: 5,  bytes: [255] },
    Task { id: 12, bytes: b"learn rust" },
]
worker_count = 2
```

任务按它在输入 `Vec` 中的**位置**轮流分配，而不是按 `id` 分配：

```text
输入位置 0 / id 8  -> worker 0
输入位置 1 / id 3  -> worker 1
输入位置 2 / id 5  -> worker 0
输入位置 3 / id 12 -> worker 1
```

返回时按 `id` 升序排列：

```text
Ok([
    Checksum { id: 3,  value: 0,                  worker_id: 1 },
    Checksum { id: 5,  value: 255,                worker_id: 0 },
    Checksum { id: 8,  value: 1026,               worker_id: 0 },
    Checksum { id: 12, value: 2944394269393984,   worker_id: 1 },
])
```

运行 `cargo run -p l16_threads_channels` 时，`main.rs` 会打印：

```text
流水线结果（按任务 ID 排序）：
任务  3: 校验值                    0，处理者 worker 1
任务  5: 校验值                  255，处理者 worker 0
任务  8: 校验值                 1026，处理者 worker 0
任务 12: 校验值     2944394269393984，处理者 worker 1

零工作线程的错误演示：
得到预期的错误：ZeroWorkers
```

边界行为：

```text
run_pipeline([], 3)          -> Ok([])
run_pipeline([任意任务], 0)   -> Err(ZeroWorkers)
```

- 只有一个 worker 时，所有结果的 `worker_id` 都是 `0`。
- worker 多于任务时，仍然按输入位置从 worker 0 开始分配；空闲 worker 应正常结束。
- 测试中的任务 ID 唯一；重复 ID 的行为不在本练习保证范围内。

### 12.5 可替换的计算步骤

`run_pipeline_with` 的第三个参数是：

```rust,ignore
work: fn(&[u8]) -> Option<u64>
```

这是一个**函数指针**：只保存一段函数代码的位置，不携带周围变量，因此可以复制给每个工作线程，不需要共享所有权。`Option<u64>` 表示：

- `Some(value)`：这个任务产生一条结果；
- `None`：这个任务不产生结果，工作线程继续处理后面的任务。

正常流水线通过 `run_pipeline` 使用 `Some(checksum(bytes))`。测试会用会 panic 的函数和总是返回 `None` 的函数，分别检查两条错误路径。这不是“为测试而测试”：把变化的部分作为参数传入，能让同一套线程协议复用于不同计算步骤。

### 12.6 并发协议与错误要求

- `worker_count == 0` 时立即返回 `ZeroWorkers`，不启动线程。
- 为每个 worker 创建独立的任务 channel；主线程按 round-robin（轮流分配）发送任务。
- 所有 worker 共享结果 channel 的发送端克隆；每个 worker 处理完自己的任务后退出。
- 每个 `Task` 的 `Vec<u8>` 必须随 `Task` 移动给 worker，不得调用 `bytes.clone()` 做深拷贝（把整块数据复制一份，而不是只复制指向它的引用）。
- 原始结果发送端和所有任务发送端都必须关闭，接收循环才有确定的结束条件。
- 关闭任务发送端后收集结果，再 `join` 每个已经创建的 worker。
- 返回结果按 `id` 升序排列，不能依赖调度或消息到达顺序。
- 任一 worker panic 时返回 `WorkerPanicked { worker_id }`；多个线程 panic 时报告编号最小的那个。
- 所有 worker 都 `join` 后，如果结果数量与任务数量不同，返回 `ResultCountMismatch { expected, actual }`。
- 任务发送失败时不能直接提前返回而遗留其他未 `join` 的 worker；应先完成关闭与清理，再返回适当错误。
- 错误优先级：`ZeroWorkers` 最先检查；其次检查 worker panic；最后检查结果数量。也就是说，如果既 panic 又数量不足，返回 `WorkerPanicked`。

### 12.7 约束

- 只使用标准库的 `std::thread` 和 `std::sync::mpsc`，不引入第三方线程池（预先创建一组线程并重复使用）或 channel。
- 不使用 `Arc`（可跨线程共享所有权的引用计数指针）、`Mutex`（互斥锁，同一时刻只允许一个线程访问）、`RwLock`（读写锁）或全局共享状态；它们是下一课的主题。
- 不使用 `unsafe`（需要程序员额外保证安全的高级代码块）、全局变量（整个程序都能访问的变量）或 `Box::leak`（故意让一块内存永不释放）。
- 不用 `sleep` 猜测任务何时完成，也不用“多跑几次碰运气”制造顺序。
- 正常路径和错误路径都不能让接收端因仍存活的 sender 永久等待。

### 12.8 推荐实现顺序

1. 完成 `checksum`，先通过空块、普通块和回绕测试。
2. 画出任务 channel、结果 channel 以及每个 sender 的所有者。
3. 启动固定数量 worker，确保每个 worker 在任务 channel 关闭后退出。
4. 分发任务、关闭 sender、收集结果并 `join`。
5. 最后检查 panic、数量不匹配、排序和空任务边界。

卡住时按顺序看本课[HINTS.md](HINTS.md)的三级提示。

### 12.9 初始状态与验收命令

初始状态下，脚手架已经通过格式检查、编译和严格 Clippy。运行全部测试会看到 **2 个概念测试通过，22 个行为测试在 `todo!()` 处失败**。这是预期现象。

```bash
cargo fmt -p l16_threads_channels -- --check
cargo check -p l16_threads_channels --all-targets
cargo test -p l16_threads_channels --no-run
cargo clippy -p l16_threads_channels --all-targets -- -D warnings
cargo test -p l16_threads_channels
cargo run -p l16_threads_channels
```

完成后：

1. 全部 24 个测试通过；
2. `cargo run -p l16_threads_channels` 输出的流水线结果与 12.4 节一致；
3. 严格 Clippy 没有警告；
4. 能用自己的话回答：哪些值被移动到哪个线程、为什么结果必须带任务 ID、哪些 sender 必须在何时销毁、`join` 的错误与业务错误有什么区别。

---

## 十三、课后巩固：并行范围统计

主练习让工作线程拥有任务数据，并通过 channel 送回结果。这个任务换一种数据（有符号整数）和一种借用方式：输入切片由调用者拥有，工作线程只在函数返回前借用它。它练习 `thread::scope` 的适用场景。

代码在 `src/consolidation.rs`。你需要完成 `parallel_stats`。

### 13.1 数据类型

```rust
pub struct Stats {
    pub count: usize,
    pub min: i64,
    pub max: i64,
    pub sum: i128,
}
```

- `count`：参与统计的整数个数；
- `min` / `max`：最小值和最大值；
- `sum`：全部整数的总和，使用 `i128`，避免很多 `i64` 相加时溢出。

### 13.2 行为与例子

```text
parallel_stats([], 3)              -> None
parallel_stats([-5], 4)            -> Some(Stats { count: 1, min: -5, max: -5, sum: -5 })
parallel_stats([3, -1, 7, 7, 0], 2)
    -> Some(Stats { count: 5, min: -1, max: 7, sum: 16 })
```

边界和约定：

- 输入为空返回 `None`，因为最小值、最大值都不存在；
- `workers == 0` 按 `1` 个工作线程处理，不 panic；
- `workers` 大于整数个数时也能工作，多出来的线程不应改变结果；
- 重复出现的整数按位置分别计数，例如 `[4, 4, 4]` 的 `count` 是 `3`；
- 所有整数都是负数时，最小值和最大值仍要正确处理符号；
- 参数是只读切片，函数只借用数据，不复制整个 `Vec`；
- 工作线程 panic 时让 panic 传播，不要伪装成正常结果。

### 13.3 约束

- 必须使用 `std::thread::scope` 借用 `values` 的分段。
- 不要为每个工作线程克隆一份完整输入。
- 不需要 channel；这个任务的重点是作用域线程的借用。
- 不使用 `unsafe`、全局变量或第三方依赖。

### 13.4 验收命令

```bash
cargo test -p l16_threads_channels consolidation
```

初始状态下，这个模块的 8 个测试都会在 `todo!()` 处失败；完成实现后应全部通过。卡住时看[HINTS.md](HINTS.md)的“课后巩固”部分。

---

## 十四、本课总结

- **线程**是程序内部独立的执行路线；操作系统调度它们，创建和切换都有成本。
- `thread::spawn` 启动可能独立于当前函数运行的线程，要求闭包和返回值满足 `Send + 'static`。
- `move` 闭包按值捕获环境：非 `Copy` 值被移动，`Copy` 值被复制。
- `'static` 表示没有短期借用，不表示数据永远不会释放。
- `JoinHandle::join` 等待线程结束，返回 `Ok(返回值)` 或 `Err(panic 值)`；工作线程 panic 不会自动让主线程 panic。
- `thread::scope` 在作用域结束前等待所有线程，允许线程安全借用当前函数的局部数据。
- `mpsc` channel 通过转移所有权传递消息；`Sender` 可克隆，`Receiver` 只有一个。
- 所有发送端销毁后，接收端才知道消息流结束；`drop(sender)` 是协议动作。
- 跨发送者的消息顺序不确定；需要稳定顺序时携带任务编号并在接收后排序。
- `sync_channel` 有容量上限，能形成背压，防止生产过快导致内存增长。
- 并发测试依赖 `join`、通道关闭、任务编号和排序，而不是 `sleep` 或运行次数。

---

## 下一课

消息传递能避免很多共享状态，但有些计数器、缓存和索引确实需要多个线程共同访问。下一课学习 [`Arc<T>`、`Mutex<T>`、`RwLock<T>`、`Send` 与 `Sync`](../17-shared-concurrency/README.md)（`Sync` 表示一个值可以安全地被多个线程同时引用）：怎样在多个线程之间共享所有权，并用锁把“同时修改”变成“轮流修改”。
