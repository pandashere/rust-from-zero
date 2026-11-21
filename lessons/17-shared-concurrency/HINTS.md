# 第十七课提示：主练习与课后巩固

提示分三级，卡到哪一级看哪一级：

- **一级：思考方向**——只提示该想什么，不涉及代码结构。
- **二级：局部分解**——把函数拆成几个判断或数据变换步骤，仍不写完整代码。
- **三级：接近完整的结构**——给出带空缺的骨架，你需要自己补上关键调用。

先看一级；想清楚了就不要继续往下看。三级提示故意不给出可直接提交的完整代码。

建议顺序：先完成主练习的 `index_documents`，再 `merge_into`、`finalize`、`join_workers`、`build_index`；主练习全部通过后再做课后巩固。每完成一个函数就运行对应测试，不要等五个函数都写完才编译。

```bash
cargo test -p l17_shared_concurrency exercise
cargo test -p l17_shared_concurrency consolidation
```

测试失败时先读测试名称和断言，再读函数上方的行为注释。不要修改测试来绕过 `todo!()`。

---

# 主练习：`index_documents`

## 一级：思考方向

这个函数不创建线程，也不碰共享索引。它只需要回答两个问题：怎样把“当前文档的单词”变成“每个单词只对应一次当前编号”，以及“什么时候算这份文档处理完成”。同一份文档里重复出现的单词要去重，但不同文档之间不去重——每个文档都要独立贡献自己的编号。

## 二级：局部分解

1. 准备一个只属于本次调用的 `Index`，初始为空。
2. 遍历传入的每一份文档。
3. 对当前文档调用 `normalized_words(&document.text)`，得到单词列表。
4. 需要一个临时集合，记录“这份文档里已经见过的单词”。集合里没有这个单词时，才把 `document.id` 追加到本地索引的对应列表；已经见过就跳过。
5. 一份文档的全部单词处理完后，把 `processed` 加一。加法发生在文档处理完成之后，使用 `Ordering::Relaxed`。
6. 返回本地索引。空文档也要执行第 5 步。

想一想：如果把“已见过”集合放到循环外面，会发生什么？不同文档的同名单词会被错误地去重，后面的文档编号就丢了。

## 三级：接近完整的结构

```rust
pub fn index_documents(documents: Vec<Document>, processed: &AtomicUsize) -> Index {
    let mut local = /* 新建一个空 Index */;

    for document in documents {
        let mut seen = /* 新建一个空集合，用来记录当前文档见过的单词 */;

        for word in normalized_words(/* 当前文档的正文 */) {
            if seen./* 集合的“插入并返回是否新插入”方法 */(word.clone()) {
                local
                    ./* 取得或创建词条的方法 */(word)
                    .or_default()
                    .push(/* 当前文档编号 */);
            }
        }

        processed.fetch_add(1, /* 只保证本值原子性的内存顺序 */);
    }

    local
}
```

顶部的 `use` 目前没有引入集合类型和 `Ordering`；实现时按编译器的提示补上需要的 `use`。不要为了消掉“未使用导入”而删掉后面要用的类型。

---

# 主练习：`merge_into`

## 一级：思考方向

这个函数只做“把局部索引搬进共享索引”这一件事。共享索引在 `Mutex` 里面，所以要先拿到独占访问。局部索引里的每个单词可能已经存在，也可能不存在；追加编号时不需要排序或去重，那是 `finalize` 的职责。拿到锁失败时，不要把错误吞掉。

## 二级：局部分解

1. 调用共享索引的 `lock()`。成功得到守卫，失败得到 `PoisonError`。
2. 把 `PoisonError` 转成题目定义的 `IndexError::IndexPoisoned`。可以用 `map_err`，也可以 `match`；不要 `unwrap()`，也不要用 `into_inner()` 绕过。
3. 遍历局部索引的每个 `(单词, 编号列表)`。
4. 在共享索引中为这个单词取得列表；不存在就创建空列表。
5. 把局部列表里的所有编号追加到共享列表末尾。
6. 守卫在函数结束时自动释放；返回 `Ok(())`。

想一想：为什么这里可以用 `extend` 一次追加多个编号，而不是对每个编号单独 `push`？这两种写法在锁的持有时间上有什么区别？

## 三级：接近完整的结构

```rust
pub fn merge_into(shared: &SharedIndex, local: Index) -> Result<(), IndexError> {
    let mut shared_index = shared
        .lock()
        .map_err(|_| /* 中毒对应的错误变体 */)?;

    for (word, ids) in local {
        shared_index
            ./* 取得或创建词条的方法 */(word)
            .or_default()
            ./* 把 ids 中所有元素追加到列表末尾的方法 */(ids);
    }

    /* 成功时返回什么 */
}
```

