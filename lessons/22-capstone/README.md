# 第二十二课：综合项目——从零写一个命令行数据工具

## 本课定位

这是主线课程的最后一课。前面二十一课分别讲过如何保存会变化的值（变量）、把几个值组合成新类型（结构体）、表达互斥状态（枚举）、决定谁负责释放数据（所有权）、临时访问别人的数据（借用）、表示可能为空的 `Option` 和可能失败的 `Result`、按顺序存放多个值（集合）并逐项处理它们（迭代器）、让同一份代码适用于多种类型（泛型）、定义可共享行为（trait，行为接口）、描述引用关系（生命周期）、带额外管理行为的指针（智能指针）、同一程序内并行执行的执行流（线程）、文件读写、命令行参数和测试；本课不再引入一整套新语法，而是把这些知识放进一个能真正运行的小程序里。

你要完成的项目叫 `datakit`：一个读取文本记录、保存到本地文件、按条件筛选、做统计并输出表格的命令行工具。它的规模不大，但包含了真实项目必须面对的问题：

- 用户输入可能写错，程序不能崩溃，要指出哪一行错了；
- 写文件可能中途失败，不能把原数据毁掉；
- 同一份数据必须产生同样的输出，方便脚本比较；
- 查询结果应该借用原记录，而不是到处复制；
- 命令行参数要与业务逻辑分开，才能独立测试。

完成本课后，你不需要再按固定语法顺序学习，可以带着这套方法进入系统工具、后端服务、嵌入式、编译器或性能工程等方向。

## 前置知识

本课默认你读过并至少运行过以下内容。忘记时不必重学整课，回到对应课复习那一节即可。

- [第一课](../01-basics/README.md)：变量（保存值的名字）、函数（一段可以反复调用的代码）、`main`（程序开始执行的位置）、编译（把源码变成可执行程序）与运行。
- [第二课](../02-data-modeling/README.md)：结构体（把几个值组合成一个新类型）和枚举（一组互斥的取值）。
- [第三课](../03-ownership-borrowing/README.md)与[第十四课](../14-lifetimes/README.md)：所有权（谁负责释放数据）、借用（临时访问别人的数据）和生命周期（引用能活多久的约束）。
- [第四课](../04-option-result/README.md)：`Option`（可能没有值）和 `Result`（可能失败），以及 `?` 运算符。
- [第五课](../05-vec-slices/README.md)、[第六课](../06-hash-collections/README.md)、[第十课](../10-iterators/README.md)：`Vec`（可变长度列表）、切片（列表中的一段连续元素）、`HashMap`（按键快速查找但迭代顺序不固定的映射表）、`BTreeMap`（按键有序保存的映射表）和迭代器（按顺序逐项处理数据的方式）。
- [第七课](../07-modules-crates/README.md)：模块（组织代码的命名空间）、package（Cargo 管理的代码单元）、library（供别人调用的库）与 binary（可直接运行的程序）。
- [第八课](../08-testing-quality/README.md)：单元测试（测试单个函数或模块）、集成测试（从外部调用公开接口的测试）、`cargo test` 和 Clippy（Rust 的静态检查工具）。
- [第十八课](../18-files-cli/README.md)：文件读写、命令行参数和退出码。

如果你还没有安装 Rust，先看仓库根目录 [README.md](../../README.md) 的“安装 Rust”一节。本课只使用标准库，不需要联网下载第三方 crate（Rust 的代码包）。

### 学习准备

本课所有命令都在课程仓库根目录执行，例如：

```bash
cargo test -p l22_capstone
```

`-p` 是 `--package` 的缩写，意思是“只操作名为 `l22_capstone` 的这个 package”。课程根目录是一个 Cargo workspace（工作区），里面有很多独立 package；指定名字可以避免编译其他课。

你还需要一个终端（输入命令的窗口）和一个文本编辑器。不会用终端也没关系，本课出现的命令都会解释用途。

## 一、先看要解决的问题

假设你在整理学习资料，想记录每条资料的编号、分类、数值、标签和备注。最初可以把它们写在一个文本文件里：

```text
1|book|89|rust;systems|The Rust Book
2|course|120|rust;video|Ownership course
3|book|-5|draft;rust|Correction
```

这比随手写在纸上好，但马上会遇到几个具体问题：

1. **想筛选时只能靠眼睛或 `grep`（命令行文本搜索工具）。** `grep rust` 会把备注里出现 `rust` 的行也找出来，也可能匹配到分类名，而不是只匹配标签。
2. **写错数字没人提醒。** 第三列本来应该是整数，写成 `oops` 也能存进文本文件，等到统计时才发现。
3. **重复编号没人拦。** 同一个 `id` 出现两次，后面的统计就会重复计算。
4. **排序和统计每次都要临时想办法。** 用 shell 拼命令，遇到负数、空标签、对齐格式时很容易出错。

我们需要一个小程序，把“格式是什么、什么算合法、输出长什么样”固定下来。这样的程序叫**命令行程序**（command-line program）：用户不点按钮，而是在终端里输入一行命令和若干参数来使用它。

几个术语先说清楚：

- **参数**（argument）：命令后面用空格分开的单词，例如 `--category book` 中的 `--category` 和 `book`。
- **选项**（option）：以 `--` 开头、用来调整行为的参数，例如 `--sort value`。
- **子命令**（subcommand）：紧跟在程序名后面的动作名，例如 `import`、`list`。一个程序可以有好几个子命令。
- **标准输出**（standard output，stdout）：程序正常结果的去向。终端默认把它显示出来，脚本也可以把它接走。
- **标准错误**（standard error，stderr）：错误信息的去向。它和标准输出分开，脚本可以只收集正常结果。
- **退出码**（exit code）：程序结束时交给操作系统的小整数。按惯例 `0` 表示成功，非 `0` 表示失败。
- **路径**（path）：文件在文件系统中的位置，例如 `data/records.db`。

