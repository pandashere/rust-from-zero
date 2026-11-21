# 第十四课提示：主练习与课后巩固

提示分三级，卡到哪一级看哪一级：

- **一级：思考方向**——只提示该想什么，不涉及代码结构。
- **二级：局部分解**——把函数拆成几个判断步骤，仍不写完整代码。
- **三级：接近完整的结构**——给出带空缺的骨架，你需要自己补上关键调用。

先看一级；想清楚了就不要继续往下看。三级提示故意不给出可以直接提交的完整代码。

主练习按 README 推荐的顺序完成：先 `parse_record`，再 `Catalog::parse` 和 `records`，然后 `get`，最后 `longest_value`。课后巩固建议先完成单词切分相关的三个方法，再处理最长单词和前缀查询。

## 主练习：`parse_record`（`src/exercise.rs`）

### 一级：思考方向

这个函数的返回值只包含两个 `&str` 字段，没有 `String`。这意味着键和值必须直接指向参数 `line` 内部的片段，不能创建新文本。先处理“整行是否值得解析”，再处理“在哪里切分”，最后处理“切出来的两段是否合格”。

注意规则里有两处容易漏掉：整行去掉空白后可能是空行或注释；键和值各自去掉空白后，键可能为空，但值允许为空。

### 二级：局部分解

1. 先把整行两端的空白去掉，得到一个仍然借用原缓冲区的切片。
2. 如果这个切片为空，或者以 `#` 开头，直接返回 `None`。
3. 找第一个 `=`，把行切成“左边”和“右边”。如果找不到 `=`，返回 `None`。
4. 左右两边分别去掉两端空白。
5. 如果左边（键）为空，返回 `None`。
6. 用左右两段构造 `Record`，返回 `Some(...)`。值即使为空也可以保留。

### 三级：接近完整的结构

```rust
let line = line./* 去掉两端空白 */();
if line./* 是否为空 */() || line./* 是否以 # 开头 */("#") {
    return None;
}
let (key, value) = line./* 在第一个 = 处切成两段 */('=')?;
let key = key./* 去掉两端空白 */();
let value = value./* 去掉两端空白 */();
if key./* 是否为空 */() {
    return None;
}
Some(Record { key, value })
```

`?` 在返回 `Option` 的函数里表示“如果是 `None` 就提前返回 `None`”，第四课已经用过。

## 主练习：`Catalog::parse` 与 `records`

### 一级：思考方向

`Catalog` 拥有的是一个 `Vec<Record<'a>>`，不是文本本身。`parse` 的职责是把 `source` 按行拆开，把每一行交给 `parse_record`，只保留成功的结果。`records` 的职责是把内部 `Vec` 借出去给调用者看，不复制、不重建。

### 二级：局部分解

1. `parse`：准备一个空的 `Vec<Record<'a>>`。
2. 逐行遍历 `source`；每一行拿到的类型是 `&'a str`，可以直接传给 `parse_record`。
3. 如果解析成功，把记录追加到 `Vec`；失败就跳过。
4. 用收集好的 `Vec` 构造 `Catalog` 返回。
5. `records`：返回内部 `Vec` 的只读切片。`&Vec<T>` 可以自动转换成 `&[T]`。

### 三级：接近完整的结构

```rust
pub fn parse(source: &'a str) -> Self {
    let mut records = Vec::new();
    for line in source./* 按行遍历的方法 */() {
        if let Some(record) = parse_record(line) {
            records./* 追加元素的方法 */(record);
        }
    }
    Self { records }
}

pub fn records(&self) -> &[Record<'a>] {
    &self./* 保存记录的字段 */
}
```

## 主练习：`get`

### 一级：思考方向

返回值是 `Option<&str>`，它应该指向目录保存的原始文本，而不是指向查询参数 `key`。查询参数只在比较时被读一次，比较完就可以销毁。按输入顺序扫描，遇到第一条匹配就返回，后面的重复键不再看。

### 二级：局部分解

1. 依次遍历 `self.records` 中的每一条记录（只读借用）。
2. 比较记录的键字段和查询参数 `key`。两者都是 `&str`，比较会逐字符判断是否完全相等，并且区分大小写。
3. 相等就返回 `Some(记录的值字段)`。
4. 循环结束仍没有匹配，返回 `None`。

### 三级：接近完整的结构

```rust
for record in &self.records {
    if record./* 键字段 */ == key {
        return Some(record./* 值字段 */);
    }
}
None
```

不要为了“让生命周期通过”而把 `key` 改成 `&'a str`。那会把查询字符串也绑到原始文本的有效范围上，短命的查询会无法调用。

## 主练习：`longest_value`

### 一级：思考方向

需要一边扫描一边记住“目前最长的值”。有两个状态：当前最佳值，以及它的长度。并列时保留先出现的那个，所以只有新值**严格更长**时才替换。长度按 UTF-8 字节数计算，不是按字符个数。

### 二级：局部分解

1. 准备一个 `Option<&str>` 保存当前最佳值，初始为 `None`。
2. 依次遍历每条记录的值。
3. 如果还没有最佳值，当前值直接成为最佳值。
4. 如果已有最佳值，比较当前值和最佳值的字节长度；只有当前值严格更长才替换。
5. 扫描结束后返回最佳值；目录为空时它仍是 `None`。

### 三级：接近完整的结构

```rust
let mut best: Option<&str> = None;
for record in &self.records {
    let is_longer = match best {
        None => true,
        Some(current) => {
            record.value./* 字节长度 */()
                > current./* 字节长度 */()
        }
    };
    if is_longer {
        best = Some(record.value);
    }
}
best
```

