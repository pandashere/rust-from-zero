# 第十八课：文件、路径、命令行参数与输入输出错误

到目前为止，课程里的程序大多在内存（程序运行时存放数据的地方）中接收数据、计算、再返回结果。真实程序还要和程序外部打交道：用户希望在运行时告诉程序“处理哪个文件”，程序需要打开这个文件，读取里面的内容，把结果写到屏幕上；文件可能不存在，内容可能不是文本，屏幕输出也可能失败。

本课从一个很小的需求出发：统计一个文本文件有多少字节、多少个词、多少行。我们会把它做成一个真正的命令行工具 `rstat`，并在这个过程中认识操作系统（管理文件和程序运行的系统软件，例如 Linux、Windows）边界上的四类新问题：命令行参数、路径、文件读写和输入输出错误。

## 本课定位

本课解决的具体问题是：

- 程序不能在编译（把源码转换成可执行程序）时就把文件名写死，否则每换一个文件都要改源码、重新编译。
- 用户输入的“路径”和代码里的普通文本不是同一种东西；路径可能包含当前平台特有的分隔符，也可能不是合法文本。
- 文件读取、内容解码和屏幕输出都可能失败，失败原因不同，程序应当给出不同信息，而不是统一崩溃。
- 这些行为必须能被自动测试，不能每次都靠人工敲命令观察。

本课结束后，你会得到一个分层清楚的 CLI：

```text
命令行参数 ──> parse_args ──> Config
                                │
                                ▼
文件字节 ────────────────> stats_from_bytes ──> Stats ──> format_stats ──> 文本
                                                                              │
                                                                              ▼
                                                                    main 写 stdout
```

每一层只做一件事：解析参数、纯计算、格式化、连接操作系统。这样测试可以直接调用中间层，不需要启动一个新进程。

### 前置知识

不需要学过其他编程语言。下面这些内容前面已经出现过，这里各用一句话复习；如果完全想不起来，先回到对应课程看一眼再继续。

- **变量、函数、`if`、循环、`main`**：程序从 `main` 开始执行，函数把一段操作起个名字，便于重复使用。见[第一课](../01-basics/README.md)。
- **结构体、枚举、`match`、方法**：`struct` 把几个相关值放在一起；`enum` 表示“只能是这几种情况之一”；`match` 按不同情况分支。见[第二课](../02-data-modeling/README.md)。
- **所有权、借用、`String` 与 `&str`**：一份数据同一时间只有一个所有者；借用是临时访问而不夺取所有权；`String` 拥有文本，`&str` 借用一段文本。见[第三课](../03-ownership-borrowing/README.md)。
- **`Option`、`Result`、`?`、自定义错误**：`Option<T>` 表示“可能有值，也可能没有”；`Result<T, E>` 表示“成功得到 `T`，或失败得到错误 `E`”；`?` 在失败时提前返回。见[第四课](../04-option-result/README.md)。
- **模块与 crate**：一个 package（Cargo 管理的代码包）可以同时有库（`src/lib.rs`）和可执行程序（`src/main.rs`），前者方便测试，后者负责启动；crate 是一次编译的代码单元，库和可执行程序各自是一个 crate。见[第七课](../07-modules-crates/README.md)。
- **自动测试**：`#[test]` 标记的函数由 `cargo test` 运行，断言（检查某个条件是否成立）失败就报告。见[第八课](../08-testing-quality/README.md)。
- **泛型与迭代器**：泛型让同一份代码适用于多种类型；迭代器按顺序逐个产出元素。见[第十课](../10-iterators/README.md)和[第十一课](../11-generics/README.md)。本课只用到很浅的一层，遇到时会重新解释。

### 本课边界

- 只使用 Rust 标准库（Rust 安装时自带的代码库），不引入第三方命令行解析库。这样你能看清每一步，而不是把行为交给一个不透明的工具。
- 文件 I/O（input/output，输入输出）是同步的：读取时当前线程（程序内部一条独立的执行流）会等待操作完成才继续。异步 I/O（等待时不阻塞当前线程，先去推进别的任务）在[第十九课](../19-async/README.md)讨论。
- 主练习允许一次把整个文件读进内存，适合中小文件。文件很大时的流式做法（边读边处理，不把整个文件同时放进内存）会在正文中演示，但不作为作业要求。
- 不涉及文件锁（防止多个程序同时修改同一文件的机制）、并发（多个执行流同时工作）写同一文件、原子替换（要么完全换成新文件，要么保持旧文件）等更复杂的文件系统主题。

## 如何学习本课

建议按下面的顺序使用本课，预计需要两到三次学习时段：

1. 顺着“具体问题 → 旧办法为什么不够 → 新概念”读正文，每段代码都先猜输出，再运行或对照解释。
2. 读“编译错误与排错”时，不要只看结论；先试着自己解释 `expected` 和 `found` 分别是什么。
3. 完成“代码预测”，再打开 `src/exercise.rs` 做主练习。
4. 主练习测试全部通过后，完成 `src/consolidation.rs` 的课后巩固。
5. 最后用自己的话回答“课后复盘”的问题。

正文里标为“完整示例”的代码可以单独编译运行；其他函数片段只展示当前主题相关的函数，省略了 `main`、重复的 `use` 或错误类型的 `Display` 实现，前后文字会说明它们的上下文。

所有命令都在仓库根目录执行。`cargo run -p l18_files_cli` 里的 `cargo` 是 Rust 自带的构建与包管理工具，`-p` 是 `--package` 的缩写，用来指定运行哪个 package。命令中的 `--` 用来告诉 Cargo：“后面的内容不是给 Cargo 的，而是传给我们的程序”。例如：

```bash
cargo run -p l18_files_cli -- --bytes notes.txt
```

在完成练习之前，这条命令会因为 `todo!()` 而中止；完成之后，它才会真正统计文件。

## 一、先从具体问题开始

假设你有一个文件 `notes.txt`，想知道它有多少行。最容易写出的程序是：

```rust
fn main() {
    let text = std::fs::read_to_string("notes.txt").expect("cannot read notes.txt");
    println!("{}", text.lines().count());
}
```

逐行解释：

- `std::fs` 是标准库中负责文件系统的模块。`fs` 是 file system 的缩写。
- `read_to_string` 一次读出整个文件，并检查内容是不是合法文本；成功时得到 `String`。
- `expect` 在失败时立刻让程序崩溃（panic），并把括号里的文字作为提示。它适合“程序自身写错了”的情况，不适合处理用户输入错误。
- `text.lines()` 把文本按行切开，逐行产出；`.count()` 数一共有多少项。
- `println!` 把结果写到标准输出（stdout，下面会解释）。

这个程序能工作，但有几个具体困难：

1. 文件名写死在源码里。想统计另一个文件，就得修改代码并重新编译。
2. 失败只有一种反应：崩溃。用户看不到“文件不存在”和“没有权限”的区别，也拿不到一个可判断的退出码。
3. 输出只有数字，没有说明这个数字是什么，其他程序难以稳定解析。
4. 逻辑全部塞在 `main` 里，自动测试只能启动整个进程，难以单独验证参数解析或统计规则。

要解决第一个问题，程序必须在运行时从外部接收信息。这个外部接口就是**命令行界面**（command-line interface，缩写 CLI）：用户在终端（输入命令的窗口程序）里输入程序名和若干段文字，程序读取这些文字并执行。

