# 第六课提示：主练习与课后巩固

提示分三级，卡到哪一级看哪一级：

- **一级：思考方向**——只提示该想什么，不涉及代码结构。
- **二级：局部分解**——把方法拆成几个判断步骤，仍不写完整代码。
- **三级：接近完整的结构**——给出带空缺的骨架，你需要自己补上关键调用。

先看一级；想清楚了就不要继续往下看。三级提示故意不给出可直接提交的完整代码。

主练习建议按 README 13.5 节的顺序完成：`new` → `count` / `tags_for` → `add_document`（先不做替换）→ `remove_document` → 补替换逻辑 → `documents_with` → `most_common`。

## 主练习：`new`（`src/exercise.rs`）

### 一级：思考方向

`TagIndex` 有两个字段，都是映射。一个空索引应该让它们各自处于什么状态？想想 `HashMap::new()` 和 `HashSet::new()` 分别创建什么。

### 二级：局部分解

1. 用 `HashMap::new()` 创建空计数表。
2. 用 `HashMap::new()` 创建空文档表。
3. 用结构体字面量把两个字段装进 `Self` 返回。字段名和局部变量同名时，可以写简写。

### 三级：接近完整的结构

```rust
pub fn new() -> Self {
    Self {
        counts: HashMap::/* 创建空映射的方法 */,
        documents: HashMap::/* 创建空映射的方法 */,
    }
}
```

## 主练习：`count` 与 `tags_for`

### 一级：思考方向

两个方法都只读索引。注释说明“标签不存在返回 0”“文档不存在返回 `None`”，所以都不能用会 panic 的索引语法。标准库里哪个查询方法返回 `Option`？`count` 的返回类型是 `u32`，而 `get` 给的是引用，差在哪一步？`tags_for` 还要注意：保存文档名时去过两端空白，查询时也要先去，才能命中。

### 二级：局部分解

1. `count`：用 `self.counts.get(tag)` 查询，得到 `Option<&u32>`。`u32` 是 `Copy`，可以用 `copied()` 把 `Option<&u32>` 变成 `Option<u32>`，再用 `unwrap_or(0)` 在 `None` 时给出 0。
2. `tags_for`：先把 `name` 去掉两端空白，再调用 `self.documents.get(...)`；它的类型正好是 `Option<&HashSet<String>>`，与返回类型一致，不需要转换。
3. 两个方法都不修改 `self`，所以参数是 `&self`。

### 三级：接近完整的结构

```rust
pub fn count(&self, tag: &str) -> u32 {
    self.counts
        ./* 查询方法 */(tag)
        ./* 把 Option<&u32> 变成 Option<u32> */()
        ./* 没有值时给 0 */()
}

pub fn tags_for(&self, name: &str) -> Option<&HashSet<String>> {
    self.documents./* 查询方法 */(/* name 去掉两端空白 */)
}
```

## 主练习：`add_document`

这是本练习最难的方法，建议分两遍写：第一遍只处理“新文档”，第二遍再补“替换已有文档”。

### 一级：思考方向

先想清楚“失败时索引不能变”意味着什么：所有可能失败的检查必须发生在修改 `self.counts` 或 `self.documents` **之前**。标签规范化会生成新的字符串，这一步完全可以先在一个临时集合里完成。再想：替换旧文档时，旧标签集合需要被遍历，同时又要修改计数表——如果只是从 `documents` 里借出旧集合，还能同时可变借用 `counts` 吗？两个字段是分开的，但方法里的借用关系要想清楚；把旧集合从 `documents` 里“拿走”往往更简单。

### 二级：局部分解

1. 先把 `name` 去掉两端空白：`let name = name.trim();`。
2. 如果 `name.is_empty()`，立刻返回 `Err(IndexError::EmptyDocumentName)`。此时还没有碰过任何字段，所以状态天然不变。
3. 构造规范化标签的临时集合：
   - 遍历 `tags` 里的每个 `&str`；
   - 对每个标签 `trim()`；
   - 把 ASCII 大写转成小写（标准库有专门只处理 A–Z 的方法；注意不要用会处理整个 Unicode 的 `to_lowercase`）；
   - 跳过空字符串；
   - 收集进 `HashSet<String>`。集合自动完成“同一篇文档内去重”。
