# 第七课提示：主练习与课后巩固

提示分三级，卡到哪一级就看哪一级：

- **一级：思考方向**——只说该想什么问题，不涉及代码结构。
- **二级：局部分解**——把任务拆成几个判断或步骤，仍不写完整代码。
- **三级：接近完整的结构**——给出带空缺的骨架，关键调用由你自己补。

先看一级；想清楚了就不要再往下翻。三级提示故意留空，不提供可以直接提交的完整代码。

排错顺序提示：先确认 `cargo test --no-run` 能编译（编译错误优先修复），再看测试失败信息。`todo!()` 导致的 panic 是预期状态，不是环境问题；`should_panic`、编译失败、找不到测试文件才是真问题。

## 主练习一：`Endpoint::host`（`src/endpoint/model.rs`）

### 一级：思考方向

函数签名是 `pub fn host(&self) -> &str`。注意 `&self`：它只是借用这个端点，不拿走所有权；返回类型也是借用。字段是 `String`，而你要返回 `&str`。想清楚：`String` 和 `&str` 之间是什么关系？你需要新建一个字符串吗？现有数据已经在哪里？

### 二级：局部分解

1. `self` 后面可以直接用 `.` 访问字段，因为你在 `impl Endpoint` 内部，私有字段在这里是可见的。
2. 字段类型是 `String`，目标是 `&str`。`String` 能自动借用成 `&str`（这个转换由标准库提供的类型转换约定完成，不需要你转换类型）。
3. 按返回类型写一个借出字段的表达式即可，不要分配新字符串，不要 `clone()`。

### 三级：接近完整的结构

```rust
pub fn host(&self) -> &str {
    /* 借出 self 的 host 字段，而不是复制它 */
}
```

想验证自己没写错：如果代码里出现了 `String::from`、`to_owned`、`clone`、`format!`，说明你在复制，方向错了。

## 主练习二：`Endpoint::port`（`src/endpoint/model.rs`）

### 一级：思考方向

返回类型是 `u16`，不是 `&u16`。`u16` 是无符号 16 位整数，实现了 `Copy`，所以复制一个 `u16` 值不需要分配内存，也不会把字段移走。想想：读字段和被借出字段，在语法上有什么不同？

### 二级：局部分解

1. 仍然在 `impl Endpoint` 内部，`self.port` 直接可读。
2. 因为 `u16: Copy`，直接返回字段就完成了一次复制，`self` 的所有权不受影响。
3. 不需要 `.clone()`（对 `Copy` 类型调用它虽然能编译，但会被严格 Clippy 提示多余）。

### 三级：接近完整的结构

```rust
pub fn port(&self) -> u16 {
    /* 返回端口数值本身 */
}
```

## 主练习三：`parse_port`（`src/endpoint/parser.rs`）

### 一级：思考方向

输入可能两端带空白，而数字解析不接受这些空白，所以第一步不是解析。第二件事：`str::parse::<u16>()` 返回 `Result<u16, ParseIntError>`，但函数要返回 `Result<u16, ParseEndpointError>`。两种错误类型不同，`?` 不会自动替你做转换。最后一个细节：错误里要保存的是去掉空白后的文本，不是原始输入。

### 二级：局部分解

1. 先把输入 `trim()`，把结果存进一个局部变量，后面都用它。否则“解析用 trim 后的文本、错误存原文本”会自相矛盾。
2. 对去掉空白后的文本调用 `parse::<u16>()`。用 turbofish（`::<u16>`）明确目标类型，编译器就不必猜。
3. 解析结果是 `Result<u16, ParseIntError>`。需要把 `Err` 分支的 `ParseIntError` 换成 `ParseEndpointError::InvalidPort(...)`，`Ok` 分支原样保留。第四课讲过专门做这件事的方法：它只接受一个“如何转换错误”的函数，并把 `Result` 里的错误替换掉。
4. 转换时要把 `&str` 变成 `String`（`InvalidPort` 的字段类型是 `String`）。用 `to_owned()` 或 `String::from`。
5. 用 `?` 传播转换后的结果，最后返回 `Ok(port)`。`?` 在这里省掉了一次 `match`。

### 三级：接近完整的结构

```rust
fn parse_port(input: &str) -> Result<u16, ParseEndpointError> {
    let text = input./* 去掉两端空白 */;

    let port = text
        .parse::<u16>()
        ./* 把 ParseIntError 换成 InvalidPort(text 的拥有副本) */(...)?;

    Ok(port)
}
```