用户在程序名后面输入的每一段文字叫一个**参数**（argument）。例如：

```text
cargo run -p l18_files_cli -- --bytes notes.txt
```

对我们自己的程序来说，真正的参数是 `--bytes` 和 `notes.txt`。终端通常按空白把一行拆成多段；如果路径里有空格，就用引号包起来，例如 `"input file.txt"`，终端会把引号里的内容当成一个参数。

程序运行时，操作系统会为它建立一个**进程**（process）：一段正在执行的程序，加上它自己的内存、打开的文件和输入输出通道。参数由操作系统在启动进程时交给程序。

## 二、参数从哪里来：`env::args_os`

Rust 标准库的 `std::env` 模块提供“环境”相关信息（进程启动时从操作系统继承的配置和参数），其中 `args_os` 返回一个迭代器，逐项产出启动参数。

**迭代器**（iterator）是一个能按顺序一次次交出元素的值；`for` 循环就是在使用迭代器。第十课已经系统讲过，这里只需要知道：`args_os()` 不会一次性把所有参数做成列表，而是每取一次给一个。

下面是一个完整程序，打印它收到的每个参数：

```rust
use std::env;

fn main() {
    for (index, argument) in env::args_os().enumerate() {
        println!("{index}: {argument:?}");
    }
}
```

- `enumerate()` 给每个元素配一个从 0 开始的序号。
- `{argument:?}` 使用 `Debug` 格式。`Debug` 是给开发者看的表示方式，会给字符串加引号、转义特殊字符，适合检查原始内容。
- 第 0 项通常是程序自身的路径；用户输入从第 1 项开始。某些平台允许启动时不给程序名，所以严格来说“第 0 项是程序名”是常见约定而不是 Rust 类型系统保证的规则；本课按常见约定处理。

为什么函数名带 `_os`？因为操作系统参数不一定是合法文本。

### 文本、字节与 UTF-8

计算机存储的最小单位是**字节**（byte），一个字节是 8 个二进制位（每个位只能是 0 或 1），可以表示 0 到 255 的整数。

人类看到的字符要按某种**编码**（encoding）规则变成字节。**Unicode** 是一个给世界上大多数字符编号的标准；**UTF-8** 是其中一种把编号变成字节的规则：一个字符可能占 1 到 4 个字节。

例如：

- 字母 `a` 在 UTF-8 中占 1 个字节。
- 汉字“语”占 3 个字节。
- 并不是任意一串字节都能按 UTF-8 解释成文本。`0xff` 这个字节单独出现时就不是合法 UTF-8。

在 Unix 类系统上，文件名本质上是字节序列，不保证是 UTF-8；Windows 上文件名使用 UTF-16（另一种编码规则）。Rust 因此提供了专门的类型 `OsString` / `OsStr`（OS 是 operating system 的缩写），用来无损保存“操作系统自己的字符串”。

- `env::args()` 产出 `String`。遇到不是合法 Unicode 的参数时，它会 panic。
- `env::args_os()` 产出 `OsString`。它不要求合法文本，因此适合路径。

规则可以记成：

- 参数语义是**路径**：保留为 `OsString`，再转成 `PathBuf`。
- 参数语义确实是**文本**，例如选项名 `--bytes`：才尝试转成 `&str`，并处理“不是文本”的情况。

本课脚手架里的 `parse_args` 接受任意“能变成 `OsString`”的元素：

```rust,ignore
fn parse_args<I, S>(args: I) -> Result<Config, AppError>
where
    I: IntoIterator<Item = S>,
    S: Into<OsString>,
```

这是泛型写法，读作：“`I` 是任何能逐个产出 `S` 的东西，`S` 是任何能转换成 `OsString` 的类型。”实际程序传入 `env::args_os()`，元素是 `OsString`；测试传入 `["rstat", "--bytes", "input.txt"]`，元素是 `&str`。两者都满足要求，所以同一份解析逻辑既能用于真实进程，也能在测试里直接调用。泛型的完整规则在第十一课，这里只需要会读这个签名。

## 三、路径不是一段普通文本：`Path` 与 `PathBuf`

**路径**（path）是操作系统用来定位文件或目录的说明，例如 `data/notes.txt`。它不是普通字符串，原因有三类：

1. 分隔符不同：Unix 类系统用 `/`，Windows 也接受 `\`，还有盘符（例如 `C:`）和网络路径前缀。
2. 路径可能不是合法 UTF-8，不能强行塞进 `String`。
3. 路径有结构：父目录、文件名、扩展名，操作系统对它们的解释有专门规则。

Rust 用两个类型区分“借用”和“拥有”，和第三课的 `&str` / `String` 对应：

- `Path`：借用的路径视图，类似 `&str`，本身不拥有字符数据。
- `PathBuf`：拥有并可增长的路径，类似 `String`。

完整示例：

```rust
use std::path::{Path, PathBuf};

fn describe(path: &Path) {
    println!("完整路径: {}", path.display());

    match path.file_name() {
        Some(name) => println!("文件名: {}", name.to_string_lossy()),
        None => println!("没有文件名（可能是根目录或以 .. 结尾）"),
    }

    match path.extension() {
        Some(extension) => println!("扩展名: {}", extension.to_string_lossy()),
        None => println!("没有扩展名"),
    }
}

fn main() {
    let mut output = PathBuf::from("build");
    output.push("reports");
    let file = output.join("daily report.txt");

    describe(&file);
    println!("父目录: {:?}", file.parent());
}
```

在 Linux 上，输出大致是：

```text
完整路径: build/reports/daily report.txt
文件名: daily report.txt
扩展名: txt
父目录: Some("build/reports")
```

在 Windows 上，`display()` 可能显示反斜杠。源码里始终写 `/` 也可以，Rust 的路径类型会按平台解释；但更推荐用 `join` / `push`，不要手工拼接分隔符字符串。

几个方法的区别：

- `Path::new("data")`：从一段文本借用出 `&Path`。
- `PathBuf::from("data")`：创建拥有的路径。
- `path.join("child")`：返回一个新 `PathBuf`，不修改原路径。
- `path.push("child")`：直接修改 `PathBuf`，追加一段。
- `path.display()`：返回一个专门用于显示的值，遇到非文本路径会用替换字符表示。它适合日志，不适合重新打开文件。
- `path.to_str()`：只有路径是合法 UTF-8 时才返回 `Some(&str)`；否则返回 `None`。
- `path.to_string_lossy()`：总是返回一个可能是借用、也可能是新建的字符串，非法部分会被替换。它同样是有损的。

函数参数写 `&Path`，调用时可以传 `&PathBuf`。这是**解引用强制转换**（deref coercion）：`PathBuf` 能借用成 `Path`，编译器自动完成这层转换。完整机制在[第十三课](../13-box-recursion/README.md)讲 `Box<T>` 与 `Deref` 时展开；当前只需要知道“拥有型可以借给接受借用型的函数”。

结论：路径参数一路保存为 `PathBuf`；只有要显示给用户或要与文本选项比较时，才做有损转换。

## 四、把参数变成配置：为什么要一个纯函数

参数解析的任务是：把一串原始参数变成一个结构清楚、后续代码容易使用的配置。

如果直接在 `main` 里边读参数边干活，会带来两个问题：

- 测试必须修改整个进程的参数，多个测试并行时会互相干扰。
- `main` 里一旦出错就要决定退出码、打印错误，业务逻辑和进程控制混在一起。

更好的做法是把解析写成**纯函数**：相同输入总得到相同输出，不读全局状态，不打印，不退出进程。它只返回 `Result<Config, AppError>`。

本课配置的含义：

```rust
pub struct Config {
    pub path: PathBuf,
    pub bytes: bool,
    pub words: bool,
    pub lines: Option<LineMode>,
}
```

`lines` 不是简单的 `bool`，因为它有三种状态：

- `None`：完全不统计行。
- `Some(LineMode::All)`：统计所有行。
- `Some(LineMode::NonEmpty)`：只统计去掉两端空白后非空的行。

用 `Option<LineMode>` 而不是两个布尔值，可以让“不可能的状态”无法表示，例如“不统计行，但又要求只统计非空行”。

### 一个完整但不同的解析示例

下面这个程序只接受两个位置参数（按出现顺序对应固定含义的参数），把第一个当作来源、第二个当作目标。它没有开关，因此不会泄露主练习的开关处理；你可以先运行它，理解“注入迭代器 + 返回 Result”的形状。

```rust
use std::env;
use std::ffi::OsString;
use std::path::PathBuf;
use std::process::ExitCode;