注意变量名：外层的 `shared` 是 `Arc<Mutex<Index>>` 的引用，`shared_index` 是守卫。守卫借用共享索引，函数返回前一直有效。

---

# 主练习：`finalize`

## 一级：思考方向

最终结果不能依赖 worker 完成顺序，所以每个编号列表都要排序并去掉重复。排序是纯计算，不需要保护共享数据；因此更合理的顺序是“短暂拿锁复制一份快照，释放锁，再在快照上排序去重”。中毒时仍然返回错误，不要返回半成品。

## 二级：局部分解

1. 获取共享索引的锁，把中毒转成 `IndexError::IndexPoisoned`。
2. 在锁内把整个 `Index` 复制出来（`HashMap` 的 `clone` 会复制键和值列表）。
3. 让守卫在语句或花括号结束时释放，之后的排序不再持锁。
4. 遍历快照中每个值（编号列表）：
   - 按从小到大排序；整数排序有稳定/不稳定版本，题目不要求保持相同编号的相对顺序，因为最终会去重。
   - 去掉相邻重复项。
5. 返回快照。

想一想：如果直接在共享索引上排序，虽然结果也对，但会让锁的持有时间包含整个排序过程。为什么这不划算？

## 三级：接近完整的结构

```rust
pub fn finalize(shared: &SharedIndex) -> Result<Index, IndexError> {
    let mut index = shared
        .lock()
        .map_err(|_| /* 中毒变体 */)?
        ./* 复制内部 Index 的方法 */();

    for ids in index./* 取得所有值可变引用的迭代方法 */() {
        ids./* 排序方法 */();
        ids./* 去除连续重复项的方法 */();
    }

    Ok(index)
}
```

`?` 之后的表达式仍然借用守卫；守卫会在整条 `let` 语句结束时销毁，所以复制出来的 `index` 已经和共享索引无关。不要在这里使用 `Arc::try_unwrap`，因为函数参数只是 `&SharedIndex`，并不拥有 `Arc`。

---

# 主练习：`join_workers`

## 一级：思考方向

这个函数是错误处理的核心。它要同时处理两种完全不同的失败：线程自己 panic（外层 `join` 返回 `Err`），以及线程正常结束但带回业务错误（内层 `Result` 是 `Err`）。前者优先级最高。还有第三种失败：所有线程都正常，但处理计数不对。无论发生哪种，都必须先把所有句柄 join 完，不能提前 `return`。

## 二级：局部分解

1. 准备两个“记录位置”的变量：一个保存最先 panic 的线程编号，一个保存第一个业务错误。
2. 用 `enumerate()` 遍历句柄，这样同时得到编号和句柄。
3. 对每个句柄调用 `join()`：
   - 返回 `Ok(Ok(()))`：这个线程成功，什么也不做；
   - 返回 `Ok(Err(error))`：记录第一个业务错误，后面的不要覆盖；
   - 返回 `Err(_)`：线程 panic；记录第一个编号，后面的不要覆盖。
4. 循环结束后，先检查 panic 记录：有就返回 `WorkerPanicked { worker_id }`。
5. 再检查业务错误：有就原样返回。
6. 读取 `processed` 的当前值，与 `expected` 比较；不相等返回 `ProcessedCountMismatch { expected, actual }`。
7. 都通过后调用 `finalize(shared)`。

为什么不能在第 3 步一发现错误就 `return`？因为后面的线程还没 join；如果它们还在运行，`shared` 的 `Arc` 克隆还在，程序可能提前离开，且计数检查也不完整。

## 三级：接近完整的结构

```rust
fn join_workers(
    handles: Vec<thread::JoinHandle<Result<(), IndexError>>>,
    expected: usize,
    processed: &AtomicUsize,
    shared: &SharedIndex,
) -> Result<Index, IndexError> {
    let mut first_panic = None;
    let mut first_business_error = None;

    for (worker_id, handle) in handles.into_iter().enumerate() {
        match handle.join() {
            Ok(Ok(())) => {}
            Ok(Err(error)) => {
                if first_business_error.is_none() {
                    first_business_error = Some(error);
                }
            }
            Err(_) => {
                if first_panic.is_none() {
                    first_panic = Some(/* 当前线程编号 */);
                }
            }
        }
    }

    if let Some(worker_id) = first_panic {
        return Err(IndexError::WorkerPanicked { worker_id });
    }
    if let Some(error) = first_business_error {
        return Err(error);
    }

    let actual = processed./* 读取原子值的方法 */(/* 与写入时相同的 Ordering */);
    if actual != expected {
        return Err(IndexError::ProcessedCountMismatch { expected, actual });
    }

    /* 最后交给哪个函数整理 */
}
```