`datakit` 的约定是：成功时退出码为 `0`；参数写错时退出码为 `2`；参数能读懂但读文件、验证数据或写文件失败时退出码为 `1`。分开这两种失败，脚本才能判断“是命令写错了”还是“数据有问题”。

## 二、先约定行为，再写代码

真实项目不能一边写一边猜需求。先把数据格式和命令协议写清楚，后面的模块才有共同目标。

### 2.1 一条记录的格式

每条记录占一行，五个字段用竖线 `|` 分开：

```text
id|category|value|tags|note
```

例如：

```text
1|book|89|rust;systems|The Rust Book
```

| 字段 | 含义 | 规则 |
|---|---|---|
| `id` | 编号 | 非负整数，使用 `u64`；同一个 store 中不能重复 |
| `category` | 分类 | 去掉两端空白后不能为空；不转小写 |
| `value` | 数值 | 64 位有符号整数 `i64`，允许负数和 `0` |
| `tags` | 标签 | 多个标签用 `;` 分开；每个标签去空白、转小写、去重（重复标签只保留一个）、按字典序（像查字典一样逐字符比较顺序）排序 |
| `note` | 备注 | 保留用户原文，不自动去空白；可以为空 |

这里的**字段**（field）指一行里被分隔符分开的一段文本；**分隔符**（separator）就是用来分段的字符。`tags` 字段内部又用 `;` 分隔多个标签。

为了不让“一行一条记录”的约定被破坏，以下字符不允许出现在字段内部：

- `category`、`note`、`tag` 中不能有 `|`、换行 `\n`、回车 `\r`、制表符 `\t`；
- `tag` 中还不能有 `;`，否则无法区分它是分隔符还是标签内容。

**为什么不用 CSV？** CSV（comma-separated values，逗号分隔值）是很常见的表格文本格式，但它允许字段用引号包起来，引号里还能出现逗号和换行。正确解析 CSV 需要处理很多边界，通常应该交给成熟库。本课为了只用标准库，选择更窄的自定义格式：一行、固定五个字段、字段内禁止分隔符。它不是通用格式，但对本项目的演示足够，而且规则可以完全讲清楚。真实产品如果要做通用导入，应换成 `csv`、`serde_json` 这类库，而不是继续扩展手写解析器。

`store` 指程序保存数据的文件。默认文件名是当前目录下的 `datakit.db`，内容与导入格式相同。每行一条记录，空行会被忽略。

### 2.2 命令协议

```text
datakit [--store PATH] import <file> [--replace]
datakit [--store PATH] list [options]
datakit [--store PATH] stats [--group category]
datakit [--store PATH] check
```

- `--store PATH`：指定 store 文件。不写时使用 `datakit.db`。
- `import <file>`：把 `<file>` 中的记录加入 store。
  - 默认追加；任何 `id` 与现有 store 或本次输入重复，整个命令失败。
  - `--replace`：用输入完整替换 store，而不是逐条覆盖。
  - 必须先把输入完整读完、验证完，再写 store；不能导入一半后失败。
  - 成功输出 `imported N records; store has M records`，替换时输出 `imported N records; store replaced`。
- `list`：筛选并输出表格。选项：
  - `--category C`：分类精确匹配（查询值会去掉两端空白）；
  - `--tag T`：标签匹配（查询值去空白并转小写）；
  - `--min N` / `--max N`：数值下界/上界，包含边界；
  - `--contains TEXT`：在备注中查找子串（一段连续文本）；匹配区分大小写（大写和小写算不同字符）；
  - `--sort id|value|category`：排序主键，默认 `id`；
  - `--desc`：反转主键顺序；
  - `--limit N`：排序后最多输出 N 条。
  - 所有筛选条件按“并且”组合；`--min` 大于 `--max` 是参数错误。
- `stats`：输出全局统计 `count`、`sum`、`min`、`max`、`mean`。加 `--group category` 时按分类分组，组名升序。
- `check`：逐行检查 store 并输出 `ok: N records`；任何一行有问题都报告行号并失败。

`--limit` 在筛选和排序之后才生效。`--desc` 只反转主键的比较方向；主键相等时仍然按 `id` 从小到大。这两条顺序约定是输出稳定的关键，后面会反复用到。

## 三、完整示例：从空目录到结果

下面是一次真实运行。先准备 `demo.txt`：

编译出的程序名是 `datakit`。在课程仓库里可以先运行 `cargo build -p l22_capstone`，再执行 `./target/debug/datakit ...`；也可以用 `cargo run -p l22_capstone -- ...`，其中 `--` 后面的内容会传给程序。为了简洁，下面直接写 `datakit`。

```text
1|book|89|rust;systems|The Rust Book
2|course|120|rust;video|Ownership course
3|book|-5|draft;rust|Correction
```

导入：

```console
$ datakit import demo.txt
imported 3 records; store has 3 records
```

列出全部记录：

```console
$ datakit list
ID  CATEGORY  VALUE  TAGS          NOTE
1   book      89     rust,systems  The Rust Book
2   course    120    rust,video    Ownership course
3   book      -5     draft,rust    Correction
```

表头固定为五列。前四列按该列最宽的内容补齐，列之间两个空格；最后一列不补空格。`tags` 用逗号连接，方便阅读；存储文件里仍然用分号。

按标签筛选，并按数值从大到小：

```console
$ datakit list --tag rust --sort value --desc
ID  CATEGORY  VALUE  TAGS          NOTE
2   course    120    rust,video    Ownership course
1   book      89     rust,systems  The Rust Book
3   book      -5     draft,rust    Correction
```

三条记录都含 `rust` 标签，所以都保留；`120 > 89 > -5`。

