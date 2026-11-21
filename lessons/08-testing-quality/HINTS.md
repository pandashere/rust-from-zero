# 第八课提示：主练习与课后巩固

提示分三级，卡到哪一级看哪一级：

- **一级：思考方向**——只提示该想什么、该读哪几条测试，不涉及代码结构。
- **二级：局部分解**——把任务拆成几个判断步骤，说明每一步用到哪个标准库能力，仍不写完整代码。
- **三级：接近完整的结构**——给出带空缺的骨架，你需要自己补上关键调用。

先看一级；想清楚了就不要继续往下看。三级提示故意不给出可直接提交的完整代码。

本课和前面的课不同：**测试已经全部写好**。所以最有用的第一步不是写代码，而是读测试——把测试当成一份用代码写成的需求说明。下面的提示会反复把你指回具体测试名。

主练习按 README 推荐的顺序完成：先 `add` 的前三步（去空白、空标题、零时长），再 `add` 的去重与容量，再 `add` 的追加，然后 `total_seconds`、`find`、`longest`。

## 开工前：怎么读这些测试

`src/playlist/queue.rs` 底部的 `mod tests` 里有一个辅助函数 `sample_playlist()`，它返回一个装了 `"晴天"`（269 秒）和 `"七里香"`（299 秒）的歌单，容量 4。很多测试用它当起点。

四个方法的契约不只在测试里，也在**方法上方的文档注释**里。本课的注释按函数自身的数据说明：返回哪个字段、去空白发生在哪一步、错误顺序是什么、并列时保留哪一首。注释不指定实现语法，但它规定了行为。

对照着读的顺序建议：

1. `add_rejects_titles_that_are_empty_after_trimming`
2. `add_rejects_zero_seconds`
3. `add_rejects_a_duplicate_title_after_trimming`
4. `add_reports_full_with_the_capacity_when_no_room_is_left`
5. `empty_title_is_reported_before_zero_length_and_full`、`duplicate_title_is_reported_before_a_full_playlist`、`zero_length_is_reported_before_duplicate_and_full`
6. `add_outcome_table_covers_normal_and_error_cases`

前四条各管一种失败，接下来三条专管**顺序**，最后一条用一张表把它们串起来。读完这八条，`add` 的行为就没有歧义了。

## 主练习：`Playlist::add` 的去空白与空标题

### 一级：思考方向

测试 `add_rejects_titles_that_are_empty_after_trimming` 的第二个 case 是 `"   \t\n "`，第三个是 `"　"`（全角空格）。第一个 case 是空字符串。

想一想：如果先把标题“原样”与空字符串比较，这三个 case 会怎样？为什么它们必须都算失败？标准库的字符串有没有一个方法能一次去掉所有种类的空白？

再看 `add_trims_whitespace_on_both_sides_of_the_title`：输入是 `"\t 稻香 \n"`，期望保存的是 `"稻香"`。这说明去掉空白只发生一次，而且保存的是去掉之后的结果，不是原始输入。

### 二级：局部分解

1. 先对参数做一次变换，把结果存进一个新变量（类型仍是 `&str`，不复制文本）。
2. 判断这个变量是不是空的。空就提前返回对应的错误变体。
3. 后面所有比较（查重、构造）都用这个变量，不要再用原始参数。否则会出现“查出重复了，但存进去的标题带空白”的不一致。

`str` 上有一个方法专门做“去掉两端空白”，名字就是英文的“修剪”。它认得空格、制表符、换行，也认得全角空格这类 Unicode 空白字符。

### 三级：接近完整的结构

```rust
pub fn add(&mut self, title: &str, seconds: u32) -> Result<(), PlaylistError> {
    let title = title./* 去掉两端空白的方法 */();
    if title./* 判断空字符串的方法 */() {
        return Err(PlaylistError::EmptyTitle);
    }
    // ... 后面三步
}
```

注意变量**遮蔽**（shadowing）：`let title = title.trim();` 让后面的 `title` 都指向去空白后的文本。这是 Rust 允许的写法，也是这里最不容易出错的做法。

如果你写 `let trimmed = ...` 也可以，但记得后面每一处都用 `trimmed`。表驱动测试里的 `"晴天 "` case 会抓住“有一处忘了用去空白版本”的错误。

## 主练习：`Playlist::add` 的零时长与重复标题

### 一级：思考方向

- `add_rejects_zero_seconds` 只有一行断言，输入是 `"静音"` 配 `0`。
- `add_rejects_a_duplicate_title_after_trimming` 先加 `"晴天"`，再加 `"  晴天  "`。期望的错误里带的文本是什么？是 `"  晴天  "` 还是 `"晴天"`？为什么？
- `add_treats_title_comparison_as_case_sensitive` 加 `"Yesterday"` 再加 `"yesterday"`，两次都成功。这说明比较时**不能**做什么？