fn parse_pair(mut args: impl Iterator<Item = OsString>) -> Result<(PathBuf, PathBuf), String> {
    let _program = args.next();
    let source = args
        .next()
        .ok_or_else(|| "missing source path".to_string())?;
    let destination = args
        .next()
        .ok_or_else(|| "missing destination path".to_string())?;

    if args.next().is_some() {
        return Err("too many paths".to_string());
    }

    Ok((PathBuf::from(source), PathBuf::from(destination)))
}

fn main() -> ExitCode {
    match parse_pair(env::args_os()) {
        Ok((source, destination)) => {
            println!("{} -> {}", source.display(), destination.display());
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::from(2)
        }
    }
}
```

解释：

- `impl Iterator<Item = OsString>` 表示“参数是任何能逐个产出 `OsString` 的迭代器”。`env::args_os()` 正好满足。
- 第一行 `let _program = args.next();` 跳过程序名。变量名前面加下划线，表示“我拿到它但不使用”，避免未使用变量警告。
- `ok_or_else` 把 `Option` 转成 `Result`：有值时成功，没值时调用闭包生成错误。闭包是能捕获周围变量的小段代码，第九课讲过。
- `?` 在 `Err` 时提前返回。这里错误类型是 `String`，函数返回的也是 `Result<_, String>`，所以不需要转换。
- 多出来的第三个路径也被拒绝，避免用户误以为程序会处理它。
- 返回值直接构造 `PathBuf`，不尝试转成 UTF-8 字符串。
- `main` 返回 `ExitCode`，成功用 `ExitCode::SUCCESS`，参数错误用 `ExitCode::from(2)`。

### 退出码是什么

程序结束时会给操作系统一个整数，叫**退出码**（exit code）。终端和脚本可以读取它来判断成功还是失败：

- `0`：成功。`ExitCode::SUCCESS` 就是 0。
- 非零：失败。具体数字由程序约定。

常见约定是：`1` 表示运行失败，`2` 表示用法错误（参数不对）。这不是 Rust 强制规定，但很多命令行工具都这样区分，本课也沿用。

在 Unix shell（接收命令的终端程序）中，可以用 `echo $?` 查看上一条命令的退出码；在 Windows PowerShell（Windows 的命令行程序）中可以用 `$LASTEXITCODE`。

### 主练习的开关语义（只讲行为）

主练习的命令形式是：

```text
rstat [--bytes] [--words] [--lines] [--non-empty] <path>
```

行为规则：

- 四个以 `--` 开头的名字叫**长选项**（long option）。
- `--bytes`、`--words`、`--lines` 分别请求字节、词、行统计；同一个开关重复出现不改变结果。
- 三个指标开关一个都没出现时，默认三项全开。
- `--non-empty` 只修改行统计方式，不算“请求了指标”。只写它时，仍然默认三项全开，但行只数非空行。
- 只要出现任意一个指标开关，就只统计出现过的指标。例如只写 `--bytes`，就只输出字节数。
- 未知的 `--` 选项、缺少路径、多个路径都是用法错误。
- 路径保留为 `PathBuf`，不要求路径文本合法。

这些规则会在“主练习”一节用真实输入和输出完整列出。

## 五、读取文件：一次读入

在 Rust 中，路径和已打开的文件是两个概念：

- `Path` / `PathBuf` 只是“去哪里找”。
- `File` 表示一个已经打开的文件句柄（程序里代表这个已打开文件的凭据），可以读、写或同时读写。

最简单的读取方式：

```rust
use std::fs;
use std::io;
use std::path::Path;

fn byte_length(path: &Path) -> io::Result<u64> {
    let data = fs::read(path)?;
    Ok(data.len() as u64)
}

fn main() {
    match byte_length(Path::new("notes.txt")) {
        Ok(length) => println!("{length} bytes"),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            eprintln!("file not found");
        }
        Err(error) => eprintln!("read failed: {error}"),
    }
}
```

解释：

- `fs::read(path)` 一次读出全部内容，返回 `io::Result<Vec<u8>>`。
- `Vec<u8>` 是字节列表，`u8` 是 0 到 255 的无符号整数。它不保证内容是文本。
- `io::Result<T>` 是 `Result<T, io::Error>` 的别名。`io::Error` 保存操作系统返回的错误信息。
- `?` 在读取失败时提前返回错误。
- `error.kind()` 返回 `ErrorKind`，这是错误的稳定分类，例如 `NotFound`、`PermissionDenied`、`AlreadyExists`。只在程序对不同类别采取不同动作时才匹配它；不要用错误字符串做判断，因为文字不是稳定接口。
- `eprintln!` 写到标准错误（stderr），下面会解释它和标准输出的区别。

如果确定内容是文本，可以用 `fs::read_to_string(path)`，它直接返回 `String`。但它会在内容不是合法 UTF-8 时返回一个 `io::Error`（类别通常是 `InvalidData`），你拿不到底层的 `Utf8Error`，也就无法知道“第几个字节开始不合法”。主练习需要区分“读取失败”和“内容不是文本”，所以脚手架先用 `fs::read` 拿字节，再自行解码。

### 如果文件很大：缓冲与流式读取

`fs::read` 会把整个文件放进内存。几十 MB 的文件通常没问题；几个 GB 的文件就可能耗尽内存。需要逐行处理时，可以打开 `File`，套一层 `BufReader`：

```rust
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

