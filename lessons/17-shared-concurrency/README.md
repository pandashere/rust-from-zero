# 第十七课：让多个线程安全地共享同一份数据

前一课（[第十六课](../16-threads-channels/README.md)）让每份数据在某一时刻只属于一个线程，线程之间通过 channel（通道，一条把消息从发送端送到接收端的队列）转移消息的所有权。这一课处理另一类真实需求：几个线程必须一起读取、修改**同一份**数据。

例如，四个线程同时为一批文档建立索引，最后需要得到一张所有线程共同贡献的索引表。把表复制四份会得到四张互不相干的表，还要额外合并；把表只交给一个线程，另外三个线程又无法写入。Rust 为这种场景提供了共享所有权和同步工具：`Arc<T>`、`Mutex<T>`、`RwLock<T>` 与原子类型。

本课不会要求你学过 C、C++、Python、Java 或任何其他编程语言。线程、锁、原子操作这些概念会先用日常语言解释，再给出 Rust 写法。读完后你应当能判断：什么时候该共享状态，什么时候该传消息，以及怎样让共享状态不出错。

## 本课定位

**要解决的具体问题**

假设你写了一个命令行程序，要把 1 万份文档建成“单词 → 文档编号”的索引。单线程可以完成，但速度慢。你启动四个线程，每个线程处理四分之一文档。四个线程最终都要把结果写进同一张表。这里同时出现两个问题：

1. 这张表由谁拥有？四个线程都要用它，不能只放在某一个线程的栈上。
2. 四个线程同时改它时，怎样避免互相覆盖？

**为什么现在学**

- [第三课](../03-ownership-borrowing/README.md)讲过所有权（每个值有一个负责人，负责人离开时值被释放）和借用（临时借用某个值，`&T` 只读，`&mut T` 可写且必须独占）。
- [第十五课](../15-smart-pointers/README.md)讲过 `Rc<T>`（引用计数指针；**指针**是记录另一个值在内存中位置的值，引用计数则记录有多少个所有者）。
- [第十六课](../16-threads-channels/README.md)讲过 `thread::spawn`、`join`、`move` 闭包和 channel。

本课把这些知识接在一起：`Rc` 的计数不是线程安全的，所以跨线程要换成 `Arc`；`Arc` 只解决“共同拥有”，不解决“同时修改”，所以还要加锁或原子操作。

**前置知识速查**

如果下面的词已经陌生，建议先回到对应课复习，不需要背下来，只要知道大概在说什么：

- **值、所有权、移动**：一个值在同一时刻只有一个所有者；把值赋给另一个变量或传进函数时，所有权可能被“移动”，原来的变量之后不能再用。
- **借用**：`&T` 是只读借用，可以同时存在多个；`&mut T` 是可写借用，同一时刻只能有一个，而且不能和只读借用同时存在。
- **`Rc<T>`**：单线程里的共享所有权。`Rc::clone` 增加引用计数，最后一个 `Rc` 销毁时释放内部值。
- **`RefCell<T>`**：把“借用规则检查”从**编译期**（编译器把源码翻译成机器码的阶段）推迟到**运行期**（程序实际执行的阶段），允许通过共享引用修改内部值；违反规则时运行期 panic。
- **`HashMap`**：键值对集合，用键快速找值；本课的索引就是 `HashMap<String, Vec<usize>>`。
- **`Result<T, E>`**：表示“成功时得到 `T`，失败时得到 `E`”；`?` 可以把错误向上传递。
- **`thread::spawn`**：创建一个新线程；它返回 `JoinHandle`，调用 `join()` 等待线程结束并取回结果。
- **闭包**：可以捕获周围变量的匿名函数，写作 `|参数| 表达式`；`move` 让闭包按值拿走捕获的变量。
- **panic**：程序遇到无法继续的错误时中止当前线程的机制；它和“返回 `Err`”是两种不同的错误处理方式。

**本课边界**

本课只讨论操作系统线程和**标准库**（standard library，Rust 随编译器一起提供的现成函数与类型集合）里的同步工具，不讨论 `async`/`await`（[第十九课](../19-async/README.md)）、无锁数据结构、分布式系统，也不使用 `unsafe`（Rust 中允许绕过部分安全检查的关键字，[第二十一课](../21-unsafe-ffi/README.md)讲解）。原子类型只介绍最基础的独立计数，不教你手写复杂的无锁协议。

## 如何学习本课

建议按下面顺序，预计需要 4 到 6 小时，卡住时不要跳读：

1. 读第一到第九节，理解“为什么需要”和每个工具解决哪一层问题。
2. 把“完整示例”亲手输入并运行一次；不要只看。
3. 做“代码预测”，先在纸上写答案，再展开解析。
4. 完成主练习，按推荐顺序逐个函数实现，用测试验证。
5. 完成课后巩固；它换了一种数据结构和锁。
6. 回到“常见误区”和“课后复盘”，用自己的话解释一遍。

课程目录就是本课的 **Cargo package**（Cargo 是 Rust 的构建与依赖管理工具；一个 package 是一组一起编译的源码和它的 `Cargo.toml` 配置）。所有命令在本课目录或仓库根目录执行：

```bash
cargo test -p l17_shared_concurrency
cargo run -p l17_shared_concurrency
```

第一次运行时，测试会因为练习里的 `todo!()` 而失败，`cargo run` 也会 panic。这是设计好的起点，不是环境坏了。

---

## 一、线程：同一程序里同时前进的多条执行路线

**程序**是磁盘上的一组指令和数据。**运行**程序时，**操作系统**（管理硬件、文件和程序运行的系统软件，例如 Linux、Windows）把它加载进内存，创建一个**进程**（process）。进程有自己的内存空间、打开的文件等资源。

**线程**（thread）是进程内部的一条执行路线。一个进程至少有一个线程；`main` 函数运行在主线程里。程序可以创建更多线程，让它们“同时”执行不同函数。

线程之间最常打交道的两类内存：

- **栈**（stack）：每个线程有自己的栈，存放当前函数的局部变量和调用信息。一个线程通常不能直接借走另一个线程栈上的数据。
- **堆**（heap）：程序运行时动态申请的内存，例如 `String` 的字符缓冲区、`Vec` 的元素。堆上的数据可以被多个线程访问，但前提是所有权和同步规则允许。

**并发**（concurrency）指多个任务在时间上重叠地推进；**并行**（parallelism）指多个任务真的在同一时刻由多个 **CPU**（中央处理器，执行机器指令的芯片）核心执行。操作系统里的**调度器**（scheduler）决定哪个线程何时运行、运行多久。你无法假设两个线程谁先谁后，也无法假设一个线程会连续运行到结束。

第十六课的最小例子：

```rust
use std::thread;

fn main() {
    let handle = thread::spawn(|| {
        for number in 1..=3 {
            println!("工作线程: {number}");
        }
    });

    for number in 1..=3 {
        println!("主线程: {number}");
    }

    handle.join().unwrap();
}
```

`thread::spawn` 的参数是一个闭包，闭包里的代码在新线程运行。它返回 `JoinHandle`（线程句柄）。`join()` 做两件事：等待目标线程结束；取回闭包的返回值。这里闭包没有返回值，所以返回 `()`；线程 panic 时 `join()` 返回 `Err`，`unwrap()` 会把那个错误变成主线程的 panic。

输出行的先后顺序不固定，可能主线程先打印完，也可能交替出现。这不是 bug，而是调度器的工作方式。

**为什么要多线程**：利用多核 CPU、在等待网络或磁盘时继续处理其他工作、同时服务多个请求。**代价**：多个执行路线会碰同一份内存，顺序又不固定，于是产生**数据竞争**（data race）。

数据竞争的定义是：两个线程同时访问同一块内存，其中至少一个是写操作，而且没有同步机制。数据竞争会让程序读到“写了一半”的值，甚至破坏内存。Rust 的安全代码（不使用 `unsafe` 的代码）承诺在编译期阻止数据竞争；但 Rust 不能阻止**逻辑竞争**，例如两个线程按不同顺序做两件事，最后业务结果仍然可能不对。

一个最小的逻辑竞争例子是计数器加一：

```text
counter 的初始值是 0。

时刻 1：线程 A 读出 counter 的值 0
时刻 2：线程 B 读出 counter 的值 0
时刻 3：线程 A 把 0 + 1 = 1 写回
时刻 4：线程 B 把 0 + 1 = 1 写回

两个线程都执行了一次“加一”，最终却是 1，不是 2。
```