`first_panic` 和 `first_business_error` 的类型可以从后面的 `Some(...)` 推出来；如果编译器要求标注，再补 `Option<usize>` / `Option<IndexError>`。

---

# 主练习：`build_index`

## 一级：思考方向

先把“不能开始”的情况挡掉：0 个 worker 直接报错，连空输入也一样。然后保存输入总数（后面 `join_workers` 要用），把文档按位置轮流放进固定数量的桶。每个桶移动给一个线程；线程内部只做“本地索引 → 加锁合并”。最后把全部句柄交给 `join_workers`，不要自己再写一遍 join 逻辑。

## 二级：局部分解

1. `if worker_count == 0 { return Err(IndexError::ZeroWorkers); }`。
2. 保存 `let expected = documents.len();`，必须在 `documents` 被移动之前。
3. 创建共享索引 `Arc::new(Mutex::new(Index::new()))` 和共享计数 `Arc::new(AtomicUsize::new(0))`。
4. 创建 `worker_count` 个空桶，类型是 `Vec<Vec<Document>>`。
5. 用 `enumerate()` 遍历 `documents.into_iter()`，得到位置和文档；把文档 `push` 到 `position % worker_count` 号桶。这里没有复制 `text`，只是移动 `Document`。
6. 创建一个保存 `JoinHandle` 的 `Vec`，容量可以是 `worker_count`。
7. 遍历桶，对每个桶：
   - 克隆共享索引和共享计数；
   - `thread::spawn(move || ...)`，闭包里先 `index_documents(bucket, &worker_processed)`，再 `merge_into(&worker_shared, local)`；
   - 闭包的返回值就是 `Result<(), IndexError>`；
   - 把句柄 push 进向量。
8. 调用 `join_workers(handles, expected, &processed, &shared)` 并返回它的结果。

注意：`index_documents` 接收的是拥有型 `Vec<Document>`，所以桶要按值移动进闭包；不要在闭包里再克隆文档。

## 三级：接近完整的结构

```rust
pub fn build_index(documents: Vec<Document>, worker_count: usize) -> Result<Index, IndexError> {
    if worker_count == 0 {
        return Err(/* 哪个错误变体 */);
    }

    let expected = documents./* 长度 */;
    let shared: SharedIndex = Arc::new(Mutex::new(/* 空 Index */));
    let processed = Arc::new(AtomicUsize::new(0));

    let mut buckets: Vec<Vec<Document>> = (0..worker_count)
        .map(|_| Vec::new())
        .collect();

    for (position, document) in documents.into_iter().enumerate() {
        buckets[position % worker_count].push(document);
    }

    let mut handles = Vec::with_capacity(worker_count);
    for bucket in buckets {
        let worker_shared = /* 克隆共享索引 */;
        let worker_processed = /* 克隆共享计数 */;

        handles.push(thread::spawn(move || {
            let local = index_documents(bucket, /* 计数引用 */);
            merge_into(/* 共享索引引用 */, local)
        }));
    }

    join_workers(handles, expected, /* 计数引用 */, /* 共享索引引用 */)
}
```

闭包里 `worker_shared` 的类型是 `Arc<Mutex<Index>>`，`merge_into` 需要 `&SharedIndex`，所以传 `&worker_shared`。`worker_processed` 同理。不要写成 `Arc::clone(&shared)` 之后又把外层 `shared` 移走；外层还要交给 `join_workers`。

---

# 课后巩固：`summarize_batch`

## 一级：思考方向

这是纯本地计算，不涉及线程或锁。每个传感器只保留一条统计；同一个传感器再次出现时，更新已有的统计，而不是插入第二条。最低温、最高温和总和都要在更新时维护。

## 二级：局部分解

1. 创建一个 `BTreeMap<String, SensorSummary>`。类型注解可能必要，因为编译器一开始不知道值的类型。
2. 遍历读数；把 `Reading { sensor, celsius }` 拆开，避免后面移动 `sensor` 时和借用冲突。
3. 用 `get_mut(&sensor)` 查找：
   - 找到：`count += 1`；`min` 取当前值和 `celsius` 的较小者；`max` 取较大者；`sum += i64::from(celsius)`。
   - 没找到：插入一条新的 `SensorSummary { count: 1, min: celsius, max: celsius, sum: i64::from(celsius) }`。
