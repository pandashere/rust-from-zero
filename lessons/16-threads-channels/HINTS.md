# 第十六课提示：主练习与课后巩固

提示分三级。先看一级；想清楚了就不要继续往下看。

- **一级：思考方向**——只提示该想什么，不给代码结构。
- **二级：局部分解**——把任务拆成几个判断步骤，仍不写完整代码。
- **三级：接近完整的结构**——给出带空缺的骨架，关键调用和边界要你自己补。

三级提示故意不给出可以直接提交的完整实现。

主练习建议按 README 的顺序完成：先 `checksum`，再 `run_pipeline_with` 的 worker 启动，然后分发、关闭、收集、join，最后处理错误和排序。

---

## 主练习：`checksum`（`src/exercise.rs`）

### 一级：思考方向

你有一个从 `0` 开始的累加器，要按顺序“吞掉”每个字节。每个字节要做什么运算，题目已经写成公式。关键在溢出：普通乘加在 debug 构建中可能 panic，而题目要求“回绕”。标准库的 `u64` 有没有名字里带 `wrapping` 的方法？空切片时循环会发生什么？

### 二级：局部分解

1. 准备一个 `u64` 累加器，初值 `0`。
2. 用 `for` 循环按顺序取出每个字节。字节在切片里是 `&u8`，需要取出它代表的 `0..=255` 整数，再转成 `u64`。
3. 每个字节执行一次“乘 31 再加字节值”。乘法和加法都要用回绕版本。
4. 循环结束后返回累加器。

### 三级：接近完整的结构

```rust
let mut acc = 0_u64;
for byte in bytes {
    acc = acc
        ./* 回绕乘法 */(31)
        ./* 回绕加法 */(u64::from(*byte));
}
acc
```

如果这里用普通的 `*` 和 `+`，`[255; 13]` 这组测试会在 debug 构建中 panic。想想 `wrapping_mul` 与 `wrapping_add` 分别替代哪个运算符。

---

## 主练习：`run_pipeline_with`（`src/exercise.rs`）

### 一级：思考方向

先把问题拆成四个独立阶段：

1. **配置检查**：`worker_count == 0` 时，必须在创建任何线程之前返回错误。
2. **建立通信**：任务需要“主线程分给某个 worker”，结果需要“多个 worker 送回主线程”。这是两个方向相反的通道。
3. **关闭与收集**：worker 怎样知道没有新任务了？主线程怎样知道所有结果都到齐了？分别由哪些 sender 的销毁决定？
4. **清理与判定**：先收集还是先 join？panic 和结果数量不足，哪个错误优先？

在移动 `tasks` 之前，先把任务总数保存下来；移动之后就数不了了。任务 ID 用来恢复顺序，worker ID 只说明实际处理者。

### 二级：局部分解

1. 如果 `worker_count == 0`，立即返回 `ZeroWorkers`。
2. 记录 `expected = tasks.len()`。
3. 创建结果通道：一个发送端，一个接收端。
4. 对每个 `worker_id in 0..worker_count`：
   - 创建这个 worker 专属的任务通道；
   - 克隆结果发送端，给这个 worker 用；
   - 启动线程，把任务接收端、结果发送端克隆、`worker_id` 和 `work` 移进去；
   - 保存任务发送端和 `JoinHandle`。
5. worker 内部：不断接收任务；对每个任务的 `bytes` 调用 `work`；得到 `Some(value)` 就构造 `Checksum` 并发送；得到 `None` 就继续下一个任务。任务通道关闭时循环结束。
6. 所有 worker 启动后，主线程不再发送结果，销毁原始结果发送端。
7. 按输入位置 `position % worker_count` 选择任务发送端，发送任务。发送失败也不要在这里 `return`：后面还有关闭、收集和 join 要做。
8. 所有任务发送完，销毁全部任务发送端。
9. 从结果接收端收集结果，直到通道关闭。
10. 逐个 `join` 已经创建的 worker，记录是否有 panic；多个 panic 时保留编号最小的。
11. 全部 join 完成后，按优先级返回：先 panic，再结果数量不匹配，最后排序并返回 `Ok`。

### 三级：接近完整的结构