`counter += 1` 看起来是一条语句，实际包含“读、加、写”三个步骤。若两个线程的步骤交错，就会丢失一次更新。后面所有同步工具，本质上都在解决“怎样让这段读改写不被别的线程插进来”。

---

## 二、前课复习：所有权、借用与 `Rc`

**所有权**（ownership）是 Rust 管理内存的规则：每个值有一个所有者；所有者离开作用域时，值被自动释放；值被移动后，原变量失效。这样可以避免“两个地方都以为自己在负责释放同一块内存”。

**借用**（borrowing）让你在不转移所有权的情况下使用值：

- `&T`：只读借用，可以有多个；
- `&mut T`：可写借用，同一时刻只能有一个；
- 只读借用和可写借用不能同时存在。

这套规则在单线程里很好用，但遇到“多个线程都要拥有一份数据”时不够：如果把值移动给线程 A，线程 B 就没有了；如果借用给线程 A，编译器又无法保证 A 不会比当前函数活得更久。

第十五课的 `Rc<T>`（Reference Counted，引用计数）解决单线程里的共享所有权：

```rust
use std::rc::Rc;

fn main() {
    let shared = Rc::new(String::from("hello"));
    let second = Rc::clone(&shared);

    println!("所有者数量: {}", Rc::strong_count(&shared));
    println!("内容: {second}");
}
```

`Rc::clone(&shared)` 不会复制字符串，只增加“有多少个 `Rc` 指向同一份数据”的计数。计数降到 0 时，内部的 `String` 才被释放。

问题在于：**引用计数的加一和减一本身不是原子操作**。**原子操作**（atomic operation）指其他线程无法观察到它“做到一半”的操作；它要么还没发生，要么已经完整发生。普通整数加法在机器层面可能要读、加、写几步，两个线程同时改计数会丢更新。计数一旦算错，可能出现“还有线程在使用，数据却被释放”，这破坏了**内存安全**（不访问已释放、越界或不属于自己的内存）。

因此 Rust 把 `Rc<T>` 标记为不能跨线程移动、不能跨线程共享。`Rc<RefCell<T>>` 也不行：`RefCell` 的“当前有几个借用”的计数同样不是线程安全的。第十五课学的“共享可变”方案，到了多线程就要换成这一课的工具。

**关键区分**：共享所有权和同步访问是两个问题。

- 谁负责让数据活着？——所有权问题。
- 谁允许谁在什么时候读写？——同步问题。

`Rc` 只解决第一个问题，而且只在单线程成立；`Arc` 解决第一个问题的线程安全版本；`Mutex`/`RwLock`/原子类型解决第二个问题。

---

## 三、先看全貌：四种工具各管什么

| 工具 | 中文名 | 解决的问题 | 不解决的问题 |
| --- | --- | --- | --- |
| `Arc<T>` | 原子引用计数指针 | 多个线程共同拥有同一个值，最后一个所有者释放它 | 不允许修改内部值，也不提供互斥 |
| `Mutex<T>` | 互斥锁 | 同一时刻只允许一个线程访问内部值 | 单独使用时不能跨线程共享所有权 |
| `RwLock<T>` | 读写锁 | 多个读者同时读，或一个写者独占写 | 不保证比 `Mutex` 快，也不消除死锁 |
| 原子类型，如 `AtomicUsize` | 原子类型 | 对单个小状态做不可分割的读改写 | 不保护多个字段组成的整体不变量 |

最常见的组合是 `Arc<Mutex<T>>`：

```text
Arc 负责“大家都能拿到同一个盒子”
Mutex 负责“盒子里的东西一次只给一个人用”
```

`Arc` 和 `Mutex` 都不是“魔法开关”。`Arc<T>` 只有在内部 `T` 也满足线程安全条件时才能跨线程；`Mutex` 只能防止同时访问，不能防止你把锁拿太久、按相反顺序拿两把锁，或在持锁时 panic。

---

## 四、`Arc<T>`：让多个线程共同拥有一个值

`Arc` 是 **Atomically Reference Counted** 的缩写，意思是“原子引用计数”。它和 `Rc` 的用法很像，但计数更新是原子的。

```rust
use std::sync::Arc;
use std::thread;

fn main() {
    let shared = Arc::new(String::from("shared"));
    println!("创建后，所有者数量: {}", Arc::strong_count(&shared));

    let worker_value = Arc::clone(&shared);
    println!("克隆后，所有者数量: {}", Arc::strong_count(&shared));

    let handle = thread::spawn(move || worker_value.len());
    let length = handle.join().unwrap();

    println!("工作线程看到的长度: {length}");
    println!("主线程仍然可以读取: {}", &*shared);
}
```

输出：

```text
创建后，所有者数量: 1
克隆后，所有者数量: 2
工作线程看到的长度: 6
主线程仍然可以读取: shared
```

逐行解释：

- `Arc::new(String::from("shared"))` 在堆上放一个 `String`，外面包一层原子引用计数。此时只有一个 `Arc` 指向它。
- `Arc::clone(&shared)` 增加计数，返回另一个 `Arc`。两个 `Arc` 指向**同一个** `String`，不是两份字符串。
- `move` 闭包把 `worker_value` 这个 `Arc` 的所有权移进新线程。主线程的 `shared` 仍然有效，因为它是另一个所有者。
- `worker_value.len()` 能调用 `String` 的方法，是因为 `Arc<T>` 实现了 `Deref`（解引用转换）：通过 `Arc` 可以像通过 `&T` 一样访问 `T`。
- `&*shared` 先解引用得到 `String`，再取引用，得到 `&String`；打印时按字符串内容输出。
- 工作线程结束后，它的 `Arc` 被销毁，计数减一；主线程的 `Arc` 仍然让数据活着。

**为什么写 `Arc::clone(&shared)` 而不是 `shared.clone()`**

两种写法都能增加引用计数。`shared.clone()` 会根据方法解析找到 `Arc` 的 `Clone` 实现，效果相同。但 `Arc::clone(&shared)` 把“我在克隆 Arc，不是克隆里面的 String”写得更清楚。若写成 `(*shared).clone()`，`*shared` 得到 `String`，再调用 `clone()` 会**深拷贝字符串内容**，这是完全不同的操作。读代码时要注意 `*` 和括号。

**`Arc` 只解决“活着”，不解决“能改”**

下面代码不能编译：

```rust,compile_fail
use std::sync::Arc;

fn main() {
    let shared = Arc::new(Vec::<i32>::new());
    shared.push(1);
}
```

原因不是“没有用 `mut`”，而是 `Arc<T>` 只提供共享的 `Deref`，没有提供可写的 `DerefMut`。多个 `Arc` 所有者同时存在时，编译器不能把内部值借给某一个所有者做可写借用，否则就违反了“可写借用必须独占”的规则。

要修改，需要把“独占访问”交给一个同步工具，通常是 `Mutex<T>`。`Arc` 负责共享盒子，`Mutex` 负责盒子的钥匙。

**注意**：`Arc<T>` 并不自动让任意 `T` 变得线程安全。`Arc<RefCell<T>>` 仍然不能跨线程，因为 `RefCell<T>` 不是 `Sync`（后面第七节解释）。正确的组合是 `Arc<Mutex<T>>` 或 `Arc<RwLock<T>>`。

---

## 五、`Mutex<T>`：一次只允许一个线程进入

`Mutex` 是 **mutual exclusion** 的缩写，中文叫“互斥锁”。它像一个房间，门口只有一把钥匙：

- 一个线程调用 `lock()` 拿到钥匙，进入房间；
- 其他线程再调用 `lock()` 时只能等待，直到钥匙被还回来；
- 拿到钥匙的线程离开房间时自动还钥匙。

在 Rust 里，`lock()` 返回的不是裸数据，而是一个**锁守卫**（lock guard，`MutexGuard`）。守卫代表“我当前持有锁”，并且实现了 `Deref` 和 `DerefMut`，所以可以像使用 `&mut T` 一样使用内部值。守卫被销毁时自动解锁。

### 5.1 最小例子

```rust
use std::sync::Mutex;

fn main() {
    let counter = Mutex::new(0_u64);

    {
        let mut guard = counter.lock().unwrap();
        *guard += 1;
    }

    println!("计数: {}", *counter.lock().unwrap());
}
```

输出：

```text
计数: 1
```

逐行解释：