fn print_numbered(path: &Path) -> io::Result<()> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    for (index, line) in reader.lines().enumerate() {
        let line = line?;
        println!("{}: {line}", index + 1);
    }

    Ok(())
}
```

- `File::open` 以只读方式打开文件。
- `BufReader` 在内存中准备一块**缓冲区**（buffer），一次向操作系统要一大块数据，之后逐行切分，减少系统调用次数。系统调用是程序请求操作系统做事的操作，次数太多会变慢。
- `reader.lines()` 每次产出一个 `io::Result<String>`。它去掉行尾的 `\n`，如果行尾是 `\r\n` 也去掉 `\r`。
- 每一行都会新建一个 `String`。如果追求更少的内存分配，可以使用 `read_line` 复用同一个 `String`，或使用 `read_until` 处理字节。当前阶段不必优化到这一步。

本课主练习为了保持逻辑简单，允许一次读完整个文件；上面的流式写法是你在真实项目中处理大文件时应当知道的方向。

## 六、从字节到文本：字节、字符、词、行

统计之前必须先明确“统计口径”，也就是每个数字到底数什么。

### 字节数

`data.len()` 是字节数。它对任何文件都成立，不需要内容是文本。

### 字符数

`text.chars().count()` 数的是 Unicode 标量值（Unicode 为字符分配的编号；Rust 的 `char` 保存一个这样的值）。一个中文字符通常是一个 `char`，但它在 UTF-8 中占 3 个字节。组合字符和 emoji 还可能由多个 `char` 组成一个“用户看到的字符”；本课不处理这种字素簇（grapheme cluster）级别的细节。

完整示例：

```rust
fn main() {
    let text = "héllo 世界";

    println!("bytes: {}", text.len());
    println!("chars: {}", text.chars().count());
    println!("words: {}", text.split_whitespace().count());

    for word in text.split_whitespace() {
        println!("word: {word}");
    }
}
```

输出：

```text
bytes: 13
chars: 8
words: 2
word: héllo
word: 世界
```

`é` 在 UTF-8 中占 2 个字节，所以 5 个拉丁字符占 6 字节；两个汉字占 6 字节；空格 1 字节，总共 13 字节。

`split_whitespace()` 按 Unicode 空白切分，连续空白只算一次分隔，并且不会产出空字符串。它很适合“词数”这种口径。

### 行数

`str::lines()` 的行为需要精确理解：

| 输入 | `lines()` 产出的行数 | 说明 |
| --- | --- | --- |
| `""` | 0 | 空文本没有行 |
| `"a"` | 1 | 没有换行也算一行 |
| `"a\n"` | 1 | 末尾换行结束这一行，不额外产生空行 |
| `"a\n\n"` | 2 | 中间的空行算一行 |
| `"\n"` | 1 | 一个空行 |
| `"a\r\nb"` | 2 | `\r\n` 是一个行尾 |

所以“行”的规则是：行与行之间用 `\n` 分隔；末尾的 `\n` 只是结束最后一行，不会凭空多出一行。`\r\n` 中的 `\r` 不会留在行内容里。

### 解码字节

如果手上是 `&[u8]`，要变成 `&str`，可以调用 `std::str::from_utf8`：

```rust
fn decode(data: &[u8]) {
    match std::str::from_utf8(data) {
        Ok(text) => println!("valid UTF-8: {text}"),
        Err(error) => {
            println!("invalid at byte {}", error.valid_up_to());
            match error.error_len() {
                Some(length) => println!("bad sequence length: {length}"),
                None => println!("input ends in the middle of a character"),
            }
        }
    }
}
```

- 成功时返回 `Ok(&str)`，它借用原来的字节，不复制。
- 失败时返回 `Err(Utf8Error)`。`valid_up_to()` 是“从开头到第几个字节为止是合法的”；`error_len()` 是“非法序列有多长”，如果是 `None`，表示字节在字符中间就结束了。
- 这个函数不分配新字符串，适合先验证再统计。

主练习的策略是：

- 只请求字节数时，完全不解码，任何字节都能统计。
- 请求词数或行数时，必须解码；解码失败就返回带路径和原始 `Utf8Error` 的错误，不输出“部分成功”的数字。

## 七、写文件：覆盖、追加与刷新

主练习只读文件；课后巩固要写文件。先看三种常见写法。

### 一次性覆盖写入

```rust
use std::fs;
use std::io;
use std::path::Path;

fn write_report(path: &Path, content: &str) -> io::Result<()> {
    fs::write(path, content.as_bytes())
}
```

`fs::write` 在文件不存在时创建，存在时把长度截断为 0 再写入。它适合“整个文件内容我都有”的场景。

### 追加写入

```rust
use std::fs::OpenOptions;
use std::io::{self, Write};

fn append_event(message: &str) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("events.log")?;

    writeln!(file, "event={message}")?;
    file.flush()?;
    Ok(())
}
```

`OpenOptions` 让你逐项指定打开方式：

- `read(true)`：允许读。
- `write(true)`：允许写。
- `truncate(true)`：打开时清空。
- `append(true)`：每次写到文件末尾，隐含可写。
- `create(true)`：不存在时创建。
- `create_new(true)`：必须新建，已存在则失败，适合避免覆盖别人的文件。

### 缓冲写入与 `flush`

对大量小段写入，可以使用 `BufWriter`：

```rust
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::Path;

fn write_numbers(path: &Path, values: &[u32]) -> io::Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    for value in values {
        writeln!(writer, "{value}")?;
    }

    writer.flush()?;
    Ok(())
}
```

`BufWriter` 先把内容攒在内存里，攒够或显式刷新时才真正交给操作系统。它减少了系统调用次数。

关键在于最后一行的 `flush()`。`BufWriter` 被销毁时也会尝试刷新，但销毁（`Drop`）不能把失败返回给调用者；如果磁盘满了或管道断了，错误会被忽略。因此重要输出要显式 `flush()`，让失败进入 `Result`。

还要注意：`flush()` 只保证数据交给了操作系统，不保证已经写到物理磁盘。需要更强持久化保证时，还要使用 `File::sync_all`、临时文件加 `rename` 等机制；这超出本课范围。

## 八、错误类型：把失败分类并保留上下文

如果所有函数都返回 `Result<T, String>`，调用者只能看到一段文字，无法稳定判断“是文件不存在还是权限不足”，也拿不到原始错误。Rust 的常见做法是为一个应用定义自己的错误枚举。

本课脚手架的 `AppError` 已经提供，不需要你实现：

```rust
#[derive(Debug)]
pub enum AppError {
    Usage(String),
    Read { path: PathBuf, source: io::Error },
    InvalidUtf8 {
        path: PathBuf,
        source: std::str::Utf8Error,
    },
    Output(io::Error),
}
```

每个变体存在的理由：

- `Usage(String)`：参数错误。具体原因放在字符串里，例如缺少路径；固定的 usage 行由 `Display` 统一附加，避免每个调用点重复拼接。
- `Read { path, source }`：读取失败。保留用户给出的路径，错误信息才能指出“哪个文件”失败；保留原始 `io::Error`，调用者才能检查 `source.kind()`。
- `InvalidUtf8 { path, source }`：内容不是合法 UTF-8。和读取失败分开，因为处理方式不同：前者可能换个文件，后者可能加 `--bytes`。
- `Output(io::Error)`：写标准输出失败，例如下游程序提前关闭管道。输出失败也是失败，不能假装成功。

`Display` 决定用户看到的文字：

```rust
impl std::fmt::Display for AppError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Usage(message) => write!(formatter, "{message}\n{USAGE}"),
            AppError::Read { path, source } => {
                write!(formatter, "failed to read {}: {source}", path.display())
            }
            AppError::InvalidUtf8 { path, source } => {
                write!(formatter, "{} is not valid UTF-8: {source}", path.display())
            }
            AppError::Output(source) => write!(formatter, "failed to write stdout: {source}"),
        }
    }
}
```

`std::error::Error` 的 `source()` 返回底层错误，让调用者可以沿着错误链继续检查：

```rust
impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AppError::Usage(_) => None,
            AppError::Read { source, .. } | AppError::Output(source) => Some(source),
            AppError::InvalidUtf8 { source, .. } => Some(source),
        }
    }
}
```

`dyn Error` 读作“某个实现了 Error 行为的类型”，具体类型在运行时才知道，这叫动态分发（运行时再决定调用哪个具体实现）。这里的 trait（特征，描述一组共同行为的接口）是 `std::error::Error`；第十二课讲过 trait 和动态分发，这里只需要知道它让不同底层错误能放在同一个返回类型里。

### `?` 如何转换错误

`?` 不只是“失败就返回”。它还会尝试用 `From` 把底层错误转换成外层错误。下面是一个独立示例：

```rust
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug)]
enum SettingsError {
    Read { path: PathBuf, source: io::Error },
}

