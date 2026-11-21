# 第八课：测试、文档测试、Clippy 与可维护验证

前七课里，你每次改完代码都要做一件事：运行程序，用眼睛看输出对不对。这个办法在你的程序只有十几行时够用；一旦函数变多、输入变多，它就会失效——你不可能每次都把所有情况手工试一遍，而且改了一处之后，你也记不清哪些旧结论需要重新确认。

本课讲的是把这件事交给机器：**把“我认为应该发生什么”写成代码**，让 Cargo 一遍遍替你检查。这类代码叫**测试（test）**。

## 本课定位

### 你将学会

- 在 Rust 里写**单元测试**（unit test，跟着代码放在同一个 crate 里的小检查）和**集成测试**（integration test，假装自己是外部用户来调用你的库的检查）。
- 写**文档测试**（doc test，写在文档注释里、会被自动执行的例子）。
- 用 `assert!`、`assert_eq!`、`assert_ne!` 表达期望，并让失败消息足以定位问题。
- 精确检查 `Result` 的**错误变体**（error variant，枚举里的某一个失败类别），而不是只检查“失败了”。
- 用表格驱动（table-driven）的写法覆盖同一行为的多种输入，同时保留可定位的失败信息。
- 判断哪些东西该测、哪些不该写进测试，避免测试阻碍正常重构。
- 用 `cargo fmt`、`cargo clippy`、`cargo test` 组成一套**质量门禁**（quality gate，提交前必须通过的自动检查集合）。

### 前置知识

本课不要求你学过任何其他编程语言。下面这些内容前面课里讲过，这里用一两句话复习，需要细节时点链接。

- **模块**（module，crate 内部的命名空间和访问边界）、`mod`、`pub use`、私有与公开：见[第七课](../07-modules-crates/README.md)。复习：`mod playlist;` 把文件纳入模块树；不带 `pub` 的项只在当前模块及其后代可见；`pub use` 把内部名字重新导出到 crate 根。
- **crate**（Rust 编译器一次处理的源码单位）：见[第七课](../07-modules-crates/README.md)。复习：一个 library 是一个 crate，`tests/` 下每个文件又各自是一个独立 crate。
- `Result`、`Option`、`?`、自定义错误枚举：见[第四课](../04-option-result/README.md)。复习：`Result<T, E>` 表示“要么成功给出 `T`，要么失败给出 `E`”；`Option<T>` 表示“可能有值，也可能没有”。
- `Vec`、切片 `&[T]`、`&mut` 借用：见[第五课](../05-vec-slices/README.md)。复习：`Vec<T>` 拥有可变长的一串数据；`&[T]` 是对其中一段的只读视图，不拥有数据。
- 结构体、方法、`&self`：见[第二课](../02-data-modeling/README.md)。复习：`&self` 表示方法借数据来看看，不夺走所有权。
- `for` 循环、`if`、字符串 `&str` 与 `String`：见[第一课](../01-basics/README.md)和[第三课](../03-ownership-borrowing/README.md)。复习：`&str` 是借来看的文本，`String` 是自己拥有的文本，`to_owned()` 把借来的文本复制成自己的。

### 本课边界

- 本课会用到 `assert_eq!` 背后的 `Debug` 和 `PartialEq` 两个 **trait**（特征，描述“这个类型能做什么”的约定）。现在够用的解释是：`Debug` 让值能被打印成给程序员看的调试文本，`PartialEq` 让两个值能用 `==` 比较。`trait` 的完整规则在[第十二课](../12-traits/README.md)。
- 本课会用到 `Box<dyn std::error::Error>`，现在够用的解释是：一个“能装下任何标准错误类型的盒子”，用来让测试函数用 `?` 直接返回失败。动态分发的原理在第[十二课](../12-traits/README.md)和[第十五课](../15-smart-pointers/README.md)。
- 本课会看到 `for case in cases` 遍历数组，以及 `.iter()`、`.map()` 这类迭代器方法。现在够用的解释是：它们逐项处理集合。迭代器的完整内容在[第十课](../10-iterators/README.md)。
- 本课不写基准测试（benchmark，测性能的工具），不测并发、网络或文件系统；文件与命令行的测试在[第十八课](../18-files-cli/README.md)讨论。
- 本课不覆盖第三方测试框架。Rust 内置的测试工具已经够用，本课不为“更熟悉某个库”而引入依赖；本课 package 的 `Cargo.toml` 因此没有任何依赖。

## 如何学习本课

1. 先读第一到第三节，理解“测试就是可执行的期望”这句话。
2. 每段代码先自己预测结果，再往下读解释。本课有专门的“代码预测与解析”一节。
3. 读“真实诊断”一节时，不要只看修正后的代码；先看报错原文，练习判断它在说什么。
4. 打开 `src/playlist/` 和 `src/statistics.rs`，按主练习的推荐顺序实现 `todo!()`。
5. 做完主练习再做课后巩固。卡住时按顺序看 `HINTS.md` 的三级提示。

建议分两次进行：概念与诊断部分约 60 分钟，练习约 60 分钟。

## 一、为什么需要测试：手工验证会在哪里崩掉

### 1.1 一个具体场景

假设你写了一个歌单库，规则是：标题两端空白要去掉、标题不能为空、时长不能是 0、标题不能重复、歌单有容量上限。你在 `main` 里打印几次结果，看起来都对了。

现在产品经理问：“标题是三个空格算不算空？”“容量 0 的歌单会怎么样？”“往满了的歌单里加一个重复的歌名，应该报哪个错？”你要一次一次改 `main.rs`、重新运行、用眼睛比对。改到第五次时，你已经不记得第一版的结论还成不成立。

### 1.2 旧办法的三个具体困难

1. **结论不持久。** 眼睛看到的“对了”没有记录。下次改代码，没有任何东西提醒你重新确认。
2. **覆盖不全。** 手工试的永远是少数输入。真正出问题的是边界：0、最大值、空字符串、两端空白。
3. **失败不定位。** 输出里出现一行不对，你得自己回想是哪次改动造成的。

### 1.3 Rust 引入什么

Rust 语言和 Cargo 内置了测试工具，不需要额外安装或引入依赖：

- 你把期望写成函数，用 `#[test]` 标记；
- 运行 `cargo test`，Cargo 编译并执行它们，逐条报告通过还是失败；
- 断言失败时，测试框架会打印实际值和期望值，指出哪个测试、哪一行失败。

“测试”这个词从此有两个意思：**测试函数**（一段检查代码）和**测试运行**（`cargo test` 这个动作）。两者都靠同一套机制。

## 二、第一个测试：最小可运行结构

新建一个 library 项目时，Rust 的约定是：测试写在**被测代码同一个文件**里，放进一个用 `#[cfg(test)]` 标记的模块。

```rust
// src/lib.rs

pub fn double(value: i32) -> i32 {
    value * 2
}

#[cfg(test)]
mod tests {
    use super::double;

    #[test]
    fn double_of_two_is_four() {
        assert_eq!(double(2), 4);
    }
}
```

逐行解释：

- `#[cfg(test)]`：**条件编译属性**。`cfg` 是 configuration（编译配置）的缩写。意思是“只有配置里启用了 `test` 时才编译这个模块”。`cargo test` 会启用它，`cargo build` 不会——所以测试代码不会进入你发布给用户的程序。
- `mod tests { ... }`：一个普通模块，名字叫 `tests` 只是社区约定（用 `#[cfg(test)]` 标记的测试模块常这么命名）。
- `use super::double;`：`super` 指父模块，也就是 crate 根。这条导入让下面的代码能直接写 `double`。
- `#[test]`：**属性**（attribute，写在项上面的 `#[...]` 标记）。它告诉测试框架“这是一个测试函数”。
- `fn double_of_two_is_four()`：测试函数必须是**无参数**的。框架统一调用它们，所以不能有参数。
- `assert_eq!(double(2), 4)`：**断言宏**。`assert` 是“断言、坚称”的意思。它比较两个值，相等就什么都不做，不相等就 **panic**（panic 是 Rust 的“立即中止当前执行并报告错误”机制，见[第四课](../04-option-result/README.md)）。

### 2.1 为什么测试模块能访问私有函数

这一点在第七课出现过，这里再确认一次，因为它是选择测试层级的关键。

假设 `double` 前面没有 `pub`：

```rust
fn double(value: i32) -> i32 {
    value * 2
}
```