- `Mutex::new(0_u64)` 创建一个锁，里面保护一个 `u64`（无符号的 64 位整数，只能表示 0 和正数）。`0_u64` 的后缀表示这个字面量的类型。
- `counter.lock()` 尝试获得锁。成功时返回 `Ok(MutexGuard)`，失败时返回 `Err(PoisonError)`（中毒，后面 5.4 节解释）。这里用 `unwrap()` 表示“失败就让当前线程 panic”。
- `let mut guard`：守卫本身要绑定为可变，才能通过它取得内部值的可写借用。`mut` 修饰的是 `guard` 这个变量，不是 `counter`。
- `*guard += 1`：`*` 解引用守卫，得到内部的 `u64`，再加一。由于守卫实现了 `DerefMut`，这里可以写。
- 花括号结束时 `guard` 被销毁，锁自动释放。即使中间 `return` 或 panic，守卫也会在栈展开时销毁，锁不会永远不释放。这种“资源在离开作用域时自动清理”的模式叫 **RAII**（Resource Acquisition Is Initialization，资源获取即初始化）。这个名字来自其他语言传统，你只需记住：**守卫离开作用域就解锁**。
- 最后一行又临时获取一次锁，读取当前值。临时守卫在语句结束时销毁。

也可以显式提前释放：

```rust
use std::sync::Mutex;

fn main() {
    let counter = Mutex::new(0_u64);
    let mut guard = counter.lock().unwrap();
    *guard += 1;
    drop(guard); // 立刻解锁，而不是等到函数结束

    println!("计数: {}", *counter.lock().unwrap());
}
```

`drop(guard)` 调用标准库的 `drop` 函数，立即销毁守卫并解锁。后面再 `lock()` 才不会卡住。

### 5.2 `Mutex` 为什么还要配 `Arc`

`Mutex` 保护数据，但它本身不是“可被多个线程共同拥有”的容器。把 `Mutex` 直接移动进一个线程，其他线程就拿不到它；在线程闭包里借用局部的 `Mutex`，又过不了 `'static` 生命周期检查。**生命周期**（lifetime）是编译器用来追踪“一个引用能安全使用多久”的规则；`'static` 在这里表示值不包含指向当前函数栈帧的短期引用。

`Arc<Mutex<T>>` 把两件事分开：

```rust
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    let counter = Arc::new(Mutex::new(0_u64));
    let mut handles = Vec::new();

    for _ in 0..4 {
        let counter = Arc::clone(&counter);
        handles.push(thread::spawn(move || {
            for _ in 0..10_000 {
                *counter.lock().unwrap() += 1;
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("最终计数: {}", *counter.lock().unwrap());
    assert_eq!(*counter.lock().unwrap(), 40_000);
}
```

输出：

```text
最终计数: 40000
```

这里每个线程都拿到一个 `Arc` 克隆，四个 `Arc` 指向同一个 `Mutex<u64>`。每次 `lock()` 只有一个线程能进入，`+= 1` 的读、加、写三步不会被别的线程插进来，所以 4 × 10 000 次更新不会丢失。

注意循环里的变量遮蔽：

```rust
let counter = Arc::clone(&counter);
```

等号右边的 `counter` 是外层那个 `Arc`，等号左边新建了一个同名变量。随后 `move` 闭包拿走的是新克隆，外层原来的 `counter` 仍然保留。如果不克隆就直接 `move`，第一个线程会把外层 `Arc` 拿走，后面的循环就不能再用了。

**不要**写成 `let counter = (*counter).clone()`。那会尝试克隆内部的 `Mutex<u64>`，而 `Mutex` 通常不实现 `Clone`；即使实现了，也会得到一把**新的锁**，四个线程各锁各的，计数仍然会错。

### 5.3 锁的作用域要尽量小

锁的作用是保护共享数据，不是“把整段程序锁起来”。持有锁的时间越长，其他线程等待越久，程序越像单线程。

下面是一种常见但糟糕的结构：

```text
let mut data = shared.lock().unwrap();
data.prepare();
send_over_network(&data);   // 网络可能很慢
data.finish();
```

`send_over_network` 期间其他线程完全无法访问 `shared`。更好的做法是只在锁内做必须受保护的操作，把结果复制或移动出来，释放锁后再做慢操作：

```text
let snapshot = {
    let mut data = shared.lock().unwrap();
    data.prepare();
    data.snapshot()          // 生成一份可以独立使用的数据
};                           // 这里 guard 销毁，锁释放

send_over_network(&snapshot);
```

原则：

- 锁内只做必须互斥的读写；
- 不在持锁时睡眠、等待网络、读写大文件、阻塞接收 channel；
- 不在持锁时调用你无法控制的回调，因为回调可能再次获取同一把锁；
- 能先在锁外算好，再进锁提交，就不要把整个计算放进锁里；
- 需要明确释放点时，用花括号或 `drop(guard)`，不要依赖“临时值大概什么时候销毁”的猜测。

一个容易忽略的细节：`*counter.lock().unwrap() += 1;` 里的临时守卫在**整条语句结束时**销毁，所以这条语句是安全的；但如果你写 `let mut guard = counter.lock().unwrap();`，守卫会活到当前作用域结束。看到“锁一直不释放”时，先检查守卫变量的作用域。

### 5.4 中毒（poisoning）：持锁线程 panic 之后

如果线程在持有 `MutexGuard` 时 panic，标准库会把这把锁标记为**中毒**（poisoned）。之后其他线程调用 `lock()` 会得到 `Err(PoisonError)`，而不是正常守卫。

这不是说数据一定坏了，而是提醒：**某个不变量（必须始终成立的条件）可能在更新到一半时被 panic 打断**。例如一个转账操作先扣了 A 的钱，还没来得及加给 B，线程就 panic 了，账目处于半完成状态。

```rust
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    let data = Arc::new(Mutex::new(vec![1]));
    let worker_data = Arc::clone(&data);

    let _ = thread::spawn(move || {
        let mut guard = worker_data.lock().unwrap();
        guard.push(2);
        panic!("worker failed while holding the lock");
    })
    .join();

    let guard = match data.lock() {
        Ok(guard) => guard,
        Err(poisoned) => {
            eprintln!("锁已中毒，需要检查数据是否完整");
            poisoned.into_inner()
        }
    };

    println!("当前数据: {guard:?}");
}
```

运行时会先看到工作线程的 panic，再看到主线程的处理。`PoisonError::into_inner()` 可以强行取出守卫，但**是否应该这样做取决于业务**：

- 如果这个不变量关键（钱、库存、配置），应该向上返回错误或终止，不能假装没事；
- 如果更新本身是原子的、可以验证（例如只是追加一条完整记录），可以检查后恢复；
- 如果数据是可重建的缓存，可以丢弃并重建。

不要习惯性地写 `lock().unwrap()` 然后忽略中毒。`unwrap()` 在中毒时会 panic，这有时是合理的“快速失败”，但你要知道自己在选择什么。

### 5.5 死锁：互相等对方的钥匙

**死锁**（deadlock）指两个或多个线程各自持有对方需要的锁，谁也不肯先释放，程序永久卡住。

```text
线程 A：先锁 left，再锁 right
线程 B：先锁 right，再锁 left

A 拿到 left，B 拿到 right
A 等 right，B 等 left
两边都永远等下去
```

Rust 的类型系统不会阻止死锁。编译器只能检查借用和数据竞争，无法知道你的程序在运行时会不会按某种顺序拿锁。

同一把非可重入锁也可能被同一个线程锁两次而卡住：

```rust,no_run
use std::sync::Mutex;

fn main() {
    let data = Mutex::new(1);
    let first = data.lock().unwrap();
    let second = data.lock().unwrap(); // 永远等不到，因为 first 还没释放
    println!("{first:?} {second:?}");
}
```

标准库的 `Mutex` 不是**可重入**（reentrant，同一线程可以再次获得自己已持有的锁）的。第二次 `lock()` 会阻塞自己。

主要防范方式：

- 全局规定多把锁的获取顺序，所有线程都按同一顺序拿；
- 尽量一次只持有一把锁；
- 把需要一起修改的状态放进同一把锁，减少多锁组合；
- 用 `try_lock()` 尝试获取，拿不到就做别的事或退避重试，而不是无限等待；
- 通过 channel 把状态集中到一个线程，其他线程发消息（第十六课）；
- 不在持锁时调用未知代码或回调。

`try_lock()` 返回 `Result`，常见错误是 `TryLockError::WouldBlock`（锁被占用）和 `TryLockError::Poisoned`。它适合“拿不到就跳过”的场景，不是死锁的万能解药：你需要设计重试、退避和放弃策略。

### 5.6 锁的粒度

