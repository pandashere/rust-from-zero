# 第十九课提示：主练习与课后巩固

提示分三级，卡到哪一级看哪一级：

- **一级：思考方向**——只提示该想什么，不涉及代码结构。
- **二级：局部分解**——把方法拆成几个判断步骤，仍不写完整代码。
- **三级：接近完整的结构**——给出带空缺的骨架，你需要自己补上关键调用。

先看一级；想清楚了就不要继续往下看。三级提示故意不给出可直接提交的完整代码。

主练习按 README 推荐的顺序完成函数：`with_timeout` → `send_with_retries` → `run_batch`。前一个不对，后一个一定会别扭。

---

## 主练习一：`with_timeout`（`src/exercise.rs`）

### 一级：思考方向

你要写的是“套在别人 future 外面的壳”。壳自己不产生结果，它只做两件事：转发 poll，和数 `Pending`。

两个障碍先想清楚：

1. `Future::poll` 的接收者是 `Pin<&mut Self>`，而普通的 `let mut f = future;` 是**不固定**的。标准库有没有一个宏，能把一个值固定在当前栈帧上、然后给你 `Pin<&mut T>`？
2. 你不能直接调用 `.poll()`（那是 trait 方法，需要 `Context`）。标准库有一个工具，能把“一个接收 `&mut Context`、返回 `Poll<T>` 的闭包”变成 future。它叫什么？

数配额的时机要想准确：一个 future 返回 `Pending` 才消耗一次配额。当配额已经用完、而这次 poll 又是 `Pending` 时，才判定超时。

### 二级：局部分解

1. 用固定宏把参数 `future` 固定住，得到一个能反复交出 `Pin<&mut F>` 的绑定。
2. 设一个可变计数器，初值 0。
3. 用把闭包变成 future 的工具包住一个闭包，闭包参数是 `&mut Context`：
   - 先 poll 内部 future；
   - 得到 `Ready(value)` → 立刻返回 `Ready(Ok(value))`；
   - 得到 `Pending` → 判断：计数器是否已经达到 `patience`？
     - 已达到 → 返回 `Ready(Err(TimedOut))`；
     - 未达到 → 计数器加一，返回 `Pending`。
4. 对第 3 步得到的 future 使用 `.await`，把它当作本函数的返回值表达式。

### 三级：接近完整的结构

```rust
pub async fn with_timeout<F: Future>(
    future: F,
    patience: usize,
) -> Result<F::Output, TimedOut> {
    let mut future = std::pin::/* 固定在当前栈帧上的宏 */(future);
    let mut pending_polls = 0;

    std::future::/* 把闭包变成 future 的函数 */(|cx| {
        match future.as_mut().poll(cx) {
            Poll::Ready(value) => Poll::Ready(Ok(value)),
            Poll::Pending => {
                if pending_polls >= patience {
                    Poll::Ready(Err(TimedOut))
                } else {
                    pending_polls += 1;
                    /* 这次还不能完成 */
                }
            }
        }
    })
    .await
}
```

验证三个边界：

- `patience = 0` 且 future 第一次就完成 → 走 `Ready` 分支，返回 `Ok`；
- `patience = 0` 且 future 第一次 `Pending` → 计数器 0 已达到 0，立刻超时；
- `patience = 2` 且内部恰好 `Pending` 两次 → 两次都走“加一”分支，第三次 `Ready` 成功。

---

## 主练习二：`send_with_retries`

### 一级：思考方向

这个函数是三层结果的“翻译器”。三层从内到外是：

```text
panic?  →  超时?  →  服务返回什么?
```

别急着写循环。先在纸上把“一次尝试”的表达式写出来：**调用 `service.send(job, n)` → 包超时 → 包 panic 捕获**。这个表达式求值后是一个 `Result<Result<Result<String, ServiceFault>, TimedOut>, TaskPanic>`。

然后再想：什么情况下继续下一轮？只有最内层是 `Temporary` 且次数还没用完。其余情况全部“立即返回”。

`attempts` 字段要填**当前尝试编号**，不是剩余次数。

### 二级：局部分解

1. 用 `for attempt in 1..=policy.max_attempts` 遍历尝试编号。
2. 循环体里构造一次尝试：
   - `catch_panic(with_timeout(service.send(job, attempt), policy.patience))`：
     先造出 `send` 的 future，交给超时包装，再用 `catch_panic` 把**整个**
     超时包装包住；
   - 只对最外层这个 future `.await` 一次，得到
     `Result<Result<Result<String, ServiceFault>, TimedOut>, TaskPanic>`；
   - 不要先 `.await` 超时结果、再对结果调用 `catch_panic`：那时 `send` 的
     panic 早就发生了，而且 `Result` 不是 future，代码既抓不到 panic 也
     编译不过。