4. 返回 map。

空输入自然返回空 map，不需要额外特判。

## 三级：接近完整的结构

```rust
pub fn summarize_batch(readings: Vec<Reading>) -> BTreeMap<String, SensorSummary> {
    let mut summaries: BTreeMap<String, SensorSummary> = /* 新建空 map */;

    for reading in readings {
        let Reading { sensor, celsius } = reading;

        if let Some(summary) = summaries./* 取得可变引用的方法 */(&sensor) {
            summary.count += 1;
            summary.min = summary.min./* 取较小值的方法 */(celsius);
            summary.max = summary.max./* 取较大值的方法 */(celsius);
            summary.sum += /* 把 i32 转成 i64 */;
        } else {
            summaries.insert(
                sensor,
                SensorSummary {
                    count: 1,
                    min: /* 初始最低温 */,
                    max: /* 初始最高温 */,
                    sum: /* 初始总和 */,
                },
            );
        }
    }

    summaries
}
```

为什么 `sum` 不直接用 `i32`？想一想很多条 20 亿度（玩具数据）相加会怎样。

---

# 课后巩固：`merge_summaries`

## 一级：思考方向

和主练习的 `merge_into` 结构相似，但这里是合并统计值，不是追加列表。需要用写锁，因为可能修改表；锁中毒时返回 `SensorError::Poisoned`。已存在的传感器要合并四个字段，新传感器直接插入。

## 二级：局部分解

1. 调用 `shared.write()` 获取写守卫；`map_err` 把中毒转成 `SensorError::Poisoned`。
2. 遍历局部 map 的 `(sensor, incoming)`。
3. 用 `get_mut(&sensor)`：
   - 已存在：`count` 相加；`sum` 相加；`min` 取两者较小；`max` 取两者较大。
   - 不存在：把 `incoming` 插入。
4. 返回 `Ok(())`。

注意：不要先 `contains_key` 再 `get_mut` 再 `insert`，那样会多次查找，也容易在“检查和修改”之间写出多余逻辑。`get_mut` 一次就能判断并修改。

## 三级：接近完整的结构

```rust
pub fn merge_summaries(
    shared: &SharedSummary,
    local: BTreeMap<String, SensorSummary>,
) -> Result<(), SensorError> {
    let mut summaries = shared
        ./* 获取写锁的方法 */()
        .map_err(|_| /* 中毒变体 */)?;

    for (sensor, incoming) in local {
        if let Some(existing) = summaries./* 取得可变引用的方法 */(&sensor) {
            existing.count += incoming.count;
            existing.min = existing.min./* 较小值 */(incoming.min);
            existing.max = existing.max./* 较大值 */(incoming.max);
            existing.sum += incoming.sum;
        } else {
            summaries./* 插入 */(sensor, incoming);
        }
    }

    Ok(())
}
```

`RwLock` 的 `write()` 返回 `Result`，和 `Mutex::lock()` 一样需要处理中毒；不要直接 `unwrap()`，否则测试里的中毒场景会让测试线程 panic 而不是返回 `Poisoned`。

---

# 课后巩固：`join_summaries`

## 一级：思考方向

和主练习的 `join_workers` 几乎同构，但少了 `expected` 计数检查，最后读取的是统计表而不是索引。仍然要：先 join 所有线程；panic 优先；业务错误其次；全部成功后再读锁取快照。读锁也可能中毒，要返回 `Poisoned`。

## 二级：局部分解

1. 准备 `first_panic` 和 `first_business_error`。
2. `enumerate()` 遍历句柄，`join()`：
   - `Ok(Ok(()))`：成功；
   - `Ok(Err(error))`：记录第一个业务错误；
   - `Err(_)`：记录第一个 panic 编号。
3. 循环结束后先返回 `WorkerPanicked`，再返回业务错误。
4. 调用 `shared.read()`，中毒转成 `Poisoned`。
5. 把 `BTreeMap` 的每个 `(key, value)` 克隆成 `(String, SensorSummary)`，收集成 `Vec`。`BTreeMap` 的迭代顺序已经是键升序，不需要再排序。
6. 返回 `Ok(vec)`。

## 三级：接近完整的结构