fn load_settings(path: &Path) -> Result<String, SettingsError> {
    let text = fs::read_to_string(path).map_err(|source| SettingsError::Read {
        path: path.to_path_buf(),
        source,
    })?;

    Ok(text)
}
```

这里用 `map_err` 显式构造错误，因为需要把当前路径一起存进去：`map_err` 只改失败时的错误，成功得到的值保持原样。若错误类型实现了 `From<io::Error>`，也可以直接写 `?`，让编译器自动转换：

```rust
impl From<io::Error> for SettingsError {
    fn from(source: io::Error) -> Self {
        SettingsError::Read {
            path: PathBuf::from("<unknown>"),
            source,
        }
    }
}
```

但这种自动转换拿不到具体路径，所以本课的主练习采用 `map_err`，在错误发生的那一层附上正确的路径。两种方式都合法，选择取决于你需要保留多少上下文。

## 九、`main` 是边界层：stdout、stderr 与退出码

一个进程默认有三个标准通道：

- **标准输入**（stdin）：程序读取输入的地方。
- **标准输出**（stdout）：正常结果写这里。`println!` 写的就是它。
- **标准错误**（stderr）：诊断、警告、错误写这里。`eprintln!` 写的是它。

分开的原因是用户可能只想保存正常结果：

```bash
rstat notes.txt > counts.txt
```

`>` 把 stdout 重定向到文件，stderr 仍然显示在终端。如果错误也写进 stdout，结果文件就会被污染，其他程序也无法稳定解析。

`main` 的职责应当尽量小：读取进程参数、调用库函数、把结果写到 stdout、把错误写到 stderr、返回退出码。真正的解析、统计、格式化都在库里。

本课脚手架的 `src/main.rs` 已经写好：

```rust
use l18_files_cli::exercise::{AppError, parse_args, run};
use std::env;
use std::io::{self, Write};
use std::process::ExitCode;

fn main() -> ExitCode {
    let config = match parse_args(env::args_os()) {
        Ok(config) => config,
        Err(error) => {
            eprintln!("error: {error}");
            return ExitCode::from(2);
        }
    };

    let result = run(&config).and_then(|output| {
        let mut stdout = io::stdout().lock();
        stdout
            .write_all(output.as_bytes())
            .and_then(|()| stdout.flush())
            .map_err(AppError::Output)
    });

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}
```

逐段解释：

- `parse_args(env::args_os())` 把真实参数交给库函数。失败说明用户用法不对，退出码用 2。
- `run(&config)` 只借用配置，不夺取所有权，因为 `main` 后面不再需要它，但借用仍然是更清晰的默认选择。
- `and_then` 在 `run` 成功时继续执行闭包；`run` 失败时原样保留错误。它和 `?` 的区别是这里最后还要统一处理错误，而不是立刻返回。
- `io::stdout().lock()` 取得标准输出的锁（同一时间只允许一个写入者使用的标记）。多个线程同时写 stdout 时，锁能避免内容交错；单线程程序也可以直接写，但锁定后连续写更快，也更明确。
- `write_all` 保证把所有字节写完。`Write::write` 可能只写一部分，必须自己循环；`write_all` 替你处理了这件事。
- `flush()` 把缓冲区内容立刻交给操作系统，并让失败进入 `Result`。
- `map_err(AppError::Output)` 把 `io::Error` 包装成应用错误。这里用函数名作为闭包，是因为 `AppError::Output` 本身就是“接收 `io::Error` 返回 `AppError`”的函数。
- 成功返回 `ExitCode::SUCCESS`（0），运行失败返回 `ExitCode::FAILURE`（1）。

为什么不用 `println!("{output}")`？因为 `println!` 在写入失败时会 panic，程序无法选择退出码，也无法给出统一错误信息。对于要参与脚本和管道的工具，输出失败应当是可处理的错误。

### broken pipe

当下游程序提前关闭管道时，写 stdout 会失败，这通常叫 **broken pipe**。例如：

```bash
rstat big.txt | head -n 1
```

`head` 读一行就退出，`rstat` 再写就可能收到管道关闭的错误。把输出纳入 `Result`，就能决定是安静退出还是报告错误，而不是让程序崩溃。

## 十、完整示例：`firstline` 命令

下面是一个完整、可编译、可运行的小程序。它和主练习不同：只接受一个路径，不统计任何指标，只打印文件第一行。它把本课的参数、路径、读取、错误和退出码串了起来。

```rust
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

fn parse_path(mut args: impl Iterator<Item = std::ffi::OsString>) -> Result<PathBuf, String> {
    let _program = args.next();
    let path = args.next().ok_or_else(|| "missing input path".to_string())?;

    if args.next().is_some() {
        return Err("expected exactly one path".to_string());
    }

    Ok(PathBuf::from(path))
}

fn first_line(path: &Path) -> io::Result<Option<String>> {
    let text = fs::read_to_string(path)?;
    Ok(text.lines().next().map(|line| line.to_string()))
}

fn main() -> ExitCode {
    let path = match parse_path(env::args_os()) {
        Ok(path) => path,
        Err(message) => {
            eprintln!("error: {message}");
            return ExitCode::from(2);
        }
    };

    match first_line(&path) {
        Ok(Some(line)) => {
            println!("{line}");
            ExitCode::SUCCESS
        }
        Ok(None) => {
            eprintln!("error: {} is empty", path.display());
            ExitCode::FAILURE
        }
        Err(error) => {
            eprintln!("error: failed to read {}: {error}", path.display());
            ExitCode::FAILURE
        }
    }
}
```

数据流：

1. `env::args_os()` 产出程序名和用户参数。
2. `parse_path` 跳过程序名，取出唯一路径，拒绝多余参数，返回 `PathBuf`。
3. `first_line` 读取整个文件，`lines().next()` 取第一行；空文件得到 `None`。
4. `main` 根据三种结果选择 stdout、stderr 和退出码。

假设 `haiku.txt` 的内容是：

```text
古池や
蛙飛びこむ
水の音
```

把上面的代码保存为临时文件 `firstline.rs`，在同一个目录编译：

```bash
rustc --edition 2024 firstline.rs -o firstline
```

然后运行（Windows 上编译产物是 `firstline.exe`）：

```bash
./firstline haiku.txt
```

会输出：

```text
古池や
```

如果文件为空，退出码为 1，stderr 显示：

```text
error: haiku.txt is empty
```

如果路径不存在，退出码为 1，stderr 显示类似：

```text
error: failed to read haiku.txt: No such file or directory (os error 2)
```

这个示例没有放进本课 package，以免和主练习的入口混在一起。你可以把它复制到任意临时目录，用上面的 `rustc` 命令单独编译，再对照主练习的 `src/main.rs` 理解同样的分层。

## 十一、编译错误与排错

下面四条诊断来自真实的 `rustc` 运行结果，只省略了临时文件路径。刚开始看编译错误时，不要被大段文字吓到；先找 `error[编号]` 的第一行和 `expected` / `found`。

### 1. 把 `&str` 传给需要 `&Path` 的函数

```rust,compile_fail
use std::fs;
use std::io;
use std::path::Path;