```rust
if worker_count == 0 {
    return /* 配置错误 */;
}

let expected = tasks.len();
let (result_sender, result_receiver) = /* 创建结果通道 */;
let mut task_senders = Vec::with_capacity(worker_count);
let mut handles = Vec::with_capacity(worker_count);

for worker_id in 0..worker_count {
    let (task_sender, task_receiver) = /* 创建这个 worker 的任务通道 */;
    let worker_results = /* 克隆结果发送端 */;

    let handle = std::thread::spawn(move || {
        for task in task_receiver {
            // 调用 work(&task.bytes)；
            // Some(value) => 构造 Checksum { id, value, worker_id } 并发送；
            // None => 这个任务不产生结果，继续；
            // 发送失败说明接收端已经不在，可以结束这个 worker。
        }
    });

    task_senders.push(task_sender);
    handles.push(handle);
}

drop(result_sender);

let mut send_failed = false;
for (position, task) in tasks.into_iter().enumerate() {
    let worker_id = position % worker_count;
    if task_senders[worker_id].send(task).is_err() {
        send_failed = true; // 不要在这里 return
    }
}

drop(task_senders);

let results: Vec<_> = /* 收集结果，直到通道关闭 */;

let mut panicked_worker = None;
for (worker_id, handle) in handles.into_iter().enumerate() {
    if handle.join().is_err() && panicked_worker.is_none() {
        // 记录最小的 panic worker 编号
    }
}

// 先判断 panicked_worker，再判断 send_failed / results.len() != expected，最后排序。
```

几个自检问题：

- `result_sender` 为什么要在启动完所有 worker 之后 `drop`？如果忘了，收集循环会怎样？
- 如果某个 worker panic，它持有的结果发送端会怎样？主线程给它的任务发送会怎样？
- 为什么不能先 `return` 再清理？哪些 worker 会因此没有被 join？
- `work` 返回 `None` 与 worker panic 都会让结果变少，但错误类型为什么不同？

---

## 课后巩固：`parallel_stats`（`src/consolidation.rs`）

### 一级：思考方向

先处理“没有数据”的情况：没有整数时，最小值和最大值都不存在，应该返回 `None`。

然后考虑分工：

- `workers` 为 `0` 时不能拿它做除数，应该把它当作几个 worker？
- `workers` 比整数个数还多时，需要限制实际使用的 worker 数吗？
- 每个 worker 处理多长的一段？怎样把“总数除以 worker 数”向上取整？
- 作用域线程可以直接借用 `values` 的分段吗？需要复制整个 `Vec` 吗？

每个 worker 可以先算自己那一段的局部统计，主线程再把局部结果合并。合并最小值用“取较小者”，合并最大值用“取较大者”，总和用加法；总和要转成 `i128` 再累加。

### 二级：局部分解

1. `values.is_empty()` 时返回 `None`。
2. 把 worker 数规范成至少 `1`，最多不超过整数个数。
3. 计算每段长度：`values.len().div_ceil(实际 worker 数)`。`div_ceil` 是“向上取整的除法”。
4. 用 `thread::scope` 创建作用域；用 `values.chunks(chunk_size)` 切分。`chunks(n)` 把切片切成若干连续子切片，每段最多 `n` 个元素，最后一段可能更短；每个 chunk 都非空。
5. 每个作用域线程处理一个 chunk：
   - `count` 就是 chunk 长度；
   - `min` 和 `max` 从 chunk 的第一个元素开始；
   - 遍历 chunk，更新最小值和最大值；
   - 把每个元素转成 `i128` 后累加。
6. 收集所有 handle，逐个 `join`，把局部结果合并。
7. 返回 `Some(Stats { ... })`。

### 三级：接近完整的结构

```rust
if values.is_empty() {
    return None;
}

let workers = /* 至少 1，且不超过 values.len() */;
let chunk_size = values.len()./* 向上取整的除法 */(workers);

Some(std::thread::scope(|scope| {
    let handles: Vec<_> = values
        .chunks(chunk_size)
        .map(|chunk| {
            scope.spawn(move || {
                // 返回 (count, min, max, sum)
                // min/max 从 chunk[0] 开始；
                // sum 用 i128 累加。
            })
        })
        .collect();

    // join 每个 handle，合并 count/min/max/sum。
}))
```

自检：

- `chunk[0]` 为什么在这里一定安全？`chunks` 会不会产生空段？
- 如果 `workers` 大于 `values.len()`，`chunk_size` 是多少？会创建多少个 chunk？
- 为什么 `sum` 不直接用 `i64` 累加？用 `[i64::MAX, i64::MIN, i64::MIN]` 走一遍：真实和是 `i64::MIN - 1`，已经低于 `i64::MIN`；如果用 `i64` 回绕，会得到一个错误的正数。
- 这里为什么不需要 channel？作用域线程的借用边界解决了什么问题？