自检清单：

- `parse_port(" 443 ")` 返回 `Ok(443)`；
- `parse_port("65536")` 返回 `Err(InvalidPort("65536"))`；
- `parse_port(" -1 ")` 返回 `Err(InvalidPort("-1"))`——注意错误里没有空格；
- `parse_port("")` 返回 `Err(InvalidPort(""))`；
- `parse_port("4 43")` 返回 `Err(InvalidPort("4 43"))`——中间的空白不会被删除。

如果测试报的是“类型不匹配”，大概率是第 3 步的错误类型没有换掉；如果报“期望 `String` 找到 `&str`”，是第 4 步漏了转换。

## 课后巩固一：`ModulePath::segments`（`src/module_path/model.rs`）

### 一级：思考方向

字段是 `Vec<String>`，返回类型是 `&[String]`。`&[String]` 是切片引用，表示“借用一段连续的字符串”，不复制、不交出所有权。想清楚：如何从一个 `Vec<String>` 得到一个指向它内容的切片？需要重新分配吗？

### 二级：局部分解

1. 在 `impl ModulePath` 内部，`self.segments` 就是那个 `Vec<String>`。
2. 借用 `Vec<T>` 得到的是 `&Vec<T>`。而函数要 `&[T]`。Rust 有一个自动转换：`&Vec<T>` 会在需要时转成 `&[T]`（第五课讲过，这叫 deref 强制转换）。
3. 所以写法上只需要“借出字段”，类型会自动匹配。不要 `to_vec()`、不要 `clone()`。

### 三级：接近完整的结构

```rust
pub fn segments(&self) -> &[String] {
    /* 借出 segments 字段 */
}
```

如果编译器提示“expected `&[String]`, found `&Vec<String>`”，检查你有没有多写了 `&`：写 `&self.segments` 得到 `&Vec<String>`，在返回位置会自动转成 `&[String]`；如果写成 `&&self.segments` 或加了别的包装，就不能自动转换。

## 课后巩固二：`ModulePath::starts_at_crate_root`（`src/module_path/model.rs`）

### 一级：思考方向

“以 `crate` 开头”只看第一段，不看整条路径，也不要求路径只有一段。因此关键问题是：从 `Vec<String>` 里安全地取第一段，得到的类型是什么？“可能没有第一段”这种情况如何处理？取到之后，比较的对象是 `String` 还是 `&str`？

### 二级：局部分解

1. 取第一段有两个常用起点：`self.segments.first()`（返回 `Option<&String>`）或按下标 `self.segments[0]`。前者不 panic，后者在空 `Vec` 上 panic。虽然本类型的构造流程保证至少有一段，但用 `first()` 更稳，也让“空”这种情况在类型上被显式处理。
2. 如果选了 `first()`，你拿到 `Option<&String>`，要把它变成一个 `bool`。标准库有专门“有值时做判断、没有值时给默认值”的方法（`map_or` 系列）；也有 `is_some_and` 这类组合方法。选一个你读得懂的即可。
3. 比较 `&String` 和 `"crate"` 时，可以直接用 `==`：标准库实现了这种比较，不需要先转成 `&str`。若你觉得不直观，可以先解引用再比较。

### 三级：接近完整的结构

```rust
pub fn starts_at_crate_root(&self) -> bool {
    self.segments
        ./* 取第一段 */()
        ./* 有值时判断是否等于 "crate"，没有值时返回 false */;
}
```

自检清单：

- `["crate", "endpoint"]` → `true`；
- `["endpoint", "crate"]` → `false`（只比较第一段）；
- `["crate"]` → `true`。

## 课后巩固三：`is_valid_segment`（`src/module_path/parser.rs`）

### 一级：思考方向

四条规则要按“先能判断、再能继续”的顺序执行，因为后面的检查依赖前面的结论。想一想：如何把一段文本拆成“第一个字符”和“其余字符”？空文本时，拆第一个字符会得到什么？你还得记住一件事：**至少有一个小写字母**，这意味着你需要在遍历过程中记录一个布尔状态。这些字符判断都有现成方法（`is_ascii_lowercase`、`is_ascii_digit`），不要手写字符范围比较。

### 二级：局部分解