**锁粒度**（lock granularity）指一把锁保护多少数据、**临界区**（critical section，持有锁并访问共享数据的那段代码）多长。

- 一把全局大锁：设计简单，不容易出现多锁死锁；高竞争时并行度低。
- 多把细粒度锁：不相关的操作可以并行；但锁顺序、事务一致性和测试都更复杂。

不要一开始就为了“性能”拆成很多锁。先选一把能清楚证明正确的锁，测量到真实瓶颈后再细分。绝大多数初学者程序里，锁的竞争不是瓶颈，错误的同步才是。

---

## 六、`RwLock<T>`：多个读者，或一个写者

`RwLock` 是 **read-write lock**，读写锁。规则是：

- 可以同时存在多个**读守卫**（read guard），大家都能读；
- **写守卫**（write guard）必须独占，写的时候不允许任何读者或其他写者；
- 读和写不能同时进行。

```rust
use std::sync::RwLock;

fn main() {
    let config = RwLock::new(vec!["a", "b"]);

    {
        let first_reader = config.read().unwrap();
        let second_reader = config.read().unwrap();
        println!("两个读者同时看到: {:?} {:?}", *first_reader, *second_reader);
    }

    config.write().unwrap().push("c");
    println!("写入后: {:?}", *config.read().unwrap());
}
```

`read()` 返回读守卫，`write()` 返回写守卫；两者都用 `unwrap()` 处理中毒。读守卫只实现 `Deref`，写守卫实现 `Deref` 和 `DerefMut`。写守卫存在时，新的读请求会等待；读守卫存在时，写请求会等待。

**什么时候值得用**

读操作远远多于写操作，而且读临界区不短、读者很多时，`RwLock` 可能让多个读者真正并行。反过来：

- 写很频繁：读写锁的额外记账成本可能超过收益；
- 临界区极短：锁本身的开销占比高，`Mutex` 往往更简单；
- 竞争很低：两者差别很小，优先选更容易理解的 `Mutex`；
- 某些平台的读写锁可能让写者饥饿（writer starvation，写者一直等不到机会），具体调度行为依赖实现。

所以：**从 `Mutex` 开始，确认有读多写少的真实需求并测量后，再考虑 `RwLock`**。不要因为名字里有“读写”就默认它更高级。

`RwLock` 也有中毒：写线程持锁 panic 后，后续 `read()`/`write()` 会返回 `PoisonError`。处理原则和 `Mutex` 一样。

---

## 七、`Send` 与 `Sync`：编译器怎样判断能不能跨线程

`Send` 和 `Sync` 是标准库里的两个 **trait**（特征，描述一组类型共同具备的能力）。它们没有方法，只作为标记使用，所以叫**标记 trait**（marker trait）。编译器会根据类型的字段自动推导它们是否成立，这类自动推导的 trait 叫**自动 trait**（auto trait）。

- 若 `T: Send`，可以把 `T` 的**所有权移动**到另一个线程。
- 若 `T: Sync`，可以把 `&T`（共享引用）交给多个线程同时使用。
- 一个实用的等价直觉：`T: Sync` 大致等于 `&T: Send`。

常见类型：

| 类型 | `Send` | `Sync` | 原因 |
| --- | --- | --- | --- |
| `i32`、`String`、`Vec<i32>` | 是 | 是 | 数据本身不包含线程不安全的内部状态 |
| `Rc<T>` | 否 | 否 | 引用计数不是原子的 |
| `RefCell<T>` | 当 `T: Send` 时是 | 否 | 运行期借用计数不是原子的 |
| `Cell<T>` | 当 `T: Send` 时是 | 否 | 内部可变，且没有同步 |
| `Mutex<T>` | 当 `T: Send` 时是 | 当 `T: Send` 时是 | 互斥访问把并发读写串行化 |
| `RwLock<T>` | 当 `T: Send` 时是 | 当 `T: Send + Sync` 时是 | 读守卫会把 `&T` 同时给多个线程 |
| `Arc<T>` | 当 `T: Send + Sync` 时是 | 当 `T: Send + Sync` 时是 | 多个线程共享同一个 `T`，所以 `T` 必须既能移动又能共享 |
| `AtomicUsize`、`AtomicBool` | 是 | 是 | 操作本身是原子的 |

这张表解释了一个常见困惑：`Arc<T>` 并不是“只要套上 Arc 就能跨线程”。`Arc<RefCell<T>>` 里的 `RefCell` 不是 `Sync`，所以 `Arc<RefCell<T>>` 不能安全地跨线程共享；编译器会拒绝它。正确做法是 `Arc<Mutex<T>>` 或 `Arc<RwLock<T>>`。

**不要手动实现 `Send` 或 `Sync`**，除非你在编写底层 `unsafe` 抽象，并且能证明所有并发不变量。错误的实现会让编译器放行真正危险的操作，导致**未定义行为**（程序的结果不再有任何保证，可能崩溃、读到乱码，甚至看似正常运行）。安全 Rust 里，你通常只需要选择正确的组合，让编译器自动推导。

当编译器报出很长的 `Send`/`Sync` 错误链时，从**最内层**那个“不满足 trait 的类型”开始读。例如 `Arc<RefCell<i32>>` 的错误会先指出 `RefCell<i32>` 不是 `Sync`，再说这导致 `Arc<RefCell<i32>>` 不能 `Send`。不要只看最外层就盲目套 `Arc`。

---

## 八、原子类型：小而独立的状态

**原子类型**（atomic type）提供不可分割的读改写操作。标准库有 `AtomicUsize`、`AtomicBool`、`AtomicI32` 等，位于 `std::sync::atomic`。

```rust
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

fn main() {
    let counter = Arc::new(AtomicUsize::new(0));
    let mut handles = Vec::new();

    for _ in 0..4 {
        let counter = Arc::clone(&counter);
        handles.push(thread::spawn(move || {
            counter.fetch_add(1, Ordering::Relaxed);
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("计数: {}", counter.load(Ordering::Relaxed));
    assert_eq!(counter.load(Ordering::Relaxed), 4);
}
```

- `AtomicUsize::new(0)` 创建一个原子整数。
- `fetch_add(1, Ordering::Relaxed)` 原子地把值加一，并返回旧值；这里不需要旧值，所以直接忽略返回值。
- `load(...)` 原子地读出当前值。
- `Arc` 仍然负责让四个线程共同拥有这个原子整数。

**`Ordering` 是什么**

`Ordering`（内存顺序）描述原子操作和其他内存操作之间的可见性约束。`Relaxed` 只保证**这一个原子值**的修改不会撕裂、所有线程看到的修改顺序一致；它不建立“我改了 A，所以其他线程一定能看到我先改了 B”这种跨变量保证。

本课只在“独立统计计数”里使用 `Relaxed`。如果你要发布一个对象、维护多个字段之间的不变量、实现无锁队列，就不要凭直觉选 `Ordering`。优先使用锁、channel 或成熟的并发库。原子操作的正确性很难靠试出来。

**什么时候用原子类型**

- 单个计数器、序号生成器；
- 一个布尔开关（例如“是否收到停止信号”）；
- 只读的统计指标。

**什么时候不要用**

- 需要同时保护两个以上字段的整体一致性；
- 需要“检查后再修改”的复合条件（例如库存足够才扣减）；
- 逻辑复杂到需要证明内存顺序。

这些场景用 `Mutex` 往往更简单、更容易证明正确。原子类型不是“更快的锁”，它是另一种适用面更窄的工具。

---

## 九、channel 还是共享状态

第十六课的 channel 模型是：数据的所有权通过消息从一个线程移动到另一个线程，同一时刻只有一个线程拥有它。共享状态模型是：多个线程通过 `Arc` 共同拥有，再用锁或原子操作协调访问。

**更适合 channel 的情况**

- 状态有自然的单一所有者，例如一个专门管理数据库连接的线程；
- 操作可以表达成命令或事件（“请处理这份任务”“这是结果”）；
- 希望隔离失败：一个工作线程 panic 不会直接破坏共享数据结构；
- 数据可以转移，不需要多个线程同时读同一份快照；
- 想避免锁顺序和持锁阻塞。

**更适合共享状态的情况**

- 多个线程需要频繁查询同一份只读或少量写入的数据；
- 操作围绕一个共享数据结构，例如缓存、索引、配置；
- 把每次访问都改成“发请求、等回复”会让协议变得复杂；
- 临界区短，且不包含阻塞操作。