fn load(path: &Path) -> io::Result<String> {
    fs::read_to_string(path)
}

fn main() {
    let _ = load("notes.txt");
}
```

诊断：

```text
error[E0308]: mismatched types
  --> src/main.rs:10:18
   |
10 |     let _ = load("notes.txt");
   |             ---- ^^^^^^^^^^^ expected `&Path`, found `&str`
   |             |
   |             arguments to this function are incorrect
   |
   = note: expected reference `&Path`
              found reference `&'static str`
```

`&str` 是一段文本的借用，`&Path` 是路径的借用；两者在类型层面不同。修复方向是用 `Path::new("notes.txt")` 创建路径视图，或者一开始就保存 `PathBuf`。不要为了通过编译把函数参数改成 `&str`，那会丢掉路径语义和非 UTF-8 支持。

### 2. `?` 找不到错误转换

```rust,compile_fail
use std::fmt;
use std::fs;

#[derive(Debug)]
enum AppError {
    Read(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Read(message) => write!(f, "{message}"),
        }
    }
}

fn load(path: &str) -> Result<String, AppError> {
    let text = fs::read_to_string(path)?;
    Ok(text)
}
```

诊断的关键部分：

```text
error[E0277]: `?` couldn't convert the error to `AppError`
  --> src/main.rs:18:40
   |
18 |     let text = fs::read_to_string(path)?;
   |                ------------------------^ the trait `From<std::io::Error>` is not implemented for `AppError`
   |
note: `AppError` needs to implement `From<std::io::Error>`
```

`read_to_string` 返回 `Result<_, io::Error>`，函数却声明返回 `Result<_, AppError>`。`?` 会尝试用 `From` 转换，但 `AppError` 没有实现 `From<io::Error>`。两种修复方向：

- 为 `AppError` 实现 `From<io::Error>`，适合错误不需要额外上下文的情况。
- 用 `map_err` 在错误发生处构造带路径的变体，适合需要保留“哪个文件失败”的情况。主练习属于后者。

### 3. 把 `Option<&str>` 当成 `&str`

```rust,compile_fail
use std::path::Path;

fn main() {
    let path = Path::new("notes.txt");
    let text: &str = path.to_str();
    println!("{text}");
}
```

诊断：

```text
error[E0308]: mismatched types
 --> src/main.rs:5:22
  |
5 |     let text: &str = path.to_str();
  |               ----   ^^^^^^^^^^^^^ expected `&str`, found `Option<&str>`
  |               |
  |               expected due to this
  |
  = note: expected reference `&str`
                  found enum `Option<&str>`
help: consider using `Option::expect` to unwrap the `Option<&str>` value, panicking if the value is an `Option::None`
```

`to_str()` 返回 `Option`，因为路径可能不是合法 UTF-8。编译器建议 `expect`，但那会让程序在非文本路径上崩溃。更合适的处理是 `match`、`ok_or`，或者只在显示时用 `display()` / `to_string_lossy()`。不要把“可能没有”硬拆成“一定有”。

### 4. 路径被移动后再次使用

```rust,compile_fail
use std::path::PathBuf;

fn consume(path: PathBuf) {
    println!("consumed {}", path.display());
}

fn main() {
    let path = PathBuf::from("notes.txt");
    consume(path);
    println!("still available: {}", path.display());
}
```

诊断的关键部分：

```text
error[E0382]: borrow of moved value: `path`
   |
 8 |     let path = PathBuf::from("notes.txt");
   |         ---- move occurs because `path` has type `PathBuf`, which does not implement the `Copy` trait
 9 |     consume(path);
   |             ---- value moved here
10 |     println!("still available: {}", path.display());
   |                                     ^^^^ value borrowed here after move
```

`PathBuf` 拥有数据，传给 `consume` 时所有权被移走，之后不能再用。修复方向：

- 函数只需要读路径时，参数改成 `&Path`，调用时传 `&path`。
- 确实需要保留两份所有权时，调用 `path.clone()`；但要先想清楚是否真的需要复制。

本课的函数大多接受 `&Config` 或 `&Path`，就是为了避免不必要的移动。

### 排错顺序

1. 从第一条 `error` 开始修，后面的错误常常是第一条引起的连锁反应。
2. 找 `expected` 和 `found`：它们直接告诉你“这里需要什么类型，实际给了什么类型”。
3. 读 `note` 和 `help`：编译器经常给出可用方向，但不一定是最适合业务的方向。
4. 一次只改一个小地方，改完重新编译，避免同时引入多个新问题。
5. 如果错误涉及所有权，先问“这个函数是只读，还是需要拥有数据？”，再决定用借用还是移动。

## 十二、代码预测与解析

先自己在纸上写出答案，再展开对照。

### 预测 1：行数

```rust
fn main() {
    println!("{}", "".lines().count());
    println!("{}", "a".lines().count());
    println!("{}", "a\n".lines().count());
    println!("{}", "a\n\n".lines().count());
    println!("{}", "a\r\nb\r\n".lines().count());
}
```

<details><summary>解析</summary>

输出依次是 `0`、`1`、`1`、`2`、`2`。

空文本没有行；没有换行的 `a` 也算一行；末尾换行不会额外产生空行；`a\n\n` 中间的空行算第二行；`\r\n` 是一个行尾，`\r` 不计入行内容。

</details>

### 预测 2：字节数与字符数

```rust
fn main() {
    let text = "héllo 世界";
    println!("{}", text.len());
    println!("{}", text.chars().count());
}
```

<details><summary>解析</summary>

输出 `13` 和 `8`。

`len()` 数的是 UTF-8 字节：`h`、`é`（2 字节）、`l`、`l`、`o`、空格、两个汉字（各 3 字节），合计 13。`chars()` 数的是 Unicode 标量值，共 8 个。

如果需求是“用户看到的字符数”，这个数字仍可能不等于字素簇数量；本课不处理那层细节。

</details>

### 预测 3：借用之后还能用吗

```rust
use std::path::{Path, PathBuf};

fn show(path: &Path) {
    println!("{}", path.display());
}

fn main() {
    let path = PathBuf::from("data.txt");
    show(&path);
    println!("{}", path.display());
}
```

<details><summary>解析</summary>

可以编译并输出两行。`show` 只借用 `PathBuf`，没有取得所有权；借用结束后，`path` 仍然有效。

如果 `show` 的参数是 `PathBuf` 而不是 `&Path`，第一次调用就会把值移动走，第二次使用会触发 E0382。

</details>

### 预测 4：忘记 flush 会怎样

```rust
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::Path;