全局统计：

```console
$ datakit stats
count: 3
sum: 204
min: -5
max: 120
mean: 68.00
```

`sum` 是 `89 + 120 + (-5) = 204`；平均值 `204 / 3 = 68`，固定显示两位小数。

按分类分组：

```console
$ datakit stats --group category
CATEGORY  COUNT  SUM  MIN  MAX  MEAN
book      2      84   -5   89   42.00
course    1      120  120  120  120.00
```

`book` 有两条：`89` 和 `-5`，和是 `84`，平均 `42.00`。分组按分类名字典序排列，不依赖哈希表（`HashMap`，按键快速查找但迭代顺序不固定）的随机顺序。

检查 store：

```console
$ datakit check
ok: 3 records
```

此时 `datakit.db` 的内容就是：

```text
1|book|89|rust;systems|The Rust Book
2|course|120|rust;video|Ownership course
3|book|-5|draft;rust|Correction
```

这个例子覆盖了正常路径。错误路径也会在“真实诊断”一节展示。

## 四、项目结构

本课的 package 在 `lessons/22-capstone/`：

```text
Cargo.toml
README.md
HINTS.md
src/
  lib.rs        库的根：声明模块
  main.rs       极薄的二进制入口
  model.rs      Record、文本格式、字段规范化
  query.rs      Query、SortKey、筛选与排序
  stats.rs      Summary、分组统计
  store.rs      读写 store、原子替换
  output.rs     表格和统计文本
  cli.rs        参数解析
  error.rs      应用错误与上下文
  app.rs        把命令连接到各模块
  sessions.rs   课后巩固的数据脚手架
tests/
  model.rs
  query.rs
  stats.rs
  store.rs
  output.rs
  cli.rs
  cli_e2e.rs
  sessions.rs
```

几个术语：

- **package**：一个 Cargo 管理单元，由 `Cargo.toml` 描述。本项目 package 名是 `l22_capstone`。
- **crate**：一次编译的单位。package 可以同时产生一个 library crate 和一个 binary crate。
- **library**（库）：可以被其他代码调用的部分，根文件是 `src/lib.rs`。
- **binary**（二进制程序）：可以直接运行的部分，根文件是 `src/main.rs`。
- **模块**（module）：用 `mod` 组织代码的命名空间，例如 `model`、`query`。

`main.rs` 里的 `main` 函数只有三行，它把操作系统参数交给 `app::main_entry`：

```rust
fn main() {
    std::process::exit(l22_capstone::app::main_entry(std::env::args().skip(1)));
}
```

`std::env::args()` 产生程序名和后面所有参数；`.skip(1)` 丢掉程序名，只留用户写的部分。`std::process::exit(code)` 用给定退出码结束进程。

业务代码放在库里，集成测试才能直接调用 `app::run`，不必每次都启动子进程。`lib.rs` 声明了所有模块：

```rust
pub mod app;
pub mod cli;
pub mod error;
pub mod model;
pub mod output;
pub mod query;
pub mod sessions;
pub mod stats;
pub mod store;
```

模块之间的依赖方向是“边界指向领域”：`cli`、`store`、`output` 使用 `model`；`app` 使用所有人；`model` 不依赖文件系统或命令行。这样领域规则可以单独测试。

### 为什么没有第三方依赖

本课 `Cargo.toml` 的 `[dependencies]` 是空的。原因不是“第三方库不好”，而是这个项目的格式足够窄：

- 一行一条记录，字段固定，没有引号和转义；
- 解析只需要 `split` 和 `parse`；
- 不需要日期、正则、序列化框架或参数解析框架。

引入依赖会带来版本、许可证、编译时间和安全更新成本。只有在它解决的实际问题大于成本时才引入。反过来说，如果需求变成“导入任意 CSV”或“输出 JSON（一种通用的文本数据格式）”，就应该使用成熟库，而不是继续手写。这是本课希望建立的判断，而不是“永远不用库”。

## 五、类型与所有权设计

### 5.1 一条记录

`model.rs` 中的核心类型是：

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Record {
    pub id: u64,
    pub category: String,
    pub value: i64,
    pub tags: Vec<String>,
    pub note: String,
}
```

`struct`（结构体）把几个值组合成一个有名字的整体。每个字段都有自己的类型：

- `u64`：无符号 64 位整数，只能表示 `0` 到 `18446744073709551615`，适合编号。
- `i64`：有符号 64 位整数，可以表示负数和正数，适合可正可负的数值。
- `String`：拥有自己内存的文本。`&str` 是“借用的一段文本”，不能比它借用的数据活得更久。记录需要在导入结束后继续存在，所以这里用拥有型 `String`，而不是借用导入文件的内容。
- `Vec<String>`：一串文本，长度可以变化。

`#[derive(...)]` 让编译器自动生成常用能力：`Debug` 用于 `{:?}` 打印，`Clone` 用于显式复制，`PartialEq`/`Eq` 用于相等比较。它们不会自动帮你验证数据，验证仍然要自己写。

### 5.2 规范化与验证

创建记录不能只是把五个值塞进结构体，因为用户可能给出空白分类、大小写混乱的标签或带分隔符的备注。规范化和验证集中在 `Record::try_new`：

```rust
pub fn try_new(
    id: u64,
    category: String,
    value: i64,
    tags: Vec<String>,
    note: String,
) -> Result<Self, ValidationError>
```

`Result<Self, ValidationError>` 表示“成功时得到一条记录，失败时得到一个说明原因的 `ValidationError`”。`ValidationError` 是枚举：

```rust
pub enum ValidationError {
    EmptyCategory,
    ForbiddenCharacter { field: &'static str, character: char },
}
```