3. 用 `match` 逐层拆（从外到内：panic、超时、服务结果）：
   - 外层 `Err(TaskPanic)` → 返回 `Failed { error: Panicked, attempts: attempt }`；
   - 中间层 `Ok(Err(TimedOut))` → 返回 `Failed { error: TimedOut, attempts: attempt }`；
   - 内层 `Ok(Ok(Ok(output)))` → 返回 `Success { id: job.id, output, attempts: attempt }`；
   - 内层 `Ok(Ok(Err(ServiceFault::Permanent { reason })))` → 返回
     `Failed { error: Permanent { reason }, attempts: attempt }`；
   - 内层 `Ok(Ok(Err(ServiceFault::Temporary { reason })))` → 如果
     `attempt == policy.max_attempts`，返回
     `Failed { error: TemporaryExhausted { last_reason: reason }, attempts: attempt }`；
     否则 `continue` 进入下一轮。
4. 循环结束后理论上不会到达（`max_attempts >= 1` 保证至少跑一轮，且每轮要么返回要么继续）。用一个 `unreachable!` 或等价写法表达这个不变量。

### 三级：接近完整的结构

```rust
pub async fn send_with_retries(
    service: &FakeService,
    job: &Job,
    policy: RetryPolicy,
) -> JobOutcome {
    for attempt in 1..=policy.max_attempts {
        let attempt_result = /* 最外层：捕获 panic */ (
            /* 中间层：加超时 */ (service.send(job, attempt), policy.patience),
        )
        .await;

        match attempt_result {
            /* 外层：panic */ => return JobOutcome::Failed {
                id: job.id,
                error: JobError::Panicked,
                attempts: attempt,
            },
            /* 中间层：超时 */ => return JobOutcome::Failed {
                id: job.id,
                error: JobError::TimedOut,
                attempts: attempt,
            },
            Ok(Ok(Ok(output))) => return JobOutcome::Success {
                id: job.id,
                output,
                attempts: attempt,
            },
            Ok(Ok(Err(ServiceFault::Permanent { reason }))) => return JobOutcome::Failed {
                id: job.id,
                error: JobError::Permanent { reason },
                attempts: attempt,
            },
            Ok(Ok(Err(ServiceFault::Temporary { reason }))) => {
                if attempt == policy.max_attempts {
                    return JobOutcome::Failed {
                        id: job.id,
                        error: JobError::TemporaryExhausted { last_reason: reason },
                        attempts: attempt,
                    };
                }
                /* 还有次数，进入下一轮 */
            }
        }
    }
    unreachable!("max_attempts >= 1，循环必然在某一轮返回")
}
```

为什么 `catch_panic` 必须在 `with_timeout` 外面，而不是里面？想一想：谁负责把 `panic` 变成 `Err(TaskPanic)`——如果超时包装先 panic 了，还能返回 `Err(TimedOut)` 吗？

---

## 主练习三：`run_batch`

### 一级：思考方向

分三段写，每段独立可验证：

1. **配置检查**：两个判断，顺序固定（先 `limit`，后 `max_attempts`）。这一点有专门的测试。
2. **切批**：把 `jobs` 按输入顺序切成每批最多 `limit` 条。标准库的 `chunks` 或 `chunks_mut` 正好做这件事（注意参数是每批大小，不是批数）。
3. **跑批并收集**：每批为每条作业构造一个 `send_with_retries(...)` 的 future，用 `join_all` 并发推进，把结果追加到总表。全部批跑完后按 `id` 排序。

排序放在最后，不要指望 `join_all` 的输出顺序等于输入顺序——虽然本课的 `join_all` 确实按输入顺序返回，但依赖这个顺序会让代码在未来换运行时后出错。

### 二级：局部分解

1. 配置检查：
   - `if config.limit == 0 { return Err(BatchError::ZeroLimit); }`
   - `if config.policy.max_attempts == 0 { return Err(BatchError::ZeroAttempts); }`
2. 准备一个可变的 `Vec<JobOutcome>` 收集结果。
3. `for chunk in jobs.chunks(config.limit)`：
   - 对 `chunk` 里每个 `job` 构造 `send_with_retries(service, job, config.policy)`；
   - 收集成一个 `Vec<future>`；
   - `join_all(futures).await` 得到 `Vec<JobOutcome>`；
   - 把它们追加进总表。
4. 总表 `sort_by_key(|outcome| outcome.id())`。
5. 返回 `Ok(BatchReport { outcomes: 总表 })`。

注意第 3 步的借用：`job` 是从 `jobs` 里借出来的 `&Job`，而 `send_with_retries` 的签名正好接受 `&Job`，所以不用克隆。但 `join_all` 的 future 会借用 `jobs`，因此 `jobs` 必须活到循环结束——它本来就是函数参数，没问题。

### 三级：接近完整的结构