1. 用 `chars()` 得到一个字符迭代器，再用 `next()` 取第一个字符，返回 `Option<char>`。`None` 就说明这段是空的，返回 `false`。
2. 检查第一个字符：必须是小写 ASCII 字母或 `_`，否则返回 `false`。顺便：如果第一个字符本身是小写字母，那么“至少有一个小写字母”已经有保证了，可以把状态先记为 `true`。
3. 遍历剩下的字符（第 1 步的迭代器现在正好停在第二个字符上，可以继续 `for` 循环）。每个字符如果小写字母，把状态设为 `true`；如果是数字或 `_`，允许通过；其他情况一律返回 `false`。
4. 循环结束后返回那个状态。这样 `"_"`、`"_1"`、`"__"` 虽然前缀合法，但因为从来没有出现小写字母，会被正确判为非法。

### 三级：接近完整的结构

```rust
fn is_valid_segment(segment: &str) -> bool {
    let mut characters = segment.chars();
    let Some(first) = characters.next() else {
        return false;
    };

    if !(/* first 是小写字母或下划线 */) {
        return false;
    }

    let mut has_lowercase = /* first 是否是小写字母 */;
    for character in characters {
        if /* 小写字母 */ {
            has_lowercase = true;
        } else if !/* 数字或下划线 */ {
            return false;
        }
    }

    has_lowercase
}
```

`let ... else` 语法第四课讲过：模式匹配失败时执行 `else` 块，那里必须提前返回或 panic，因为后面的代码只能看到匹配成功的情况。

如果测试只错在 `"_1"` 或 `"__"` 这类输入，说明你漏了“至少一个小写字母”这条；如果错在 `"9lives"`，说明首字符检查写成了“字母或数字”。

## 课后巩固四：`parse_module_path` 的循环体（`src/module_path/parser.rs`）

### 一级：思考方向

函数外壳已经把“整段输入去空白”和“空输入”处理完了，你只需要处理切分后的每一段。循环变量给了你两样东西：序号和段文本。对每一段，先判断“空不空”，再判断“格式合不合法”，两者失败原因不同，错误类型也不同。全部通过后，这一段需要变成 `String` 存进 `segments`。想一想：为什么只能在这里 `to_owned()`，而不能在检查之前就转？

### 二级：局部分解

1. `input.split("::").enumerate()` 会按 `"::"` 切开并给每段编号。注意 `"crate::"` 会切出两段，第二段是空字符串；`"crate::::parser"` 中间也会切出空段。这就是为什么必须单独判断空段。
2. 第一道检查：`segment.is_empty()` 为真时返回 `Err(ParseModulePathError::EmptySegment { index })`。这里用提前返回，循环不用继续。
3. 第二道检查：调用已经写好的私有函数 `is_valid_segment(segment)`。为假时返回 `Err(InvalidSegment { index, text: segment.to_owned() })`。`text` 必须是拥有所有权的 `String`，因为你之后不能再借用这段输入。
4. 两道检查都通过后，把这一段存进 `segments`（需要 `String`，同样用 `to_owned()` 或 `String::from`）。
5. 循环结束后不要动外壳里已经写好的 `Ok(ModulePath::new(segments))`。特别注意：构造 `ModulePath` 只能通过 `ModulePath::new`，因为字段私有；而 `new` 是 `pub(super)`，正好对 `module_path::parser` 开放。

### 三级：接近完整的结构

```rust
let mut segments = Vec::new();
for (index, segment) in input.split("::").enumerate() {
    if segment./* 是空文本 */ {
        return Err(ParseModulePathError::EmptySegment { index });
    }

    if !/* 调用 is_valid_segment 判断这一段 */ {
        return Err(ParseModulePathError::InvalidSegment {
            index,
            text: /* 这一段的拥有副本 */,
        });
    }

    segments.push(/* 这一段的拥有副本 */);
}
```

自检清单：

- `"crate::endpoint::parser"` → 三段都合法，顺序不变；
- `"::parser"` → `EmptySegment { index: 0 }`；
- `"crate::"` → `EmptySegment { index: 1 }`；
- `"crate::::parser"` → `EmptySegment { index: 1 }`（不是 2，因为第一个空段就返回了）；
- `"crate::Parser"` → `InvalidSegment { index: 1, text: "Parser" }`；
- `"crate:: endpoint"` → `InvalidSegment { index: 1, text: " endpoint" }`——只 trim 整段输入，不 trim 每一段。

如果报错说 `ModulePath::new` 不可见，检查你是不是在别的地方（比如集成测试里）试图调用它；它只对 `module_path` 范围开放。