**可以组合**：用 channel 分发工作，用 `Arc<AtomicUsize>` 记录独立指标，用 `Arc<RwLock<Config>>` 保存少量共享配置。不要为了“纯粹”强行只用一种。

选择时问自己三个问题：

1. 这份数据有没有一个自然的单一所有者？有，优先 channel。
2. 其他线程是需要“拿走数据”，还是只需要“读/改同一份数据”？前者 channel，后者共享。
3. 如果共享，临界区是否短、是否不阻塞？不是，就重新设计，别用锁硬扛。

---

## 十、完整示例：并发词频统计

下面是一个独立可运行的程序。它把三行文本分给三个线程，统计每个单词出现多少次，最后合并成一张表。这个例子和主练习不同：主练习建“单词 → 文档编号”的倒排索引，这里建“单词 → 出现次数”的计数表。

```rust
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    let chunks = vec![
        String::from("rust makes sharing safe"),
        String::from("sharing data needs rules"),
        String::from("rust rules"),
    ];

    let counts = Arc::new(Mutex::new(HashMap::<String, usize>::new()));
    let mut handles = Vec::new();

    for chunk in chunks {
        let counts = Arc::clone(&counts);
        handles.push(thread::spawn(move || {
            let mut local = HashMap::new();
            for word in chunk.split_whitespace() {
                *local.entry(word.to_string()).or_insert(0) += 1;
            }

            let mut shared = counts.lock().unwrap();
            for (word, count) in local {
                *shared.entry(word).or_insert(0) += count;
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let snapshot = counts.lock().unwrap();
    let mut entries: Vec<_> = snapshot.iter().collect();
    entries.sort_by(|left, right| left.0.cmp(right.0));
    for (word, count) in entries {
        println!("{word}: {count}");
    }
}
```

输出（顺序固定，因为最后排了序）：

```text
data: 1
makes: 1
needs: 1
rules: 2
rust: 2
safe: 1
sharing: 2
```

逐步解释数据和所有权的变化：

1. `chunks` 是一个 `Vec<String>`，拥有三行文本。`for chunk in chunks` 每次把一行 `String` 的所有权移动进循环变量。
2. `counts` 是 `Arc<Mutex<HashMap<String, usize>>>`：外层 `Arc` 让多个线程共同拥有；`Mutex` 让合并时互斥；里面是单词计数表。
3. 每轮循环先 `Arc::clone(&counts)`，得到一个新的 `Arc`。然后 `move` 闭包把这个克隆移进线程。主线程的 `counts` 始终保留。
4. 线程内部先建 `local`：一个只属于当前线程的普通 `HashMap`。`split_whitespace()` 按空白切分，`word.to_string()` 创建拥有型键，`entry(...).or_insert(0)` 在键不存在时插入 0，然后 `+= 1`。
5. 本地统计完成后，才调用 `counts.lock().unwrap()` 拿共享锁。锁内只做“把 local 的计数加到 shared”。锁在 `shared` 守卫离开作用域时释放。
6. 三个线程都 `join` 之后，主线程再锁一次，取得快照并排序打印。此时不会再有线程修改，但读取仍然需要锁，因为 `Mutex` 不区分“现在是读还是写”；如果确定不再修改且想省掉锁，可以在最后用 `Arc::try_unwrap` 取回内部值，但前提是确认没有其他 `Arc` 克隆。

**为什么先本地再合并**

如果每读到一个单词就锁一次共享表，锁的获取和释放会非常频繁，而且大部分时间花在等待其他线程上。先在本地 `HashMap` 里完成分词和计数，最后只锁一次批量合并，通常更快，也更容易控制临界区。

**为什么结果稳定**

线程完成顺序不确定，但每个单词的总数是一个确定的和：三个线程分别加上自己的局部计数，加法满足交换律，最终总数不变。最后又对键排序，所以打印顺序也稳定。**不要依赖线程的完成顺序来保证业务结果**；要么让结果与顺序无关，要么显式排序/合并。

---

## 十一、编译错误与排错

下面展示的是 `rustc 1.94` 上的典型诊断。播报里的文件路径和编译器内部位置做了简化，错误码、类型名和约束文字保留原意。你的行号可能不同。

### 11.1 `Rc<T>` 不能跨线程

```rust,compile_fail
use std::rc::Rc;
use std::thread;

fn main() {
    let value = Rc::new(42);
    let worker_value = Rc::clone(&value);
    thread::spawn(move || println!("{worker_value}"));
}
```

诊断：

```text
error[E0277]: `Rc<i32>` cannot be sent between threads safely
  --> src/main.rs:7:19
   |
 7 |     thread::spawn(move || println!("{worker_value}"));
   |     ------------- -------^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |     |             |
   |     |             `Rc<i32>` cannot be sent between threads safely
   |     |             within this `{closure}`
   |     required by a bound introduced by this call
   |
   = help: within `{closure}`, the trait `Send` is not implemented for `Rc<i32>`
note: required because it's used within this closure
  --> src/main.rs:7:19
   |
 7 |     thread::spawn(move || println!("{worker_value}"));
   |                   ^^^^^^^
note: required by a bound in `spawn`
   |
   | pub fn spawn<F, T>(f: F) -> JoinHandle<T>
   |        ----- required by a bound in this function
...
   |     F: Send + 'static,
   |        ^^^^ required by this bound in `spawn`
```

读法：

- `E0277` 是“某个 trait 约束不满足”的错误码。
- 关键句是 `Rc<i32> cannot be sent between threads safely`。
- `thread::spawn` 要求闭包 `F: Send + 'static`。闭包捕获了 `Rc`，所以闭包不是 `Send`。
- 修复不是给 `Rc` 加 `mut`，而是换成 `Arc`。如果内部还需要修改，再根据需求配 `Mutex`/`RwLock`。

### 11.2 `Arc<T>` 不能直接修改

```rust,compile_fail
use std::sync::Arc;

fn main() {
    let shared = Arc::new(Vec::<i32>::new());
    shared.push(1);
}
```

诊断：

```text
error[E0596]: cannot borrow data in an `Arc` as mutable
 --> src/main.rs:5:5
  |
5 |     shared.push(1);
  |     ^^^^^^ cannot borrow as mutable
  |
  = help: trait `DerefMut` is required to modify through a dereference, but it is not implemented for `Arc<Vec<i32>>`
```

`E0596` 是“不能按可变方式借用”。`Arc` 只提供 `Deref`，不提供 `DerefMut`，因为多个所有者同时存在时不能保证独占。修复方式：

```rust
use std::sync::{Arc, Mutex};

fn main() {
    let shared = Arc::new(Mutex::new(Vec::<i32>::new()));
    shared.lock().unwrap().push(1);
    println!("{:?}", *shared.lock().unwrap());
}
```

### 11.3 `Arc<RefCell<T>>` 仍不能跨线程

```rust,compile_fail
use std::cell::RefCell;
use std::sync::Arc;
use std::thread;

fn main() {
    let shared = Arc::new(RefCell::new(0));
    let worker = Arc::clone(&shared);
    thread::spawn(move || {
        *worker.borrow_mut() += 1;
    });
}
```

诊断：

```text
error[E0277]: `RefCell<i32>` cannot be shared between threads safely
   --> src/main.rs:8:19
    |
  8 |     thread::spawn(move || {
    |  _____-------------_^
    | |     |
    | |     required by a bound introduced by this call
  9 | |         *worker.borrow_mut() += 1;
 10 | |     });
    | |_____^ `RefCell<i32>` cannot be shared between threads safely
    |
    = help: the trait `Sync` is not implemented for `RefCell<i32>`
    = note: if you want to do aliasing and mutation between multiple threads, use `std::sync::RwLock` instead
    = note: required for `Arc<RefCell<i32>>` to implement `Send`
```

这段诊断非常有代表性：最内层的问题是 `RefCell<i32>` 不满足 `Sync`；因为 `Arc<T>` 要跨线程发送需要 `T: Send + Sync`，所以外层 `Arc<RefCell<i32>>` 也不能 `Send`。修复不是“给 Arc 再加一层”，而是把 `RefCell` 换成 `Mutex` 或 `RwLock`。

### 11.4 死锁没有编译错误

```rust,no_run
use std::sync::Mutex;

fn main() {
    let data = Mutex::new(1);
    let first = data.lock().unwrap();
    let second = data.lock().unwrap();
    println!("{first:?} {second:?}");
}
```

这段代码能编译、能启动，然后永久卡住。编译器不知道 `first` 还没有释放。排错方法：

