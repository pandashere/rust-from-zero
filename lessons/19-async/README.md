# 第十九课：`async`/`.await`、`Future` 与运行时边界

一个程序向外发送 50 条通知。如果按老办法一条一条地等，每条都要等对方服务器回应，50 条就是 50 段等待时间的总和。可是这段时间里，我们的程序几乎什么计算都没做——它只是在等。

**异步（asynchronous，简称 async）** 要解决的就是这件事：让一段代码在“等”的时候先让开，把执行机会交给别的代码，等条件满足后再回来继续。

Rust 给出的机制叫 `async fn` 与 `.await`。它们看上去像语法糖，背后却是一套明确的模型：**future（未来值）**、**poll（轮询）** 和一个负责反复推进任务的 **executor（执行器）**。本课从零解释这三个词，然后动手写一个有界并发、带超时和重试的调度器。

## 本课定位

### 你将学会

- 解释 `async fn` 调用后为什么“什么都没发生”。
- 用自己的话说明 `Future`、`poll`、`Poll::Pending`、`Poll::Ready` 与 `Waker` 的关系。
- 区分并发（concurrency）、并行（parallelism）、线程（thread）与异步任务。
- 判断两段 `.await` 是顺序等待还是并发等待。
- 说明运行时（runtime）与执行器（executor）各自负责什么，以及为什么标准库不提供完整运行时。
- 用“poll 次数”而不是真实时间来表达超时，从而写出可重复的异步测试。
- 区分可重试错误与不可重试错误，并说明重试为什么不能写成裸循环。
- 用“分批”限制同时在跑的作业数量，并解释取消（取消就是丢弃 future）时资源如何释放。
- 把 future 里的 panic 转成结构化错误，而不是让它炸掉整个调度器。

### 前置知识

本课默认你已经完成前十八课。需要用到时会再用一两句话复习并给出链接。

- 变量、函数、`main`：见[第一课](../01-basics/README.md)。
- struct（结构体）、enum（枚举）、`impl` 方法：见[第二课](../02-data-modeling/README.md)。复习：`enum` 的每个变体可以携带不同的数据，`match` 用来按变体分别处理。
- 所有权、借用、`&T` 与 `&mut T`：见[第三课](../03-ownership-borrowing/README.md)。本课会大量依赖“借用不能活得比被借的数据更久”这条规则。
- `Option`、`Result`、`?`：见[第四课](../04-option-result/README.md)。复习：`Result<T, E>` 表示“要么成功得到 `T`，要么失败得到 `E`”；`?` 在失败时提前返回。
- 迭代器与闭包：见[第九课](../09-closures/README.md)和[第十课](../10-iterators/README.md)。复习：闭包是能捕获外部变量的匿名函数；迭代器是逐个产出元素的“流水线”。
- 泛型与 trait（特征，描述“具备哪些行为”的接口）：见[第十一课](../11-generics/README.md)和[第十二课](../12-traits/README.md)。复习：`fn f<T: Trait>(x: T)` 的意思是“`f` 对任何实现了 `Trait` 的类型都可用”。
- `Box`（把值放到堆上）：见[第十三课](../13-box-recursion/README.md)。
- `Drop`（值被销毁时自动执行的清理）：见[第十五课](../15-smart-pointers/README.md)。
- 线程、`move` 闭包、`'static`（数据能活到程序结束）：见[第十六课](../16-threads-channels/README.md)。本课会反复对比“线程”和“异步任务”。
- `Send` 与 `Sync`（能安全地跨线程移动 / 共享）：见[第十七课](../17-shared-concurrency/README.md)。

### 本课边界

本课**不**使用 Tokio 或其他第三方运行时，原因见 6.1 节：我们要亲眼看 `poll`，并且让测试不依赖真实时间来等待。因此本课不涉及真实网络、真实定时器、`tokio::select!`、`Arc<Semaphore>` 或跨线程任务调度。

真实项目里请使用成熟运行时。学完本课你会知道它替你做了什么，以及哪些坑仍然要自己躲。

## 如何学习本课

1. 先只读第一到第六节，建立“future 是被推着走的”这个直觉。
2. 运行仓库里的两个可运行示例（第二节和第八节会告诉你命令），对照输出去读代码。
3. 读第十四节的真实编译错误。异步代码的报错信息初看很绕，但模式非常固定。
4. 先做「代码预测」的几道小题，再动手做 `src/exercise.rs` 的主练习。
5. 主练习的三个函数按 README 给的顺序实现：`with_timeout` → `send_with_retries` → `run_batch`。前一个不通，后一个很难写对。

## 一、从一个具体问题说起：等待很贵

假设程序要发 3 条通知，每次发送要等对方回应 200 毫秒（毫秒是千分之一秒）。

**办法一：一条一条地等。** 总耗时 600 毫秒。在这段实际经过的时间里，CPU（中央处理器，真正执行计算的硬件）几乎都在空闲等待，而不是忙着计算。

**办法二：每条开一个操作系统线程。** 线程（thread）是操作系统调度的执行单位，可以真正同时干活。这确实能缩短总耗时，但：

- 每个线程都要占用栈空间（操作系统预先分配的一块内存）和内核里的记录，常以兆字节计；
- 1 万条通知就对应 1 万个线程，内存和调度开销会先压垮程序；
- 线程之间的切换由操作系统决定，程序无法精确控制。

**办法三：一条线程上跑很多“会等的任务”。** 让每个任务在等待时**主动让开**，执行器立刻去推进别的任务。这样 1 万条通知只需要很少的线程和很少的内存。

办法三就是异步。它换来的能力是：**用极低的内存代价同时管理大量“大部分时间在等”的任务**。