枚举的每个变体（variant）是一种可能情况。`field` 保存出问题的字段名，`character` 保存具体字符。调用者可以用 `match` 区分“分类为空”和“备注里有竖线”，而不是只收到一句无法处理的字符串。

**规范化**（normalization）指把等价写法变成统一写法。本课的规则是：

- 分类去两端空白；
- 标签去空白、转小写、丢空项、去重、排序；
- 备注保持原样，只检查禁用字符。

规范化必须集中在创建记录的地方。如果导入时转小写、查询时又用另一套规则，同一条记录就会有两种行为。

### 5.3 文本与记录互相转换

`Record::parse_line` 把一行文本变成记录：

```rust
pub fn parse_line(line: &str) -> Result<Self, ParseRecordError>
```

`ParseRecordError` 区分：

```rust
pub enum ParseRecordError {
    FieldCount { found: usize },
    InvalidId { text: String },
    InvalidValue { text: String },
    Validation(ValidationError),
}
```

这样“字段数不对”“编号不是数字”“数值不是数字”“字段能读但业务规则不通过”不会混成同一种错误。错误里保存原始文本，用户才能看到自己到底写了什么。

`Record::to_line` 做反向转换，返回不带换行的 `String`。两个函数必须互相对应：`parse_line` 接受 `to_line` 写出的内容，`to_line` 写出的内容也能被 `parse_line` 读回。这个性质叫**往返一致**（round trip），测试会检查它。

### 5.4 查询为什么返回引用

查询函数的签名是：

```rust
pub fn execute<'a>(
    records: &'a [Record],
    query: &Query,
) -> Result<Vec<&'a Record>, QueryError>
```

`&[Record]` 是“借来的一串记录”，函数不会取得它们的所有权。返回值 `Vec<&'a Record>` 是一串引用，每个引用指向原切片中的一条记录。

`'a` 是**生命周期参数**（lifetime parameter）。它不延长任何值的生命，只是在告诉编译器：“返回的引用最多和输入的 `records` 活得一样久。”因为 `records` 的所有者在调用期间一直存在，返回引用是安全的，也避免了为每条查询结果复制字符串。

如果返回 `Vec<Record>`，每条结果都要克隆 `category`、`tags`、`note`，既慢又掩盖了“结果只是原数据的一个视图”这一事实。只有在确实需要独立拥有数据时才克隆。

### 5.5 错误如何向上传递

`Result<T, E>` 的 `?` 运算符表示：“如果是 `Ok`，取出值继续；如果是 `Err`，立刻把错误返回给调用者。”例如：

```rust
let records = store::load(store_path)?;
```

这行要求当前函数返回的 `Result` 能接收 `StoreError`。`app.rs` 里通过 `impl From<StoreError> for AppError` 做转换：`impl` 是“为某个类型实现某个 trait（行为接口）”的语法，`From` 表示“可以从另一种类型转换过来”。这样底层错误被包装成应用错误，同时保留路径和行号上下文。

不要把错误压成一个 `String` 再往上扔；那样调用者和用户都失去了结构化信息。也不要在库函数里调用 `process::exit`，否则测试和复用都无从谈起。

## 六、各模块要做什么

下面的函数是**学习者负责**的部分，源码里用 `todo!()` 标出。`todo!()` 是一个宏（macro，编译期展开的代码模板），运行到这里会 panic（程序因无法继续而中止），并打印 `not yet implemented`。在完成实现前，测试失败是预期行为；但代码必须能编译。

### 6.1 model：规范化和单行格式

需要完成三个函数：

- `Record::try_new`：按 5.2 的规则规范化并验证。
- `Record::parse_line`：按 `|` 切分，检查字段数，解析 `id` 和 `value`，把标签按 `;` 拆开，再交给 `try_new`。
- `Record::to_line`：按同样格式写回一行。

边界要特别注意：

- 空行不是合法记录：`"".split('|')` 得到一个字段，所以字段数是 1，应该报 `FieldCount { found: 1 }`；
- 空标签字段 `1|book|0||` 是合法的，表示没有标签、没有备注；
- `u64` 和 `i64` 的解析失败都要保留原始文本；
- 分类在 trim 后为空才算空，`"   "` 应报 `EmptyCategory`；
- 标签里的 `;` 是结构字符，不能出现在单个标签内部；
- 备注不 trim，`" hi "` 要原样保留。

### 6.2 query：筛选、排序、截断

`execute` 的顺序必须是：

1. 检查 `min <= max`，否则返回 `QueryError::InvalidRange`；
2. 依次应用分类、标签、下界、上界、备注子串五个筛选；
3. 按主键排序，主键相等时按 `id` 升序；
4. 最后应用 `limit`。

顺序错了就会产生可观察差异。例如先 `limit` 再排序，会得到“输入中的前 N 条被重新排序”，而不是“排序后的前 N 条”。

`SortKey` 是枚举：

```rust
pub enum SortKey {
    Id,
    Value,
    Category,
}
```

`--desc` 只反转主键，不反转 `id` 次级顺序。假设两条记录数值都是 89，`id` 分别是 4 和 1，那么无论升序还是降序，它们之间都按 `1`、`4` 排列。这样输出不依赖输入文件顺序。

排序比较可以用 `sort_by` 加一个返回 `Ordering` 的闭包（闭包是可以捕获周围变量的匿名函数）。`Ordering` 是“小于/等于/大于”的枚举。`then_with` 可以把主键比较和次级比较串起来。不要用 `sort_by_key` 后再手动交换，相等键的次序会变得难以推理。

### 6.3 stats：聚合

`Summary::summarize` 计算：

- `count: u64`：条数；
- `sum: i128`（128 位有符号整数，范围比 `i64` 大得多）：所有 `value` 的和。为什么不用 `i64`？两个接近 `i64::MAX` 的值相加会溢出；先把和算对，再考虑展示限制。
- `min`、`max`：`Option<i64>`，空集合时是 `None`，不是 `0`。`0` 是一个真实的数值，不能拿来冒充“没有值”。