fn write_report(path: &Path) -> io::Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    writeln!(writer, "done")?;
    Ok(())
}
```

<details><summary>解析</summary>

这个函数可能“看起来成功”，但缓冲区里的内容还没有保证写出。`BufWriter` 在销毁时会尝试刷新，但销毁不能把错误返回给调用者；如果写入失败，调用者已经拿到了 `Ok(())`。

修复方向是在返回前调用 `writer.flush()?`，让失败进入 `Result`。`flush` 也不等于物理落盘，但对“数据是否交给了操作系统”这件事，它比依赖析构更可靠。

</details>

## 十三、主练习：文件统计 CLI `rstat`

### 目标

完成 `src/exercise.rs` 中的四个函数，使 `rstat` 命令可以：

- 从命令行接收任意顺序的开关和一个路径；
- 读取任意文件（包括二进制文件，即内容不是文本、可以是任意字节的文件）；
- 按请求统计字节、词、行；
- 用固定格式输出；
- 用结构化错误报告参数、读取、解码和输出失败；
- 让 `main` 根据错误类型选择退出码。

四个待实现函数：

```rust,ignore
pub fn parse_args<I, S>(args: I) -> Result<Config, AppError>
where
    I: IntoIterator<Item = S>,
    S: Into<OsString>;

pub fn stats_from_bytes(data: &[u8], config: &Config) -> Result<Stats, AppError>;

pub fn format_stats(stats: &Stats) -> String;

pub fn run(config: &Config) -> Result<String, AppError>;
```

### 输入、输出与边界

假设文件 `input file.txt` 的内容如下，最后一行末尾**没有**换行：

```text
Rust rust

last line
```

它的原始长度是 20 字节：第一行 9 字节，两个换行各 1 字节，最后一行 9 字节。

默认命令：

```bash
cargo run -p l18_files_cli -- "input file.txt"
```

stdout：

```text
bytes: 20
words: 4
lines: 3
```

只统计非空行：

```bash
cargo run -p l18_files_cli -- --lines --non-empty "input file.txt"
```

stdout：

```text
lines: 2
```

只写 `--non-empty`、不写指标开关时，仍然默认请求三项：

```bash
cargo run -p l18_files_cli -- --non-empty "input file.txt"
```

stdout：

```text
bytes: 20
words: 4
lines: 2
```

组合 `--words --bytes` 时，字段顺序仍然固定为 bytes、words、lines，与命令行顺序无关：

```text
bytes: 20
words: 4
```

空文件的默认输出：

```text
bytes: 0
words: 0
lines: 0
```

二进制文件 `binary.dat` 的内容是两个字节 `[0xff, 0x00]`。只请求字节：

```bash
cargo run -p l18_files_cli -- --bytes binary.dat
```

stdout：

```text
bytes: 2
```

如果对同一个文件使用默认请求（包含词和行），因为内容不是合法 UTF-8，程序应当返回 `InvalidUtf8`，错误写 stderr，退出码为 1，并且**不能**先输出一份看起来成功的部分统计。

如果路径不存在，`run` 返回 `AppError::Read`，stderr 类似：

```text
error: failed to read missing.txt: No such file or directory (os error 2)
```

退出码为 1。参数错误（缺少路径、未知选项、多个路径）退出码为 2。

### 参数行为

- 第一个参数是程序名，必须跳过；之后是任意顺序的开关和唯一路径。
- `--bytes`、`--words`、`--lines`、`--non-empty` 是四个仅有的长选项。
- 重复开关不改变结果。
- 三个指标开关一个都没出现时，默认全部开启；`--non-empty` 不算指标开关。
- 只要出现任意指标开关，就只请求出现过的指标。
- `--non-empty` 把行统计改为 `LineMode::NonEmpty`；它不影响 bytes 或 words。
- 未知的 `--` 开头参数返回 `AppError::Usage("unknown option: <参数>")`。
- 没有路径返回 `AppError::Usage("missing input path")`。
- 第二个路径返回 `AppError::Usage("multiple input paths")`。
- 路径保留为 `PathBuf`，不要求路径文本合法 UTF-8。

### 统计行为

- bytes 是原始字节数，不是字符数。
- words 按 Unicode 空白切分，连续空白只起一次分隔作用。
- lines 为 `None` 时不统计。
- `LineMode::All` 统计所有行，包括空行和只有空白的行。
- `LineMode::NonEmpty` 只统计 `trim()` 后非空的行；`trim()` 去掉两端空白。
- 空输入是 0 行；末尾换行不额外产生一行；`\r\n` 是一个行尾。
- 只有请求 words 或 lines 时才需要合法 UTF-8；bytes-only 可以处理任意二进制。
- 解码失败返回 `AppError::InvalidUtf8`，保留 `config.path` 和原始 `Utf8Error`。

### 输出行为

- 字段顺序固定为 `bytes`、`words`、`lines`。
- 每个被请求的字段输出一行：`字段名: 数值\n`。
- 未请求的字段整行跳过。
- 三个字段都没有时返回空字符串。
- `format_stats` 只生成字符串，不写 stdout。

### 脚手架与约束

- package：`l18_files_cli`，`edition = "2024"`，`publish = false`。
- 主要文件：`src/exercise.rs`（主练习）、`src/consolidation.rs`（课后巩固）、`src/main.rs`（已写好，不需要改）。
- 只替换四个函数体里的 `todo!()`；不要修改类型定义、函数签名和底部测试。
- 不使用第三方 crate。
- 不手工用 `/` 或 `\` 拼接路径。
- 不用 `unwrap`、`expect` 或 `panic!` 处理用户输入和 I/O 失败。
- 不通过错误显示文字判断 `io::Error` 类型；需要分类时使用 `ErrorKind`。
- 文件读取和统计逻辑不写 stdout/stderr；输出和退出码只在 `main`。
- 测试不依赖当前工作目录中的固定文件；`TestDir` 会为每个测试创建独立临时目录并在结束时删除。

### 推荐完成顺序

1. `parse_args`：先处理“跳过程序名、收集开关和路径、应用默认值”，再补错误分支。
2. `stats_from_bytes`：先完成 bytes，再决定是否需要文本；分别验证空输入、Unicode 空白、行尾和非法 UTF-8。
3. `format_stats`：先写一个字段，再处理 `None` 和空结果。
4. `run`：读取失败时附加路径上下文，然后组合统计和格式化。
5. 运行全部测试，再用真实命令观察 stdout、stderr 和退出码。

### 测试说明与验收命令

本课在 Unix 类系统上一共有 27 个自动测试（其中 1 个非 UTF-8 路径测试只在 Unix 上编译，Windows 上为 26 个）：

- 2 个概念测试验证标准库行为，初始状态就通过；
- 17 个主练习行为测试；
- 8 个课后巩固测试。

初始状态下，25 个行为测试（Windows 上 24 个）都只失败在 `todo!()` 处，这是预期现象。编译、格式和严格 Clippy 应当立即通过。命令都在仓库根目录执行：

```bash
cargo fmt -p l18_files_cli -- --check
cargo check -p l18_files_cli --all-targets
cargo test -p l18_files_cli --no-run
cargo clippy -p l18_files_cli --all-targets -- -D warnings
```

只跑主练习测试：

```bash
cargo test -p l18_files_cli exercise
```

只跑课后巩固测试：

```bash
cargo test -p l18_files_cli consolidation
```

实现完成后，再手动验证三种真实场景：

```bash
# 1. 默认统计（先准备一个 input file.txt）
cargo run -p l18_files_cli -- "input file.txt"

# 2. 只统计二进制文件的字节
cargo run -p l18_files_cli -- --bytes binary.dat

