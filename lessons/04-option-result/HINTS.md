# 第四课提示：主练习与课后巩固

提示分三级，卡到哪一级看哪一级：

- **一级：思考方向**——只提示该想什么，不涉及代码结构。
- **二级：局部分解**——把函数拆成几个判断步骤，仍不写完整代码。
- **三级：接近完整的结构**——给出带空缺的骨架，你需要自己补上关键调用。

先看一级；想清楚了就不要继续往下看。三级提示故意不给出可直接提交的完整代码。

## 主练习：`parse_port`（`src/exercise.rs`）

### 一级：思考方向

函数拿到的 `text` 可能两端有空白。先想清楚三件事：空白在哪一步去掉？
`text.parse::<u16>()` 失败时拿到的是什么类型的错误，而函数需要返回的又是
什么类型？解析成功后，是不是所有 `u16` 数字都能直接返回？

### 二级：局部分解

1. 对 `text` 调用 `trim()`，之后的判断都用去掉空白后的文本。
2. 解析成 `u16`。解析失败的错误类型是标准库的 `ParseIntError`，不是
   `ParseEndpointError`，需要在失败分支做一次错误转换：把任何解析错误
   变成 `InvalidPort`，并把 trim 后的文本存进去（用 `to_owned()` 或
   `String::from` 把 `&str` 变成拥有的 `String`）。
3. 解析成功后再检查一个特殊值：`0` 要走 `ZeroPort` 分支。
4. 其余情况返回 `Ok(port)`。

### 三级：接近完整的结构

```rust
pub fn parse_port(text: &str) -> Result<u16, ParseEndpointError> {
    let text = text.trim();
    let port = text
        .parse::<u16>()
        ./* 把 Err 转换成 InvalidPort(text 的拥有副本） */(...)?;

    if port == 0 {
        return Err(ParseEndpointError::ZeroPort);
    }

    Ok(port)
}
```

回想 README 中“把别人的错误翻译成自己的错误”一节：哪个方法只在
`Err` 时调用、并把旧错误换成新错误？`?` 在这里替你省掉了哪个 `match`？

## 主练习：`parse_endpoint`（`src/exercise.rs`）

### 一级：思考方向

按注释里的顺序做四道“关卡”：空输入、没有 `:`、主机名为空、端口交给
`parse_port`。每一道关卡失败都直接返回对应的错误。想想哪个标准库方法
能把字符串按“第一个 `:`”切成两段，它返回的是什么类型？这个类型和
`Result` 之间怎么转换？

### 二级：局部分解

1. 对 `input` 调用 `trim()`，为空就返回 `EmptyInput`。
2. 用 `split_once(':')` 按第一个冒号切分。它返回 `Option<(&str, &str)>`：
   没找到冒号时是 `None`。用一个方法把 `None` 变成
   `Err(MissingSeparator)`，再用 `?` 取出成功时的两段。
3. 主机名那段再 `trim()` 一次，为空就返回 `EmptyHost`。
4. 端口那段直接调用 `parse_port`，失败时让 `?` 原样传播错误。
5. 构造 `Endpoint`：`host` 字段需要拥有的 `String`，而你手上是 `&str`。

### 三级：接近完整的结构

```rust
pub fn parse_endpoint(input: &str) -> Result<Endpoint, ParseEndpointError> {
    let input = input.trim();

    if input.is_empty() {
        return Err(ParseEndpointError::EmptyInput);
    }

    let (host, port_text) = input
        .split_once(':')
        ./* None 变成 Err(MissingSeparator) */(...)?;
    let host = host.trim();

    if host.is_empty() {
        return Err(ParseEndpointError::EmptyHost);
    }

    let port = /* 调用 parse_port 并传播失败 */(...)?;

    Ok(Endpoint {
        host: /* 把 &str 变成 String */,
        port,
    })
}
```

回想 README 中“把‘没有’升级成‘失败’”一节：哪个方法能把 `Option`
变成 `Result`？

## 课后巩固：`unit_seconds`（`src/consolidation.rs`）

### 一级：思考方向

输入只有三种合法情况，其余一律“没有值”。用 `match` 按文本内容分支最
直接；注意字符串字面量区分大小写，不需要你做额外处理。

### 二级：局部分解

1. `match unit`，为 `"s"`、`"m"`、`"h"` 各写一个分支，返回对应的
   `Some(秒数)`。
2. 用一个兜底分支 `_` 覆盖其余所有输入，返回 `None`。

### 三级：接近完整的结构

```rust
pub fn unit_seconds(unit: &str) -> Option<u64> {
    match unit {
        "s" => Some(1),
        "m" => /* ... */,
        "h" => /* ... */,
        _ => /* ... */,
    }
}
```

## 课后巩固：`parse_duration`（`src/consolidation.rs`）

### 一级：思考方向

函数可以拆成两段工作：先找到“数字到哪里结束”，把输入切成数字部分和
单位部分；再分别校验数字、单位，最后做乘法。找一个能遍历字符位置、并
在“第一个不是数字的字符”处停下来的方法组合；找不到这样的字符意味着
整段都是数字。乘法本身也可能失败——标准库有一种“乘法会返回 `Option`”
的方法，想想它叫什么。

### 二级：局部分解

1. `trim()` 后判空，空则返回 `Empty`。
2. 用 `char_indices()` 配合 `find` 定位第一个非 ASCII 数字字符的位置：
   - 位置是 `Some(0)`：开头就没有数字，返回 `MissingNumber`；
   - 位置是 `None`：整段都是数字，没有单位，返回 `MissingUnit`；
   - 其他 `Some(index)`：用 `split_at(index)` 切成数字段和单位段。
3. 数字段 `parse::<u64>()`，失败时转成 `InvalidNumber(数字段文本)`。
4. 单位段 `trim()` 后交给 `unit_seconds`；`None` 时转成
   `UnknownUnit(单位文本)`。
5. 用 `checked_mul` 做乘法；`None` 表示乘积超出 `u64`，转成 `Overflow`。
6. 成功返回 `Ok(总秒数)`。

### 三级：接近完整的结构

```rust
pub fn parse_duration(input: &str) -> Result<u64, DurationError> {
    let input = input.trim();

    if input.is_empty() {
        return Err(DurationError::Empty);
    }

    let split = input
        .char_indices()
        .find(|(_, character)| !character.is_ascii_digit())
        .map(|(index, _)| index);

    let (digits, unit) = match split {
        Some(0) => return Err(DurationError::MissingNumber),
        None => return Err(DurationError::MissingUnit),
        Some(index) => input.split_at(index),
    };

    let amount = digits
        .parse::<u64>()
        ./* 转换成 InvalidNumber(digits 的拥有副本） */(...)?;

    let unit = unit.trim();
    let seconds_per_unit = unit_seconds(unit)
        ./* None 变成 Err(UnknownUnit(unit 的拥有副本）) */(...)?;

    let total = amount
        .checked_mul(seconds_per_unit)
        ./* None 变成 Err(Overflow) */(...)?;

    Ok(total)
}
```

`find` 返回的是 `Option<(usize, char)>`，其中位置信息才是你需要的；
`map` 可以只保留位置。三次“`Option` 变 `Result`”用的是同一个方法。
