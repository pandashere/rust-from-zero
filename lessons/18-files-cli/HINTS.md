# 第十八课提示：主练习与课后巩固

提示分三级，卡到哪一级看哪一级：

- **一级：思考方向**——只提示该想什么，不涉及代码结构。
- **二级：局部分解**——把函数拆成几个判断步骤，仍不写完整代码。
- **三级：接近完整的结构**——给出带空缺的骨架，你需要自己补上关键调用。

先看一级；想清楚了就不要继续往下看。三级提示故意不给出可直接提交的完整代码。所有验收命令都在仓库根目录执行。

---

## 主练习：`parse_args`（`src/exercise.rs`）

### 一级：思考方向

先把“读取参数”和“决定配置”分开：

- 哪些参数是选项，哪些是路径？
- 三个指标开关是否出现过，决定了最后是否套用默认值；`--non-empty` 只影响行模式，不参与“是否出现过指标”的判断。
- 路径只能有一个。怎样区分“还没见过路径”“已经有一个路径”“这是第二个路径”？
- 不是合法 UTF-8 的参数仍可能是路径，不要因为它转不成 `&str` 就当成未知选项。

### 二级：局部分解

1. 把参数迭代器变成可逐个取出的形式，先取走第一项（程序名）。
2. 准备几个状态：路径 `Option<PathBuf>`、三个指标布尔值、一个 `non_empty` 布尔值。
3. 遍历剩余参数。对每一项：
   - 如果内容正好是四个已知选项之一，更新对应状态；
   - 如果内容以 `--` 开头但不是已知选项，返回带原参数的 `Usage`；
   - 否则把它当作路径：如果已经有路径，返回“多个路径”错误；没有就保存。
4. 遍历结束后检查路径是否存在，没有就返回缺少路径错误。
5. 根据“是否出现过任意指标开关”决定是否套用默认三项。
6. 根据 `non_empty` 决定 `lines` 是 `Some(LineMode::NonEmpty)` 还是 `Some(LineMode::All)`；如果行统计被关闭，则保持 `None`。

判断一个 `OsString` 是不是某个文本选项，需要先尝试把它看成 `&str`。转换失败说明它不是文本，应当继续按路径处理，而不是报未知选项。

### 三级：接近完整的结构

```rust
let mut args = args.into_iter();
let _program = args.next();

let mut path: Option<PathBuf> = None;
let mut bytes = false;
let mut words = false;
let mut lines = false;
let mut non_empty = false;
let mut metric_seen = false;

for argument in args {
    let argument: OsString = argument.into();

    match argument.to_str() {
        Some("--bytes") => {
            bytes = true;
            /* 记录“出现过指标” */
        }
        Some("--words") => {
            words = true;
            /* 记录“出现过指标” */
        }
        Some("--lines") => {
            lines = true;
            /* 记录“出现过指标” */
        }
        Some("--non-empty") => {
            non_empty = true;
        }
        Some(text) if text.starts_with("--") => {
            return Err(AppError::Usage(format!("unknown option: {text}")));
        }
        _ => {
            // 这里既可能是普通路径，也可能不是合法 UTF-8 的参数。
            // 如果 path 已经是 Some，要返回什么错误？
            // 否则把 argument 转成 PathBuf 保存。
            todo!()
        }
    }
}

// 路径缺失检查：返回哪个 Usage 原因？
// 默认指标：metric_seen 为 false 时把三项都打开。

let line_mode = if lines {
    Some(if non_empty {
        LineMode::NonEmpty
    } else {
        LineMode::All
    })
} else {
    None
};

Ok(Config {
    path: todo!(),
    bytes,
    words,
    lines: line_mode,
})
```

注意：上面的 `match` 不能直接 `return` 一个“未知选项”给非 UTF-8 参数；`_` 分支要把它当路径。

---

## 主练习：`stats_from_bytes`（`src/exercise.rs`）

### 一级：思考方向

先问“这次运行到底需要文本吗”：

- bytes 只依赖字节长度，不需要解码。
- words 和 lines 都依赖文本；只要请求了其中任意一个，就必须解码。
- 解码失败时，错误里要带哪个路径？
- 行模式有两种：全部行、非空行。空输入、末尾换行、CRLF 分别如何影响结果？