`tests` 模块仍然是 `double` 的**后代模块**。Rust 的隐私规则是“不带 `pub` 的项对当前模块及其后代可见”。所以测试能调用它，而 crate 外部的代码不能。

这条规则的价值是：你可以为内部实现细节写测试，而不必为了测试把它们改成 `pub`。公开什么应该由 API 设计决定，不能由“测试方便不方便”决定。

### 2.2 `cargo test` 到底跑了哪些东西

在一个既有 library 又有 binary 的 package 里，`cargo test` 会构建并运行四类东西：

| 类型 | 位置 | 编译成什么 | 能看到什么 |
|---|---|---|---|
| library 单元测试 | `src/**/*.rs` 里的 `#[cfg(test)]` | library crate 的测试版本 | crate 内一切，包括私有项 |
| binary 单元测试 | `src/main.rs` 或 `src/bin/*.rs` 里的 `#[cfg(test)]` | binary crate 的测试版本 | 该 binary 内一切 |
| 集成测试 | `tests/*.rs` | 每个文件一个**独立 crate** | 只有 library 的公共 API |
| 文档测试 | 源码里文档注释中的代码块 | 每个代码块一个小程序 | 只有公共 API |

注意第三、四行：集成测试和文档测试都是“外部用户视角”。这是本课反复使用的分层思路。

### 2.3 读懂 `cargo test` 的输出

在完成实现的项目上，运行一次带名字过滤器的 library 测试：

```bash
cargo test -p l08_testing_quality --lib trims
```

输出：

```text
running 2 tests
test playlist::queue::tests::add_trims_whitespace_on_both_sides_of_the_title ... ok
test playlist::queue::tests::find_trims_the_argument_but_stays_case_sensitive ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 34 filtered out
```

要点：

- 测试的完整名字是“模块路径 + 函数名”，例如 `playlist::queue::tests::add_trims_whitespace_on_both_sides_of_the_title`。
- `trims` 是名字过滤器：`cargo test <文本>` 只运行名字里含这段文本的测试。
- `filtered out` 表示被过滤器排除的测试数量。本课 library 一共 36 个测试，这里跑了 2 个，剩下 34 个被排除；不加过滤器时，36 个全部运行。
- 失败时，框架会把该测试的 panic 消息打印在 `---- 测试名 stdout ----` 下面。

## 三、断言：把期望写成机器能检查的句子

### 3.1 三个基本宏

```rust
assert!(value > 0);          // 条件为 true 才通过
assert_eq!(actual, expected); // 两个值相等才通过
assert_ne!(left, right);      // 两个值不相等才通过
```

`assert!` 只接受一个 `bool`。`assert_eq!` 和 `assert_ne!` 需要两个值同时满足两个 trait：

- `PartialEq`：能用 `==` 比较，否则宏没法判断相等；
- `Debug`：能用 `{:?}` 打印，否则失败时无法展示两边到底是什么。

如果你自己定义的类型没有 `#[derive(Debug, PartialEq)]`，`assert_eq!` 会直接编译失败。第十五节有一条真实报错。

### 3.2 为什么 `assert_eq!` 比手写 `if` 好

手写：

```rust
if double(2) != 4 {
    panic!("double(2) 不对");
}
```

失败时你只得到一句“不对”。`assert_eq!` 会打印：

```text
assertion `left == right` failed
  left: 5
 right: 4
```

左边是实际值，右边是期望值。这个区别在复杂测试里非常关键——你不需要再去加打印语句。

### 3.3 加自定义消息

当同一个测试里有多次断言，或者输入来自一张表，默认输出不够定位时，给断言加消息：

```rust
assert_eq!(actual, expected, "case {:?}，输入 {:?}", name, input);
```

消息参数和 `println!` 一样，支持 `{}` 和 `{:?}`。**失败消息应该带出输入**，否则你只知道“某处不相等”。

README 里的主练习测试就是这么写的：每个 case 有名字，失败时直接告诉你哪一行数据坏了。

## 四、测试 `Result`：不要只问“失败了吗”

前面的库大多返回 `Result`。这时最容易写出的弱测试是：

```rust
let result = playlist.add("晴天", 0);
assert!(result.is_err());
```

它能通过，但几乎什么都没证明：如果实现把“时长为 0”错报成了“歌单已满”，这个测试照样通过。用户看到的错误提示就错了。

### 4.1 直接比较整个 `Result`

如果错误类型实现了 `PartialEq`（本课的错误类型都派生了它），可以一步比较到底：

```rust
assert_eq!(playlist.add("晴天", 0), Err(PlaylistError::ZeroLength));
assert_eq!(playlist.add("晴天", 269), Ok(()));
```

`Err(PlaylistError::ZeroLength)` 是一个完整的 `Result` 值。这样写同时检查了三件事：失败了、失败原因是这个变体、没有附带意外的上下文。

### 4.2 变体带数据时怎么办

有的变体携带信息，例如 `DuplicateTitle(String)`。同样可以直接比较：

```rust
assert_eq!(
    playlist.add(" 晴天 ", 200),
    Err(PlaylistError::DuplicateTitle(String::from("晴天"))),
);
```

注意这里比较的是**去掉空白之后**的文本——测试同时在检查“错误里保存的上下文是否正确”。如果实现保存了带空白的 `" 晴天 "`，测试会失败，这是对的：调用者拿到错误后要展示给用户，上下文必须干净。

### 4.3 错误类型不能比较时：`match`

如果错误类型没有实现 `PartialEq`（例如它包着一个不能比较的东西），可以用 `match` 拆开检查：

```rust
match result {
    Err(PlaylistError::DuplicateTitle(title)) => assert_eq!(title, "晴天"),
    other => panic!("期望 DuplicateTitle，实际是 {other:?}"),
}
```

`other => panic!(...)` 这一支非常重要。只匹配你期望的那一种，其余情况会落进一个通配分支；如果你不处理它，`match` 就不完整（编译器会报错），而如果你写了 `_ => {}`，错误变体就会被悄悄接受。

### 4.4 还要检查“失败没有留下痕迹”

一个常见的实现缺陷是：先修改数据，再检查条件。比如先 `push` 再判断容量，满了再弹出来——中间状态可能已经被破坏。

所以错误测试除了检查错误类型，还应该检查状态没变：

```rust
let mut playlist = Playlist::new(2);
playlist.add("晴天", 269).unwrap();

assert_eq!(
    playlist.add("稻香", 0),
    Err(PlaylistError::ZeroLength)
);
assert_eq!(playlist.len(), 1, "失败不该改动歌单");
```

`unwrap()` 的意思是“如果是 `Ok` 就取出里面的值，如果是 `Err` 就 panic”。在测试的准备阶段用它很合适：准备步骤失败说明测试本身有问题，应该立刻暴露。

### 4.5 让测试函数自己返回 `Result`

测试函数可以返回 `Result`，这样就能用 `?`：

```rust
#[test]
fn longest_track_can_be_read_back() -> Result<(), Box<dyn std::error::Error>> {
    let mut playlist = Playlist::new(2);
    playlist.add("晴天", 269).unwrap();
    playlist.add("七里香", 299).unwrap();

    let longest = playlist.longest().ok_or("歌单为空")?;
    assert_eq!(longest.title(), "七里香");
    Ok(())
}
```

逐项解释：

- 返回类型 `Result<(), Box<dyn std::error::Error>>`：成功时返回 `Ok(())`（`()` 读作 **unit**，表示“没有值”），失败时返回任意标准错误。
- `?`（见[第四课](../04-option-result/README.md)）：遇到 `Err` 就立刻把错误返回给测试框架，测试被判为失败，并打印错误。
- `ok_or(...)`：把 `Option` 变成 `Result`；这里的错误值是 `&str`，`?` 会把它转换成 `Box<dyn Error>`。歌单里没有曲目时，测试就会以这个错误失败。
- `Box<dyn std::error::Error>`：`Box` 是把值放到堆上的容器，`dyn Error` 表示“任何实现了 `Error` trait 的类型”。细节在第[十二课](../12-traits/README.md)和[第十五课](../15-smart-pointers/README.md)；现在只需知道它是一个“什么错误都能装”的类型。

**重要限制**：一旦用 `?` 提前返回，你就失去了对错误的精细检查。如果这个测试的目的是“验证返回值确实是某个错误变体”，不要用 `?`，用第 4.1 节的直接比较。

## 五、表格驱动测试：一份逻辑，多行数据