```rust
fn join_summaries(
    handles: Vec<thread::JoinHandle<Result<(), SensorError>>>,
    shared: &SharedSummary,
) -> Result<Vec<(String, SensorSummary)>, SensorError> {
    let mut first_panic = None;
    let mut first_business_error = None;

    for (worker_id, handle) in handles.into_iter().enumerate() {
        match handle.join() {
            Ok(Ok(())) => {}
            Ok(Err(error)) => {
                if first_business_error.is_none() {
                    first_business_error = Some(error);
                }
            }
            Err(_) => {
                if first_panic.is_none() {
                    first_panic = Some(worker_id);
                }
            }
        }
    }

    if let Some(worker_id) = first_panic {
        return Err(SensorError::WorkerPanicked { worker_id });
    }
    if let Some(error) = first_business_error {
        return Err(error);
    }

    let summaries = shared
        ./* 获取读锁的方法 */()
        .map_err(|_| /* 中毒变体 */)?;

    Ok(summaries
        .iter()
        .map(|(sensor, summary)| (/* 克隆键 */, /* 克隆值 */))
        .collect())
}
```

`read()` 返回的守卫只实现 `Deref`，不能修改；这正是“读者可以同时读”的体现。不要在守卫还活着时尝试 `write()`，同一线程可能死锁。

---

# 课后巩固：`summarize_readings`

## 一级：思考方向

先挡住 0 个 worker。保存不需要了，因为没有计数检查；但需要把读数按位置轮流分桶。每个线程先 `summarize_batch`，再 `merge_summaries`；最后交给 `join_summaries`。和主练习一样，局部统计必须在写锁之外。

## 二级：局部分解

1. `if worker_count == 0 { return Err(SensorError::ZeroWorkers); }`。
2. 创建 `Arc::new(RwLock::new(BTreeMap::new()))` 作为共享统计表。
3. 创建 `worker_count` 个空桶 `Vec<Vec<Reading>>`。
4. 用 `enumerate()` 遍历 `readings.into_iter()`，按 `position % worker_count` push 到桶里。`Reading` 整体移动，不克隆 `sensor`。
5. 遍历桶，为每个桶克隆共享表；`thread::spawn(move || { let local = summarize_batch(bucket); merge_summaries(&worker_shared, local) })`；保存句柄。
6. 调用 `join_summaries(handles, &shared)` 并返回。

空输入 + 正数 worker 时，桶都是空的；每个线程的 `summarize_batch` 返回空 map，`merge_summaries` 什么也不插入；最后返回空 `Vec`。不要为“空输入”写提前返回，否则就绕过了“仍然创建线程”的要求。

## 三级：接近完整的结构

```rust
pub fn summarize_readings(
    readings: Vec<Reading>,
    worker_count: usize,
) -> Result<Vec<(String, SensorSummary)>, SensorError> {
    if worker_count == 0 {
        return Err(/* 哪个错误变体 */);
    }

    let shared: SharedSummary = Arc::new(RwLock::new(/* 空 BTreeMap */));
    let mut buckets: Vec<Vec<Reading>> = (0..worker_count)
        .map(|_| Vec::new())
        .collect();

    for (position, reading) in readings.into_iter().enumerate() {
        buckets[position % worker_count].push(reading);
    }

    let mut handles = Vec::with_capacity(worker_count);
    for bucket in buckets {
        let worker_shared = /* 克隆共享表 */;
        handles.push(thread::spawn(move || {
            let local = summarize_batch(bucket);
            merge_summaries(/* 共享表引用 */, local)
        }));
    }

    join_summaries(handles, /* 共享表引用 */)
}
```

闭包里 `worker_shared` 的类型是 `Arc<RwLock<BTreeMap<...>>>`，`merge_summaries` 需要 `&SharedSummary`，传 `&worker_shared`。外层 `shared` 还要给 `join_summaries`，不要被 `move` 拿走。

---

# 卡住时的通用排错

1. 先运行单个测试，例如 `cargo test -p l17_shared_concurrency index_documents`，把问题缩小到一个函数。
2. 读编译器的第一条错误，不要从最后一条开始。`Send`/`Sync` 错误从最内层类型看。
3. 如果测试卡住不返回，检查是否在同一线程里重复 `lock()`/`read()`/`write()`，或者守卫活到了下一个锁调用之后。
4. 如果测试报 `PoisonError`，检查是不是前一个测试故意让子线程 panic；在“中毒测试”里这是预期路径，要返回错误而不是 `unwrap()`。
5. 如果结果偶尔不对，先检查是否有 worker 的结果没有合并、计数是否在文档处理完成前就加了、最终是否排序去重；不要用 `sleep` 掩盖顺序问题。
6. 如果 Clippy 报未使用变量或导入，按实际需要调整 `use`；不要用全局 `#![allow(...)]` 关闭警告。