### 二级：局部分解

1. 如果请求了 bytes，先得到 `Some(字节长度)`；否则是 `None`。
2. 判断是否需要文本：`config.words` 为真，或 `config.lines` 是 `Some`。
3. 不需要文本时，直接返回只有 bytes 的结果（words 和 lines 都是 `None`）。
4. 需要文本时，把 `data` 解码成 `&str`。失败就用 `config.path.clone()` 和原始解码错误构造 `InvalidUtf8`。
5. 请求了 words 时，按 Unicode 空白切分并计数；没有请求时保持 `None`。
6. 根据 `config.lines` 的三种状态计算行数：`None` 不统计；`All` 数所有行；`NonEmpty` 过滤掉 `trim()` 后为空的行。
7. 构造 `Stats`，只填被请求的字段。

`str::from_utf8` 返回的是借用文本，不复制字节。不要先做有损转换，那会把非法字节替换掉，无法报告真正的解码位置。

### 三级：接近完整的结构

```rust
let bytes = if config.bytes {
    Some(data.len() as u64)
} else {
    None
};

let needs_text = config.words || config.lines.is_some();

let text = if needs_text {
    Some(
        std::str::from_utf8(data).map_err(|source| AppError::InvalidUtf8 {
            path: config.path.clone(),
            source,
        })?,
    )
} else {
    None
};

let words = if config.words {
    Some(todo!("在 text 上按 Unicode 空白切分并计数"))
} else {
    None
};

let lines = match config.lines {
    None => None,
    Some(LineMode::All) => Some(todo!("数 text 的所有行")),
    Some(LineMode::NonEmpty) => Some(todo!("只数 trim 后非空的行")),
};

Ok(Stats { bytes, words, lines })
```

`text` 是 `Option<&str>`。在 `words` 和 `lines` 分支里，你确定它一定是 `Some`，可以用 `text.unwrap()` 吗？想一想：这个“确定”来自哪里？有没有更清楚地表达它的办法？

---

## 主练习：`format_stats`（`src/exercise.rs`）

### 一级：思考方向

这个函数只做字符串拼接，不做 I/O。先确定输出顺序，再逐个判断字段是不是 `Some`。注意每一行末尾都要有换行，未请求的字段不能留下空行。

### 二级：局部分解

1. 准备一个可增长的 `String`，初始为空。
2. 依次检查 `stats.bytes`、`stats.words`、`stats.lines`。
3. 每个 `Some(value)` 追加一行 `名字: 数值\n`；`None` 跳过。
4. 返回这个字符串。三个都是 `None` 时，自然得到空字符串。

可以用 `format!` 生成一行再 `push_str`，也可以使用写字符串的格式化宏。若使用后者，需要引入 `std::fmt::Write`，注意它和 `std::io::Write` 同名。

### 三级：接近完整的结构

```rust
use std::fmt::Write as _;

let mut output = String::new();

if let Some(value) = stats.bytes {
    /* 追加 "bytes: "、数值和换行 */
}

if let Some(value) = stats.words {
    /* 追加 "words: "、数值和换行 */
}

if let Some(value) = stats.lines {
    /* 追加 "lines: "、数值和换行 */
}

output
```

写入 `String` 的格式化操作不会失败，但仍会返回 `Result`。你需要决定如何处理这个“不可能失败”的结果；不要因此让函数签名变成 `Result`。

---

## 主练习：`run`（`src/exercise.rs`）

### 一级：思考方向

`run` 是“带 I/O 的组合层”：读取文件、调用纯统计、调用纯格式化。它不打印，只返回字符串或结构化错误。读取失败时，错误里要同时有用户路径和原始 `io::Error`。

### 二级：局部分解

1. 用路径读取原始字节。
2. 读取失败时，用 `map_err` 构造 `AppError::Read`；路径要和 `config.path` 一致。
3. 把字节和配置交给 `stats_from_bytes`，用 `?` 传播错误。
4. 把统计结果交给 `format_stats`。
5. 返回格式化后的字符串。

不要在这里写 stdout，也不要把错误转成字符串后丢失原始 `io::Error`。