当你要测的是“同一个函数的很多组输入”，把每组都写成一个 `#[test]` 会产生大量重复。这时用**表格驱动**（table-driven）：把输入和期望排成一张表，循环处理。

```rust
#[test]
fn median_cases() {
    let cases = [
        ("空", &[][..], None),
        ("一个值", &[12.5][..], Some(12.5)),
        ("多个值", &[20.0, 15.5, 21.0][..], Some(20.0)),
    ];

    for (name, values, expected) in cases {
        let actual = median(values);
        assert_eq!(actual, expected, "case {name:?}");
    }
}
```

几个细节：

- `&[][..]` 是把“空数组字面量”变成一个空的 `&[f64]` 切片。`[..]` 是切片索引语法，两端都省略表示整段。这样写是为了让三个元素的类型一致（都是 `&[f64]`）。
- `for (name, values, expected) in cases` 是**解构**：数组元素是三元组，循环变量一次拆成三个名字。
- `assert_eq!` 的第三个参数带出 `name`，失败时能立刻知道是哪一行。

这里的 `median` 是本课课后巩固里的函数（第十七节），现在只需看表格本身的结构。

### 5.1 表格的适用边界

表格适合“同一种断言、不同数据”。不适合这些情况：

- 每个 case 需要完全不同的准备步骤（比如一个要建歌单、一个要建文件）；
- 每个 case 检查的是不同性质（一个查错误、一个查排序、一个查总时长）。

硬塞进一张表的后果是：表里出现 `if case.name == "..."` 这种分支，可读性比分开写更差，而且失败时一个 case 中断，后面的 case 不会执行。

判断标准很简单：**表格里的每一行，断言的那句话应该是一样的。**

### 5.2 一个测试表达一个主题

一个测试可以有多个断言，只要它们说的是同一件事：

```rust
#[test]
fn add_stores_the_trimmed_title_and_seconds_in_order() {
    let playlist = sample_playlist();
    let titles: Vec<&str> = playlist
        .tracks()
        .iter()
        .map(|track| track.title())
        .collect();

    assert_eq!(titles, ["晴天", "七里香"]);
    assert_eq!(playlist.len(), 2);
    assert_eq!(playlist.tracks()[0].seconds(), 269);
}
```

这四条断言都在回答“添加之后歌单里是什么”。可以放在一起。

反过来，不要把“添加成功”“重复标题”“歌单已满”“总时长溢出”塞进一个测试。第一个断言失败后，后面的断言根本不会执行，你无法知道到底坏了几个地方。

### 5.3 测试命名

名字要写清“条件和结果”，让失败列表能直接当问题清单读：

```text
add_rejects_titles_that_are_empty_after_trimming
add_reports_full_with_the_capacity_when_no_room_is_left
longest_keeps_the_earliest_track_when_lengths_are_tied
```

对比 `test1`、`works`、`test_add`：它们出现在失败列表里等于没有信息。

## 六、测试辅助函数：共享准备，不共享状态

多个测试需要同样的初始数据时，写一个普通函数返回它：

```rust
fn sample_playlist() -> Playlist {
    let mut playlist = Playlist::new(4);
    playlist.add("晴天", 269).unwrap();
    playlist.add("七里香", 299).unwrap();
    playlist
}
```

几点说明：

- 它没有 `#[test]`，所以不会被当作测试执行。
- 每个测试调用一次，各拿到**自己的**歌单。测试之间没有共享状态，所以单独运行和整组运行结果一致。
- 在测试框架里，这种“固定的准备数据”常被称为 **fixture**（夹具，指测试开始前就位的环境或数据）。

不要写成全局静态变量再让测试互相修改。Rust 测试默认**并行**运行（多个测试同时跑），共享的可变状态会让结果依赖执行顺序，出现“单独跑通过、整组跑失败”的偶发问题。

## 七、单元测试与集成测试：两个不同的边界

### 7.1 单元测试

单元测试（unit test）跟代码放在一起，测的是“这一块内部逻辑对不对”。它的优势是能访问私有项，能直接测一个私有 helper 的边界。

### 7.2 集成测试

集成测试（integration test）放在 package 根目录的 `tests/` 下，每个 `.rs` 文件是一个**独立 crate**。它只能通过 library 的公共 API 使用代码，就像一位外部用户在写程序。

```rust
// tests/playlist_public_api.rs

use l08_testing_quality::{Playlist, PlaylistError};

#[test]
fn public_api_builds_a_playlist_and_reads_it_back() {
    let mut playlist = Playlist::new(2);

    playlist.add(" 晴天 ", 269).unwrap();

    assert_eq!(playlist.tracks()[0].title(), "晴天");
}
```

注意导入路径：`l08_testing_quality::Playlist`，直接从 crate 根写起。如果 `playlist` 模块对外是私有的，写 `l08_testing_quality::playlist::...` 会编译失败——这正是集成测试的价值：它替你验证“外部用户能不能真的用上这个 API”。

集成测试能发现这些问题：

- 公共函数其实没有 `pub`，或者被藏在私有模块里；
- 公共 API 需要用户先构造一个私有类型才能调用；
- 返回类型不可用（例如返回了一个外部无法命名的私有类型）。

### 7.3 怎么选

| 你关心的是 | 写到哪 |
|---|---|
| 一个私有 helper 的边界计算 | 该模块内的 `#[cfg(test)]` |
| 公开 API 的完整行为 | `tests/` 下的集成测试 |
| 文档里的示例能不能用 | 文档注释里的代码块 |
| 内部实现细节（会被重构掉的） | 尽量少测；测了就准备接受重构时的修改 |

一个常见误区是“为了测试把所有东西都改成 `pub`”。不要这样做。需要测私有项时，把测试写在那个模块里就行。

## 八、文档测试：让例子不会过期

Rust 的文档注释（`///` 或 `//!`）里的代码块，会被 `cargo test` 编译并执行。这叫**文档测试**（doc test）。

```rust
/// 计算一组测量值的平均数。
///
/// ```
/// use l08_testing_quality::statistics::mean;
///
/// assert_eq!(mean(&[2.0, 4.0, 6.0]), Some(4.0));
/// ```
pub fn mean(values: &[f64]) -> Option<f64> {
    // ...
}
```

它解决的具体问题是：文档里的例子，随着代码演进会变成谎言。用户复制过去发现跑不通，或者跑出来结果和文档写的不一样。文档测试把“例子正确”变成自动检查的一部分。

### 8.1 文档测试的运行环境

每个代码块被包装成一个独立的小程序，并以“外部用户”的身份链接你的 library。所以你必须在例子开头写 `use`：

```rust
/// ```
/// use my_crate::Endpoint;
///
/// let endpoint = Endpoint::parse("localhost:80")?;
/// assert_eq!(endpoint.port(), 80);
/// # Ok::<(), my_crate::ParseEndpointError>(())
/// ```
```

- 第一行的 `use` 不能省。文档测试不会自动把 crate 里的东西放进作用域。
- 最后一行以 `# ` 开头：`#` 后的内容在渲染出的文档里**隐藏**，但仍然参与编译。它用来说明“这个例子的外层其实是一个返回 `Result` 的函数”，让 `?` 合法。不要隐藏理解示例所必需的核心步骤。

### 8.2 四种代码块属性

| 写法 | 含义 |
|---|---|
| ``` ```rust ```（或空标记） | 编译并运行 |
| ``` ```rust,no_run ``` | 只编译，不运行 |
| ``` ```rust,compile_fail ``` | 期望**编译失败**，编译成功则测试失败 |
| ``` ```rust,ignore ``` | 完全跳过，最容易腐烂 |

`no_run` 在什么时候用？当例子必须有副作用（读写文件、监听端口），或者它调用的功能还没实现（本课脚手架里就是这样：`Playlist::add` 还是 `todo!()`，例子只验证“能编译”）。

`compile_fail` 适合说明“类型系统禁止的用法”。本课用它展示私有字段不能被外部构造：

```rust
/// ```compile_fail
/// use l08_testing_quality::Track;
///
/// let track = Track { title: String::from("晴天"), seconds: 269 };
/// ```
```

一个陷阱：`compile_fail` 只要求“编译不过”。如果例子因为拼写错误而编译不过，它照样“通过”。所以例子要尽量短，只保留你要证明的那一件事。

## 九、`#[should_panic]` 与 panic 的边界

有些行为本身就是 panic，比如数组越界。这时用 `#[should_panic]`：