### 二级：局部分解

1. 零时长：`seconds` 是 `u32`。判断它等于 0 就返回 `ZeroLength`。这一行要放在去重和容量检查**之前**——看 `zero_length_is_reported_before_duplicate_and_full`：容量 1 且已满，输入重复标题 + 0 秒，期望得到 `ZeroLength`。
2. 去重：遍历已有曲目，比较每首的标题。注意用 `Track` 的公开方法取标题，而不是访问字段（字段是私有的，而且你在 `queue` 模块里其实也访问不到——它在 `model` 模块）。
3. 找到重复时，返回的 `DuplicateTitle` 里装的必须是**已经去空白**的标题，用 `to_owned()` 复制成 `String`。
4. 比较用 `==`（区分大小写）。不要用“忽略大小写”的方法，那会让 `add_treats_title_comparison_as_case_sensitive` 失败。

### 三级：接近完整的结构

```rust
    if seconds == 0 {
        return Err(PlaylistError::/* 哪个变体 */);
    }

    if self.tracks./* 遍历并判断是否存在同名曲目的方法 */(|track| track.title() == title) {
        return Err(PlaylistError::DuplicateTitle(title./* 变成自己拥有的 String 的方法 */()));
    }
```

`|track| ...` 是一个**闭包**（closures，可以像值一样传递的一小段代码），第九课会细讲。现在只需理解成“对每个元素执行这段判断，返回 `bool`”。`iter().any(...)` 的语义是“存在任意一个满足条件的元素吗”。

为什么 `DuplicateTitle` 里要 `to_owned()`？因为参数 `title` 是借来的文本，函数返回时原始输入可能已经不在了；错误要带回自己拥有的副本。

## 主练习：`Playlist::add` 的容量与追加

### 一级：思考方向

`add_reports_full_with_the_capacity_when_no_room_is_left` 期望 `Full { capacity: 2 }` —— 带上容量，而不是只报“满了”。这个值从哪个字段来？

`a_zero_capacity_playlist_rejects_every_track`：容量 0，第一次添加就失败。用 `==` 比较“当前数量等于容量”还是用 `>=`，在这个 case 下结果一样；但在容量 0 时想清楚为什么不能漏判。

`playlist.add("晴天", 269).unwrap()` 之后，测试会检查 `playlist.tracks()[0].title()`。曲目是哪个类型、怎么构造？

顺序再看一遍 `duplicate_title_is_reported_before_a_full_playlist`：容量 1 已满，输入 `" 晴天 "`。期望是 `DuplicateTitle`，不是 `Full`。所以容量检查必须是**最后**一步。

### 二级：局部分解

1. 判断当前数量是否已经达到或超过容量。是就返回 `Full { capacity: ??? }`，字段值是歌单自己的容量。
2. 全部检查通过后，用 `Track::new(去掉空白的标题的 String, seconds)` 造曲目。这个构造器在 `src/playlist/model.rs`，可见性是 `pub(super)`，`queue` 和 `model` 是兄弟模块，共同父模块是 `playlist`，所以你能调用它。
3. 把曲目追加到 `self.tracks`。
4. 返回 `Ok(())`。注意函数返回类型是 `Result<(), PlaylistError>`，成功值是 `()`，不是曲目本身。

`self.tracks` 是 `Vec<Track>`，容量检查用 `len()`，追加用 `push()`（第五课）。

### 三级：接近完整的结构

```rust
    if self.tracks.len() >= self./* 容量字段 */ {
        return Err(PlaylistError::Full {
            capacity: self./* 同一个字段 */,
        });
    }

    self.tracks./* 追加元素的方法 */(Track::new(title./* 复制成 String 的方法 */(), seconds));
    Ok(())
```

写完后，把开头那句 `let _ = (&self, title, seconds);` 删掉——它只是为了让脚手架在没有实现时也能通过 Clippy 的未使用参数检查。

### 检查点：`add` 全绿之前不要往下走

`cargo test -p l08_testing_quality add` 应该让所有 `add_*` 和优先级测试通过。如果 `add_outcome_table_covers_normal_and_error_cases` 失败，先看失败消息里的 `case "..."`，它会告诉你是哪一行数据。

## 主练习：`total_seconds`

### 一级：思考方向

每首曲目的时长是 `u32`，函数返回 `u64`。测试 `total_seconds_does_not_overflow_when_the_sum_exceeds_u32` 加了两首 `u32::MAX`，期望 `8589934590`。