- 用 `try_lock()` 代替 `lock()`，观察是否返回 `WouldBlock`；
- 缩小守卫作用域，或显式 `drop`；
- 检查是否在持锁时调用了会再次加锁的函数；
- 用日志记录“准备获取哪把锁”，确认多线程的获取顺序是否相反。

### 11.5 中毒后 `unwrap()` 的运行时 panic

如果工作线程持锁 panic，主线程又写 `data.lock().unwrap()`，会看到类似：

```text
thread '<unnamed>' panicked at src/main.rs:11:9:
worker failed while holding the lock
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

thread 'main' panicked at src/main.rs:15:29:
called `Result::unwrap()` on an `Err` value: PoisonError { .. }
```

第一段是工作线程自己的 panic；第二段是主线程对 `PoisonError` 调 `unwrap()` 造成的 panic。`PoisonError` 是一个错误值，不是“锁坏了不能用”，而是“之前有线程在持锁时失败了”。要恢复，就显式 `match` 并用 `into_inner()`，同时想清楚数据是否可信。

### 11.6 排错顺序

遇到并发编译错误时，按这个顺序看：

1. 找到最内层“不满足 `Send`/`Sync`”的类型；
2. 问它为什么不是线程安全的（引用计数？运行期借用？内部可变？）；
3. 选择对应的线程安全替代：`Rc` → `Arc`，`RefCell` → `Mutex`/`RwLock`；
4. 确认 `Arc` 里面是同步类型，而不是只把外层换成 `Arc`；
5. 如果错误提到 `'static`，检查线程是否借用了当前函数的局部变量；改用拥有型数据，或使用第十六课的 `thread::scope`。

---

## 十二、代码预测

先写下答案，再展开。

### 预测 1：`Arc::clone` 和 `(*arc).clone()`

```rust
use std::sync::Arc;

fn main() {
    let text = Arc::new(String::from("hi"));
    let same = Arc::clone(&text);
    let copy = (*text).clone();

    println!("{} {} {}", Arc::strong_count(&text), same, copy);
}
```

输出是什么？`copy` 和 `text` 是同一份字符串吗？

<details><summary>解析</summary>

输出是：

```text
2 hi hi
```

`Arc::clone(&text)` 只增加引用计数，`same` 和 `text` 指向同一个 `String`，所以计数是 2。`(*text).clone()` 先解引用得到 `String`，再调用 `String` 的 `clone()`，复制出一份独立的字符串内容。之后修改 `copy` 不会影响 `text`。这也是为什么共享状态时不能靠“克隆内部值”来代替 `Arc`：每个线程会得到自己的副本，最后还要合并。

</details>

### 预测 2：原子计数

```rust
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

fn main() {
    let counter = Arc::new(AtomicUsize::new(0));
    let mut handles = Vec::new();

    for _ in 0..8 {
        let counter = Arc::clone(&counter);
        handles.push(thread::spawn(move || {
            for _ in 0..1_000 {
                counter.fetch_add(1, Ordering::Relaxed);
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("{}", counter.load(Ordering::Relaxed));
}
```

结果一定是 8000 吗？为什么？

<details><summary>解析</summary>

是 8000。每个 `fetch_add` 都是原子的读改写，8 个线程各做 1000 次，不会丢失更新。这里 `Relaxed` 足够，因为我们只关心这个计数器自己的总数，不用它来发布其他数据。若你还用这个计数器判断“另一块数据是否已经准备好”，就需要更强的内存顺序或直接用锁。

</details>

### 预测 3：守卫的作用域

```rust
use std::sync::Mutex;

fn main() {
    let data = Mutex::new(vec![1]);

    {
        let mut guard = data.lock().unwrap();
        guard.push(2);
    }

    println!("{:?}", *data.lock().unwrap());
}
```

输出是什么？如果去掉花括号，把 `guard` 留到 `println!` 之后，会怎样？

<details><summary>解析</summary>

输出 `[1, 2]`。花括号结束时 `guard` 销毁，锁释放，最后一行可以重新获取锁。如果去掉花括号，`guard` 活到 `main` 末尾，最后一行再次 `lock()` 时同一线程会等待自己，程序死锁。`println!` 的参数里再 `lock()` 也不会自动帮你释放前面的守卫。

</details>

### 预测 4：中毒之后

工作线程在持锁时 panic，主线程随后执行：

```rust
let value = match data.lock() {
    Ok(guard) => *guard,
    Err(poisoned) => {
        let guard = poisoned.into_inner();
        *guard
    }
};
```

这段代码能编译吗？运行结果是什么？`into_inner()` 是否检查了数据完整性？

<details><summary>解析</summary>

能编译。只要线程 panic 前已经把数据改成某个值，`into_inner()` 会返回那个值，例如 `[1, 2]`。但 `into_inner()` **不检查**业务不变量，它只是绕过中毒标记把守卫交给你。是否安全取决于 panic 发生在哪一步、数据是否可能只更新了一半。对于账目、库存等关键状态，应该返回错误或重建，而不是无条件 `into_inner()`。

</details>

### 预测 5：两把锁的顺序

```rust
let left = Mutex::new(1);
let right = Mutex::new(2);

let first = left.lock().unwrap();
let second = right.lock().unwrap();
```

单独看这段代码会死锁吗？什么情况下会？

<details><summary>解析</summary>

单独一个线程按 `left`、`right` 的顺序获取，不会死锁。但如果另一个线程先获取 `right` 再获取 `left`，两个线程就可能各自持有对方需要的锁，形成循环等待。是否真的发生取决于调度，可能运行很多次都没事，也可能偶尔卡死。多锁代码必须规定全局一致的获取顺序，或者合并成一把锁。

</details>

---

## 十三、主练习：并发倒排索引

### 目标与行为

你要在一个独立 Cargo package 中完成 `src/exercise.rs` 的五个函数，用多个工作线程把一批文档建成倒排索引。**倒排索引**（inverted index）是一张“单词 → 包含它的文档编号列表”的表。搜索引擎先用它快速找到“哪些文档包含这个单词”。

练习重点是：

- 用 `Arc<Mutex<Index>>` 共享一张索引；
- 让分词、逐文档去重和局部索引构建发生在锁外；
- 正确处理线程 panic、锁中毒、处理计数不一致；
- 保证最终结果与线程完成顺序无关。

### 数据结构

脚手架已经给出这些类型，不要修改：

```rust
pub struct Document {
    pub id: usize,
    pub text: String,
}

pub type Index = HashMap<String, Vec<usize>>;
pub type SharedIndex = Arc<Mutex<Index>>;

pub enum IndexError {
    ZeroWorkers,
    WorkerPanicked { worker_id: usize },
    IndexPoisoned,
    ProcessedCountMismatch { expected: usize, actual: usize },
}
```

- `Document.id` 是文档编号，类型 `usize`（平台相关的无符号整数，常用来表示长度和位置编号）；编号允许重复，`text` 是正文。
- `Index` 的键是规范化后的单词，值是按升序排列、无重复的文档编号列表。
- `SharedIndex` 是“可被多个线程共同拥有、并且能互斥修改”的索引。
- `IndexError` 是一个**枚举**（enum，只能取列出的几种值之一的类型），四个变体分别表示：没有工作线程、某个线程 panic、锁中毒、处理数量不对。

### 输入、输出与边界

**正常示例**

输入三份文档，使用两个工作线程：

```text
Document { id: 8, text: "Rust rust, RUST!" }
Document { id: 3, text: "Rust 并发" }
Document { id: 5, text: "并发基础" }
```

`build_index(documents, 2)` 的逻辑结果是：

```text
"rust"      -> [3, 8]
"并发"      -> [3]
"并发基础"  -> [5]
```

同一份文档里的 `Rust`、`rust`、`RUST!` 都规范化为 `rust`，编号 8 只记录一次；不同文档中的 `Rust` 和 `rust` 分别贡献编号 3 和 8。`HashMap` 不保证键的顺序，所以测试比较的是“每个键对应的列表”，不是打印顺序。

**边界和错误**

```text
build_index([], 3)                    -> Ok({})
build_index([任意文档], 0)            -> Err(ZeroWorkers)
build_index([], 0)                    -> Err(ZeroWorkers)
build_index([只有标点的文档], 4)      -> Ok({})
build_index([id=5 "same", id=5 "same"], 2) -> "same" -> [5]
worker_count > 文档数                 -> 仍然创建全部线程，正常返回
```