```rust
#[test]
#[should_panic(expected = "index out of bounds")]
fn indexing_beyond_the_end_panics() {
    let values = vec![1, 2];
    let _ = values[10];
}
```

`expected = "..."` 是消息子串匹配：panic 的文本必须包含这段文字。

### 9.1 `#[should_panic]` 为什么容易“通过但没测到”

看这个测试：

```rust
#[test]
#[should_panic]
fn setup_panics_before_the_target_assertion() {
    let values = vec![1];
    let _ = values[10];      // 这里先 panic
    assert_eq!(2 + 2, 5);    // 永远执行不到
}
```

运行结果是：

```text
test tests::setup_panics_before_the_target_assertion - should panic ... ok
```

它“通过”了，但它完全没有检查最后那行 `assert_eq!(2 + 2, 5)`。`should_panic` 只证明“发生了某个 panic”，不证明“发生了你以为的那个 panic”。

### 9.2 什么时候该用 panic，什么时候该用 `Result`

| 场景 | 合适的设计 |
|---|---|
| 用户输入的文本格式不对 | 返回 `Err` |
| 配置文件缺失、网络请求失败 | 返回 `Err` |
| 调用者传了 0 作为除数（明确违反契约） | 可以 panic |
| 内部不可能状态（理论上不会发生） | 可以 panic |
| 标准库的索引越界 | 固有 panic |

判断原则：**失败是调用者可以合理处理的，就用 `Result`；失败意味着程序有 bug，就用 panic。**

本课的歌单库全部用 `Result`：空标题、零时长、重复、已满，都是调用者能理解并展示给用户的失败，不应该 panic。所以本课的测试里不出现 `#[should_panic]`——这不是遗漏，是设计选择。

如果确实需要检查 panic 的具体内容，可以用 `std::panic::catch_unwind` 捕获，但它复杂、且不适合作为普通输入验证手段。本课不展开。

## 十、浮点数、容差与无序集合

### 10.1 浮点数不能随便 `assert_eq!`

十进制小数在二进制里通常存不下精确值：

```rust
let actual = 0.1_f64 + 0.2;
let expected = 0.3_f64;
println!("{actual:?}");
```

输出是 `0.30000000000000004`，不是 `0.3`。所以这类计算要用**容差**（tolerance，允许的误差范围）比较：

```rust
assert!((actual - expected).abs() < 1e-12);
```

- `.abs()` 取绝对值；
- `1e-12` 是科学计数法，等于 $10^{-12}$，也就是 0.000000000001。

容差不是越小越好，也不是随手写个大数。它应该来自“这个领域允许多大误差”。对“几个小数相加”这种量级，`1e-12` 足够。

本课课后巩固的 `mean` 和 `median` 都会踩到这一点。

### 10.2 也可以用近似比较的辅助函数

```rust
fn assert_option_close(actual: Option<f64>, expected: Option<f64>, case: &str) {
    match (actual, expected) {
        (None, None) => {}
        (Some(actual), Some(expected)) => {
            assert!(
                (actual - expected).abs() < TOLERANCE,
                "case {case:?}：期望 {expected}，实际 {actual}"
            );
        }
        (actual, expected) => panic!("case {case:?}：期望 {expected:?}，实际 {actual:?}"),
    }
}
```

这个辅助函数把“`Option` 的存在性”和“`f64` 的容差”两件事一起处理了。注意最后一支：`Some` 对 `None` 必须失败，不能悄悄通过。

### 10.3 无序集合

`HashMap` 和 `HashSet`（见[第六课](../06-hash-collections/README.md)）的迭代顺序不保证。测试时要么直接比较集合本身（`HashMap` 实现了 `PartialEq`，比较的是“键值对是否相同”，与顺序无关），要么先排序再比较。

不要写“断言第一个元素是 X”——它依赖内部哈希顺序，换编译器版本就可能失败。

## 十一、测试隔离：不要让测试互相影响

Rust 的测试默认**并行**执行。这意味着任何全局资源都会被多个测试同时触碰：

| 全局资源 | 会出什么问题 |
|---|---|
| 环境变量 | 一个测试改了，另一个测试读到改后的值 |
| 当前工作目录 | 一个测试 `chdir` 之后，其他测试的相对路径全错 |
| 固定的临时文件名 | 两个测试写同一个文件，内容互相覆盖 |
| 固定 TCP 端口 | 第二个测试绑定端口失败 |
| 全局静态可变状态 | 数据竞争（Rust 会用类型系统拦住大部分，但不是全部） |

规则：

1. 每个测试从自己的初始状态开始，不依赖前一个测试留下的东西。
2. 需要文件时，为每个测试创建**独立**的路径（可以拼上进程号或测试名），并在结束时清理。
3. 不要通过 `chdir` 改进程的工作目录；把路径作为参数传给函数。
4. 不要为了让测试通过而把整个套件设成单线程（`--test-threads=1`）。那会掩盖设计问题。

`--test-threads=1` 只应该在**排查**“是不是并行导致的”时临时使用，不是长期配置。

## 十二、完整示例：一个温度记录库

这个例子和主练习不同，可以放心细读。它演示本课所有层次：单元测试、表格驱动、错误检查、返回 `Result` 的测试、集成测试和文档测试。

### 12.1 库代码

```rust
// src/lib.rs

#[derive(Debug, PartialEq, Eq)]
pub enum LogError {
    NotANumber,
    OutOfRange,
}

/// 一串温度读数。
///
/// 字段私有：只能通过 `record` 追加，读数因此都经过范围检查。
///
/// ```compile_fail
/// use temperature::TemperatureLog;
///
/// let log = TemperatureLog {
///     readings: vec![20.0],
/// };
/// ```
#[derive(Debug, Default)]
pub struct TemperatureLog {
    readings: Vec<f64>,
}

impl TemperatureLog {
    /// 记录一个摄氏温度读数。
    ///
    /// `NaN`（“不是一个数字”）不行，超出 -90 到 60 的读数也不好——
    /// 这些都不是真实的地面气温。
    ///
    /// ```
    /// use temperature::{LogError, TemperatureLog};
    ///
    /// let mut log = TemperatureLog::default();
    ///
    /// assert_eq!(log.record(20.0), Ok(()));
    /// assert_eq!(log.record(f64::NAN), Err(LogError::NotANumber));
    /// assert_eq!(log.len(), 1);
    /// ```
    pub fn record(&mut self, celsius: f64) -> Result<(), LogError> {
        if celsius.is_nan() {
            return Err(LogError::NotANumber);
        }
        if !(-90.0..=60.0).contains(&celsius) {
            return Err(LogError::OutOfRange);
        }
        self.readings.push(celsius);
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.readings.len()
    }

    /// 返回最高读数；没有读数时返回 `None`。
    pub fn warmest(&self) -> Option<f64> {
        self.readings
            .iter()
            .copied()
            .fold(None, |best: Option<f64>, value| match best {
                Some(best) if best >= value => Some(best),
                _ => Some(value),
            })
    }

    /// 返回最高读数与最低读数之差；没有读数时返回 `None`。
    ///
    /// ```
    /// use temperature::TemperatureLog;
    ///
    /// let mut log = TemperatureLog::default();
    /// log.record(20.0).unwrap();
    /// log.record(15.5).unwrap();
    ///
    /// assert_eq!(log.span(), Some(4.5));
    /// assert_eq!(TemperatureLog::default().span(), None);
    /// ```
    pub fn span(&self) -> Option<f64> {
        if self.readings.is_empty() {
            return None;
        }
        let mut lowest = self.readings[0];
        let mut highest = self.readings[0];
        for &value in &self.readings[1..] {
            if value < lowest {
                lowest = value;
            }
            if value > highest {
                highest = value;
            }
        }
        Some(highest - lowest)
    }
}
```

`fold`（折叠）现在只需理解成“从初始值出发，把序列里的每个元素依次合并进去”。完整解释在[第十课](../10-iterators/README.md)。

### 12.2 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::{LogError, TemperatureLog};

    fn log_with(readings: &[f64]) -> TemperatureLog {
        let mut log = TemperatureLog::default();
        for &value in readings {
            log.record(value).unwrap();
        }
        log
    }

    #[test]
    fn record_rejects_nan_and_out_of_range_values() {
        let mut log = TemperatureLog::default();

        assert_eq!(log.record(f64::NAN), Err(LogError::NotANumber));
        assert_eq!(log.record(-90.5), Err(LogError::OutOfRange));
        assert_eq!(log.record(60.5), Err(LogError::OutOfRange));
        assert_eq!(log.len(), 0, "失败的记录不应留下任何读数");
    }

    #[test]
    fn record_accepts_the_range_boundaries() {
        let mut log = TemperatureLog::default();

        assert_eq!(log.record(-90.0), Ok(()));
        assert_eq!(log.record(60.0), Ok(()));
        assert_eq!(log.len(), 2);
    }

    #[test]
    fn span_of_an_empty_log_is_none() {
        assert_eq!(TemperatureLog::default().span(), None);
    }

    #[test]
    fn span_cases() {
        let cases = [
            ("空", &[][..], None),
            ("一个读数", &[12.5][..], Some(0.0)),
            ("多个读数", &[20.0, 15.5, 21.0][..], Some(5.5)),
        ];

        for (name, readings, expected) in cases {
            let actual = log_with(readings).span();
            assert_eq!(actual, expected, "case {name:?}");
        }
    }

    #[test]
    fn warmest_needs_a_tolerance() -> Result<(), Box<dyn std::error::Error>> {
        let log = log_with(&[0.1, 0.2, 0.3]);
        let warmest = log.warmest().ok_or("日志为空")?;

        assert!((warmest - 0.3).abs() < 1e-12, "warmest 是 {warmest}");
        Ok(())
    }
}
```