用 `>` 而不是 `>=`，并列时才会保留先出现的值。

## 课后巩固：`new` 与 `source`（`src/consolidation.rs`）

### 一级：思考方向

`WordIndex` 只保存一个对文本的引用。`new` 把参数放进字段；`source` 把字段原样借出去。注意 `source` 返回的是 `&'a str`，不是 `&self` 推导出来的更短生命周期——它承诺返回的切片和原始文本一样有效，可以比索引本身活得久。

### 二级：局部分解

1. `new`：用结构体字面量构造 `Self`，把 `text` 参数放进唯一的字段。
2. `source`：返回 `self.text`。字段本身就是 `&'a str`，复制这个引用不会复制文本。

### 三级：接近完整的结构

```rust
pub fn new(text: &'a str) -> Self {
    Self {
        /* 唯一字段 */: text,
    }
}

pub fn source(&self) -> &'a str {
    self./* 保存文本的字段 */
}
```

## 课后巩固：单词切分、`word_count`、`first_word`、`last_word`

### 一级：思考方向

四个方法共享同一个“单词”定义：先按空白切开，再去掉每个片段两端的标点，最后丢掉空片段。可以把这段逻辑重复写在每个方法里，也可以写一个私有的辅助函数统一处理。注意 `first_word` 和 `last_word` 返回的切片来自原始文本，类型是 `Option<&'a str>`。

### 二级：局部分解

1. 用按空白切分的方法得到片段迭代器。
2. 对每个片段，删除两端属于模块文档所列集合的字符。`trim_matches` 可以接收一个“这个字符要不要删”的判断。
3. 过滤掉删除后变成空字符串的片段。
4. `word_count` 数剩下的片段个数。
5. `first_word` 取第一个；`last_word` 取最后一个；没有元素时它们自然得到 `None`。

### 三级：接近完整的结构

```rust
self.text
    ./* 按空白切分 */
    .map(|piece| piece./* 去掉两端列出的标点 */())
    .filter(|word| !word./* 是否为空 */())
```

在这个迭代器后面接：

- `word_count`：统计元素个数的方法；
- `first_word`：取第一个元素的方法（返回 `Option`）；
- `last_word`：取最后一个元素的方法（返回 `Option`）。

如果写私有辅助函数，注意它的返回类型要能让调用者继续拿到 `&'a str`，而不是被 `&self` 的短生命周期限制。

## 课后巩固：`longest_word`

### 一级：思考方向

和主练习的 `longest_value` 结构相似，但比较的是字符个数，不是字节数。仍然用“严格更长才替换”来保证并列时返回先出现的单词。

### 二级：局部分解

1. 准备 `Option<&'a str>` 保存当前最佳单词。
2. 遍历所有单词（复用上一组的切分逻辑）。
3. 最佳为空时直接保存当前单词。
4. 否则比较字符个数；只有当前单词的字符数严格大于最佳单词时才替换。
5. 返回最佳值。

### 三级：接近完整的结构

```rust
let mut best: Option<&'a str> = None;
for word in /* 单词迭代器 */ {
    let is_longer = match best {
        None => true,
        Some(current) => {
            word./* 字符个数 */()
                > current./* 字符个数 */()
        }
    };
    if is_longer {
        best = Some(word);
    }
}
best
```

## 课后巩固：`count` 与 `first_with_prefix`

### 一级：思考方向

`count` 要数“完全相等”的单词，查询字符串不参与清洗：文本里的 `rust,` 先被清洗成 `rust`，但查询参数 `"rust,"` 仍然带逗号，所以不会匹配。`first_with_prefix` 要按文本顺序找第一个满足前缀的单词；空前缀匹配第一个单词。两个方法的查询参数都只需要在调用期间有效。

### 二级：局部分解

1. `count`：遍历清洗后的单词，对每个单词判断是否与参数完全相等，统计相等次数；区分大小写。
2. `first_with_prefix`：遍历清洗后的单词，找第一个以 `prefix` 开头的单词并返回；找不到返回 `None`。
3. 前缀为空时，`starts_with("")` 对任何字符串都为真，因此会自然匹配第一个单词；没有单词时仍然返回 `None`。
4. 不要清洗 `prefix` 或 `word` 参数本身，只清洗文本中的单词。

### 三级：接近完整的结构

```rust
// count
self.text
    ./* 按空白切分 */
    .map(|piece| piece./* 去掉两端标点 */())
    .filter(|candidate| !candidate./* 是否为空 */())
    .filter(|candidate| *candidate == word)
    .count()

// first_with_prefix
self.text
    ./* 按空白切分 */
    .map(|piece| piece./* 去掉两端标点 */())
    .filter(|candidate| !candidate./* 是否为空 */())
    .find(|candidate| candidate./* 是否以 prefix 开头 */())
```

`find` 返回 `Option<&str>`，其中的引用来自原始文本；`prefix` 的生命周期与它无关。

## 卡住时先检查

- 是否把 `String` 和 `&str` 混用了？`Record` 的字段必须是切片，不能存 `String`。
- 是否给所有引用都写了同一个 `'a`？只有真正需要一起有效的引用才共享生命周期。
- 是否在函数里创建了 `String` 又试图返回它的切片？那会得到“返回局部变量引用”的错误。
- 测试失败信息里如果显示“slice ... is not inside source”，说明返回的文本内容也许对，但已经不是原输入中的片段，可能发生了复制。