- 空文档没有索引项，但仍计入“已处理文档数”。
- 不同输入文档使用相同编号时，最终列表要排序并去重。
- 工作线程数大于文档数时，多出来的线程拿到空桶，仍然要创建、启动、join。
- 工作线程数为 0 时，即使输入为空也返回 `ZeroWorkers`。

**错误语义**

- `WorkerPanicked { worker_id }`：编号为 `worker_id` 的线程 panic。若多个线程 panic，报告编号最小的那个。所有线程都必须先 join，不能一发现 panic 就提前返回。
- `IndexPoisoned`：共享索引的 mutex 已被 panic 标记为中毒。不能通过 `into_inner()` 静默忽略。
- `ProcessedCountMismatch { expected, actual }`：所有线程结束后，实际处理计数与输入文档数不同。不能返回部分索引。
- 多个错误同时存在时，优先级是：`WorkerPanicked` → `IndexPoisoned` → `ProcessedCountMismatch`。

**分词规则**

`normalized_words(text: &str) -> Vec<String>` 已经由脚手架提供，不需要你实现。它的行为是：

- 连续的 **Unicode**（一套给世界上各种文字、数字和符号统一编号的标准）字母或数字字符组成一个单词，判断依据是 `char::is_alphanumeric`；
- 空白、标点、下划线和其他非字母数字字符都是分隔符；
- 每个字符通过 `char::to_lowercase` 转小写，一个字符可能变成多个小写字符；
- 不做额外的 Unicode 规范化、兼容折叠或整词大小写折叠；
- 没有单词的文本返回空列表。

实现里用 `std::mem::take` 把已经完成的单词缓冲区取走，并留下一个空缓冲区继续收集下一个词。这是标准库的辅助函数，`normalized_words` 已经写好，不需要修改。

例如：

```text
"C++ Rust_2026 你好，世界！" -> ["c", "rust", "2026", "你好", "世界"]
"Rust_rust's"                -> ["rust", "rust", "s"]
"İ"                          -> ["i\u{307}"]
```

`İ` 是带点的土耳其大写 I，转小写后是 `i` 加一个组合点，所以长度是 2。这里不要求你处理更复杂的 Unicode 等价性；规则以提供的函数为准。

### 并发协议与约束

- **只使用标准库**的 `thread`、`Arc`、`Mutex`、`AtomicUsize` 和已有集合类型；不要添加第三方依赖。
- 不沿用上一课的 channel；本练习要求共享索引 + `Mutex`。
- 不使用 `RwLock` 代替题目要求的 `Mutex`。
- 按文档在输入 `Vec` 中的位置轮流分配：第 0 份给 0 号线程，第 1 份给 1 号，依此类推，分完一轮再从 0 号继续。
- 每份文档及其 `String` 必须**移动**给某个线程，不得深拷贝正文。
- 每个线程先调用 `index_documents` 构建局部索引，再调用 `merge_into` 合并；分词、逐文档去重和局部构建都不能在共享索引锁内进行。
- 每个线程只进行一次（或少量）加锁批量合并，不要每遇到一个单词就锁一次。
- 每完整处理一份文档，`processed` 计数加一，使用 `Ordering::Relaxed`。这个计数只用于检查进度，不能代替索引锁。
- 所有线程都必须 join，即使某个线程已经 panic 或返回错误。
- worker 闭包返回的 `Result::Err(IndexPoisoned)` 是业务错误；`join()` 返回的 `Err` 是线程 panic。两者不能混为一层。
- 成功返回前，把每个文档编号列表排序并去重，结果不能依赖线程完成顺序。
- 不使用 `unsafe`、全局可变状态、`Box::leak`，也不用 `sleep` 猜测线程何时完成。

### 脚手架与要修改的文件

本课 package 名是 `l17_shared_concurrency`，目录是 `lessons/17-shared-concurrency/`。你要修改：

- `src/exercise.rs`：主练习。只替换 `index_documents`、`merge_into`、`finalize`、`join_workers`、`build_index` 里的 `todo!()`。
- 不要修改类型定义、函数签名和文件底部的测试。

`join_workers` 是 `build_index` 的内部步骤，签名是私有函数，但测试会直接调用它来验证“所有线程都 join”“错误优先级”等行为。它的参数里包含一组 `JoinHandle<Result<(), IndexError>>`，第 `i` 个句柄对应 `i` 号线程。

### 推荐完成顺序

1. 先实现 `index_documents`，通过“文档内去重”“空文档计数”“初始计数不为 0”这几个测试。此时不需要考虑锁。
2. 实现 `merge_into`：拿锁、把局部索引追加到共享索引、把中毒转成 `IndexPoisoned`。用测试确认合并和中毒行为。
3. 实现 `finalize`：拿锁、复制快照、释放锁，再排序和去重。不要在锁内做无必要的排序。
4. 实现 `join_workers`：遍历所有句柄，先全部 join；分别记录最先 panic 的编号和第一个业务错误；按优先级返回；最后检查计数并调用 `finalize`。
5. 实现 `build_index`：先处理 0 线程；保存输入总数；按位置轮流分桶；为每个桶启动线程；把句柄交给 `join_workers`。
6. 运行完整测试和 `cargo run -p l17_shared_concurrency`，确认输出与示例一致。

### 测试说明与验收命令

本课共有 **31 个自动测试**：主练习 19 个，课后巩固 12 个。初始脚手架状态下：

- `normalized_words_follows_the_documented_rules` 通过（这个函数已经提供）；
- 其余 **30 个**测试会在 `todo!()` 处 panic 而失败；
- 格式、`cargo check --all-targets`、`cargo test --no-run` 和严格 Clippy 仍然通过。

这是预期起点。实现过程中，测试失败信息会指出哪个函数还没有完成，不要通过修改测试来“修复”。

在仓库根目录运行：

```bash
cargo fmt -p l17_shared_concurrency -- --check
cargo check -p l17_shared_concurrency --all-targets
cargo test -p l17_shared_concurrency --no-run
cargo test -p l17_shared_concurrency
cargo clippy -p l17_shared_concurrency --all-targets -- -D warnings
cargo run -p l17_shared_concurrency
```

完成后，`cargo test` 应显示 31 个测试全部通过；`cargo run` 应输出：

```text
rust: [3, 8]
并发: [3]
并发基础: [5]
```

如果测试输出里出现工作线程的 panic 信息，但对应测试仍然 `ok`，那是“中毒测试”故意让子线程 panic 造成的，不是测试失败。

---

## 十四、课后巩固：并发汇总传感器读数

主练习处理文本和文档编号，用 `Mutex`；这个任务换一种数据、换一种锁：汇总带正负值的温度读数，用 `RwLock` 保存一张按传感器名字排序的统计表。

### 目标与行为

修改 `src/consolidation.rs`，完成四个函数：

- `summarize_batch(readings) -> BTreeMap<String, SensorSummary>`：顺序汇总一个线程分到的读数。
- `merge_summaries(&shared, local) -> Result<(), SensorError>`：把局部统计合并进共享表。
- `join_summaries(handles, &shared) -> Result<Vec<(String, SensorSummary)>, SensorError>`：等待所有线程，处理错误，读取最终快照。
- `summarize_readings(readings, worker_count) -> Result<Vec<(String, SensorSummary)>, SensorError>`：分桶、启动线程、调用上面三个函数。

数据结构：

```rust
pub struct Reading {
    pub sensor: String,
    pub celsius: i32,
}

pub struct SensorSummary {
    pub count: usize,
    pub min: i32,
    pub max: i32,
    pub sum: i64,
}

pub type SharedSummary = Arc<RwLock<BTreeMap<String, SensorSummary>>>;

pub enum SensorError {
    ZeroWorkers,
    WorkerPanicked { worker_id: usize },
    Poisoned,
}
```

`count` 是读数条数；`min`/`max` 是最低/最高温度；`sum` 用 `i64`（64 位有符号整数，可表示正负），因为很多 `i32`（32 位有符号整数）相加可能超出 `i32` 范围。`BTreeMap`（一种按键自动排序的键值集合）会按字符串键自动排序，所以最终列表天然有序。

### 输入、输出与边界

输入五条读数，使用三个线程：

```text
Reading { sensor: "kitchen", celsius: 21 }
Reading { sensor: "harbor",  celsius: 18 }
Reading { sensor: "kitchen", celsius: -3 }
Reading { sensor: "balcony", celsius: 30 }
Reading { sensor: "harbor",  celsius: 22 }
```

期望输出（按传感器名升序）：