`Summary::mean` 在 `count == 0` 时返回 `None`，否则返回 `sum as f64 / count as f64`。浮点数（带小数的近似数值表示）可能有精度损失，所以原始总和仍保存在 `sum` 里。

`group_by_category` 用 `BTreeMap<String, Summary>` 按分类聚合。`BTreeMap` 的键按顺序保存，迭代结果稳定；`HashMap` 的迭代顺序不保证，直接输出会让同一份数据产生不同文本。最后用 `into_iter().collect()` 转成 `Vec<(String, Summary)>`。

### 6.4 store：读取与原子替换

`load` 的规则：

- 文件不存在返回空 `Vec`，这样第一次导入不需要额外命令；
- 空行跳过，但行号仍按文件真实行数计算；
- 每个非空行用 `Record::parse_line` 解析，错误带路径和从 1 开始的行号；
- 读完检查 `id` 唯一，重复时报两条记录的行号；
- 不信任文件内容，因为用户可以手工编辑。

`save` 不能直接打开目标文件并清空后写入。假如写到一半失败，原 store 已经被破坏。正确流程是：

1. 在目标文件**同一个目录**创建临时文件（同目录才能用 rename 跨过同一个文件系统）；
2. 写入全部记录，每行末尾一个换行；
3. `flush` 把缓冲区内容交给操作系统，`sync_all` 请求把数据落到存储设备；
4. 关闭临时文件；
5. 用 `rename` 把临时文件替换成目标文件；
6. 任何一步失败都删除临时文件，并返回错误；原目标保持不变。

“原子替换”指其他读者要么看到旧文件，要么看到新文件，不会看到写了一半的文件。它**不等于**数据库事务，也不保证断电后一定持久；`sync_all` 只是尽量把数据交给操作系统和磁盘。这个区别要在文档里说清楚，不能夸大。

`temporary_path` 已经提供，它把进程 id 加进文件名，降低并行测试互相覆盖的概率。本实现假设同一个 store 不会被多个进程同时写；这是本课明确的范围限制。

### 6.5 output：稳定文本

`render_records` 接受 `&[&Record]`，返回一个 `String`，结尾带换行。规则：

- 表头固定：`ID`、`CATEGORY`、`VALUE`、`TAGS`、`NOTE`；
- 前四列按该列最宽的内容（包括表头）用空格补齐，列之间两个空格；
- 最后一列不补齐，避免行尾空格；
- 空标签显示 `-`，标签之间用 `,` 连接；
- 空备注显示 `-`；
- 空结果只输出表头。

宽度用 `chars().count()` 而不是 `len()`。`len()` 返回字节数，一个中文字符通常占 3 个字节，会让补齐计算错误。不过 `chars().count()` 也不是终端显示宽度，宽字符仍可能让列看起来不齐；这是本课接受的限制，真实终端 UI（用户界面）需要使用专门的宽度库。

输出函数返回字符串，不直接 `println!`，这样测试可以把结果写进 `Vec<u8>` 比较，也可以复用到文件输出。真实项目常把 writer 作为参数，写成 `fn render<W: Write>(writer: &mut W, ...)`；本课先返回 `String`，是为了让格式化逻辑更容易单独验证。

### 6.6 cli 与 app：已经提供的部分

`cli::parse_args` 已经实现。它负责：

- `--store` 只能是全局选项，且只能出现一次；
- 每个需要值的选项缺值时返回 `MissingOptionValue`；
- 数字用 `parse` 解析，失败返回 `InvalidOptionValue`；
- 同一个选项出现两次返回 `DuplicateOption`，避免“最后一次悄悄生效”；
- `--min > --max` 在解析结束时返回 `InvalidRange`。

`app` 已经实现命令连接。重点看 `import` 的顺序：

```text
读取现有 store
→ 读取并解析全部输入
→ 检查输入内部重复 id
→ 检查与现有 store 重复 id
→ 在内存中合并
→ 一次性 save
```

只有最后一步会修改文件。前面任何一步失败，store 都保持不变。这就是**全有或全无**（all-or-nothing）：失败不会留下半份导入结果。

`app::run` 只做三件事：解析参数、执行命令、把错误写到标准错误并返回退出码。它不把业务逻辑塞进 `main`，所以集成测试可以直接调用它并捕获输出。

## 七、真实诊断

这一节展示真实运行得到的输出。你的行号可能因为后续修改略有不同，但错误结构和阅读方法相同。

### 7.1 `todo!()` 的测试失败

在还没有实现 `try_new` 时运行：

```console
$ cargo test -p l22_capstone --test model try_new_trims_category_and_preserves_note
running 1 test
test try_new_trims_category_and_preserves_note ... FAILED

failures:

---- try_new_trims_category_and_preserves_note stdout ----

thread 'try_new_trims_category_and_preserves_note' panicked at lessons/22-capstone/src/model.rs:149:9:
not yet implemented
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

怎么读：

- `panicked at ...:149:9` 指出 panic 发生在 `src/model.rs` 第 149 行第 9 列；
- `not yet implemented` 是 `todo!()` 的标准消息；
- 测试名 `try_new_trims_category_and_preserves_note` 告诉你哪条契约还没有满足；
- 这不是编译错误。代码能编译，只是运行到未实现的位置停了。

如果测试在 `todo!()` 之外失败，例如断言（用表达式检查结果是否符合预期的语句）左边和右边不相等，那才是实现行为不对。先看测试名和断言差异，再回到函数文档核对规则。

### 7.2 不能返回临时值的引用

规范化标签时，很容易写出这样的函数：

```rust
fn normalize(tag: &str) -> &str {
    tag.trim().to_lowercase().as_str()
}
```

真实编译结果：

```text
error[E0515]: cannot return value referencing temporary value
 --> src/main.rs:2:5
  |