> 注意“同时管理”不等于“同时计算”。做大量 CPU 计算时，异步不会让计算变快；那是并行（多个执行单元同一时刻各算各的）要解决的问题。第十七课讲的多线程才是并行的工具。

### 1.1 异步不涉及的东西

- 它不需要操作系统提供新能力；
- 它不会自动创建线程；
- 它不能让一段耗时 1 秒的纯计算变成 0.1 秒。

它只是把“等待”这段时间从浪费变成可复用。

## 二、`async fn` 返回一个 future

在函数签名前加 `async`，就得到一个**异步函数**：

```rust
async fn double(value: i32) -> i32 {
    value * 2
}
```

签名读作“调用后得到一个最终会产出 `i32` 的东西”。这个“最终会产出结果的东西”就叫 **future（未来值）**。

关键事实：**调用 `double(21)` 时，函数体一行都不会执行。** 它只构造了一个描述“将来怎么算”的值。`future` 这个词的含义正在于此——它代表一件还没发生、但已经安排好怎么做的事。

要真正推进它，需要一个执行器反复去“问”它进展如何。本课自带的入口叫 `block_on`：在当前线程上反复推进，直到结果出现。

仓库里的可运行示例（`lessons/19-async/examples/future_laziness.rs`）完整展示了这一点：

```text
$ cargo run -p l19_async --example future_laziness
future 已创建，函数体还没有运行
函数体开始运行
block_on 之后得到 42
```

第一行输出来自 `main`，说明这时 `double` 的函数体还没跑。第二、三行发生在 `block_on` 内部。

### 2.1 忘了推进会怎样

只在原地创建一个 future 而不推进它，什么事都不会发生。编译器会用一条警告提醒你：

```text
$ rustc --edition 2024 b.rs
warning: unused implementer of `Future` that must be used
 --> b.rs:2:5
  |
2 | /     async {
3 | |         println!("这行不会执行");
4 | |     };
  | |_____^
  |
  = note: futures do nothing unless you `.await` or poll them
  = note: `#[warn(unused_must_use)]` (part of `#[warn(unused)]`) on by default
```

请把最后那句 note 记住：**futures do nothing unless you `.await` or poll them**。这是初学异步最常见的“代码没跑却也不报错”的原因。

### 2.2 `.await` 只能在 async 里用

`.await` 是“等这个 future 出结果”的写法，它只能出现在异步上下文（`async fn` 或 `async { ... }` 块）里：

```text
$ rustc --edition 2024 a.rs
error[E0728]: `await` is only allowed inside `async` functions and blocks
 --> a.rs:3:24
  |