```text
[
  ("balcony", SensorSummary { count: 1, min: 30, max: 30, sum: 30 }),
  ("harbor",  SensorSummary { count: 2, min: 18, max: 22, sum: 40 }),
  ("kitchen", SensorSummary { count: 2, min: -3, max: 21, sum: 18 }),
]
```

注意：

- 同一个传感器跨多个批次出现时，`count` 相加，`sum` 相加，`min` 取更小，`max` 取更大；
- 负数温度是合法读数，不能当成“没有数据”；
- 空输入 + 正数线程数返回 `Ok(vec![])`；
- `worker_count == 0` 时，即使输入为空也返回 `Err(ZeroWorkers)`；
- 线程数大于读数条数时，多出来的线程处理空批次，仍然要创建并 join；
- 返回列表按传感器名升序，不能依赖线程完成顺序；
- `WorkerPanicked` 的优先级高于业务错误；所有线程都要 join。

### 约束与验收

- 只使用标准库；数据必须移动给线程，不得深拷贝 `sensor` 字符串。
- 局部统计必须在写锁之外完成；每个线程只做少量加锁合并。
- 不使用 `unsafe`、全局可变状态、`sleep` 轮询。
- 完成后运行：

```bash
cargo test -p l17_shared_concurrency consolidation
cargo clippy -p l17_shared_concurrency --all-targets -- -D warnings
```

课后巩固共 12 个测试，覆盖正常聚合、空输入、0 线程、线程数超过读数、负数、重复传感器、锁中毒、线程 panic、处理顺序无关等边界。测试名称描述行为，先读名称再读断言。

---

## 十五、常见误区

### 误区 1：`Arc<T>` 让 `T` 变成线程安全

`Arc` 只让多个线程共同拥有 `T`，并保证最后一个所有者释放它。它不提供互斥，也不让内部类型自动满足 `Send`/`Sync`。`Arc<RefCell<T>>` 仍然不能跨线程。正确的判断是看**最内层**类型：`RefCell` 不是 `Sync`，换成 `Mutex` 或 `RwLock`。

### 误区 2：有了 `Mutex` 就不需要 `Arc`

`Mutex` 保护数据，但把 `Mutex` 移动给一个线程后，其他线程就拿不到它。普通 `thread::spawn` 还要求闭包 `'static`，不能借用当前函数的局部 `Mutex`。除非使用第十六课的 `thread::scope`，否则共享所有权仍然需要 `Arc`。

### 误区 3：锁住的时间越长越安全

锁只保证互斥，不保证性能。持锁做网络请求、睡眠、读大文件、调用未知回调，会让其他线程长时间等待，也更容易死锁。正确做法是让临界区尽可能短：先在锁外准备数据，锁内只做必须原子完成的修改。

### 误区 4：`RwLock` 一定比 `Mutex` 快

读写锁需要维护读者数量、写者等待等状态；写频繁或临界区很短时，开销可能更高。某些实现还可能让写者饥饿。没有测量就不要替换；从 `Mutex` 开始。

### 误区 5：中毒后 `into_inner()` 总是安全

`into_inner()` 只是绕过中毒标记，不验证业务不变量。若 panic 发生在“扣款后、入账前”，你拿到的就是半完成状态。要按领域决定：返回错误、重建、还是验证后恢复。

### 误区 6：用原子类型保护多个字段

两个 `AtomicUsize` 各自原子，不代表它们之间的组合原子。例如“库存减一”和“售出数加一”是两个独立操作，中间可能被其他线程看到。需要整体一致性时用一把 `Mutex` 保护一个**结构体**（struct，把多个相关字段组合成一个类型）。

### 误区 7：克隆 `Arc` 和克隆内部值一样

`Arc::clone` 共享同一份数据，计数加一；`(*arc).clone()` 复制内部值。前者用于共享状态，后者得到独立副本，之后的修改不会互相影响。两者名字都叫 `clone`，但语义完全不同。

### 误区 8：线程完成顺序是固定的

调度器不保证顺序。不要写“线程 A 一定先完成，所以 B 能看到 A 的结果”这种代码。需要顺序时，用 `join`、channel 或锁建立明确的先后关系；需要稳定输出时，显式排序。

### 误区 9：忽略 `join()` 的 `Err`

`join()` 返回 `Result`。线程 panic 时它返回 `Err`，不是自动把 panic 传给主线程。忽略它，你可能在“少了一个线程的结果”时仍然返回成功。本练习要求所有线程都 join，并按优先级报告 panic。

### 误区 10：编译通过就等于并发正确

Rust 阻止数据竞争，但不阻止死锁、逻辑竞争、持锁阻塞、错误的锁顺序、业务不变量被破坏。并发正确性需要设计协议、写测试、审查锁的作用域，不能只看“能编译”。

### 误区 11：用 `sleep` 等待另一个线程

`sleep` 只是让当前线程暂停一段猜测的时间；机器负载变化时，猜测会失效。用 `join` 等待线程结束，用 channel 等待消息，用锁/条件变量等待状态变化。

### 误区 12：为每个小任务创建一个线程

线程创建有成本，线程太多会增加调度开销。固定数量 worker + 任务队列（channel）通常更稳。本课练习要求固定 worker 数，也是这个原因。

---

## 十六、本课总结

- **线程**是进程内的执行路线，共享堆但各有栈；调度器决定运行顺序，顺序不确定。
- **数据竞争**需要“同时访问 + 至少一个写 + 无同步”三个条件；Rust 安全代码在编译期阻止数据竞争，但不阻止逻辑错误。
- **所有权**回答“谁负责释放”，**同步**回答“谁能在什么时候访问”。这是两个问题。
- **`Arc<T>`** 用原子引用计数让多个线程共同拥有一个值，最后一个所有者释放它；它不提供内部可变性。
- **`Mutex<T>`** 通过 `lock()` 返回守卫，同一时刻只允许一个线程访问；守卫离开作用域自动解锁（RAII）。
- **锁的作用域**应尽量小；不要在持锁时做慢操作、调用未知回调或再次锁同一把锁。
- **中毒**表示持锁线程 panic，数据可能处于半更新状态；用 `match` 显式决定恢复还是报错，不要无条件 `into_inner()`。
- **死锁**是运行期逻辑问题，类型系统不检查；用固定锁顺序、单锁设计、`try_lock` 或 channel 降低风险。
- **`RwLock<T>`** 允许多个读者或一个写者；只有读多写少且测量证明有收益时才考虑。
- **`Send`** 管所有权能否跨线程移动，**`Sync`** 管共享引用能否跨线程使用；`Rc`/`RefCell` 不是线程安全的，`Arc<Mutex<T>>` 才是常见组合。
- **原子类型**适合单个独立状态；`Ordering::Relaxed` 只保证该原子值自身的原子性，不建立跨变量同步。
- **channel 与共享状态**各有适用场景；先问“有没有单一所有者”，再决定是否引入锁。

---

## 十七、课后复盘

合上 README，用自己的话回答：

1. 为什么 `Rc<T>` 不能跨线程，而 `Arc<T>` 可以？`Arc` 的“原子”具体指什么？
2. `Arc<T>` 和 `Mutex<T>` 各自解决哪一半问题？为什么通常要组合使用？
3. `MutexGuard` 是什么？它为什么能在离开作用域时自动解锁？这和“忘记解锁”有什么关系？
4. 为什么锁内不应该做网络请求或 `sleep`？如果必须在锁外做，怎样把需要的数据带出来？
5. 什么是中毒？`into_inner()` 做了什么、没做什么？
6. 死锁的四个必要条件里，本课最常利用哪一条来预防？举一个固定锁顺序的例子。
7. `Send` 和 `Sync` 的区别是什么？为什么 `Arc<RefCell<T>>` 不能跨线程？
8. 原子计数器和 `Mutex` 计数器各适合什么场景？为什么不能用两个原子变量维护一个复合不变量？
9. 主练习中，为什么每个 worker 要先建局部索引再合并？锁的准确持有范围是什么？
10. 如果主练习改用 channel 把局部索引发给一个汇总线程，所有权结构会怎样变化？还需要 `Arc<Mutex<Index>>` 吗？

---

## 下一课预告

这一课处理的是“内存里的共享状态”。真实程序还要面对程序边界：命令行参数、文件路径、打开文件、按块读取、写入时的缓冲，以及怎样把 `io::Error` 穿过清晰的应用错误类型。

[第十八课](../18-files-cli/README.md)会把这些边界逐个拆开，并用一个完整的命令行工具串起来。本课的并发工具不会消失：当你需要并发处理多个文件时，仍然要判断哪些状态该共享、哪些该通过 channel 传递。