2 |     tag.trim().to_lowercase().as_str()
  |     -------------------------^^^^^^^^^
  |     |
  |     returns a value referencing data owned by the current function
  |     temporary value created here
```

`to_lowercase()` 会创建一个新的 `String`，它是函数内部的临时值，函数结束就被释放。`as_str()` 借用了这个临时值；如果把借用返回出去，调用者会拿到一个指向已释放内存的引用。Rust 拒绝编译，正是为了避免悬空引用。

解决办法不是加 `unsafe`（由程序员手动保证内存安全的关键字），而是让返回值拥有数据：返回 `String`。`Record::try_new` 接收并保存的就是 `String`，所以标签可以直接转成拥有型文本。

### 7.3 运行时的错误输出

导入一行坏数据：

```console
$ datakit import bad.txt
error: bad.txt: line 1: invalid value "oops": expected a 64-bit signed integer
$ echo $?
1
```

错误里有文件、行号、原始文本和期望类型。用户不需要打开源码猜哪里错了。

重复 id：

```console
$ datakit import dup.txt
error: duplicate id 1 (input line 1 and input line 2)
$ echo $?
1
```

参数组合错误：

```console
$ datakit list --min 5 --max 2
error: min (5) must not be greater than max (2)
usage: datakit [--store PATH] <import|list|stats|check> [options]
$ echo $?
2
```

store 被手工改坏：

```console
$ datakit --store broken.db check
error: broken.db: line 2: invalid value "oops": expected a 64-bit signed integer
$ echo $?
1
```

注意 `check` 不会因为一条坏记录 panic；它把错误作为 `Result` 返回，由 `app` 打印并设置退出码。

## 八、代码预测

先不要运行，写下你的答案，再对照后面的解释。预测能暴露“看起来懂了但规则没连起来”的地方。

**题 1**：调用

```rust
Record::try_new(7, "  Book  ".into(), -3, vec!["  Rust ".into(), "SYSTEMS".into()], "  keep  ".into())
```

得到的 `category`、`tags`、`note` 分别是什么？

**题 2**：`Record::parse_line("1|book|0|Rust;rust;|")` 的 `tags` 和 `note` 是什么？

**题 3**：记录 `id=2, value=5` 和 `id=1, value=5`，按 `value` 降序排列，顺序是什么？

**题 4**：空 store 的 `stats` 输出中，`min`、`max`、`mean` 是什么？

**题 5**：store 文件内容是 `"1|book|1||\n\n2|book|oops||\n"`，`check` 报告第几行？

参考结果：

1. `category = "Book"`（只去空白，不转小写），`tags = ["rust", "systems"]`（去空白、转小写、排序），`note = "  keep  "`（原样保留）。
2. `tags = ["rust"]`（空标签被丢弃，重复只留一个），`note = ""`。
3. `id=1` 在前，`id=2` 在后。降序只反转主键；主键相等时仍按 `id` 升序。
4. 分别是 `-`、`-`、`-`，因为空集合没有最小值、最大值和平均值，不能显示成 `0`。
5. 第 3 行。空行被跳过，但仍然占一个行号；错误行号按文件真实行数计算。

## 九、常见误区

1. **在查询里克隆整条记录。** 先用引用返回，确实需要独立数据时再克隆。克隆会掩盖所有权问题，也让结果不再反映原数据。
2. **用 `HashMap` 迭代顺序生成输出。** 同一份数据每次运行可能得到不同顺序。需要稳定顺序时用 `BTreeMap`，或收集后按明确的键排序。
3. **先 `limit` 再排序。** 这会把“输入中的前 N 条”拿去排序，而不是“排序后的前 N 条”。顺序必须按协议固定。
4. **排序只比较主键。** 主键相等时如果没有次级键，顺序会依赖输入顺序，输出不稳定。始终用 `id` 作为次级键。
5. **直接清空 store 再写。** 中途失败会毁掉原数据。先写临时文件，再 rename。
6. **把 `rename` 说成“绝对持久”。** 它主要保证读者不会看到半文件；断电、磁盘缓存和目录元数据是另一个层次的问题。
7. **在库代码里 `process::exit`。** 库函数应该返回 `Result`，由最外层决定退出码。否则测试无法捕获失败，调用者也无法复用。
8. **用 `String` 保存错误。** 调用者只能读文本，不能区分“文件不存在”和“格式错误”。用枚举和 `source` 保留结构。
9. **分类和标签用同一套大小写规则。** 本课分类只 trim、保留大小写；标签才转小写。规则不同是有意的，必须在创建记录时统一执行。
10. **用 `len()` 计算中文列宽。** `len()` 是字节数，不是字符数；至少用 `chars().count()`，并知道它仍不等于终端显示宽度。
11. **把备注当格式字符串或直接拼进 CSV/JSON。** 本课禁止分隔符进入字段，避免格式被破坏；真实序列化应交给库做转义。
12. **只检查 `is_err()`。** 错误原因错了也会通过。测试应比较具体的错误变体和必要字段。

## 十、主练习

### 10.1 目标

完成 `src/model.rs`、`src/query.rs`、`src/stats.rs`、`src/store.rs`、`src/output.rs` 中标有 `todo!()` 的函数，使 `datakit` 的四个子命令按协议工作。

不要修改测试来迁就实现；测试描述的是公开行为。也不要把某个测试的字面量直接写死成通用逻辑。

### 10.2 你负责的函数

| 文件 | 函数 | 目标 |
|---|---|---|
| `model.rs` | `Record::try_new` | 分类 trim、标签规范化、禁用字符检查 |
| `model.rs` | `Record::parse_line` | 五字段解析、数字错误、领域验证 |
| `model.rs` | `Record::to_line` | 与 `parse_line` 往返一致 |
| `query.rs` | `execute` | 范围检查、五个筛选、稳定排序、limit |
| `stats.rs` | `Summary::summarize` | count/sum/min/max，空集合语义 |
| `stats.rs` | `Summary::mean` | 空集合返回 `None` |
| `stats.rs` | `group_by_category` | 按分类聚合且顺序稳定 |
| `store.rs` | `load` | 缺失文件、空行、行号、重复 id |
| `store.rs` | `save` | 临时文件 + flush/sync + rename + 失败清理 |
| `output.rs` | `render_records` | 固定表头、对齐、空值显示 |

`cli.rs`、`app.rs`、`error.rs`、`main.rs` 已经提供，不需要改。你可以在不改变公开签名的前提下重构自己的实现。

### 10.3 输入与期望输出

用本课开头的 `demo.txt` 导入后：

- `datakit list` 必须输出第三节展示的三行表格，顺序为 `id` 1、2、3；
- `datakit list --tag rust --sort value --desc` 必须输出 `id` 2、1、3；
- `datakit stats` 必须是 `count: 3`、`sum: 204`、`min: -5`、`max: 120`、`mean: 68.00`；
- `datakit stats --group category` 必须先 `book` 后 `course`；
- `datakit check` 必须输出 `ok: 3 records`。

边界与错误示例：

- 空文件导入成功，输出 `imported 0 records; store has 0 records`，store 是空文件；
- 空 store 的 `stats` 中 `min`、`max`、`mean` 都是 `-`；
- 输入文件第 2 行数值写成 `oops`，命令退出码 `1`，stderr 包含 `line 2`，store 不被创建或修改；
- 输入内部出现重复 `id`，整个导入失败，不留下半份数据；
- 向已有 store 追加一个重复 `id`，失败且原 store 不变；
- `list --min 5 --max 2` 退出码 `2`，stderr 包含 `usage: datakit`；
- 手工破坏 store 第 2 行后，`check` 退出码 `1` 并报告第 2 行。

### 10.4 推荐完成顺序

1. **model**：先让 `try_new` 的规范化测试通过，再实现 `parse_line`/`to_line` 的往返测试。
2. **query**：先做单个筛选，再做组合筛选，最后处理排序、降序、limit 和 `min > max`。
3. **stats**：先做全局 `Summary`，再做 `mean`，最后做 `group_by_category`。
4. **store**：先做 `load` 的正常和错误行号，再做 `save` 的往返，最后做临时文件清理。
5. **output**：先输出固定表头，再计算列宽，最后处理空标签和空备注。
6. **端到端**：运行 `tests/cli_e2e.rs`，确认四个子命令串起来的行为。

每完成一组就运行对应测试，不要等全部写完再编译。这样错误范围小，诊断更容易读。

### 10.5 测试地图

测试已经全部提供，`cargo test -p l22_capstone --no-run` 应该只编译、不运行：

| 文件 | 测试数 | 覆盖内容 | 初始状态 |
|---|---:|---|---|
| `tests/model.rs` | 17 | 规范化、字段解析、往返、边界 id | 失败于 `todo!()` |
| `tests/query.rs` | 16 | 每种筛选、组合、边界、排序 tie-break（主键相等时的次级排序）、limit | 失败于 `todo!()` |
| `tests/stats.rs` | 9 | 空集、单条、负数、`i128` 大数、分组 | 失败于 `todo!()` |
| `tests/store.rs` | 11 | 缺失文件、空行、CRLF（Windows 使用的回车加换行行尾）、坏行、重复 id、保存清理 | 失败于 `todo!()` |
| `tests/output.rs` | 7 | 表头、对齐、空标签、空备注、统计文本 | 6 个失败于 `todo!()`；空分组只渲染表头的 1 个已通过 |
| `tests/cli.rs` | 22 | 参数解析、错误、重复选项、范围 | 一开始就通过（解析器已提供） |
| `tests/cli_e2e.rs` | 18 | 真实运行 binary、stdout/stderr/退出码、原子导入 | 多数先失败；1 个纯参数错误测试直接通过 |
| `tests/sessions.rs` | 15 | 课后巩固的时段数据 | 失败于 `todo!()` |

总数是 115 个测试。完成全部主练习后，`cli.rs` 和 `cli_e2e.rs` 应全部通过；`sessions.rs` 属于课后巩固，单独完成。

### 10.6 验收命令

在仓库根目录执行：

```bash
cargo fmt -p l22_capstone -- --check
cargo check -p l22_capstone --all-targets
cargo test -p l22_capstone --no-run
cargo test -p l22_capstone
cargo clippy -p l22_capstone --all-targets -- -D warnings
```

`--all-targets` 表示库、二进制、测试和示例都检查。`-D warnings` 把警告当成错误，避免“能跑但一堆警告”的代码。

`cargo test -p l22_capstone` 也会运行 `tests/sessions.rs`；完成第十一节的课后巩固之前，那 15 个测试会失败。只验收主练习时，可以用 `--test` 指定测试文件，例如 `cargo test -p l22_capstone --test model --test query`。

如果 `cargo fmt --check` 失败，运行 `cargo fmt -p l22_capstone` 自动格式化，再检查一次。格式化只改排版，不改行为。

### 10.7 卡住时

打开本课 [HINTS.md](HINTS.md)。每组任务有三层提示：

1. **一级**：只给思考方向，适合刚看完文档、想知道从哪里下手；
2. **二级**：把任务拆成几个小步骤，适合已经尝试但卡在中间；
3. **三级**：接近完整的结构，但不给可直接粘贴的最终答案。

先看一级，动手试；仍然卡住再看下一级。直接看三级会削弱练习价值，但比复制网上答案更有效。

## 十一、课后巩固

主练习的数据允许负数、有五个字段。巩固任务换一份数据，边界也不同，用来检查你是否真的理解了“先约定、再验证、稳定输出”，而不是记住了某个函数的写法。

### 11.1 时段记录

`src/sessions.rs` 已经提供脚手架。格式是三个字段：

```text
date|project|minutes
```

例如：

```text
2026-11-03|rust-from-zero|90
2026-11-03|datakit|45
2026-11-04|datakit|30
```

规则：

- `date` 去两端空白，不能为空；本课不检查日期格式，只把它当文本；
- `project` 去两端空白并转小写，不能为空；
- `minutes` 必须能解析成 `u32`，且必须在 `1..=1440`（包含 1 和 1440）之间；
- 同一天同一项目不能重复；第二次出现时报第一次出现的行号；
- 空行跳过，但仍占真实行号。

需要完成：

| 函数 | 目标 |
|---|---|
| `parse_sessions` | 解析整份文本，错误带行号 |
| `total_minutes_by_project` | 按项目汇总，项目名升序，总数用 `u64` |
| `busiest_day` | 找总分钟数最多的一天，平局取更早日期 |

与主练习不同的边界：

- 数值是 `u32` 而不是 `i64`，且不是“任意整数”，而是有上下界；
- 重复键是 `(date, project)` 组合，不是单个 id；
- 汇总结果用 `u64`，因为多条记录可以超过 `u32`；
- 平局规则是“日期字典序更小”，不是 `id`。

输入与期望输出示例：

```text
2026-11-03| Rust-From-Zero |90
2026-11-03|datakit|45
2026-11-04| Datakit |30
```

解析后是三条记录，项目名分别是 `rust-from-zero`、`datakit`、`datakit`。按项目汇总：

```text
datakit 75
rust-from-zero 90
```

最忙的一天是 `2026-11-03`，共 `135` 分钟。

错误示例：

- 第二行写成 `2026-11-03|datakit`：字段数是 2，报 `FieldCount { expected: 3, found: 2 }`，行号 2；
- `minutes = 0` 或 `1441`：报 `MinutesOutOfRange`；
- 同一天同一项目出现两次：报 `DuplicateProjectDay`，带第一次出现的行号；
- 空文本：得到空 `Vec`，不是错误。

验收：

```bash
cargo test -p l22_capstone --test sessions
```

这 15 个测试覆盖正常、边界和错误路径。完成后再运行一次完整 `cargo test -p l22_capstone`，确认没有破坏主练习。

## 十二、工程复盘问题

完成后，试着不看代码回答：

1. 每条记录的 `String` 由谁创建、谁释放？查询结果为什么不需要克隆？
2. `execute` 的 `'a` 约束了什么？如果把返回值改成 `Vec<Record>`，需要付出什么代价？
3. 哪些错误保留了路径、行号和底层 `source`？哪些错误被压扁了？
4. `save` 失败时，原 store 为什么仍然完整？临时文件在哪里被清理？
5. 输出为什么不依赖 `HashMap` 的迭代顺序？次级排序键解决了什么问题？
6. `cli` 和 `app` 的边界在哪里？为什么库代码不调用 `process::exit`？
7. 哪些 `clone` 是领域需要，哪些只是为了绕过借用检查？
8. 如果数据量变大，当前“每次加载全部记录”的方案在哪里会成为瓶颈？你会先测量什么，再决定是否引入索引、流式处理或数据库？
9. 如果要做 `--format json`，应该引入什么库？为什么不应该继续手写转义？
10. 如果两个进程同时写同一个 store，`temporary_path` 的进程 id 方案还够吗？需要什么级别的锁或数据库事务？