4. 取出旧标签集合：用 `self.documents.remove(name)` 得到 `Option<HashSet<String>>`。这里用 `remove` 而不是 `get`，因为旧集合马上要被新集合替换，而且拿走所有权后可以一边遍历旧集合、一边修改 `counts`，不会产生两个字段之间的借用冲突。
5. 如果取到了旧集合，对旧集合里的每个标签：
   - 在 `self.counts` 里把计数减 1；
   - 减到 0 时把这个标签从 `counts` 删除（不能留下 0）。
6. 对新集合里的每个标签：在 `self.counts` 里把计数加 1；不存在时从 0 开始。
7. 把 `name.to_owned()` 和新集合插入 `self.documents`。
8. 返回 `Ok(())`。

两个容易漏的细节：

- 第 5、6 步的顺序必须是“先撤销旧、再加新”。如果新旧标签有重叠，先加后撤在最终数值上可能碰巧相同，但“减到 0 就删除”的规则会让中间状态出现临时错误，而且更难推理。按规格规定的顺序做。
- 第 7 步要在计数更新之后。这样即使中途逻辑需要查看 `documents`，也不会看到“新集合已存在但计数还没加”的半状态。整个方法没有提前返回的错误分支，所以只要第 2 步的检查在最前面，失败原子性就成立。

### 三级：接近完整的结构

```rust
pub fn add_document(&mut self, name: String, tags: &[&str]) -> Result<(), IndexError> {
    let name = name.trim();
    if name.is_empty() {
        return Err(IndexError::EmptyDocumentName);
    }

    let new_tags: HashSet<String> = /* 遍历 tags，逐个 trim、ASCII 小写化、过滤空串，收集成集合 */;

    if let Some(old_tags) = self.documents./* 拿走旧集合的方法 */(name) {
        for tag in &old_tags {
            if let Some(count) = self.counts./* 取得可变计数的方法 */(tag) {
                *count -= 1;
                if *count == 0 {
                    /* 从 counts 删除这个标签；注意这里还能不能用 count */
                }
            }
        }
    }

    for tag in &new_tags {
        *self.counts
            ./* 取得或插入计数槽位 */(/* key 需要 String，而 tag 是 &String */)
            ./or_insert(0) += 1;
    }

    self.documents
        ./* 插入方法 */(name.to_owned(), new_tags);

    Ok(())
}
```

第 5 步删除零计数条目时，如果编译器抱怨借用冲突，先把“是否归零”算成一个 `bool`，再在 `count` 最后一次使用之后删除。

## 主练习：`remove_document`

### 一级：思考方向

先判断文档在不在，再决定返回 `true` 还是 `false`。如果存在，它的标签集合需要被逐个用来撤销计数；撤销方式和 `add_document` 替换旧文档时完全一样。想想怎样同时拿到“文档存在”和“它的标签集合”。

### 二级：局部分解

1. 先把 `name` 去掉两端空白，再用 `self.documents.remove(...)` 尝试拿走文档的标签集合，得到 `Option<HashSet<String>>`。
2. `None`：文档不存在，返回 `false`，不要修改 `counts`。
3. `Some(old_tags)`：遍历旧标签，把每个计数减 1；减到 0 就删除条目。全部撤销后返回 `true`。
4. 文档已经被 `remove` 拿走，不需要再单独删除一次。

### 三级：接近完整的结构

```rust
pub fn remove_document(&mut self, name: &str) -> bool {
    match self.documents./* 拿走标签集合的方法 */(/* name 去掉两端空白 */) {
        None => false,
        Some(old_tags) => {
            for tag in &old_tags {
                /* 与 add_document 撤销旧标签相同的逻辑 */
            }
            true
        }
    }
}
```

## 主练习：`documents_with`

### 一级：思考方向

你要遍历“文档 → 标签集合”这张表，挑出标签集合包含目标标签的文档。注意返回类型是 `Vec<&str>`：每个名字都借用索引内部的 `String`，不能复制。`HashMap` 的遍历顺序不稳定，而规格要求按字典序，所以收集完必须排序。