1 | fn main() {
  | --------- this is not `async`
2 |     let future = async { 42 };
3 |     let value = future.await;
  |                        ^^^^^ only allowed inside `async` functions and blocks
```

编译器甚至用 `----------` 标出了“从这一行开始就不是 async”。遇到 E0728，要么把外层函数改成 `async fn`，要么在 `main` 里用执行器（例如本课的 `block_on`）驱动。

## 三、`async fn` 到底是什么：一台状态机

把 `.await` 理解成“暂停”，编译器就必须解决一个难题：暂停时，函数已经算到一半的局部变量放在哪里？

答案是：编译器把异步函数体编译成一台 **状态机（state machine，一个记住“我现在进行到哪一步”的对象）**。它至少需要记录：

- 还没开始；
- 正卡在第一个 `.await` 上；
- 第一个 `.await` 已完成，正卡在第二个 `.await` 上；
- 全部完成，结果已产出。

跨越 `.await` 仍需使用的局部变量，必须作为状态机的一部分保存下来。这带来两个可观察后果，后面会反复用到：

1. future 的大小取决于它保存了哪些局部变量；
2. 这些被保存的变量会影响 future 能否跨线程、能否被安全移动。

## 四、`Future` 的接口：`poll`

`Future` 是一个 trait（特征，描述“具备哪些行为”的接口）。去掉细节后，它的核心只有一个方法：

```rust,ignore
trait Future {
    type Output;                       // 最终产出的类型

    fn poll(
        self: Pin<&mut Self>,          // 被推进的 future 自己
        context: &mut Context<'_>,     // 借给它的一些信息，其中包含 waker
    ) -> Poll<Self::Output>;           // 这次推进的结论
}

enum Poll<T> {
    Ready(T),      // 完成了，结果在这里
    Pending,       // 还没完成，稍后再问
}
```

逐个解释：

- **`type Output`** 是关联类型（associated type）：每个 future 自己声明“我最终产出什么”。`async fn double() -> i32` 生成的 future 里，`Output = i32`。
- **`Pin<&mut Self>`** 是把 future 固定在内存位置上的引用，第五节解释。
- **`Context`** 里能拿到一个 **`Waker`（唤醒器）**，它是一个“等条件满足了就通知执行器”的回调。
- **`Poll::Ready(value)`** 表示任务完成，`value` 就是结果。
- **`Poll::Pending`** 表示这次没法完成，执行器应该先去做别的事。

### 4.1 `Pending` 之后必须有人叫醒你

如果一个 future 返回 `Pending` 却从不安排 waker 被调用，执行器就永远不知道它什么时候能继续，任务会卡死。真实运行时里，waker 会在 socket（套接字，操作系统提供的网络通信端点）可读、定时器到点等事件发生时被唤醒，把任务重新放进待运行队列。

反过来，一个 future **不能**在 `Pending` 之后自己原地死循环等待——那会把执行器占住，别的任务全被饿死。

本课的 `block_on` 为了简单，`Pending` 之后直接继续轮询，同时用 `Waker` 的空实现满足类型要求。它演示的是“反复推进”这件事，而不是生产级的等待策略。

### 4.2 一次具体执行：`yield_now`

本课 `src/runtime.rs` 里有一个 `yield_now()`：第一次被 poll 时返回 `Pending`，第二次返回 `Ready(())`。它模拟“需要等一步”的操作，但不消耗真实时间。

逐轮看它：

| 第几次 poll | 做了什么 | 返回 |
|---|---|---|
| 第 1 次 | 把内部标记改为“已让出”，调用 `waker.wake_by_ref()` | `Pending` |
| 第 2 次 | 看到标记已是“已让出” | `Ready(())` |

测试里用 `yield_now` 代替 `sleep`，于是“等 200 毫秒”变成“等 3 次 `Pending`”。结果完全确定，不依赖机器快慢。这是本课所有可重复测试的基础。

## 五、`Pin` 为什么存在（本课够用的版本）

第三节说过，future 是状态机，可能保存指向自己内部数据的引用。如果 future 在开始被 poll 之后被搬到内存另一处，这些内部引用就会指向旧地址。

`Pin<P>` 是一个包装，含义是“通过这个指针，被指向的值不会再被安全地移动”。`Pin<&mut Self>` 就是“我借它，并且保证不会把它搬走”。

日常写异步代码很少需要手动操作 `Pin`：

- `.await` 会自动处理普通 future 的固定；
- 需要在堆上固定时用 `Box::pin(future)`——`Box` 把值放到堆上，地址稳定；
- 只有实现底层 `Future`，或需要手动 poll 别人给的 future 时才接触到 `Pin`。

本课 `runtime.rs` 里的 `JoinAll` 和 `CatchPanic` 就是这种底层结构。它们把内部 future 装在 `Pin<Box<_>>` 里，并给自己声明 `Unpin`（“整体被移动是安全的”），于是实现 `poll` 时可以用安全代码访问字段，不需要 `unsafe`。

## 六、运行时到底提供什么

Rust 语言提供了 `Future`、`async`、`.await`，但**标准库没有提供执行器**。原因是不同场景的调度策略差别太大：嵌入式设备可能只有一个线程，服务器可能有几百个；等待策略、定时器精度、任务优先级都不同。

一个完整的**运行时（runtime，让异步代码真正跑起来的整套支撑）** 通常包含：

- **executor（执行器）**：反复 poll 已就绪的任务；
- **reactor / I/O driver（事件反应器）**：监听 socket、文件、定时器等外部事件；
- **调度器**：决定哪个任务下一步运行、在哪个线程上运行；
- **异步版的网络、时间与同步工具**：`async` 版的 sleep、TCP（一种常用的网络传输协议）、mutex 等；
- **任务创建入口**：例如 Tokio 的 `tokio::spawn`。

### 6.1 本课为什么自己写一个

本课刻意不引入 Tokio，理由有三条：

1. **可观察性**：自己写几十行 executor，`poll`、`Pending`、`Ready` 就是看得见摸得着的代码，而不是被框架藏起来的行为。
2. **可重复的测试**：真实定时器依赖机器负载，测试会变得不稳定（flaky）。用“poll 次数”表达超时，测试就永远确定。
3. **依赖成本**：Tokio 是一个庞大的依赖树。本课要讲的概念用不上它的网络栈。

所以本课没有 `[dependencies]`，一行依赖都没有。等你写真实服务时，请选 Tokio 或 async-std 这类成熟运行时——它们提供的正是上面这张清单。

## 七、`.await` 不创建线程：并发与并行

先区分四个词：

| 概念 | 含义 | 例子 |
|---|---|---|
| 并发（concurrency） | 多个任务在重叠的时间段内都有进展 | 一个线程上两个任务交替推进 |
| 并行（parallelism） | 多个任务在同一时刻各用一份硬件同时执行 | 4 个核各跑一个任务 |
| 线程（thread） | 操作系统调度的执行单位 | `std::thread::spawn` |
| 异步任务（async task） | 一个被 executor 反复 poll 的 future | 本课的 `join_all` 里的每一项 |

关键结论：**`.await` 只是让当前任务暂停，它不会创建线程，也不会切换线程。** 一个线程上可以同时“管理”成千上万个任务，只要它们大部分时间在等。

反过来，如果 future 里有一次 5 秒的纯计算，中间一次 `.await` 也没有，那它就会占住这个线程 5 秒，其他任务都得排队。这就是为什么 README 后面反复强调：**不要把长时间阻塞的操作塞进异步任务**。真实运行时的做法是把这类工作交给专用线程池（Tokio 的 `spawn_blocking`）。

## 八、一起跑起来：并发等待

两个 future 依次 `.await`，是**顺序等待**：

```rust,ignore
let first = fetch("a").await;   // 等它完成
let second = fetch("b").await;  // 这时 second 才开始
```

第二个 future 其实在 `fetch("a")` 完成前根本没被创建过，谈不上并发。

要并发，必须**先创建多个 future，再让执行器一起推进它们**。本课提供了 `join_all`：

```rust,ignore
let futures = vec![fetch("a"), fetch("b")];
let results = join_all(futures).await;   // 两者交替推进
```

`join_all` 的 `poll` 每次依次询问每个还没完成的 future：

- 某个 future 返回 `Ready`，就把它的结果收好，从这一轮的名单里划掉；
- 某个 future 返回 `Pending`，不影响后面的 future 被 poll；
- 全部收集齐了才返回 `Ready(所有结果)`。

这就是“并发”在代码层面的样子：**不是把工作塞进线程，而是在一个线程里轮流推进多个 future。**

### 8.1 完整示例：交替推进

仓库的 `src/main.rs` 就是完整示例，脚手架状态下也能运行：

```text
$ cargo run -p l19_async
刚创建 future 时，任务体一行都还没运行。
丢弃 future 后，任务体仍然一行都没运行。

把两个任务交给 join_all，它们在同一个线程上交替推进：
A：第 1 步
B：第 1 步
A：第 2 步
B：第 2 步
A：第 3 步

交替出现说明一个任务让出执行权时，另一个任务获得了推进机会。
```

逐段解释数据怎么流动：

1. `task("未启动", 3)` 只创建 future。打印第一行时，`task` 的循环一次都没进入。
2. `drop(future)` 把这个 future 丢掉。它从未被 poll，因此永远不会运行——这就是“future 是惰性的”。
3. `join_all(vec![task("A", 3), task("B", 2)])` 把两个 future 收进一个组合 future，`block_on` 反复 poll 它。
4. 第一轮：A 打印第 1 步后调用 `yield_now().await`，于是 A 返回 `Pending`；`join_all` 接着 poll B，B 打印第 1 步后也 `Pending`。
5. 第二轮：A 从第一次让出中恢复，打印第 2 步后再次让出；B 同样打印第 2 步后让出。
6. 第三轮：A 从第二次让出中恢复，打印第 3 步后再次让出；B 从第二次让出中恢复，循环结束，B 完成并返回 `Ready`。
7. 第四轮：A 从第三次让出中恢复，循环结束；两个 future 都已完成，`join_all` 返回。

输出顺序说明了本课最重要的直觉：**并发来自“一个任务让开时另一个顶上”，不是来自多个 CPU。**

## 九、超时为什么用“poll 次数”

真实程序里的超时是“最多等 2 秒”。

在测试里写真实计时器有两个麻烦：机器繁忙时 2 秒可能不够；测试还要真的空等 2 秒，慢。

本课改用另一个等价但可控的量：**一个 future 最多允许返回多少次 `Pending`**。这叫**耐心值（patience）**。

- `patience = 0`：一次 `Pending` 都不许，第一次 poll 就必须完成；
- `patience = 2`：允许两次 `Pending`，第三次 poll 还没完成就算超时。

这和真实超时的结构完全一样：**允许的等待量用完 → 判定超时 → 取消内部操作**。区别只是“等待量”是 poll 次数而不是秒数。

实现这种包装需要“手动 poll 别人给的 future”，于是用到第四节那个接口：

```rust,ignore
async fn with_timeout<F: Future>(future: F, patience: usize) -> Result<F::Output, TimedOut> {
    // 1. 把 future 固定在当前栈帧上，这样才能反复拿到 Pin<&mut F>
    // 2. 用 poll_fn 造一个闭包驱动的 future，在闭包里数 Pending 次数
    // 3. 超过 patience 就返回 Ready(Err(TimedOut))，否则返回 Pending
}
```

`poll_fn` 是标准库提供的工具：把一个“`FnMut(&mut Context) -> Poll<T>`”的闭包变成一个 future。这样就绕过了“不能直接调用 `.poll()`”的限制。

主练习的三个函数里，`with_timeout` 就是这个包装——这是你要写的第一个函数。

### 9.1 超时时内部 future 会怎样

返回 `Err(TimedOut)` 之后，被包装的 future 就不再被 poll 了，它会在函数返回时被丢弃（drop）。丢弃会：

- 运行它所有局部变量的析构（`Drop`）代码；
- 释放它持有的锁、连接等资源；
- **不会**回滚已经发生的副作用（比如“请求已经发出去一半”）。

这就是 Rust 异步里最基本的**取消（cancellation）**：**取消就是丢弃 future**。没有单独的 `cancel()` 调用，也没有强制中断。第十二节展开讲。

## 十、重试：不是简单循环

服务返回错误时，先要问：这个错误**再试一次会不同吗**？

- 网络抖动、服务繁忙、连接被重置 → 可能不同，值得重试（**暂时性错误**）；
- 收件人地址不存在、参数非法、认证失败 → 再试一万次也一样（**永久性错误**），不该重试。

本课用 `ServiceFault` 表达这个区别：

```rust,ignore
pub enum ServiceFault {
    Temporary { reason: String },   // 可以重试
    Permanent { reason: String },   // 不要重试
}
```

一个合格的重试流程至少要处理：

1. **最大次数**：`max_attempts`，包含第一次尝试。所以 `max_attempts = 1` 表示“不重试”。
2. **只重试暂时性错误**：遇到永久性错误立刻放弃。
3. **每次尝试各自的超时**：否则一次卡死的请求会吞掉所有重试预算。
4. **最终错误保留上下文**：`TemporaryExhausted { last_reason }` 里要能看出最后一次失败的原因。

真实系统还要加指数退避（每次重试等更久）、随机抖动（避免大量客户端同时重试）、总 deadline（所有重试共享的截止时间）、以及幂等性判断（重复执行会不会造成重复副作用）。本课为了确定性，退避用“立即重试”，只保留次数与错误类别的结构。

### 10.1 重试与超时组合起来

单次尝试的流程是：

```text
调用 send  →  包一层超时  →  把 panic 变成错误
```

这三件事的结果是三层嵌套，必须逐层拆开看：

```text
Result<                         ← 最外层：是否 panic
    Result<                     ← 中间层：是否超时
        Result<String, ServiceFault>,   ← 最内层：服务返回什么
        TimedOut
    >,
    TaskPanic
>
```

这种“套娃”不是设计缺陷，而是三个不同的失败来源本来就不该混为一谈：**panic 是代码 bug，超时是等待超限，`ServiceFault` 是对方明确拒绝。** 把它们压成一个错误字符串，调用方就没法做决策了。

## 十一、有界并发：为什么要限制

“1 万条通知，全部同时发”听起来最快，实际会：

- 同时打开 1 万个连接，耗尽文件描述符（操作系统给每个进程的连接/文件配额）；
- 瞬间占满内存；
- 把对方服务打挂，触发限流，最后全部失败；
- 失败后一起重试，形成尖峰。

所以要设上限，比如“同时最多 20 个”。本课用的是最简单的策略：**分批**。

```text
作业 1..10，limit = 3
批 1：作业 1, 2, 3   ← 3 个 future 用 join_all 一起推进
批 2：作业 4, 5, 6
批 3：作业 7, 8, 9
批 4：作业 10
```

每批内部并发，批与批之间顺序。这样“同时进行的数量”永远不超过 `limit`，实现也容易验证。

> 生产代码常用信号量（semaphore，一种“同时最多 N 个持有者”的计数器）做滑动窗口，让空闲出来的名额立刻被新作业填上，吞吐更好。分批的代价是每批末尾会有“等最慢那个”的空档。本课选分批，是因为它更容易证明上限成立，也更容易写确定的测试。

## 十二、取消：future 被丢弃时发生什么

Rust 的异步取消只有一种形态：**不再 poll 它，然后丢掉它**。

丢弃一个 future 时：

1. 它保存的所有局部变量按逆序析构；
2. 局部变量里的 `Drop` 实现会运行；
3. 因此“持有资源”的守卫（guard）能被正确释放。

本课 `FakeService::send` 里有一段值得细看的代码：

```rust,ignore
pub async fn send(&self, job: &Job, attempt: usize) -> Result<String, ServiceFault> {
    let active = self.active.get() + 1;
    self.active.set(active);
    self.peak.set(self.peak.get().max(active));
    let _guard = ActiveGuard { service: self };   // 从这个点开始计数
    // ... 可能 await 很多次，也可能 panic ...
}
```

`ActiveGuard` 的 `Drop` 会把“进行中”计数减一。无论 `send` 是正常返回、panic，还是因为超时被丢弃，`_guard` 都会被析构，计数都会归零。

这解释了一条重要结论：**取消安全（cancellation safety）的关键，是把资源清理放进 `Drop`，而不是放在函数末尾。** 如果你写“在函数最后手动减一”，那么被取消时那行永远不会执行。

### 12.1 取消不能回滚副作用

已经发生的副作用不会因为取消而消失。发送了一半的请求、写了一半的文件、从队列里取走的消息，都需要额外设计（比如临时文件加原子替换、消息确认机制）。

## 十三、panic 不是错误，但也不能让它炸掉整批

`panic!` 在 Rust 里表示“程序遇到了不该继续的 bug”。默认情况下它会展开（unwind）当前线程的栈并结束线程。

一个作业的处理逻辑 panic 了，不应导致另外 99 个作业的结果全部丢失。所以要把 panic 捕获并转成结构化结果。标准库的 `std::panic::catch_unwind` 能做这件事，但它需要 `UnwindSafe` 断言——意思是“我保证 panic 之后不会再信任这些数据”。

本课 `runtime::catch_panic` 把它包成一个 future 适配器：

```rust,ignore
pub fn catch_panic<F: Future>(future: F) -> CatchPanic<F>
// CatchPanic 的 Output = Result<F::Output, TaskPanic>
```

这样在 `poll` 里 panic 会变成 `Err(TaskPanic)`，调度器可以把这条作业记为失败，继续收集其他结果。

真实运行时里，`tokio::spawn` 返回的 `JoinHandle` 用 `JoinError` 表达同样的意思：要么任务 panic 了，要么任务被取消。

## 十四、编译错误与诊断

异步代码的报错信息乍看很绕，但模式固定。下面是真实运行 `rustc` 得到的三条，都比对着看一遍。

### 14.1 在非 async 函数里写 `.await`

```text
error[E0728]: `await` is only allowed inside `async` functions and blocks
 --> a.rs:3:24
  |
1 | fn main() {
  | --------- this is not `async`
2 |     let future = async { 42 };
3 |     let value = future.await;
  |                        ^^^^^ only allowed inside `async` functions and blocks
```

**读法**：编译器先指出“这一段不是 async”，再指出违规的那个 `.await`。
**修法**：把外层改成 `async fn`，或在 `main` 里用执行器驱动。

### 14.2 `async move` 之后还想用原来的值

```text
error[E0382]: borrow of moved value: `name`
 --> c.rs:4:16
  |
2 |     let name = String::from("job");
  |         ---- move occurs because `name` has type `String`, which does not implement the `Copy` trait
3 |     let future = async move { name };
  |                  ----------   ---- variable moved due to use in coroutine
  |                  |
  |                  value moved here
4 |     println!("{name}");
  |                ^^^^ value borrowed here after move
```

**读法**：`async move` 把 `name` 移进了 future（注意编译器把它叫 `coroutine`，这是同一个东西的旧名字）。移走发生在**构造 future 的那一刻**，不是等它被 poll 的时候。
**修法**：不要移动（改用借用），或者先 `clone()` 一份给 future，或者把使用顺序调整到移动之前。

### 14.3 手动 poll 一个不保证位置固定的 future

```text
error[E0277]: `F` cannot be unpinned
    --> d.rs:6:40
     |
   6 |     std::future::poll_fn(|cx| Pin::new(&mut future).poll(cx)).await
     |                               -------- ^^^^^^^^^^^ the trait `Unpin` is not implemented for `F`
     |
     = note: consider using the `pin!` macro
             consider using `Box::pin` if you need to access the pinned value outside of the current scope
```

**读法**：`Pin::new` 要求被指向的类型是 `Unpin`（“移动它没关系”），而泛型 `F` 没这个保证。
**修法**：用 `std::pin::pin!` 把值固定在当前栈帧上，或者用 `Box::pin` 固定到堆上。**不要**用 `unsafe` 绕过这条检查。

### 14.4 排错顺序建议

遇到异步相关报错时按这个顺序看：

1. 最外层错误类型是什么？（`await` 位置、`Send`、`Unpin`、生命周期）
2. 编译器用 `^^^^` 指出的那个值，是什么类型？
3. 它是否跨越了某个 `.await` 仍然活着？
4. note 里通常已经给出修法方向，先试它。

## 十五、常见误区

**误区一：以为 `async fn` 调用就执行了。**
不会。它只造了一个状态机。没 `.await` 也没人 poll，就永远不跑（2.1 节的警告）。

**误区二：以为连续两个 `.await` 就是并发。**
不是。第二个 future 要等第一个完成才被创建。

**误区三：以为 `async` 会开线程。**
不会。`.await` 只是让出执行权。

**误区四：以为独立 `spawn` 一定优于在同一个任务里组合。**
独立 spawn 要求 future 拥有自己的数据（通常还得满足 `Send + 'static`），并额外产生一层“任务本身是否 panic”的错误。能在当前任务内组合就组合，除非你真的需要它在后台独立存活、或需要多线程并行。

**误区五：在 async 里调用阻塞 API。**
`std::thread::sleep`、阻塞式文件读写、同步数据库驱动都会占住执行器线程。真实项目里改成异步版本或交给阻塞线程池。

**误区六：跨 `.await` 持有同步锁。**
等待期间锁一直被占着，其他任务一碰就卡死。正确做法是先取出需要的快照，释放锁，再去 await。

**误区七：丢弃 `JoinHandle` 就不管了。**
任务失败了没人知道。必须收集并汇总。

**误区八：以为测试通过一次就证明没有竞态。**
异步测试的价值在于**确定**：用假服务、用可控的等待量、用明确的完成顺序。不要用真实 `sleep` 去“大概等一下”。

## 十六、代码预测

先自己想，再展开答案。

### 16.1 这段代码会打印几行

```rust,ignore
async fn log() {
    println!("将来");
}

fn main() {
    let _future = log();
    println!("现在");
}
```

<details><summary>解析</summary>

只有 `现在` 一行。`log()` 只构造 future，函数体没跑；`_future` 在 `main` 结束时被丢弃，从未被 poll。编译器还会给出 `unused_must_use` 警告。

</details>

### 16.2 两个任务的实际执行顺序

```rust,ignore
// 片段：`step` 是一个省略了定义的辅助函数，效果是打印它的参数。
block_on(join_all(vec![
    async { step("A1"); yield_now().await; step("A2"); },
    async { step("B1"); yield_now().await; step("B2"); },
]));
```

<details><summary>解析</summary>

`A1, B1, A2, B2`。

第一轮 `join_all` 先 poll 第一个：打印 `A1`，遇到 `yield_now` 返回 `Pending`。接着 poll 第二个：打印 `B1`，也 `Pending`。第二轮两个都从 `yield_now` 之后继续：打印 `A2`、`B2`，然后都完成。

注意：这个顺序由 `join_all` 的实现决定（它按列表顺序 poll）。真实运行时多线程下的顺序不可预测，所以测试不应依赖它。

</details>

### 16.3 超时发生在第几次 poll

`patience = 2`，内部 future 需要经历 3 次 `Pending` 才完成。

<details><summary>解析</summary>

超时。前两次 `Pending` 用掉配额，第三次 `Pending` 到来时配额已耗尽，包装立刻返回 `Err(TimedOut)`。内部 future 被丢弃，它保存的变量被析构。

</details>

### 16.4 这几个错误值得重试吗

- 连接被对方重置；
- 对方返回“这个收件人不存在”；
- 请求超过允许的等待量。

<details><summary>解析</summary>

第一个值得——连接重置通常是临时的。第二个不值得——数据本身有问题，重试结果一样。第三个要看场景：如果是网络拥塞，重试可能成功；如果是对方服务过载，马上重试只会加重问题。所以超时应该配合退避，而不是立即重试。

</details>

## 十七、主练习：有界并发通知调度器

现在把前面所有概念装进一个可测试的调度器。

### 17.1 目标

给定一批 `Job`（待发送的通知）和一个 `FakeService`（可编程的假服务），在并发上限、重试次数、单次超时都受限的前提下，把每个作业跑完，返回**每个作业恰好一个**最终结果，并按作业编号排序。

### 17.2 文件与分工

| 文件 | 内容 | 谁写 |
|---|---|---|
| `src/runtime.rs` | `block_on`、`yield_now`、`join_all`、`catch_panic` | 已提供 |
| `src/exercise.rs` | 数据类型、`Job::new`、`FakeService`、`JobOutcome::id` | 已提供 |
| `src/exercise.rs` | `with_timeout`、`send_with_retries`、`run_batch` | **你写** |
| `tests/job_tests.rs` | 主练习的行为测试 | 已提供，不要改 |
| `src/consolidation.rs` | 课后巩固的两个函数 | **你写** |
| `tests/race_tests.rs` | 课后巩固的行为测试 | 已提供，不要改 |
| `src/main.rs`、`examples/future_laziness.rs` | 可运行演示 | 已提供 |

只替换 `todo!()`。**不要修改函数签名，也不要修改测试**——测试就是规格的一部分。

### 17.3 输入与期望输出

一个具体例子：

```text
输入：
  作业 #1：user1@example.test，消息 1
  作业 #2：user2@example.test，消息 2
  配置：limit = 2，max_attempts = 3，patience = 5
  脚本：#1 → 第 1 次暂时性失败，第 2 次成功
        #2 → 第 1 次永久性失败

期望输出（BatchReport）：
  outcomes = [
    Success { id: 1, output: "已通知 user1@example.test：消息 1", attempts: 2 },
    Failed  { id: 2, error: Permanent { reason: "..." }, attempts: 1 },
  ]
```

要点：

- 作业 #2 永久失败，但作业 #1 的结果照样保留；
- 顺序按 `id`，不按完成先后；
- `attempts` 是实际发生的尝试次数。

### 17.4 数据模型速览

`src/exercise.rs` 里已经准备好这些类型。你不需要改它们，但必须理解每个字段：

| 类型 | 作用 |
|---|---|
| `Job { id, recipient, message }` | 一条待发送的通知。`id` 唯一，用于排序和查脚本。 |
| `Step` | 假服务对**一次**尝试的预设行为：成功 / 暂时失败 / 永久失败 / panic，前三种可带“让出几次”的延迟。 |
| `ServiceFault::Temporary / Permanent` | 服务返回的错误类别。 |
| `FakeService` | 按脚本响应的假服务；同时记录每个作业被调用几次、当前进行中数量、历史峰值。 |
| `RetryPolicy { max_attempts, patience }` | 重试策略与单次尝试的等待上限。 |
| `BatchConfig { limit, policy }` | 整批作业的调度配置。 |
| `TimedOut` | 单次尝试超出 `patience` 的标记。 |
| `JobError` | 一个作业最终失败的四种原因。 |
| `JobOutcome::Success / Failed` | 一个作业的最终结果。 |
| `BatchReport { outcomes }` | 整批报告。 |
| `BatchError::ZeroLimit / ZeroAttempts` | 整批作业无法开始的原因。 |

### 17.5 三个待实现函数

**`with_timeout(future, patience) -> Result<F::Output, TimedOut>`**

给任意 future 加上“最多 `patience` 次 `Pending`”的限制。`patience == 0` 表示第一次 poll 就必须完成。超时时返回 `Err(TimedOut)`，内部 future 被丢弃。

**`send_with_retries(service, job, policy) -> JobOutcome`**

对一条作业执行至多 `max_attempts` 次尝试。每次尝试都要：调用 `service.send`、加超时、捕获 panic。成功返回 `Success`；暂时性失败在还有次数时重试，否则返回 `TemporaryExhausted`；永久失败、超时、panic 都立即停止。

**`run_batch(service, jobs, config) -> Result<BatchReport, BatchError>`**

先检查配置（`limit == 0` 优先于 `max_attempts == 0`），再把作业按输入顺序切成每批最多 `limit` 条，每批用 `join_all` 并发推进，最后把全部结果按 `id` 排序返回。

### 17.6 约束

- **只能通过 `FakeService` 的公开方法**与它交互：`send`、`calls_for`、`active_now`、`peak_active`。不要为了走捷径去改 `FakeService` 内部。
- **不要真的睡眠**。所有等待都用 `yield_now`。
- **不要修改签名**。`with_timeout` 必须对任意 `F: Future` 生效，不能要求 `F: Unpin`。
- **不要留下无人观察的后台任务**。`run_batch` 返回时，所有 future 要么已完成，要么已被丢弃。
- **不要求 `Send`**。本课运行在单线程上，你的实现不需要处理跨线程移动。
- 允许使用 `runtime` 里已提供的 `join_all` 和 `catch_panic`，也允许自己写组合逻辑。

### 17.7 推荐完成顺序

1. **`with_timeout`**：先用 `std::pin::pin!` 把 future 固定住，再用 `std::future::poll_fn` 数 `Pending` 次数。写完立刻跑：

   ```text
   cargo test -p l19_async with_timeout
   ```

   这三个测试只涉及超时本身，最容易单独确认。

2. **`send_with_retries`**：把三层结果逐层拆开。先处理最内层的 `ServiceFault`，再处理 `TimedOut`，最后处理 `TaskPanic`。跑：

   ```text
   cargo test -p l19_async --test job_tests retr
   ```

   过滤词匹配的是**测试名**，所以这里用 `retr` 同时命中“重试”“不重试”“超时不重试”三个测试。

3. **`run_batch`**：先做配置检查（有专门测试检查两个错误的优先级），再做空输入，再做分批 + `join_all`，最后排序。跑：

   ```text
   cargo test -p l19_async --test job_tests
   ```

第 3 步依赖前两步正确，所以不要跳到它。这条命令只跑主练习的 23 个测试；不带 `--test` 的 `cargo test -p l19_async` 还会运行第十八节的 `race_tests`。

### 17.8 实现前会失败的测试

脚手架状态下运行测试，会看到大量 `not yet implemented` panic：

```text
$ cargo test -p l19_async
running 23 tests
test empty_batch_returns_empty_report ... FAILED
...
failures:
    empty_batch_returns_empty_report
    ...
test result: FAILED. 0 passed; 23 failed
```

（具体数字以实际输出为准。`cargo test` 在第一个失败的测试文件之后就停止，所以这次不会运行 `tests/race_tests.rs`；它属于第十八节的课后巩固。）

这是正常的学习流程。**编译错误**才说明脚手架被改坏了。

### 17.9 自动测试覆盖了什么

`tests/job_tests.rs` 共 23 个测试，覆盖：

- 超时：恰好用完配额通过、多一次就超时、`patience = 0`；
- 配置错误：0 上限、0 尝试次数，以及两者的优先级；
- 空输入；
- 一次成功、重试后成功、重试耗尽、`max_attempts = 1`；
- 永久失败不重试；
- 超时与业务错误可区分；
- panic 变成结构化失败；
- 完成乱序但返回按 `id` 排序；
- `limit` 为 1、2、大于作业数时的并发峰值；
- 单条失败不影响其他结果；
- 非 ASCII 文本在输出中保持不变；
- 脚本缺失时按永久失败处理。

### 17.10 验收命令

在仓库根目录执行：

```text
cargo test -p l19_async
```

这条命令也会运行 `tests/race_tests.rs`。在完成第十八节的课后巩固之前，那 9 个测试仍会失败；主练习本身对应的是 23 个 `job_tests` 测试。全部通过后，再跑一遍严格检查确保没有引入警告：

```text
cargo fmt -p l19_async -- --check
cargo clippy -p l19_async --all-targets -- -D warnings
```

### 17.11 完成后请能回答

- 哪些 future 是在同一个任务里用 `join_all` 组合的？哪些是独立的？为什么本课不需要独立 spawn？
- 为什么 `.await` 不等于创建线程？
- `ActiveGuard` 在什么时候被析构？超时和 panic 两条路径上有区别吗？
- 哪两类错误可以重试，哪两类不可以？依据是什么？
- 如果把 `limit` 从 2 改成 10，`peak_active` 会怎么变？如果作业只有 3 个呢？

## 十八、课后巩固

主练习关心“怎么调度”。这里换成完全不同的数据和边界，专门观察 future 的两个细节：**竞速**与**计数**。脚手架在 `src/consolidation.rs`，测试在 `tests/race_tests.rs`。

### 18.1 `first_ready(left, right) -> Winner<A::Output, B::Output>`

同时推进两个 future，返回**先完成**的那一个。

```text
输入：left 需要 2 次 Pending 才完成，right 第一次 poll 就完成
期望：Winner::Right(right 的输出)
```

规则：

- 每轮先 poll `left`，完成就返回 `Winner::Left`；
- 否则再 poll `right`，完成就返回 `Winner::Right`；
- 都 `Pending` 就进入下一轮；
- **同一轮都完成时 `Left` 获胜**（因为先检查 `left`）；
- 返回后另一个 future 被丢弃（取消），它的 `Drop` 会执行。

两个 future 的输出类型可以不同。

### 18.2 `polls_until_ready(future) -> (F::Output, usize)`

手动 poll 一个 future 直到完成，返回“输出”和“期间经历了多少次 `Pending`”。

```text
输入：第一次 poll 就完成
期望：(输出, 0)

输入：需要 3 次 Pending 才完成
期望：(输出, 3)
```

这个函数把第四节的 `Poll` 模型直接变成可观察的数字。写完它，你对“poll 次数”就不会再有模糊感。

### 18.3 巩固任务的边界（与主练习不同）

- 没有服务、没有重试、没有作业编号；
- 数据是两个任意类型的 future；
- 判定标准是**顺序**（谁先完成）而不是**汇总**（谁成功谁失败）；
- 需要处理的边界是“同一轮都完成”“立刻完成”“需要多轮”；
- 取消的观察方式是数 `Drop` 次数，而不是看服务的进行中计数。

### 18.4 验收命令

```text
cargo test -p l19_async --test race_tests
```

只跑 `polls_until_ready` 的两个测试时用：

```text
cargo test -p l19_async polls_until_ready
```

注意 `first_ready` 是函数名不是测试名，`cargo test -p l19_async first_ready` 会匹配到 0 个测试；测试按行为命名，过滤时请用行为关键词（例如 `wins`）或直接指定测试文件。

### 18.5 巩固测试覆盖了什么

`tests/race_tests.rs` 共 9 个测试：两个都立即完成时左侧获胜、右侧先完成、同一轮都完成时左侧获胜、输出类型不同、左侧获胜时右侧被丢弃、右侧获胜时左侧被丢弃、立即完成时计数为 0、多次 `Pending` 正确计数、输出是拥有型字符串。

## 十九、本课总结

- `async fn` 调用后只构造 future，不执行函数体；future 是惰性的。
- `Future::poll` 每次返回 `Ready(结果)` 或 `Pending`；`Pending` 之后必须靠 `Waker` 通知，不能忙等。
- 异步函数被编译成状态机，跨越 `.await` 的局部变量保存在其中，这决定了 future 的大小与可移动性。
- `Pin` 保证状态机不会被搬走；日常代码用 `.await`、`pin!`、`Box::pin` 就够了，不需要 `unsafe`。
- `.await` 让出执行权但不创建线程；并发来自“一个任务让开、另一个顶上”，并行才需要多个执行单元。
- 连续 `.await` 是顺序等待；并发要显式组合（`join_all`）或独立 spawn。
- 标准库只提供语言机制，执行器与 I/O 事件源由运行时提供。本课用一个几十行的最小运行时换取可观察性与确定性。
- 超时、重试、有界并发与取消是异步 API 正确性的一部分，不是“以后再补”的细节。
- 取消就是丢弃 future；清理逻辑必须放在 `Drop` 里，否则被取消时不会执行。
- 把 panic、超时、业务错误分成不同的类型，调用方才能做出正确决策。

## 下一课

下一课进入宏。我们会先用 `macro_rules!` 做语法层面的重复，再区分声明宏、derive 宏和属性宏，重点始终是同一个判断：**什么时候普通函数或泛型才是更简单的工具**。本课你已经见过 `#[derive(Debug)]` 这类属性宏，也知道 `#[tokio::main]` 那种“宏替你生成启动代码”的手法，下一课会拆开看它们到底在做什么。见[第二十课](../20-macros/README.md)。