这些问题没有标准答案，但能说明你是否真正理解了项目的取舍。

## 本课总结

完成本课后，你应该能把这些模型串起来：

- **所有权**决定谁负责释放数据；**借用**提供受约束的临时访问；生命周期标注描述引用关系，不延长值的生命。
- **结构体和枚举**让数据和状态进入类型系统；`Result` 让失败成为可处理的返回值，而不是崩溃。
- **规范化集中在边界**：导入时统一规则，查询和输出只消费已经合法的数据。
- **稳定输出**来自明确的排序键、次级键和固定的格式规则，而不是碰巧的集合迭代顺序。
- **文件更新**要先验证、再写临时文件、最后 rename；失败不能破坏原数据。
- **测试**围绕可观察行为：正常输入、边界、具体错误，以及“失败后状态没有改变”。
- **复杂度要有理由**：本课不需要第三方库、并发（多个任务在时间上重叠执行）、async（异步：等待 I/O 时让出线程）或 unsafe；当需求真的变化时，再引入并解释成本。

`datakit` 不是玩具式的“把代码敲一遍”，而是一个缩小版的真实工具：它有格式协议、错误上下文、原子更新、稳定输出和端到端测试。把这些决策讲清楚，比多写几个花哨特性更重要。

## 下一步

主线课程到这里结束。接下来不必再按固定语法课推进，可以选一个方向深入：

- **系统工具**：文件同步、日志分析、进程监控；重点是错误处理、信号和跨平台文件语义。
- **后端服务**：HTTP（网页和接口通信常用的协议）、数据库、认证；重点是并发、超时、可观测性和数据一致性。
- **编译器/解释器**：词法、语法、类型检查；重点是递归数据结构、错误恢复和测试策略。
- **嵌入式**：`no_std`、内存限制、硬件接口；重点是所有权在资源管理上的价值。
- **性能工程**：基准测试、分配分析、缓存；重点是先测量再优化。

不管选哪个方向，保留本课的习惯：先写清楚契约，再写最小实现；用测试固定行为；对每一个依赖、并发和 `unsafe` 决策说明它解决的具体问题。