### 12.3 集成测试

```rust
// tests/public_api.rs

use temperature::{LogError, TemperatureLog};

#[test]
fn public_api_records_and_reports() {
    let mut log = TemperatureLog::default();
    log.record(20.0).unwrap();
    log.record(-5.0).unwrap();

    assert_eq!(log.len(), 2);
    assert_eq!(log.warmest(), Some(20.0));
    assert_eq!(log.record(f64::NAN), Err(LogError::NotANumber));
}
```

### 12.4 从输入到输出：数据怎么流动

| 调用 | 输入 | 内部发生了什么 | 返回值 |
|---|---|---|---|
| `TemperatureLog::default()` | 无 | `readings` 是空 `Vec` | 空日志 |
| `record(20.0)` | `20.0` | 不是 NaN，在 `-90..=60` 内，`push` | `Ok(())` |
| `record(f64::NAN)` | `NaN` | `is_nan()` 为真，立即返回 | `Err(NotANumber)` |
| `record(-90.5)` | `-90.5` | 不在范围内 | `Err(OutOfRange)` |
| `record(-90.0)` | `-90.0` | 恰好在边界，`contains` 为真 | `Ok(())` |
| `span()`（先记录 20.0 和 15.5） | 两个读数 | 最高 20.0，最低 15.5，相减 | `Some(4.5)` |
| `span()`（空日志） | 无读数 | 提前返回 | `None` |

注意 `-90.0` 那一行：`(-90.0..=60.0)` 里的 `..=` 是**闭区间**，两端都包含。如果用 `..`（右端不包含），`60.0` 就会被错误拒绝。这类“恰好等于边界”的情况，是测试必须覆盖的。

### 12.5 一次 `cargo test` 的结果（节选）

```text
running 5 tests
test tests::record_accepts_the_range_boundaries ... ok
test tests::record_rejects_nan_and_out_of_range_values ... ok
test tests::span_cases ... ok
test tests::span_of_an_empty_log_is_none ... ok
test tests::warmest_needs_a_tolerance ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

     Running tests/public_api.rs (target/debug/deps/public_api-bc0b7481bb3f57e2)

running 1 test
test public_api_records_and_reports ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

   Doc-tests temperature

running 2 tests
test src/lib.rs - TemperatureLog::record (line 31) ... ok
test src/lib.rs - TemperatureLog::span (line 68) ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

running 1 test
test src/lib.rs - TemperatureLog (line 13) - compile fail ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

一次命令，三类边界（单元、集成、文档）都被验证了。这就是“质量门禁”的雏形。

## 十三、真实诊断与排错

下面每一条都是真实运行得到的输出，不是编造的形状。学会读它们，比记住结论更有用。

### 13.1 E0277：`assert_eq!` 要求 `Debug`

```text
error[E0277]: `Point` doesn't implement `Debug`
  --> src/lib.rs:17:9
   |
17 |         assert_eq!(Point { x: 1 }, Point { x: 1 });
   |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ the trait `Debug` is not implemented for `Point`
   |
  = note: add `#[derive(Debug)]` to `Point` or manually `impl Debug for Point`
help: consider annotating `Point` with `#[derive(Debug)]`
   |
 2 + #[derive(Debug)]
 3 | pub struct Point { pub x: i32 }
```

怎么读：

- `E0277` 是“trait 约束不满足”的错误码。`trait bound`（trait 约束）的意思是“某段代码要求某个类型具备某种能力”。
- `assert_eq!` 需要 `Debug`，因为失败时要打印两边。你的类型没派生出 `Debug`，所以宏无法展开。
- 修复：给类型加 `#[derive(Debug)]`。如果还要用 `assert_eq!` 比较相等，再加 `PartialEq`。

注意这是**编译错误**，不是测试失败。测试代码类型检查不过，整套测试都不会运行。这就是为什么验收命令里有一条 `cargo test --no-run`：它只编译，不执行，能最早暴露这类问题。

### 13.2 E0308：字面量的类型和函数返回类型不一致

```text
error[E0308]: mismatched types
 --> src/lib.rs:9:29
  |
9 |         assert_eq!(count(), 3u32);
  |                             ^^^^ expected `usize`, found `u32`
  |
help: change the type of the numeric literal from `u32` to `usize`
```

怎么读：

- `count()` 返回 `usize`（集合长度用的无符号整数类型），你写的是 `3u32`。
- `u32` 后缀显式指定了字面量的类型，所以 `assert_eq!` 的两个参数类型不同，`PartialEq` 无法比较。
- 修复：去掉后缀写成 `3`（类型会被推断成 `usize`），或者写 `3usize`。

这类错误在测试里很常见：**测试里的期望值和真实返回值必须是同一个类型**，不能靠隐式转换。

### 13.3 E0451：集成测试不能访问私有字段

```text
error[E0451]: fields `title` and `seconds` of struct `Track` are private
 --> tests/x.rs:3:47
  |
3 |     let _track = l08_testing_quality::Track { title: String::from("x"), seconds: 1 };
  |                                               ^^^^^ private field       ^^^^^^^ private field
```

怎么读：

- 这个错误出现在 `tests/` 下的文件里，说明它是从**外部用户**视角编译的。`x.rs` 是示意用的临时测试文件名。
- `E0451` 专门指“结构体字段不可访问”。即使结构体本身是 `pub`，字段也有自己的可见性（第七课）。
- 这不是测试写错了，而是测试**成功地证明了**外部用户不能绕过构造函数。本课的 `Track` 文档里就有一个故意触发它的 `compile_fail` 例子。

### 13.4 `f64` 没有 `Ord`，所以不能直接 `sort()`

写中位数时很自然会想“先排序”：

```rust
let mut sorted = values.to_vec();
sorted.sort();
```

```text
error[E0277]: the trait bound `f64: Ord` is not satisfied
   --> src/lib.rs:3:12
    |
  3 |     sorted.sort();
    |            ^^^^ the trait `Ord` is not implemented for `f64`
```

怎么读：

- `sort()` 要求元素的类型实现 `Ord`（全序：任意两个值都能比较，且没有“不可比”的情况）。
- `f64` 做不到全序，因为有 `NaN`：`NaN < 1.0`、`NaN > 1.0`、`NaN == 1.0` 全是假。
- 修复方向：用 `sort_by` 并提供一个比较函数，例如 `sort_by(|left, right| left.partial_cmp(right).expect("no NaN"))`。`partial_cmp` 返回 `Option<Ordering>`，`NaN` 参与时得到 `None`。

本课课后巩固的数据约定不含 `NaN`，所以 `expect` 在这里是合理的。如果真实数据可能含 `NaN`，应该显式处理，而不是 `expect`。

### 13.5 测试失败和编译错误的区别

实现写错时，你看到的是这种输出：

```text
---- playlist::queue::tests::add_trims_whitespace_on_both_sides_of_the_title stdout ----

thread 'playlist::queue::tests::add_trims_whitespace_on_both_sides_of_the_title' (161847) panicked at src/playlist/queue.rs:228:9:
assertion `left == right` failed
  left: "\t 稻香 \n"
 right: "稻香"
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

failures:
    playlist::queue::tests::add_trims_whitespace_on_both_sides_of_the_title

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 35 filtered out

error: test failed, to rerun pass `-p l08_testing_quality --lib`
```