```rust
pub async fn run_batch(
    service: &FakeService,
    jobs: Vec<Job>,
    config: BatchConfig,
) -> Result<BatchReport, BatchError> {
    if config.limit == 0 {
        return /* 哪个错误 */;
    }
    if config.policy.max_attempts == 0 {
        return /* 哪个错误 */;
    }

    let mut outcomes = Vec::with_capacity(jobs.len());

    for chunk in jobs./* 按每批大小切片的迭代方法 */(config.limit) {
        let futures = chunk
            .iter()
            .map(|job| /* 对单条作业跑重试流程 */ )
            .collect::<Vec<_>>();

        outcomes./* 把一批结果追加到总表 */(join_all(futures).await);
    }

    outcomes.sort_by_key(/* 按什么排序 */);

    Ok(BatchReport { outcomes })
}
```

验证三个边界：

- `jobs` 为空 → 不进入循环，`outcomes` 为空；
- `jobs.len() < limit` → 只有一批，`chunks` 会给出整个切片；
- `limit = 1` → 每批一个，`peak_active` 最多 1。

---

## 课后巩固一：`first_ready`（`src/consolidation.rs`）

### 一级：思考方向

这就是 README 第八节 `join_all` 的“只看第一个赢家”版本。区别只有一个：一旦某一方完成，立刻返回，不要等另一方。

难点在“两边都还没完成时怎么办”。你同样需要手动 poll 两个 future，所以又回到那两个工具：固定宏 + 把闭包变成 future。

注意 `left` 和 `right` 是**两个不同类型**，不能装进同一个 `Vec`。所以不能用 `join_all`，要分别固定、分别 poll。

“同一轮都完成时 `Left` 获胜”这条规则，是**检查顺序**自动带来的：先检查 `left`，如果它 `Ready` 就直接返回，`right` 根本没被 poll。

### 二级：局部分解

1. 用固定宏分别固定 `left` 和 `right`。
2. 用把闭包变成 future 的工具包住一个闭包，闭包参数是 `&mut Context`：
   - poll `left`：`Ready(value)` → 返回 `Ready(Winner::Left(value))`；
   - 否则 poll `right`：`Ready(value)` → 返回 `Ready(Winner::Right(value))`；
   - 两个都 `Pending` → 返回 `Pending`。
3. `.await` 这个 future，作为函数返回值。

注意每一步都要用 `as_mut()` 拿到新的 `Pin<&mut _>`，因为 poll 会消耗掉 `Pin` 本身的所有权。

### 三级：接近完整的结构

```rust
pub async fn first_ready<A, B>(left: A, right: B) -> Winner<A::Output, B::Output>
where
    A: Future,
    B: Future,
{
    let mut left = std::pin::/* 固定宏 */(left);
    let mut right = std::pin::/* 固定宏 */(right);

    std::future::/* 把闭包变成 future */(|cx| {
        if let Poll::Ready(value) = left.as_mut().poll(cx) {
            return Poll::Ready(/* 谁赢了 */);
        }
        if let Poll::Ready(value) = right.as_mut().poll(cx) {
            return Poll::Ready(/* 谁赢了 */);
        }
        /* 两个都还没完成 */
    })
    .await
}
```

为什么这个函数返回时，输掉的那个 future 一定会被丢弃？提示：它是这个 async 函数的局部变量，函数返回时局部变量会怎样。

---

## 课后巩固二：`polls_until_ready`

### 一级：思考方向

这个函数把 README 第四节的 `Poll` 模型变成两个可观察的值：输出，和一个数字。

结构上它和 `with_timeout` 几乎一样，区别是**没有上限**：数到 future 完成为止。

`Pending` 的计数含义要精确：**每出现一次 `Pending` 就加一**，所以“第一次 poll 就完成”是 0，不是 1。

### 二级：局部分解

1. 固定参数 `future`。
2. 设一个计数器，初值 0。
3. 用把闭包变成 future 的工具包住闭包：
   - `Ready(value)` → 返回 `Ready((value, 计数器))`；
   - `Pending` → 计数器加一，返回 `Pending`。
4. `.await` 它，作为返回值。

### 三级：接近完整的结构

```rust
pub async fn polls_until_ready<F: Future>(future: F) -> (F::Output, usize) {
    let mut future = std::pin::/* 固定宏 */(future);
    let mut pending_polls = 0;

    std::future::/* 把闭包变成 future */(|cx| {
        match future.as_mut().poll(cx) {
            Poll::Ready(value) => Poll::Ready((/* 输出 */, /* 计数 */)),
            Poll::Pending => {
                /* 计数如何变化 */
                /* 这次还不能完成 */
            }
        }
    })
    .await
}
```

如果把这个函数用在一个永不完成的 future 上会怎样？谁会发现它卡住了？想想 `runtime::block_on` 里的那个上限常量。