### 二级：局部分解

1. 创建一个空的 `Vec<&str>`。
2. 遍历 `&self.documents`，每项是 `(&String, &HashSet<String>)`。
3. 用 `HashSet::contains` 判断这篇文档的标签集合里有没有 `tag`。
4. 有就把文档名转成 `&str`（`name.as_str()`）放进 Vec。
5. 用 `sort_unstable()` 排序后返回。`&str` 的默认排序就是字典序。

### 三级：接近完整的结构

```rust
pub fn documents_with(&self, tag: &str) -> Vec<&str> {
    let mut names: Vec<&str> = Vec::new();

    for (name, tags) in &self.documents {
        if tags./* 成员判断方法 */(tag) {
            names.push(/* 把 &String 变成 &str */);
        }
    }

    names./* 排序方法 */();
    names
}
```

如果返回类型写成 `Vec<&String>` 会怎样？它和 `Vec<&str>` 的差别只是元素的借用视图；题目要求后者，照着签名写即可。

## 主练习：`most_common`

### 一级：思考方向

先处理“一个标签都没有”的情况：计数表为空时返回 `None`。否则要遍历计数表，一边看计数、一边记住当前最好的标签。并列时不是“先到先得”，而是比较标签文本的字典序。注意返回值借用内部键，不能复制标签。

### 二级：局部分解

1. 用一个 `Option<(&str, u32)>` 保存当前最佳结果，初始为 `None`。
2. 遍历 `&self.counts`，每项是 `(&String, &u32)`。
3. 对每个标签，判断它是否应该取代当前最佳：
   - 还没有最佳：直接成为最佳；
   - 计数比当前最佳大：取代；
   - 计数相等，且标签字典序比当前最佳小：取代；
   - 其他情况：保留当前最佳。
4. 遍历结束后返回保存的结果。标签用 `name.as_str()` 转成 `&str`，计数用 `*count` 复制出 `u32`。

### 三级：接近完整的结构

```rust
pub fn most_common(&self) -> Option<(&str, u32)> {
    let mut best: Option<(&str, u32)> = None;

    for (tag, count) in &self.counts {
        let candidate = (tag.as_str(), *count);
        best = match best {
            None => Some(candidate),
            Some((best_tag, best_count)) => {
                if /* candidate 应该胜出的条件 */ {
                    Some(candidate)
                } else {
                    Some((best_tag, best_count))
                }
            }
        };
    }

    best
}
```

如果直接在遍历里用 `best_tag.cmp(tag.as_str())`，注意 `cmp` 返回的是 `Ordering`，需要和 `Ordering::Less` 比较，不是布尔值。也可以拆成两步：先找最大计数，再在最大计数里找字典序最小的标签。

## 课后巩固：`tally`（`src/consolidation.rs`）

### 一级：思考方向

这是“按名字累加”的问题：每读一行，要么是第一次见到这个商品，要么是追加数量。哪两个条件会让一行被忽略？商品名要按**原样**保存，所以不要去改大小写。返回的映射拥有商品名，但输入是借用的 `&str`，需要在哪里创建 `String`？

### 二级：局部分解

1. 创建空的 `HashMap<String, u32>`。
2. 遍历 `lines`，每项是 `(&str, u32)`。
3. 对商品名做 `trim()`；如果结果为空，跳过这一行。
4. 如果数量是 0，也跳过。
5. 其余行：用 `entry` 找到商品名对应的计数槽位，不存在时插入 0，然后加上这一行的数量。key 需要 `String`，可以用 `name.to_owned()` 或 `String::from(name)`。
6. 遍历结束后返回映射。

### 三级：接近完整的结构

```rust
pub fn tally(lines: &[(&str, u32)]) -> HashMap<String, u32> {
    let mut totals: HashMap<String, u32> = HashMap::new();

    for (name, quantity) in lines {
        let name = name./* 去掉两端空白 */();
        if name.is_empty() || *quantity == 0 {
            continue;
        }

        *totals
            ./* 取得或插入槽位 */(name.to_owned())
            ./or_insert(0) += *quantity;
    }

    totals
}
```