逐段读（行号来自一次真实运行，你自己实现后行号会不同）：

- 第一行指出哪个测试失败。
- `panicked at src/playlist/queue.rs:228:9` 指出**断言所在的行**，不是缺陷所在的行。缺陷可能在上游很远的函数里。
- `left` 是实际值，`right` 是期望值。这里实际值是带空白的 `"\t 稻香 \n"`，说明实现没有去掉空白。
- `35 filtered out` 是因为运行时加了名字过滤器。
- 最后一行 `error: test failed, to rerun pass -p l08_testing_quality --lib` 告诉你怎样只重跑 library 单元测试：`--lib` 表示不编译、不运行集成测试和文档测试。

排错顺序：

1. 看 `left`/`right`，判断是“实现错了”还是“期望写错了”。
2. 如果是实现错了，回到被调用的函数，用这组输入走一遍。
3. 只跑相关测试（`cargo test -p l08_testing_quality --lib trims`），改完再跑完整套件。

### 13.6 `not yet implemented`：脚手架的正常状态

本课的 `todo!()` 在运行时会产生：

```text
thread 'playlist::queue::tests::add_rejects_zero_seconds' (121880) panicked at src/playlist/queue.rs:123:9:
not yet implemented: 实现 Playlist::add：去空白、检查空标题与零时长、去重、检查容量，然后追加
```

- `todo!()` 是一个**宏**，它展开成“立即 panic，并附带你写的说明”。
- 括号里的文本会原样出现在报错里，所以脚手架可以用它提示这一步要做什么。
- 看到 `not yet implemented` 时不要怀疑环境或测试：它明确表示“这里还是空的”。

### 13.7 Clippy：`for` 循环只用来索引

```text
error: the loop variable `index` is only used to index `values`
  --> src/lib.rs:11:18
   |
11 |     for index in 0..values.len() {
   |                  ^^^^^^^^^^^^^^^
   |
   = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.94.0/index.html#needless_range_loop
   = note: `-D clippy::needless-range-loop` implied by `-D warnings`
help: consider using an iterator
   |
11 -     for index in 0..values.len() {
11 +     for <item> in &values {
```

怎么读：

- `#needless_range_loop` 是 Clippy 的 lint 名字。**lint** 指“静态检查提出的一条可疑写法”。
- `-D warnings` 把警告升级为错误（`-D` 是“deny”，拒绝）。
- 建议本身是合理的：直接遍历元素比“造一个索引再拿它索引”更清楚，也省掉越界可能。但**不要盲目照抄**。如果循环里还要用索引做别的事（比如同时访问第二个切片），保留 `for index in ...` 是正确的，这时可以在最小范围加 `#[allow(clippy::needless_range_loop)]` 并写明理由。

### 13.8 警告：测试里的未使用变量

```text
warning: unused variable: `expected`
  --> src/lib.rs:11:13
   |
11 |         let expected = String::from("x");
   |             ^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_expected`
```

这条警告往往暗示一个更严重的问题：你算了期望值，但断言里没用它。

```rust
let expected = String::from("x");
assert!(!label("晴天").is_empty());   // expected 根本没用上
```

这个测试会显示 `... ok`，但它没有检查 `expected`，所以“通过”没有意义。**测试通过不等于测试有效**——这是本课最重要的观念之一。

### 13.9 `compile_fail` 的例子其实编译成功了

```text
---- src/lib.rs - TemperatureLog::record (line 20) stdout ----
Test compiled successfully, but it's marked `compile_fail`.

failures:
    src/lib.rs - TemperatureLog::record (line 20)
```

原因：那段例子写的是 `log.record(f64::NAN).expect("NaN 会被拒绝")`。`expect` 在**运行时**才 panic，编译完全没问题。`compile_fail` 检查的是编译，不是运行。

修复方向：如果目的是“证明会被拒绝”，用普通代码块（会运行），断言返回 `Err`；如果目的是“证明编译不过”，放一个真正非法的表达式（例如访问私有字段）。

## 十四、代码预测与解析

先独立判断，再展开。

### 预测 1：这个测试能通过吗

```rust
#[test]
fn add_is_ok() {
    let mut playlist = Playlist::new(2);
    let result = playlist.add("晴天", 269);
    assert!(result.is_ok());
    assert_eq!(result, Ok(()));
}
```

<details><summary>解析</summary>

能通过。两行断言都不算错，但第一行是多余的：`assert_eq!(result, Ok(()))` 已经包含了“是 Ok”这个信息，而且更强。

多余断言本身不是错误，但会让失败信息变多。更好的习惯是选**最强的那条**断言。

</details>

### 预测 2：哪个测试会失败

```rust
#[test]
fn count_matches() {
    let mut playlist = Playlist::new(1);
    playlist.add("晴天", 269).unwrap();
    assert_eq!(playlist.add("稻香", 200), Err(PlaylistError::Full { capacity: 1 }));
}
```

<details><summary>解析</summary>

通过。歌单容量 1，已经装了 1 首；新标题不重复，所以检查顺序走完前三步、在第四步返回 `Full { capacity: 1 }`。

如果把歌单改成 `Playlist::new(2)`，这个断言就会失败：实际返回 `Ok(())`，因为还有位置。

</details>

### 预测 3：为什么这个测试通过却没有价值

```rust
#[test]
fn total_is_a_number() {
    let playlist = sample_playlist();
    let total = playlist.total_seconds();
    assert!(total >= 0);
}
```

<details><summary>解析</summary>

`total_seconds()` 返回 `u64`，而无符号整数永远 `>= 0`。这条断言是恒真的，实现返回 0、返回 1、返回错误的值都能通过。

有价值的版本是写清期望：`assert_eq!(playlist.total_seconds(), 269 + 299)`。

“恒真断言”是测试里最常见的无效来源。检查方法：问自己“这个断言有可能失败吗？什么改动会让它失败？”

</details>

### 预测 4：`should_panic` 会通过吗

```rust
#[test]
#[should_panic(expected = "titles must not be empty")]
fn rejects_empty_title() {
    let mut playlist = Playlist::new(2);
    playlist.add("", 200).unwrap();
}
```

<details><summary>解析</summary>

不会通过（在这个设计下）。`add` 返回 `Err(PlaylistError::EmptyTitle)`，`unwrap()` 会 panic，实际报错是：

```text
thread 'rejects_empty_title' panicked at tests/pred.rs:25:27:
called `Result::unwrap()` on an `Err` value: EmptyTitle
note: panic did not contain expected string
      panic message: "called `Result::unwrap()` on an `Err` value: EmptyTitle"
 expected substring: "titles must not be empty"
```

这段文本里没有 `"titles must not be empty"`，所以 `expected` 匹配失败，测试被判为失败。

这个测试还有更深的问题：它把 `Result` 的失败硬转成 panic 来测，破坏了 API 契约。正确写法是 `assert_eq!(playlist.add("", 200), Err(PlaylistError::EmptyTitle))`。

</details>

### 预测 5：哪些行会参与编译

```rust
/// ```
/// use my_crate::add;
/// # let hidden = 1;
/// assert_eq!(add(2, 3), 5);
/// ```
```

<details><summary>解析</summary>

三行都参与编译和执行。`#` 只影响**文档渲染**（那一行在网页上看不见），不影响测试。

所以不要把 `let mut` 之类的准备语句藏起来却依赖它——它会正常工作，但读文档的人看不到它，复制例子时会缺东西。

</details>

### 预测 6：这个集成测试能编译吗

```rust
// tests/sneaky.rs
use l08_testing_quality::playlist::Playlist;

#[test]
fn works() {
    assert_eq!(Playlist::new(2).capacity(), 2);
}
```

<details><summary>解析</summary>

不能。`Playlist` 是从 crate 根**重新导出**的（`pub use playlist::Playlist`），但 `playlist` 模块本身是 `mod playlist;`，没有 `pub`。因此外部 crate 无法经过 `l08_testing_quality::playlist` 这条路径。

正确写法：`use l08_testing_quality::Playlist;`。

这正是集成测试要确认的边界：外部用户能否通过真实存在的公共路径拿到类型。

</details>

## 十五、质量标准：四类命令各证明什么