想一想：如果先用 `u32` 累加再转成 `u64`，这两首相加会怎样？溢出在 debug 构建里会 panic，在 release 构建里会回绕（绕回小数值）——两种都不对。所以转换必须发生在**加法之前**。

`total_seconds_of_an_empty_playlist_is_zero`：空歌单要返回 0，不要提前返回 `None`（返回类型根本不是 `Option`）。

### 二级：局部分解

1. 遍历所有曲目，取每首的 `seconds()`。
2. 把每个 `u32` 转成 `u64`。标准库的 `u64::from(u32值)` 是无损转换（小范围转大范围永远安全），比 `as` 更能表达“这里不会丢数据”。
3. 把转换后的值累加，返回结果。可以用 `for` 循环配一个 `let mut sum = 0u64;`，也可以用迭代器的 `.map(...).sum()`（第十课会讲；现在用 `for` 完全可以）。

### 三级：接近完整的结构

```rust
    let mut total: u64 = 0;
    for track in /* 遍历曲目的写法 */ {
        total += u64::from(track./* 时长 getter */());
    }
    total
```

不要写 `self.tracks.iter().map(|t| t.seconds()).sum::<u64>()` 就以为万事大吉——那不会编译，因为元素是 `u32` 而 `sum` 的累加类型是 `u64`。必须先 `map` 成 `u64`。

## 主练习：`find`

### 一级：思考方向

签名是 `fn find(&self, title: &str) -> Option<&Track>`。返回类型里有 `&`，意味着返回的是**歌单内部那首曲目的借用**，不是副本。

`find_trims_the_argument_but_stays_case_sensitive`：`find("  Yesterday\t")` 要能找到，说明查找参数也要去空白；`find("yesterday")` 要找不到，说明比较区分大小写。

`find_on_an_empty_playlist_returns_none`：空歌单上调用不能 panic。

### 二级：局部分解

1. 对参数去空白（和 `add` 一样）。
2. 在 `self.tracks` 上找第一个标题相等的元素。`iter()` 配合 `find(...)` 正好返回 `Option<&Track>`。
3. 闭包里用 `track.title() == trimmed`。

这里不需要 `to_owned()`：返回的是借用，闭包里的比较也是借用对借用。

### 三级：接近完整的结构

```rust
    let title = title./* 去空白 */();
    self.tracks
        ./* 返回迭代器的方法 */()
        ./* 找第一个满足条件的元素的方法 */(|track| track.title() == title)
```

如果编译报 “returns a reference to a temporary” 或 “missing lifetime specifier”，先检查你是不是写成了 `self.tracks.iter().filter(...)`——`filter` 返回的是迭代器，不是 `Option<&Track>`。

## 主练习：`longest`

### 一级：思考方向

`longest_of_an_empty_playlist_is_none` 要求空歌单返回 `None`，不能 panic、不能返回 0。

`longest_keeps_the_earliest_track_when_lengths_are_tied`：两首都是 300 秒，期望返回**先添加**的 `"先"`。这意味着：只有当新曲目**严格更长**时才替换当前最优；相等时保持原样。

`longest_returns_the_track_with_the_most_seconds`：`[100, 400, 200]` 要返回 400 那首，说明不能只看第一首。

### 二级：局部分解

1. 用一个“当前最优”的变量，初始值表示“还没有”。
2. 遍历所有曲目。
3. 对每个曲目判断：当前还没有最优，或者这首比当前最优更长 → 更新最优。
4. 遍历结束返回这个变量。

两种实现思路都可以：

- 手动 `for` 循环 + `Option<&Track>` 变量，判断条件写成 `best.is_none_or(|b| track.seconds() > b.seconds())`；
- 用 `iter().fold(None, ...)`（第十课内容，想用就用，不想用别勉强）。

### 三级：接近完整的结构

```rust
    let mut best: Option<&Track> = None;
    for track in /* 遍历 */ {
        if best./* 判断“还没有值”或“新值更长” */(|current| track.seconds() > current.seconds()) {
            best = Some(track);
        }
    }
    best
```

`is_none_or` 的语义是“是 `None`，或者里面的值满足条件”。如果它让你困惑，等价写法是：

```rust
        let better = match best {
            None => true,
            Some(current) => track.seconds() > current.seconds(),
        };
```

用 `>` 而不是 `>=`，是“并列保留先出现者”的关键。

## 主练习完成后：一次自检

1. 运行 `cargo test -p l08_testing_quality`，36 个单元测试 + 3 个集成测试 + 5 个文档测试全绿。
2. 运行 `cargo clippy -p l08_testing_quality --all-targets -- -D warnings`。
3. 做一次变异实验：把 `add` 里的 trim 去掉，确认有测试失败；然后**立刻改回来**。
4. 运行 `cargo run -p l08_testing_quality`，对照输出理解每一步。