### 三级：接近完整的结构

```rust
let data = std::fs::read(&config.path).map_err(|source| AppError::Read {
    path: todo!("哪个路径？"),
    source,
})?;

let stats = stats_from_bytes(&data, config)?;
let output = format_stats(&stats);

Ok(output)
```

注意 `config` 已经是 `&Config`，传给 `stats_from_bytes` 时不需要再借一次；`&config.path` 借用路径，不会移动配置。

---

## 课后巩固：`normalize_text`（`src/consolidation.rs`）

### 一级：思考方向

按行处理，但“行”的切分和“行内容”的清理是两件事。先想清楚：

- 行首空白要保留还是去掉？
- 行尾空白去掉后，如果什么都不剩，这一行还要不要输出？
- 输出的每一行之间用什么分隔？最后一行后面要不要换行？
- `\r\n` 应当变成什么？

### 二级：局部分解

1. 准备一个空 `String` 保存结果。
2. 遍历输入的每一行。
3. 对每行去掉末尾空白；行首不要动。
4. 如果清理后为空，跳过这一行。
5. 否则把清理后的内容追加到结果，再追加一个 `\n`。
6. 返回结果。输入为空或全是空白时，结果自然为空字符串。

不要用 `split('\n')` 自己处理 `\r`；标准库的行切分已经定义了 `\r\n` 的行为。

### 三级：接近完整的结构

```rust
let mut output = String::new();

for line in text.lines() {
    let trimmed = todo!("去掉行尾空白，保留行首");

    if trimmed.is_empty() {
        continue;
    }

    output.push_str(trimmed);
    output.push('\n');
}

output
```

---

## 课后巩固：`tidy_file`（`src/consolidation.rs`）

### 一级：思考方向

这个函数把“读、解码、转换、写”串起来。每一步失败都要保留正确的路径：

- 读取失败：输入路径。
- 解码失败：输入路径。
- 写出失败：输出路径。

还要想清楚两个计数分别从哪里来，以及空文件时它们应当是多少。

### 二级：局部分解

1. 读取输入文件的原始字节；失败时构造 `TidyError::Read`。
2. 把字节解码成 `&str`；失败时构造 `TidyError::InvalidUtf8`。
3. 调用 `normalize_text` 得到要写出的文本。
4. 把文本写到输出路径；失败时构造 `TidyError::Write`。
5. 计算写出的行数和丢弃的行数，构造 `TidyReport`。
6. 返回报告。

计算丢弃行数时，可以用“输入总行数减去写出行数”。注意空文件时 `lines().count()` 是 0，不会出现减法下溢。

### 三级：接近完整的结构

```rust
let data = std::fs::read(input).map_err(|source| TidyError::Read {
    path: todo!("输入还是输出路径？"),
    source,
})?;

let text = std::str::from_utf8(&data).map_err(|source| TidyError::InvalidUtf8 {
    path: todo!("输入还是输出路径？"),
    source,
})?;

let normalized = normalize_text(text);

std::fs::write(output, normalized.as_bytes()).map_err(|source| TidyError::Write {
    path: todo!("输入还是输出路径？"),
    source,
})?;

let written_lines = todo!("normalized 有多少行");
let removed_lines = todo!("输入总行数减去 written_lines");

Ok(TidyReport {
    written_lines,
    removed_lines,
})
```

写出用 `normalized.as_bytes()` 或直接传 `normalized` 都可以；前者明确表示写入 UTF-8 字节。注意 `fs::write` 会创建或覆盖文件，不需要手动 `OpenOptions`。

---

## 验收清单

主练习：

```bash
cargo test -p l18_files_cli exercise
```

课后巩固：

```bash
cargo test -p l18_files_cli consolidation
```

全部检查：

```bash
cargo fmt -p l18_files_cli -- --check
cargo check -p l18_files_cli --all-targets
cargo test -p l18_files_cli --no-run
cargo clippy -p l18_files_cli --all-targets -- -D warnings
cargo test -p l18_files_cli
```

如果测试失败，先看失败断言里的输入和期望值，再对照函数注释中的行为约定。不要为了让某个测试通过而写死测试数据。