```bash
cargo fmt -p l08_testing_quality -- --check
cargo check -p l08_testing_quality --all-targets
cargo test -p l08_testing_quality --no-run
cargo clippy -p l08_testing_quality --all-targets -- -D warnings
cargo test -p l08_testing_quality
```

| 命令 | 主要证明什么 | 不能证明什么 |
|---|---|---|
| `fmt --check` | 格式符合统一规则 | 代码正确性 |
| `check` | 所有构建目标能通过类型检查 | 测试会通过 |
| `test --no-run` | 所有测试代码能编译 | 测试会通过 |
| `clippy` | 没有触发启用的可疑写法 | 没有逻辑错误 |
| `test` | 已写出的断言都成立 | 需求没有遗漏 |

拆开解释：

- `cargo fmt` 是 **rustfmt**（Rust 官方格式化工具）的入口。`--check` 只检查、不修改；不加 `--check` 就直接改写文件。格式统一的价值是：代码评审时不用争论空格，diff 里不会出现“只改了缩进”的噪声。
- `--all-targets` 让 Cargo 把 library、binary、测试、例子都纳入检查。不加它时，某些测试代码可能没被编译到。
- `cargo test --no-run` 编译测试但不执行。它的价值是**最早**发现问题：编译错误和 `todo!()` 失败是两回事，前者说明测试代码本身坏了。
- `cargo clippy` 是 Rust 的静态检查工具。它比编译器严格：编译器只管“能不能编译”，Clippy 还会指出“能编译但可疑”。`-D warnings` 把警告当错误，适合在提交前跑。
- `-- -D warnings` 里的第一个 `--` 表示“后面的参数传给测试/Clippy 二进制，而不是 Cargo”。

### 15.1 Clippy 建议不是法律

处理顺序：

1. 读懂 lint 指出的具体问题；
2. 判断建议是否保持了你的领域语义；
3. 优先改代码；
4. 确实需要保留时，在**最小作用域**加 `#[allow(...)]` 并写理由。

不要在 crate 根写 `#![allow(warnings)]` 来换一片绿色。那个属性会让之后所有真实问题都被藏起来。本课脚手架只在 `Track::new` 上用了 `#[allow(dead_code)]`，并写明了原因：这个方法要等学习者实现 `add` 之后才会被调用。

### 15.2 变异测试：证明测试真的能失败

一条测试如果永远不会失败，它就没有价值。检查办法是**故意改坏实现**（这叫变异测试，mutation testing 的基本思路），看测试是否报警。

本课 README 的验证过程中做过这些实验（只统计 library 单元测试，共 36 个）：

| 故意引入的缺陷 | 结果 |
|---|---|
| 去掉标题的 `trim()` | 6 个测试失败 |
| 把 `ZeroLength` 错报成 `EmptyTitle` | 3 个测试失败 |
| 用 `u32` 累加时长后转 `u64` | 1 个测试失败（溢出用例） |
| 中位数排序前先取长度，跳过 `sort_by` | 4 个测试失败 |

如果一个“看起来很重要”的变异没有任何测试失败，说明覆盖有空洞。每次实验后**立刻恢复**正确实现，不要让故意缺陷留在代码里。

## 十六、主练习：歌单库的四个方法

### 16.1 目标

本课 package 是 `l08_testing_quality`，里面已经有一个完整的歌单库骨架：数据类型、错误类型、模块结构、公开 API 签名、**全部自动测试**都已经写好。你要实现四个方法，让整套测试通过。

不需要写测试——测试已经给你了。这也是本课的设计：先读懂测试表达的契约，再实现满足契约的代码。读懂测试是一种独立能力，和写测试同样重要。

### 16.2 四个待实现的方法

文件：`src/playlist/queue.rs`

| 方法 | 签名 | 目标 |
|---|---|---|
| `Playlist::add` | `fn add(&mut self, title: &str, seconds: u32) -> Result<(), PlaylistError>` | 去空白、按顺序检查四种失败、追加曲目 |
| `Playlist::find` | `fn find(&self, title: &str) -> Option<&Track>` | 去空白后按标题查找，返回借用 |
| `Playlist::total_seconds` | `fn total_seconds(&self) -> u64` | 用 `u64` 累加所有时长 |
| `Playlist::longest` | `fn longest(&self) -> Option<&Track>` | 空返回 `None`，并列时保留先出现的 |

文件：`src/playlist/model.rs` 里的 `Track::new` 是**已给出的**构造器（`pub(super)`，只对 `playlist` 模块可见）。你在 `add` 里调用它。

### 16.3 输入、输出与边界

以下都是真实调用及其期望结果。

**正常情况**

```text
Playlist::new(3).capacity()                    => 3
Playlist::new(3).len()                         => 0
Playlist::new(3).is_empty()                    => true

let mut p = Playlist::new(2);
p.add(" 晴天 ", 269)                            => Ok(())
p.tracks()[0].title()                          => "晴天"     （空白被去掉）
p.tracks()[0].seconds()                        => 269
p.find("  晴天\t")                              => Some(&Track{"晴天", 269})
p.total_seconds()                              => 269
```

**边界情况**

```text
Playlist::new(0).add("晴天", 269)               => Err(Full { capacity: 0 })
p.add("超长", u32::MAX)                          => Ok(())      （u32 最大值合法）
两个 u32::MAX 曲目的 total_seconds()              => 8589934590  （超过 u32，必须是 u64）
p.add("\t 稻香 \n", 223)                         => Ok(())      标题为 "稻香"
p.add("　", 200)                                 => Err(EmptyTitle)  （全角空格也是空白）
p.add("Yesterday", 125); p.add("yesterday", 125) => 两次都 Ok(())    （区分大小写）
```

**错误情况**

```text
p.add("", 200)                                  => Err(EmptyTitle)
p.add("   ", 200)                               => Err(EmptyTitle)
p.add("静音", 0)                                 => Err(ZeroLength)
p.add("  晴天  ", 200)（已有 "晴天"）              => Err(DuplicateTitle("晴天"))
满员的 2 首歌单 add("稻香", 223)                   => Err(Full { capacity: 2 })
Playlist::new(1) 已有 "晴天" 时 add(" 晴天 ", 200) => Err(DuplicateTitle("晴天"))  ← 注意不是 Full
Playlist::new(0) 时 add("  ", 0)                 => Err(EmptyTitle)               ← 不是 ZeroLength
```

最后三行是**错误优先级**：检查顺序是 `EmptyTitle` → `ZeroLength` → `DuplicateTitle` → `Full`。顺序是对外契约的一部分，测试逐条锁死了它。你在实现时要按这个顺序写 `if`，不能图省事调换。

**失败不改状态**：所有错误返回之后，`len()` 和 `tracks()` 必须和调用前完全一样。

**查找与统计**

```text
空歌单 find("晴天")                              => None
空歌单 longest()                                => None
空歌单 total_seconds()                          => 0
[100s, 400s, 200s] 的 longest()                 => 400s 那首
[300s, 300s] 的 longest()（并列）                 => 先添加的那首
```

### 16.4 脚手架与约束

- 只替换四个方法体里的 `todo!()`。不要修改签名、字段、`pub` 修饰符，也不要删除或改写测试。
- 不要为了让测试通过而删除测试或放宽断言。
- `tracks` 字段保持私有，外部只能通过 `tracks()`、`find()`、`longest()` 接触曲目。
- `find` 和 `longest` 必须**返回借用**（`&Track`），不能复制曲目。测试会检查返回值的字段，但设计意图是零拷贝。
- 不要新增依赖。本课的 `Cargo.toml` 依赖区是空的，标准库够用。
- 占位参数已经用 `let _ = (&self, title, seconds);` 处理，所以脚手架能通过 Clippy；实现时把这一行删掉。

### 16.5 推荐完成顺序

1. **`add` 的前三步**：去空白、空标题、零时长。跑 `cargo test -p l08_testing_quality add_rejects`，让空标题和零时长的测试通过。
2. **`add` 的去重与容量**：按顺序加两个检查。跑 `add_reports_full` 和 `duplicate_title_is_reported_before`。
3. **`add` 的追加**：调用 `Track::new` 并 `push`。此时 `add` 相关测试应全绿。
4. **`total_seconds`**：先写最简单的累加，再看溢出测试为什么要求 `u64`。
5. **`find`**：注意返回类型是 `Option<&Track>`，从 `iter()` 里拿引用。
6. **`longest`**：先处理空，再考虑并列时保留哪一个。
7. 跑完整套件，再做一次变异实验（见 15.2 节）确认测试有效。