注意 `lines` 是切片，`for (name, quantity) in lines` 里的 `name` 是 `&&str`，`quantity` 是 `&u32`。`name.trim()` 会自动解引用；比较数量时要写 `*quantity`。

## 课后巩固：`find_shortages`

### 一级：思考方向

只检查需求表里的商品。对每个商品，库存可能查不到；查不到时按 0 算，而不是跳过。比较的是“需求 > 库存”，相等不算缺货。结果要按字典序，所以收集完名字后排序。

### 二级：局部分解

1. 创建空的 `Vec<String>`。
2. 遍历 `demand`，每项是 `(&String, &u32)`。
3. 用 `stock.get(name)` 查库存，得到 `Option<&u32>`；`copied()` 变成 `Option<u32>`，`unwrap_or(0)` 处理查不到的情况。
4. 如果 `*needed > available`，把商品名复制成 `String` 放进结果（结果拥有文本，不能借用 `demand`）。
5. 用 `sort_unstable()` 排序后返回。
6. 不要遍历 `stock`：库存有剩但没人订的商品不进结果。

### 三级：接近完整的结构

```rust
pub fn find_shortages(
    demand: &HashMap<String, u32>,
    stock: &HashMap<String, u32>,
) -> Vec<String> {
    let mut shortages: Vec<String> = Vec::new();

    for (name, needed) in demand {
        let available = stock
            ./* 查询方法 */(name)
            ./* 复制出 u32 */()
            ./* 查不到时给 0 */();

        if /* 缺货条件 */ {
            shortages.push(/* 拥有型商品名 */);
        }
    }

    shortages./* 排序方法 */();
    shortages
}
```

用 `demand[name]` 也行，但键一定存在；关键是 `stock` 的查询必须能处理缺失。

## 课后巩固：`everyone_wants`

### 一级：思考方向

两种可行方向，选一种写清楚：

- **集合交集方向**：先把第一份心愿单去重成一个集合，再让这个集合和后面每一份心愿单“求交”。交集只会变小，最后剩下的就是每个人都想要的。
- **计数方向**：统计每个商品出现在**多少份不同**的心愿单里；出现份数等于心愿单总数时才入选。注意同一份里的重复不能重复计数，所以每处理一份，要先去重。

空输入、某一份心愿单为空，这两种边界都要单独想清楚：没有“每一份”可言，或者有一份是空集，结果都应该是空。

### 二级：局部分解（交集方向）

1. 如果 `lists` 为空，直接返回空 `Vec`。
2. 把第一份心愿单的元素收集成 `HashSet<&str>`。集合自动去掉这一份里的重复。
3. 遍历剩下的每一份心愿单：
   - 把这一份也收集成 `HashSet<&str>`（去重）；
   - 用 `retain` 只保留“当前集合里也有”的元素。`retain` 的闭包参数是元素的引用，`HashSet<&str>` 的元素是 `&str`，所以闭包里要再解一层引用。
4. 最后把集合里的每个 `&str` 复制成 `String` 放进 `Vec`。
5. 排序后返回。
6. 如果中途某一份为空，交集自然变成空集——不需要特判，但可以想一想为什么。

### 三级：接近完整的结构

```rust
pub fn everyone_wants(lists: &[&[&str]]) -> Vec<String> {
    if lists.is_empty() {
        return Vec::new();
    }

    let mut common: HashSet<&str> = /* 第一份心愿单收集成集合 */;

    for list in &lists[1..] {
        let current: HashSet<&str> = /* 这一份收集成集合 */;
        common.retain(|item| /* current 是否也包含 item */);
    }

    let mut result: Vec<String> = Vec::new();
    for item in common {
        result.push(/* &str → String */);
    }
    result.sort_unstable();
    result
}
```

`retain` 的闭包参数类型是 `&&str`：集合元素是 `&str`，`retain` 借出元素的引用。`current.contains(...)` 需要 `&str`，所以要把外层引用解开一次。

如果改用计数方向，记得每份心愿单先 `collect` 成 `HashSet`，再逐个增加该商品的“出现份数”；直接遍历原始切片会把同一份里的重复算多次。