# 3. 参数错误：观察 stderr 和退出码
cargo run -p l18_files_cli -- --unknown input.txt
```

第三条命令应返回退出码 2。Linux/macOS 可以用 `echo $?` 查看；Windows PowerShell 用 `$LASTEXITCODE`。

卡住时按顺序阅读 `HINTS.md` 的三级提示。三级提示也不给完整答案。

## 十四、课后巩固：整理文本文件

主练习只读文件；这个任务要读入、转换、再写出，数据不同，边界也不同。

### 目标

完成 `src/consolidation.rs` 中的两个函数：

```rust,ignore
pub fn normalize_text(text: &str) -> String;

pub fn tidy_file(input: &Path, output: &Path) -> Result<TidyReport, TidyError>;
```

### 真实输入与期望输出

输入文件 `raw.txt` 的内容如下（这里用 Rust 字符串字面量表示，`\r\n` 和 `\t` 是实际的控制字符；行尾有空格和制表符，中间有空行）：

```text
"  first  \r\n\r\n\tsecond\t\n   \nthird"
```

按 `normalize_text` 的规则：

- `"  first  "` 去掉末尾空白后是 `"  first"`，保留行首两个空格；
- 空行丢弃；
- `"\tsecond\t"` 去掉末尾制表符后是 `"\tsecond"`；
- `"   "` 去掉空白后为空，丢弃；
- `"third"` 保留。

输出文件 `clean.txt` 应为：

```text
  first
	second
third
```

也就是字符串 `"  first\n\tsecond\nthird\n"`。`TidyReport` 为：

```rust,ignore
TidyReport {
    written_lines: 3,
    removed_lines: 2,
}
```

边界：

- 输入为空文件：输出文件也为空，报告 `0` 行写出、`0` 行丢弃。
- 输入全是空白行：输出为空文件，所有行都计入丢弃。
- `\r\n` 统一转换成 `\n`。
- 输出文件已存在时会被完整覆盖。

错误：

- 输入不存在：`TidyError::Read`，保留输入路径和 `ErrorKind::NotFound`。
- 输入不是合法 UTF-8：`TidyError::InvalidUtf8`，保留输入路径和 `Utf8Error`。
- 输出路径不可写（例如父目录不存在、目标本身是目录）：`TidyError::Write`，保留输出路径。

### 验收命令

```bash
cargo test -p l18_files_cli consolidation
```

8 个测试覆盖正常转换、空输入、全空白输入、CRLF、读取错误、UTF-8 错误和写出错误。初始状态全部失败在 `todo!()` 处。

提示同样在 `HINTS.md` 的“课后巩固”部分。

## 十五、常见误区

### 1. 用 `env::args()` 读取路径

`env::args()` 在遇到非 UTF-8 参数时会 panic。路径应当走 `args_os()`，保存为 `PathBuf`。只有选项名、子命令名这类必须比较的文本，才转成 `&str`。

### 2. 用 `to_string_lossy()` 的结果重新打开文件

`to_string_lossy()` 会把非法字节替换成替换字符，得到的文本已经不能指回原文件。显示可以用它，打开文件必须继续用原来的 `Path` / `PathBuf`。

### 3. 把字节数当成字符数

`"世界".len()` 是 6，不是 2。需要字符数时用 `chars().count()`；需要用户可见字素数时还要更复杂的库，本课不涉及。

### 4. 认为末尾换行会多出一行

`"a\n".lines().count()` 是 1。行是由分隔符隔开的区间，末尾分隔符只结束最后一行，不产生新的空行。只有连续两个换行才会产生一个中间空行。

### 5. 所有错误都用 `unwrap`

`unwrap` 在失败时 panic，退出码和错误信息都不受控制。用户输入错误和环境失败应当返回 `Result`，在最外层统一打印和选择退出码。`unwrap` 更适合测试和“程序不变量被破坏”的情况。

### 6. 把错误写到 stdout

正常结果写 stdout，诊断写 stderr。混在一起会让 `> result.txt` 保存到错误文字，也会让管道下游收到无法解析的内容。

### 7. 用错误字符串判断错误类型

错误文字可能随平台、语言和版本变化。需要分类时检查 `ErrorKind`；需要保留原始信息时保存 `io::Error` 本身。

### 8. 忘记 `flush`

`BufWriter` 的析构会尝试刷新，但无法把失败返回。重要输出应显式 `flush()?`，让错误进入 `Result`。

### 9. 手工拼接路径分隔符

`format!("{}/{}", dir, name)` 在 Windows 上可能出错，也无法正确处理根路径、盘符和结尾分隔符。用 `Path::join` 或 `PathBuf::push`。

### 10. 把 `main` 写成全部逻辑

`main` 应当只连接进程边界。解析、统计、格式化放在库里，测试才能直接调用，错误也更容易分类。

## 十六、本课总结

- 命令行参数由操作系统在启动进程时传入；`env::args_os()` 产出 `OsString`，不要求参数是合法 UTF-8。
- 路径用 `Path` / `PathBuf` 表示，不能用普通字符串替代；拼接用 `join` / `push`，显示用 `display()`，重新打开文件继续用路径类型。
- 参数解析应当是纯函数：接收可注入的迭代器，返回 `Result<Config, AppError>`，不读全局状态、不打印、不退出进程。
- 文件可以先整体读入（`fs::read` / `fs::read_to_string`），也可以缓冲流式读取（`BufReader` + `lines`）；选择取决于文件大小和是否需要逐行处理。
- 字节、字符、词、行是不同口径。UTF-8 让一个字符占多个字节，非法字节序列不能解码成文本。
- 应用错误用枚举分类，保留路径和原始错误；`Display` 负责用户文字，`Error::source` 保留错误链，`?` 通过 `From` 或显式 `map_err` 转换。
- `main` 是边界层：正常结果写 stdout，诊断写 stderr，成功返回 0，运行失败返回 1，用法错误返回 2。
- 输出也要纳入 `Result`；`write_all` 处理部分写入，`flush` 让缓冲失败可见，broken pipe 是可处理的输出失败。

一个长期有用的模型是“三层边界”：

```text
纯逻辑（可测试）  →  带 I/O 的应用逻辑（返回结构化错误）  →  进程边界（main）
```

每逢写 CLI，先问自己：哪些部分可以做成纯函数？哪些失败需要保留上下文？最外层应当如何报告？

## 十七、课后复盘

用自己的话回答，不需要写代码：

1. 为什么路径参数用 `OsString` / `PathBuf`，而选项名可以转成 `&str`？
2. 为什么 `parse_args` 接受迭代器，而不是直接读 `env::args_os()`？
3. bytes-only 模式为什么可以接受非法 UTF-8，而 words/lines 不行？
4. `AppError::Read` 为什么要同时保存路径和原始 `io::Error`？
5. `format_stats` 为什么不直接打印，而是返回 `String`？
6. 什么时候用 `fs::read`，什么时候用 `BufReader`？
7. 为什么 `main` 要把 stdout 写入也放进 `Result`？
8. 退出码 0、1、2 在本课分别代表什么？

## 下一课

本课的文件操作是同步的：调用 `fs::read` 时，当前线程会一直等待操作系统返回。对于少量文件这没有问题；当一个程序要同时等待很多网络请求或文件操作时，线程会大量阻塞在等待上。

下一课[第十九课：`async`/`.await`、`Future` 与运行时](../19-async/README.md)会讨论如何把“等待”变成可调度的任务，让一个线程在等待期间推进其他工作，并解释为什么 `async` 不是“自动创建线程”的同义词。`Future` 是“将来才会得到的结果”的抽象，运行时是负责调度和推进这些等待任务的支撑代码；当前只需要知道它们与同步 I/O 的区别。