### 16.6 测试说明与验收命令

实现之前，测试**能编译**，运行时会在对应的 `todo!()` 处以 `not yet implemented` 失败。这是预期状态，不是环境问题。

本课共有这些测试：

| 位置 | 数量 | 覆盖 |
|---|---|---|
| `src/playlist/queue.rs` 的 `mod tests` | 23 | 初始状态、正常、边界、错误、错误优先级、查找、合计、最长、表格驱动 |
| `src/statistics.rs` 的 `mod tests` | 13 | 平均数与中位数的正常、边界、容差、输入不被修改、表格驱动 |
| `tests/playlist_public_api.rs` | 3 | 只用 crate 根导出的公共 API 行为 |
| 文档测试 | 5 | 3 个可通过/可编译的例子，1 个可运行的 `Playlist` 例子，1 个 `compile_fail` |

验收命令（在仓库根目录运行）：

```bash
cargo fmt -p l08_testing_quality -- --check
cargo check -p l08_testing_quality --all-targets
cargo test -p l08_testing_quality --no-run
cargo clippy -p l08_testing_quality --all-targets -- -D warnings
cargo test -p l08_testing_quality
cargo test -p l08_testing_quality --doc
```

全部通过即完成。`cargo run -p l08_testing_quality` 可以看演示输出。

如果某条命令失败，先判断它属于哪一类：

- `fmt --check` 失败 → 运行 `cargo fmt -p l08_testing_quality` 自动修好；
- `check` 或 `test --no-run` 失败 → 编译错误，读错误码（本课第十三节）；
- `clippy` 失败 → 读 lint 名字和它给的例子；
- `test` 失败 → 读 `left`/`right`，回到实现。

## 十七、课后巩固：平均数与中位数

换一组数据、换一类边界。这次处理浮点数，会踩到舍入误差和“不能直接 `sort()`”。

文件：`src/statistics.rs`，两个函数都是 `todo!()`。

### 17.1 目标与行为

| 函数 | 签名 | 契约 |
|---|---|---|
| `mean` | `fn mean(values: &[f64]) -> Option<f64>` | 空切片返回 `None`；否则返回总和 / 个数 |
| `median` | `fn median(values: &[f64]) -> Option<f64>` | 空切片返回 `None`；奇数个返回排序后正中间；偶数个返回中间两个的平均；不修改调用者数据 |

### 17.2 输入、输出与边界

```text
mean(&[])                    => None
mean(&[7.5])                 => Some(7.5)
mean(&[-4.0, 1.0, 3.0])      => Some(0.0)        （(-4+1+3)/3）
mean(&[0.1, 0.2])            => 约 0.15，需容差比较
mean(&[2.5, 2.5, 2.5, 2.5])  => Some(2.5)

median(&[])                  => None
median(&[42.0])              => Some(42.0)
median(&[9.0, 1.0, 5.0])     => Some(5.0)        （排序后 [1,5,9]）
median(&[4.0, 1.0, 3.0, 2.0])=> Some(2.5)        （排序后 [1,2,3,4]）
median(&[-1.0, -1.0, 2.0])   => Some(-1.0)       （重复值）
median(&[3.0, 1.0, 2.0])     => Some(2.0)，且调用者的数组仍是 [3.0, 1.0, 2.0]
```

### 17.3 约束

- 只替换两个函数体里的 `todo!()`；不修改签名和测试。
- `median` 的参数是只读切片，排序必须在一份副本上做。测试会直接检查原数组没被改动。
- 浮点比较用容差，不要用 `assert_eq!` 比较算出来的小数。辅助函数 `assert_option_close` 已经写好。
- 不要引入依赖。

### 17.4 验收

```bash
cargo test -p l08_testing_quality statistics
cargo clippy -p l08_testing_quality --all-targets -- -D warnings
```

十三个测试全绿即完成。如果 `median_of_an_even_count_averages_the_two_middle_values` 失败，检查你有没有除以 2。

## 十八、常见误区

- **只测“正常路径”。** 一个函数有五个参数，每个参数都有失败情况，只测成功调用几乎必然漏掉真实的 bug。
- **只检查 `is_err()`。** 错误原因错了也能通过，用户会看到错误的提示。
- **在 `should_panic` 里让准备代码先 panic。** 测试通过了，目标行为一次都没执行。至少用 `expected = "..."`，或者改用 `Result`。
- **断言恒真。** `assert!(total >= 0)` 对无符号整数永远成立。问自己“什么改动能让这条断言失败”。
- **测试依赖执行顺序。** 共享可变状态 + 并行执行 = 偶发失败。每个测试从自己的状态开始。
- **用固定的文件名或端口。** 两个测试同时跑就互相踩。用独立路径。
- **为了测试把私有实现全改成 `pub`。** 测试写在模块内就能访问私有项；API 可见性应由使用场景决定。
- **测试锁死实现细节。** 断言“内部先调用了 `split_once` 再调用 `parse_port`”会让正常重构全部失败。测可观察行为。
- **一个测试塞进太多无关断言。** 第一个失败后后面不执行，你不知道坏了几个地方。
- **表格里加 `if case.name == ...` 分支。** 那说明这些 case 不该放在同一张表里。
- **用 `assert_eq!` 比较浮点运算结果。** `0.1 + 0.2 != 0.3`。
- **`compile_fail` 例子写了运行时才失败的代码。** 它会报 “Test compiled successfully, but it's marked `compile_fail`”。
- **只跑 `cargo test`，不跑 fmt 和 clippy。** 格式漂移和可疑写法会慢慢积累。
- **为了让 CI 变绿而 `#![allow(warnings)]`。** 之后所有真实警告都被隐藏。
- **在提交里留下故意引入的缺陷做“变异实验”。** 实验完必须立刻恢复。

## 十九、本课总结

1. 测试是**可执行的期望**：把“应该发生什么”写成代码，让机器反复检查。
2. 单元测试写在 `#[cfg(test)] mod tests` 里，能访问私有项，只在你运行 `cargo test` 时编译。
3. 集成测试放在 `tests/*.rs`，每个文件是独立 crate，只能走公共 API——它验证的是“外部用户能不能用”。
4. 文档测试执行文档注释里的例子，保证例子不会和代码脱节。`no_run` 只编译，`compile_fail` 期望编译失败，`ignore` 会腐烂。
5. `assert_eq!` 比 `assert!` 提供更好的失败信息，代价是类型必须实现 `Debug + PartialEq`。
6. 测 `Result` 要比较具体的变体和上下文，不能停在 `is_err()`；同时检查失败没有留下部分状态。
7. 表格驱动把多组输入压成一份逻辑，前提是每行断言的是同一件事，且失败消息带出 case 名和输入。
8. 浮点用容差，无序集合不依赖顺序，测试之间不共享全局资源（测试默认并行）。
9. `#[should_panic]` 只证明“某个 panic 发生过”，容易被无关 panic 骗过。
10. 质量门禁是四类命令的组合：格式化、类型检查、编译测试、静态检查、行为测试。每类证明的东西不同，全部通过也不等于需求没有遗漏。
11. 测试通过不等于测试有效。用“故意改坏实现”的办法检查测试真的会失败。

## 二十、课后复盘

用自己的话回答，再对照正文：

1. 为什么单元测试能访问私有函数，而集成测试不能？
2. 为什么 `assert!(result.is_err())` 经常不够？举一个它会漏掉的具体缺陷。
3. `cargo test --no-run` 和 `cargo test` 各能发现什么问题？为什么两个都要跑？
4. 一个 `#[should_panic]` 测试通过了，你怎么确认它测的是你以为的那件事？
5. 为什么 `compile_fail` 的例子要写得尽量短？
6. 测试默认并行执行，这会给哪些写法带来问题？举两个具体例子。
7. 你写了一条测试，它从来没失败过。你怎么判断它有没有价值？

## 下一课预告

本课的所有测试都是“给一个值，检查结果”。下一课[第九课：闭包、环境捕获与 `Fn` 系列 trait](../09-closures/README.md)会引入一种新的值：**可以像函数一样调用的代码块**。你会看到它如何被当作参数传给别的函数，如何借用或夺走周围的数据，以及为什么标准库的排序、迭代器方法都要求你传一个闭包。

到那时，本课学到的测试能力会立刻用上：闭包捕获了什么、能调用几次，这些行为都可以用你现在会写的方式固定下来。