## 课后巩固：`mean`（`src/statistics.rs`）

### 一级：思考方向

返回类型是 `Option<f64>`：空输入返回 `None`，非空返回 `Some(平均值)`。

`mean_of_an_empty_slice_is_none` 是最容易漏的 case：如果不判断空，`sum / 0.0` 在浮点里得到的是 `NaN`（或 `inf`），而不是 `None`——测试会以“期望 None，实际 Some(NaN)”失败。

`mean_mixes_negative_and_positive_values` 输入 `[-4.0, 1.0, 3.0]`，期望 `0.0`。负数和正常相加，不需要特殊处理。

`mean_needs_a_tolerance_for_repeating_decimals` 提醒你：结果要和期望比，就不能用 `assert_eq!`——测试里已经用了容差辅助函数，你只要保证算出来的值是对的。

### 二级：局部分解

1. 先判断切片长度是否为 0，是就返回 `None`。
2. 累加所有元素，得到 `f64` 的总和。
3. 除以元素个数。元素个数是 `usize`，要转成 `f64` 才能除（`usize` 和 `f64` 不能直接运算）。
4. `Some(结果)`。

`values.len() as f64` 里的 `as` 是显式类型转换。这里安全，因为长度不会超过 `f64` 能精确表示的范围（实际数量级远小于此）。

### 三级：接近完整的结构

```rust
    if values./* 判断空 */() {
        return None;
    }
    let sum: f64 = /* 累加所有元素 */;
    Some(sum / values.len() /* 转成 f64 */)
```

## 课后巩固：`median`

### 一级：思考方向

中位数要先排序，但参数是 `&[f64]`（只读借用）。`median_leaves_the_callers_data_untouched` 会在调用后检查原数组还是 `[3.0, 1.0, 2.0]`。

所以第一步必须是**复制**出一份自己拥有的 `Vec<f64>`，在副本上排序。

第二个坑：`Vec<f64>` 的 `sort()` 不能编译。README 第 13.4 节给了完整报错——`f64` 没有 `Ord`（全序），因为 `NaN` 无法比较。你需要 `sort_by` 配一个比较函数，或者 `sort_by(|a, b| a.partial_cmp(b).unwrap())`（数据约定里没有 `NaN`）。

第三个坑：奇数个和偶数个的取法不同。`[1, 2, 3, 4]` 的中位数是 `(2 + 3) / 2`，不是 `2`。

### 二级：局部分解

1. 空输入返回 `None`。
2. `values.to_vec()` 复制成 `Vec<f64>`。
3. 用 `sort_by` 排序，比较函数用 `partial_cmp`。`partial_cmp` 返回 `Option<Ordering>`，`NaN` 参与时是 `None`；本课数据没有 `NaN`，可以 `expect`。
4. 求中间位置：`len / 2`（整数除法，向下取整）。
5. 如果 `len % 2 == 1`，返回 `sorted[middle]`。
6. 否则返回 `(sorted[middle - 1] + sorted[middle]) / 2.0`。

### 三级：接近完整的结构

```rust
    if values.is_empty() {
        return None;
    }

    let mut sorted = values./* 复制成 Vec 的方法 */();
    sorted./* 带比较函数的排序方法 */(|left, right| {
        left./* 部分比较 */(right).expect("no NaN")
    });

    let middle = sorted.len() / 2;
    if sorted.len() % 2 == 1 {
        Some(sorted[middle])
    } else {
        Some((sorted[middle - 1] + sorted[middle]) / /* 两个数的平均 */)
    }
```

`middle - 1` 在偶数长度时不会下溢：长度至少为 2（长度 0 已提前返回，长度 1 走奇数分支），所以 `middle >= 1`。

### 检查点

```bash
cargo test -p l08_testing_quality statistics
```

13 个统计测试全绿后，再跑一次完整套件和 Clippy。

## 卡住时的通用办法

1. **先跑单条测试**，不要一次跑全部：`cargo test -p l08_testing_quality add_rejects_zero`。
2. **读测试的断言**，把 `left`（实际）与 `right`（期望）对到你的实现上。
3. **加一句临时打印**：`println!` 的输出默认被测试框架吞掉，加 `-- --nocapture` 才能看到。
4. 如果报错是 `not yet implemented`，说明那条路径还没实现，不是测试坏了。
5. 如果报错是 `E0277`、`E0308`、`E0451`、`E0433`，回到 README 第十三节对照。
6. 还是不懂，把“我改了哪一行、期望什么、实际得到什么”写下来再问——这个过程往往自己就找到答案了。
