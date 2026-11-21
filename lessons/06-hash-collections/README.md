# 第六课：HashMap、HashSet 与 Entry API

第五课的 `Vec` 按**位置**保存数据：第 0 个、第 1 个、第 2 个。很多问题却不是按位置提问的，而是按**名字**提问的：

- “`guide` 这篇笔记现在有哪些标签？”
- “带 `rust` 标签的笔记一共有几篇？”
- “读书会里，`Ada` 读过哪些书？”

如果数据装在 `Vec` 里，每个问题都得从头到尾扫一遍。本课学习两种按**键（key，用来查找的名字）**组织的集合：`HashMap`（键到值的映射）和 `HashSet`（只保存不重复的键）。它们让“按名字找东西”变成一次定位，而不是一次全表扫描。同时我们会反复处理一个所有权问题：**名字被放进集合以后，谁拥有它？还能不能继续用？**

## 本课定位

### 你将学会

- 用 `HashMap<K, V>` 保存“键 → 值”的对应关系，并解释 `K`、`V` 两个类型参数各表示什么。
- 用 `insert` 添加或替换条目，读懂它返回的 `Option<V>`。
- 用 `get`、`get_mut`、`contains_key`、`remove` 查找和删除，并解释为什么 `get` 返回引用。
- 用 `&str` 查找 `HashMap<String, _>`，理解“借用视图查找”为什么不产生新的字符串分配。
- 用 `entry`（Entry API，把“这个键对应的槽位”交给调用者的接口）、`or_insert`、`and_modify`、`or_insert_with` 完成计数和“缺了才插入”。
- 用 `HashSet<T>`（只保存不重复值的集合）表达“不重复”，并做交集、并集、差集等集合运算。
- 解释键必须满足 `Eq`（能判断两个键是否相等）与 `Hash`（能算出哈希值）的原因，知道哪些类型不能直接当键。
- 理解哈希表平均很快、但遍历顺序不稳定的原因，写出不依赖顺序的测试。
- 在同时维护两张表的索引里，保证失败时不留“改了一半”的状态。

### 前置知识

- 变量、`mut`、函数、`for` 循环、数组：见[第一课](../01-basics/README.md)。简单复习：`let mut x` 表示这个变量之后允许被修改；`for item in list` 让 `item` 依次取集合里的每个元素。
- `struct`、`enum`、方法、`&self` 与 `&mut self`：见[第二课](../02-data-modeling/README.md)。简单复习：`struct` 把几个相关字段打包成一个类型；`enum` 表示“几种情况之一”；`&self` 的方法只借数据来看，`&mut self` 的方法借来修改。
- 所有权、move、`Copy`、`&T`、`&mut T`、`String` 与 `&str`：见[第三课](../03-ownership-borrowing/README.md)。简单复习：每份数据只有一个所有者；把 `String` 传给别的变量或集合通常会把所有权移过去（move），之后原变量失效；`Copy` 类型（如 `i32`）赋值时是复制，不会失效；`&T` 是只读借用，`&mut T` 是可变借用。
- `Option<T>`、`Some`/`None`、`Result<T, E>`、`if let`、`?` 与自定义错误：见[第四课](../04-option-result/README.md)。简单复习：`Option` 表示“可能有值，也可能没有”；`Result` 表示“成功（`Ok`）或失败（`Err`）”，失败时带着错误信息；两者取用前都必须处理所有情况。
- `Vec<T>`、slice、`retain`、迭代时借用还是移动：见[第五课](../05-vec-slices/README.md)。简单复习：`Vec` 拥有元素、长度可变；`&v` 借来看，`&mut v` 借来改，直接写 `v` 会把整个集合消费掉。

### 本课边界

本课不展开：泛型（generics，让同一份代码适用于多种类型的机制；第 11 课讲正式规则，本课只解释 `HashMap<K, V>` 为什么能装任意类型）、trait（一组行为的约定；第 12 课，解释 `Eq`、`Hash`、`Borrow` 这些约定的完整写法）、迭代器适配器（第 10 课，本课只用 `collect`、`copied` 这类必要操作并给出临时解释）、闭包（第 9 课，本课会在 `or_insert_with` 和 `retain` 里用到，用到时解释够用的部分）、生命周期（第 14 课，本课只说明“借用不能比原数据活得久”这一条）。哈希算法、开放寻址的具体实现细节不属于本课；我们只建立能指导写代码的概念模型。

本课不引入任何第三方依赖：`HashMap` 和 `HashSet` 都在 Rust 标准库（安装 Rust 时自带、无需下载的代码库）里。

## 如何学习本课

1. 先读第一节，想清楚“用 `Vec` 按名字查”到底慢在哪、会错在哪。
2. 每个完整程序都先自己预测输出，再运行对照。可以用 `cargo new` 在临时目录建一个自己的小项目练习。
3. 第十一节的编译器（把源代码翻译成可运行程序的工具）诊断不要跳过：本课的借用错误在实际项目里非常常见。
4. 做完第十二节的代码预测，再打开主练习。主练习卡住时按 `HINTS.md` 的三级提示逐级查看。
5. 主练习全部测试通过后，再做课后巩固。

预计用时：概念部分约 2 小时，主练习约 1.5 小时，课后巩固约 40 分钟。

## 一、问题：用位置找不到“名字”

假设读书会要记录每个人读过的书。数据先用第五课的 `Vec` 表示，每一项是“人名 + 书名列表”：

```rust,ignore
// 片段：只展示数据结构，省略 use 和 main。
let records: Vec<(String, Vec<String>)> = vec![
    (String::from("Ada"), vec![String::from("Rust Book"), String::from("Dune")]),
    (String::from("Lin"), vec![String::from("Dune")]),
];
```

现在要回答“Ada 读过哪些书”，只能写一个循环，从头比较每一项的人名：

```rust,ignore
// 片段：假设 records 已在上面定义。
for (name, books) in &records {
    if name == "Ada" {
        println!("{books:?}");
        break;
    }
}
```

这段代码能工作，但有两个具体困难：

1. **每次查询都要扫一遍。** 记录有 10 条时最多比较 10 次；有 10 万条时最多比较 10 万次。查询越多、数据越大，代价越明显。用“大 O 记号”（描述数据量变大时耗时如何增长的记号）说，这是 $O(n)$：数据翻倍，最坏耗时也大致翻倍。
2. **表达不了“唯一”。** 同一个人可能被登记两次，同一本书也可能在一个人的列表里出现两次。`Vec` 允许重复，程序必须自己记住“插入前先查一遍”，而查一遍又是一次全表扫描。

如果还要反向回答“哪些人读过 Dune”，就得再维护一份反向的 `Vec`，并且每次增删都同步更新两份数据。两份数据一旦有一份没更新，程序给出的答案就自相矛盾。

我们需要一种集合，它的核心操作是：

- **给一个键（key），直接找到对应的值（value）。** 键是查找用的名字，值是这个名字对应的数据。
- **同一个键只保留一份。** 再次放入同一个键，是“更新”，不是“再来一条”。
- **查找不依赖数据量线性增长。** 平均情况下，无论有 10 条还是 10 万条，查找耗时都接近常数。

这种“键到值的对应关系”叫**映射（map）**。日常例子很多：字典里“单词 → 释义”、通讯录里“姓名 → 电话”、身份证号 → 个人信息。Rust 标准库提供的哈希映射类型叫 `HashMap<K, V>`，读作“hash map of K to V”。`K` 是键的类型，`V` 是值的类型。

名字里的 **hash（哈希）** 指它使用的定位方法：用一个**哈希函数（hash function）**把键变成一个大整数，再用这个整数决定条目放在表里的哪个位置。第八节会展开。现在只需要知道：哈希让“按名字找”不必从头比较。

先认识三个词：

- **集合（collection）**：装一组数据的容器。`Vec`、`HashMap`、`HashSet` 都是集合。
- **条目（entry）**：映射里的一组“键 + 值”。例如 `("Ada", 3)`。
- **桶（bucket）**：哈希表内部存放条目的位置。键先被算成一个数字，再映射到某个桶。

## 二、创建 HashMap 并放入第一组数据

下面是一个完整、可运行的程序（`fn main()` 是程序入口，运行时从这里开始执行）：

```rust
use std::collections::HashMap;

fn main() {
    let mut scores: HashMap<String, i32> = HashMap::new();

    scores.insert(String::from("Ada"), 10);
    scores.insert(String::from("Lin"), 20);

    println!("{scores:?}"); // 花括号里的 :? 表示按“调试格式”打印
}
```

逐行解释：
- `println!` 和后面测试里的 `assert_eq!` 名字末尾都有 `!`，表示它们是**宏（macro）**：编译时按规则展开成代码的工具。本课只需会用；宏的机制第 20 课讲。
- `use std::collections::HashMap;`：`use` 把某个路径下的名字引入当前文件，之后可以只写 `HashMap` 而不写完整路径。`std` 是标准库，`collections` 是其中“集合类型”的模块（把相关代码分组的单位，第 7 课详讲）。
- `HashMap::new()`：创建一个空映射。`::` 表示“关联函数”——挂在类型上的函数，用类型名调用。`new` 是 Rust 里常见的“创建一个新值”的函数名。
- `HashMap<String, i32>`：键是 `String`（拥有文本的类型），值是 `i32`（32 位整数）。尖括号里的两个类型叫**类型参数**：`K` 和 `V` 先当占位符，使用时换成具体类型。
- `let mut scores`：`insert` 会修改映射，所以绑定必须是 `mut`（可修改）的。
- `scores.insert(key, value)`：放入一个键值对。

如果后面的 `insert` 已经能推断出类型，可以省略标注：

```rust
use std::collections::HashMap;

fn main() {
    let mut scores = HashMap::new(); // 此刻还不知道 K、V 是什么
    scores.insert(String::from("Ada"), 10); // 从这里推断出 HashMap<String, i32>
    assert_eq!(scores.len(), 1);
}
```

但如果创建后暂时不插入任何东西，编译器无法猜出类型，就需要标注。空映射可以写成：

```rust
use std::collections::HashMap;

fn main() {
    let empty: HashMap<String, u32> = HashMap::new();
    assert!(empty.is_empty());
}
```

也可以从一组键值对直接构造：

```rust
use std::collections::HashMap;

fn main() {
    let scores = HashMap::from([("Ada", 10), ("Lin", 20)]);
    assert_eq!(scores.len(), 2);
}
```

`HashMap::from([...])` 接收一个数组，数组元素是 `(键, 值)` 形式的**元组（tuple，把几个值按顺序打包成的组合值）**。

**不要依赖 `{:?}` 打印出来的顺序。** 上面第一个程序可能打印 `{"Lin": 20, "Ada": 10}`，也可能打印 `{"Ada": 10, "Lin": 20}`。`HashMap` 不承诺插入顺序，原因在第八节。调试时看内容即可，写测试时要比较集合本身，或者先排序再比较（第九节）。

## 三、插入、替换与所有权

### 3.1 insert 返回旧值

`insert` 不只是“放进去”，它返回一个 `Option<V>`：

- 键原本不存在：插入成功，返回 `None`。
- 键原本存在：旧值被新值替换，返回 `Some(旧值)`。

```rust
use std::collections::HashMap;

fn main() {
    let mut scores = HashMap::new();

    let first = scores.insert(String::from("Ada"), 10);
    assert_eq!(first, None);

    let second = scores.insert(String::from("Ada"), 30);
    assert_eq!(second, Some(10)); // 旧值被交还出来

    assert_eq!(scores.get("Ada"), Some(&30));
    assert_eq!(scores.len(), 1); // 仍然是 1 条，不是 2 条
}
```

这个返回值有实际用途：如果“键已经存在”在业务上是错误，就可以用它检测，而不是默默覆盖。

### 3.2 键和值会被移动进 map

`HashMap` **拥有**它的键和值。对非 `Copy` 类型（如 `String`），`insert` 会把所有权移进映射：

```rust,ignore
// 片段：省略 use 和 main，只为看清所有权变化。
let key = String::from("Ada");
let value = String::from("compiler");
let mut map = HashMap::new();

map.insert(key, value); // key 和 value 的所有权移入 map

// 从这里开始，key 和 value 都不能再使用：它们已经“搬走”了。
```

如果之后还需要键的文本，有三个选择：

1. **在移动之前用完。** 例如先打印，再 `insert`。
2. **明确复制一份。** `map.insert(key.clone(), value)` 把副本放进映射，原变量仍可用。复制有内存（程序运行时存放数据的地方）和时间成本，只在确实需要两份时使用。
3. **不要构造拥有型键。** 如果数据本来就有寿命更长的文本，可以让映射借用它：`HashMap<&str, V>`。第 10 节的词频例子会对比这两种设计。

注意 `i32` 这类 `Copy` 类型不受影响：复制一个整数没有“搬走”的概念。

```rust
use std::collections::HashMap;

fn main() {
    let mut map = HashMap::new();
    let score = 10; // i32 是 Copy 类型
    map.insert(String::from("Ada"), score);
    println!("{score}"); // 仍然可用：插入的是副本
}
```

### 3.3 键本身在 map 里不能被修改

映射不会把键的 `&mut` 借给你。原因很实际：键放在哪个桶，是由它的哈希值决定的；如果键的内容变了、哈希值跟着变，它却还待在旧桶里，之后按新内容查找就找不到了。标准库从 API 上直接禁止这件事，而不是要求程序员自觉。遍历时你拿到的是 `&K`（只读），值才可以拿到 `&mut V`。

## 四、查找：get、get_mut 与借用查找

### 4.1 get 返回 Option<&V>

```rust
use std::collections::HashMap;

fn main() {
    let mut roles = HashMap::new();
    roles.insert(String::from("alice"), String::from("admin"));
    roles.insert(String::from("bob"), String::from("reader"));

    assert_eq!(roles.get("alice").map(String::as_str), Some("admin"));
    assert_eq!(roles.get("carol"), None);
    assert!(roles.contains_key("bob"));
    assert!(!roles.contains_key("carol"));
}
```

`get` 返回 `Option<&V>`，不是 `Option<V>`：

- `None` 表示键不存在；`Some(&值)` 表示存在，但只借给你看。
- 值可能很大、可能不能复制（例如 `String`、`Vec`），映射仍然拥有它。直接返回一份拷贝既昂贵，也改变了“谁拥有数据”的事实。
- 需要文本时用 `.map(String::as_str)` 把 `&String` 变成 `&str`，不复制内容。

`contains_key` 只回答“在不在”，不返回值。它适合只做判断的场合。

### 4.2 get_mut 借来修改

```rust
use std::collections::HashMap;

fn main() {
    let mut level: HashMap<String, u32> = HashMap::new();
    level.insert(String::from("Ada"), 3);

    if let Some(points) = level.get_mut("Ada") {
        *points += 10; // points 是 &mut u32，用 * 写回它指向的位置
    }

    assert_eq!(level.get("Ada"), Some(&13));
}
```

`get_mut` 返回 `Option<&mut V>`。`if let Some(points)` 里的 `points` 是可写引用；对数字要写 `*points += 10`，因为 `points` 本身是“指向 u32 的引用”，`*` 表示“它指向的那个值”。

### 4.3 用 &str 查找 HashMap<String, V>

```rust
use std::collections::HashMap;

fn main() {
    let mut votes: HashMap<String, u32> = HashMap::new();
    votes.insert(String::from("rust"), 3);

    // 键是 String，但查询可以直接用 &str：不会创建新 String。
    assert_eq!(votes.get("rust"), Some(&3));
    assert!(votes.contains_key("rust"));
}
```

为什么可以这样？`String` 拥有文本，`&str` 是对文本的借用视图（第三课）。标准库为“拥有型键”和“兼容的借用视图”之间定义了一条协议：如果两者表示同一段文本，它们的相等比较和哈希值必须一致。`get` 接受借用视图，内部用这个视图去计算哈希、比较内容，所以不需要为了查询临时 `String::from("rust")`。

这条协议在 Rust 里由 `Borrow` 这个 trait（一组行为约定的名字，第 12 课详讲）表达。现在够用的理解是：**`HashMap<String, V>` 可以用 `&str` 查，`HashMap<Vec<T>, V>` 可以用 `&[T]` 查，前提是借用视图与被查询的键表示同一份内容。** 自己定义类型时不要手工破坏这条一致性（第七节）。

### 4.4 索引语法会在缺失时 panic

`HashMap` 也支持方括号索引：

```rust
use std::collections::HashMap;

fn main() {
    let mut votes: HashMap<String, u32> = HashMap::new();
    votes.insert(String::from("rust"), 3);

    println!("{}", votes["rust"]); // 3
    println!("{}", votes["go"]); // 运行到这里会 panic：no entry found for key
}
```

`votes["go"]` 的静态类型（编译时就确定的类型）是 `u32`（不是引用），但它要求键必须存在；不存在时程序会**中止（panic）**。这和 `Vec` 越界是同一类问题：编译器无法预知运行时的键。

选择规则很简单：

- **键不存在是程序 bug**（例如内部约定保证它一定存在）：索引可以让 bug 立刻暴露。
- **键不存在是正常输入**（用户输入、文件内容、网络数据）：用 `get`，把“没有”当成正常分支处理。

主练习的 `count` 就属于后者：查询一个从未出现过的标签是正常情况，必须返回 0 而不是 panic。

### 4.5 借用还没结束时不能修改 map

```rust,compile_fail
use std::collections::HashMap;

fn main() {
    let mut votes: HashMap<String, u32> = HashMap::new();
    votes.insert(String::from("rust"), 1);

    let count = votes.get("rust");
    votes.insert(String::from("cli"), 1);
    println!("{count:?}");
}
```

`count` 借用了映射里的值；`insert` 需要独占修改整张表。只要 `count` 之后还会被使用，借用检查器（Rust 编译器里负责检查借用规则的部分）就不允许插入。完整报错见 11.2 节。

实际修法有三种，按语义选择：

1. 先复制出需要的小值，再修改：`let count = votes.get("rust").copied();`。`u32` 是 `Copy`，复制很便宜。
2. 把修改挪到旧引用最后一次使用之后。
3. 如果本来就是“有就加一、没有就插入 1”，用下一节的 Entry API，一次解决。

## 五、Entry API：一次定位，顺便修改

### 5.1 计数：先看笨办法

统计选票是典型需求。不用 Entry 时，常见写法是“先查，再决定”：

```rust,ignore
// 片段：tally 是 HashMap<&str, u32>，name 是 &str。
if let Some(count) = tally.get_mut(name) {
    *count += 1;
} else {
    tally.insert(name, 1);
}
```

这段代码是对的，但做了两次查找：`get_mut` 找一次；键不存在时，`insert` 再找一次。更糟的是，很多初学者会写成先 `get` 拿引用、再 `insert`，直接触发 4.5 节的借用错误。

**Entry API** 把“这个键对应的槽位”本身交给你，存在与不存在两种情况在同一个值上处理，只定位一次。

```rust
use std::collections::HashMap;

fn main() {
    let ballots = ["rust", "go", "rust", "c", "rust"];
    let mut tally: HashMap<&str, u32> = HashMap::new();

    for name in ballots {
        let count = tally.entry(name).or_insert(0);
        *count += 1;
    }

    assert_eq!(tally.get("rust"), Some(&3));
    assert_eq!(tally.get("go"), Some(&1));
    assert_eq!(tally.get("c"), Some(&1));
}
```

拆开看：

- `tally.entry(name)`：向映射申请 `name` 对应的槽位，返回一个 `Entry` 值。`Entry` 是一个**枚举（enum，表示“几种情况之一”的类型）**，它要么表示“这个键已经存在”（`Occupied`），要么表示“这个键还不存在”（`Vacant`）。
- `.or_insert(0)`：如果槽位是空的，插入 `0`；如果已经有值，就什么也不插入。无论哪种情况，都返回那个槽位的**可变引用** `&mut u32`。
- `*count += 1`：通过引用把计数加一。

注意这里 `HashMap<&str, u32>` 的键是借用视图，`ballots` 里的字符串字面量活得足够久，所以映射可以借用它们。如果键需要离开这段文本继续存在，就要用拥有型 `String`（第 10 节）。

### 5.2 and_modify：只对已存在的值动手

```rust
use std::collections::HashMap;

fn main() {
    let mut bonus: HashMap<String, u32> = HashMap::new();
    bonus.insert(String::from("Ada"), 10);

    bonus
        .entry(String::from("Ada"))
        .and_modify(|score| *score += 5)
        .or_insert(1);

    assert_eq!(bonus.get("Ada"), Some(&15));

    bonus
        .entry(String::from("Lin"))
        .and_modify(|score| *score += 5)
        .or_insert(1);

    assert_eq!(bonus.get("Lin"), Some(&1));
}
```

- `and_modify(闭包)` 只在 `Occupied` 时执行闭包；`Vacant` 时跳过。**闭包（closure）** 是一段可以像值一样传递的代码，`|score| *score += 5` 接收槽位里的 `&mut u32`。
- 后面的 `.or_insert(1)` 只在 `Vacant` 时插入。
- `and_modify` 返回 `Entry`，`or_insert` 返回 `&mut V`；两者都能继续链式调用，所以可以接在一起。

### 5.3 or_insert_with：默认值很贵时才延迟构造

`or_insert(value)` 的参数**在调用方法之前就会被计算**。即使键已经存在、根本不需要默认值，`Vec::new()` 这样的表达式也已经执行了。如果构造默认值很贵，用 `or_insert_with` 接收一个闭包：

```rust
use std::collections::HashMap;

fn main() {
    let mut index: HashMap<String, Vec<String>> = HashMap::new();

    index
        .entry(String::from("rust"))
        .or_insert_with(Vec::new)
        .push(String::from("ownership"));

    index
        .entry(String::from("rust"))
        .or_insert_with(Vec::new)
        .push(String::from("borrowing"));

    assert_eq!(index.get("rust").map(Vec::len), Some(2));
}
```

闭包只在 `Vacant` 时调用。对已经存在的键，`Vec::new()` 一次也不会执行。

`or_insert_with_key` 更进一步：闭包接收键的引用，可以在“键不存在”时用键本身构造值，而不必额外复制键：

```rust
use std::collections::HashMap;

fn main() {
    let mut lengths: HashMap<String, usize> = HashMap::new();

    lengths
        .entry(String::from("rust"))
        .or_insert_with_key(|key| key.len());

    assert_eq!(lengths.get("rust"), Some(&4));
}
```

`key` 的类型是 `&String`，`key.len()` 返回字节数（第三课讲过：`str` 的长度按字节算，ASCII 文本的字节数等于字符数）。

### 5.4 显式匹配 Occupied 与 Vacant

需要更复杂的替换逻辑时，可以直接 `match` 这两种情况：

```rust
use std::collections::HashMap;
use std::collections::hash_map::Entry;

fn main() {
    let mut counts: HashMap<String, u32> = HashMap::new();
    counts.insert(String::from("rust"), 2);

    match counts.entry(String::from("rust")) {
        Entry::Occupied(mut slot) => {
            *slot.get_mut() += 10;
            println!("原来有 {}，现在是 {}", slot.key(), slot.get());
        }
        Entry::Vacant(slot) => {
            slot.insert(10);
        }
    }

    assert_eq!(counts.get("rust"), Some(&12));
}
```

- `Entry::Occupied(mut slot)`：`slot` 可以查看键（`key()`）、读值（`get()`）、改值（`get_mut()`）、替换值（`insert` 返回旧值）。
- `Entry::Vacant(slot)`：`slot` 只能插入一个新值。
- 枚举的**变体（variant）** 是枚举的每一种可能取值；`Entry::Occupied` 和 `Entry::Vacant` 就是 `Entry` 的两个变体。

`Entry` 或者它返回的 `&mut V` 还活着时，它借用了整张映射的一部分。这时不能对同一张映射再做另一次借用或修改。

### 5.5 entry 会拿走键的所有权

这一点很容易被忽略：`entry(key)` 的参数类型就是 `K`，键被**按值传入**。即使这个键最后落在 `Occupied` 分支、映射里本来就有它，传进去的那个 `String` 也已经被移动，不能再用。

```rust,compile_fail
use std::collections::HashMap;

fn main() {
    let word = String::from("rust");
    let mut counts: HashMap<String, u32> = HashMap::new();
    *counts.entry(word).or_insert(0) += 1;
    println!("{word}");
}
```

如果之后还需要 `word`，顺序要改成“先用完再移动”，或者传 `word.clone()`，或者一开始就让映射借用 `&str`。第十二节的代码预测会再考一次。

## 六、HashSet：只关心“在不在”

### 6.1 唯一性与 insert 的返回值

`HashSet<T>` 可以理解成“只有键、没有值的 `HashMap`”。它只保存每个值一次，适合去重和成员判断。

```rust
use std::collections::HashSet;

fn main() {
    let mut tags = HashSet::new();

    assert!(tags.insert("rust")); // 第一次插入：新元素，返回 true
    assert!(tags.insert("cli"));
    assert!(!tags.insert("rust")); // 已经存在：不重复保存，返回 false

    assert_eq!(tags.len(), 2);
    assert!(tags.contains("cli"));
    assert!(!tags.contains("web"));

    assert!(tags.remove("cli")); // 确实删掉了
    assert!(!tags.remove("cli")); // 本来就不在
    assert_eq!(tags.len(), 1);
}
```

`insert` 返回 `bool`（布尔值，只有 `true` 和 `false` 两种取值）：`true` 表示这次真的新增了，`false` 表示元素早就存在。这个布尔值能直接表达“是不是第一次见到”，不需要先 `contains` 再插入（那会查两次）。

### 6.2 从一组数据去重

```rust
use std::collections::HashSet;

fn main() {
    let names = vec!["Ada", "Lin", "Ada", "Kai"];
    let unique: HashSet<&str> = names.iter().copied().collect();

    assert_eq!(unique.len(), 3);
    assert!(unique.contains("Ada"));
}
```

这里用到了几个第 10 课才系统讲的操作；先给临时解释。这些操作都属于**迭代器（iterator，按顺序逐个产出元素的值）**：

- `names.iter()` 逐个借出元素，得到 `&&str`。
- `.copied()` 把 `&&str` 复制成 `&str`（`&str` 是 `Copy`，复制的是“指向文本的引用”，不是文本本身）。
- `.collect()` 把一串元素收集进目标集合；目标类型由左边的 `HashSet<&str>` 标注决定。重复元素在收集时自动只保留一个。

如果 `names` 是 `Vec<String>`，`HashSet<String>` 会拥有这些字符串；也可以收集 `HashSet<&str>` 借用它们，但集合不能比 `names` 活得久。

### 6.3 集合运算

```rust
use std::collections::HashSet;

fn main() {
    let left: HashSet<i32> = HashSet::from([1, 2, 3]);
    let right: HashSet<i32> = HashSet::from([3, 4]);

    let union: HashSet<i32> = left.union(&right).copied().collect();
    let intersection: HashSet<i32> = left.intersection(&right).copied().collect();
    let difference: HashSet<i32> = left.difference(&right).copied().collect();
    let symmetric: HashSet<i32> = left.symmetric_difference(&right).copied().collect();

    assert_eq!(union, HashSet::from([1, 2, 3, 4]));
    assert_eq!(intersection, HashSet::from([3]));
    assert_eq!(difference, HashSet::from([1, 2])); // 在 left、不在 right
    assert_eq!(symmetric, HashSet::from([1, 2, 4])); // 恰好属于一边

    assert!(HashSet::from([3]).is_subset(&left));
    assert!(left.is_superset(&HashSet::from([3])));
    assert!(left.is_disjoint(&HashSet::from([9, 10])));
}
```

- `union`（并集）：出现在任意一边的元素。
- `intersection`（交集）：同时出现在两边的元素。
- `difference`（差集）：`left.difference(&right)` 是在 `left` 中但不在 `right` 中的元素。反过来写就得到另一边的差集。
- `symmetric_difference`（对称差）：只属于其中一边、不同时属于两边的元素。
- `is_subset`：左边是不是右边的子集（左边的每个元素都在右边里）。
- `is_superset`：左边是不是右边超集（右边每个元素都在左边里）。
- `is_disjoint`：两边有没有共同元素；一个都没有时为 `true`。

这些运算**不修改**原集合，返回的是迭代器（第 10 课），逐个借出元素。元素是 `Copy` 时用 `.copied()` 复制出来再 `collect`。元素是 `String` 时，结果里会是 `&String` 或 `&str` 引用，或者需要显式 `clone` 成新的 `HashSet<String>`——这取决于结果要借用原集合，还是要独立存在。

当需求是“成员唯一”时，不要用 `Vec` 手工查重再 `push`：那既是 $O(n)$ 的重复扫描，又把“唯一”这条规则散落在每个调用点。`HashSet` 把规则放进类型里，插入时自动执行。

## 七、什么能当键：Eq 与 Hash

`HashMap` 的键必须满足两个行为约定：

- **`Eq`**：键之间可以用 `==` 比较，而且这种相等是稳定的等价关系（自己等于自己；`a == b` 时 `b == a`；`a == b`、`b == c` 时 `a == c`）。
- **`Hash`**：键可以被哈希函数算成一个整数。

为什么两者都要？哈希表先用 `Hash` 找到候选桶，再用 `Eq` 确认桶里的键是不是真的等于要查的键。只有哈希、没有相等比较，就无法处理碰撞（第八节）；只有相等、没有哈希，就无法快速定位。

最基本的要求是：

```text
如果 a == b，那么 hash(a) 必须等于 hash(b)
```

反过来不要求：不同的键可以算出同一个哈希值，这叫碰撞。碰撞会让查找变慢，但不应该改变结果。

Rust 的整数、`bool`、`char`、`String`、`&str` 以及由它们组成的元组都已经实现了 `Eq + Hash`。自定义类型可以用 `derive` 自动生成：

```rust
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct UserId(u64);

fn main() {
    let mut names = HashMap::new();

    names.insert(UserId(7), String::from("Ada"));
    names.insert(UserId(9), String::from("Lin"));

    assert_eq!(names.get(&UserId(7)).map(String::as_str), Some("Ada"));
    assert_eq!(names.get(&UserId(8)), None);
}
```

- `struct UserId(u64);` 是**元组结构体（tuple struct）**：只有一个字段，字段没有名字，用 `.0` 访问。它给 `u64` 起了一个新名字，叫 **newtype（新类型）** 惯用法。好处是 `UserId` 和普通的 `u64`、其他 `u64` 包装类型在类型层面区分开，不会把“用户编号”误传给“订单编号”。
- `#[derive(...)]` 是**派生宏（derive macro）**：让编译器按字段自动生成实现。`PartialEq` 生成 `==`，`Eq` 声明这是完整等价关系，`Hash` 生成哈希计算，`Debug` 生成 `{:?}` 打印，`Clone` 生成 `.clone()`。宏的机制第 20 课讲，这里只需要知道这一行会生成正确的配套实现。
- `names.get(&UserId(7))` 传入的是临时键的引用，`get` 只借用它来查找，不会把它存进映射。

**不要手工实现互相不一致的 `Eq` 和 `Hash`。** 如果两个值相等、哈希值却不同，它们可能被放进不同的桶，`get` 就会漏掉本应找到的条目。派生宏会按字段同时生成两者，保持一致。

**浮点数不能直接当键。** `f64` 没有实现 `Eq` 和 `Hash`，因为 `NaN != NaN`（NaN 是“不是一个数”的特殊浮点值），破坏了“自己等于自己”的等价关系；哈希表无法在这种关系上可靠工作。11.3 节会看到真实报错。如果业务真的需要按浮点数分组，先想清楚领域里的相等规则：是精确位模式相等，还是四舍五入到某个精度后相等？把规则固定成整数或可比较的键，比强行把 `f64` 塞进映射更安全。

## 八、哈希表内部如何工作（概念）

这一节建立概念模型，不讨论标准库的具体实现细节。

### 8.1 从键到桶

假设映射内部有一个固定长度的**桶数组**，每个桶可以放一个或多个条目。插入 `("rust", 3)` 时：

```text
键 "rust"
   │
   │  哈希函数：把任意数据变成一个整数
   ▼
   哈希值 0x9e37...
   │
   │  对桶的数量取余，得到 0..桶数-1 的下标
   ▼
桶数组：[ 空 ][ ("rust", 3) ][ 空 ][ 空 ]
```

查找时重复同样的计算：哈希 → 下标 → 到那个桶里找。如果桶里没有别的键，一次就比较完，和表里总共有多少条目无关。这就是“平均 $O(1)$”的来源。

### 8.2 碰撞与相等比较

桶的数量远小于所有可能的键，所以不同键算出同一个桶是必然的。这叫**碰撞（collision）**。处理方式不是“覆盖”，而是：

1. 到候选桶里，用 `Eq` 逐个比较键。
2. 找到相等的键：返回值（或替换值）。
3. 没找到：按实现规定的探测规则看下一个候选位置，直到找到空位或匹配项。

标准库当前使用**开放寻址（open addressing）** 一类的方法：条目都放在桶数组里，冲突时按规则探测下一个位置，而不是在每个桶外挂一条链表。具体策略属于实现细节，可能随版本变化；你需要记住的是“哈希定位 + 相等确认”这个模型。

最坏情况下，如果所有键都碰撞，查找会退化成逐个比较，$O(n)$。平均性能依赖两件事：哈希函数把键分散得足够均匀；表不要太满。

### 8.3 扩容与引用失效

当条目多到一定程度（装载因子过高，“装载因子”指已用位置占全部位置的比例），继续探测会越来越慢。映射会申请一个更大的桶数组，把所有条目**重新哈希**、搬到新位置。这个过程叫**扩容（resize / rehash）**。

这解释了两件前面出现过的事：

- **插入可能很慢。** 大多数插入很快，但触发扩容的那一次要搬动全部条目。这是“平均”很快、单次可能慢的典型例子。
- **拿着值的引用时不能插入。** 扩容会把条目搬到新地址，旧引用就悬空了（指向一个已经无效的位置）。第五课的 `Vec` 扩容也是同样的道理，借用检查器用同一条规则提前阻止：`&V` 活着时，不能对整张映射做 `&mut` 操作（4.5 节的 E0502）。

### 8.4 为什么遍历顺序不稳定

标准库的 `HashMap` 在创建时会给哈希函数取一个随机种子（种子是哈希计算开始前混入的一个随机数）。这带来两个结果：

1. **不同运行、不同映射的遍历顺序可能不同。** 即使插入顺序完全一样，桶的布局也可能不同。`HashMap` 不承诺按插入顺序、键的字典序（逐字符比较的顺序，像查字典）或任何其他顺序遍历。
2. **攻击者难以预先构造大量碰撞。** 如果哈希函数固定，恶意输入可以故意制造碰撞，让服务器每次查找都退化。随机种子让这种攻击难以预测。

所以：

- 测试不要写“收集出来的 `Vec` 等于某个固定顺序”。
- 需要稳定输出时，先收集条目再排序（第 9.3 节），或者改用 `BTreeMap`（按 key 有序）。
- `HashMap` 的相等比较（`==`）与顺序无关：两个映射只要键值对相同就相等，哪怕遍历顺序不同。这也是测试里直接比较两个 `HashMap` 的原因。

## 九、删除、批量保留与遍历

### 9.1 remove 与 remove_entry

```rust
use std::collections::HashMap;

fn main() {
    let mut stock: HashMap<String, u32> = HashMap::new();
    stock.insert(String::from("apple"), 3);
    stock.insert(String::from("milk"), 1);

    assert_eq!(stock.remove("apple"), Some(3)); // 返回被删除的值
    assert_eq!(stock.remove("apple"), None); // 已经不存在
    assert_eq!(stock.len(), 1);

    stock.insert(String::from("tea"), 2);
    assert_eq!(
        stock.remove_entry("tea"),
        Some((String::from("tea"), 2)) // 同时拿回键和值
    );
    assert_eq!(stock.len(), 1);
    assert_eq!(stock.get("milk"), Some(&1)); // milk 仍在表里
}
```

`remove` 接收借用视图（`&str` 可以查 `String` 键），返回 `Option<V>`；键不存在时返回 `None`，不会 panic。如果还需要拿回键本身（例如把条目转移到另一个集合），用 `remove_entry`，它返回 `Option<(K, V)>`。

### 9.2 retain：按条件批量删除

```rust
use std::collections::HashMap;

fn main() {
    let mut stock: HashMap<String, u32> = HashMap::new();
    stock.insert(String::from("apple"), 3);
    stock.insert(String::from("milk"), 0);
    stock.insert(String::new(), 5);

    stock.retain(|name, count| !name.is_empty() && *count > 0);

    assert_eq!(stock.len(), 1);
    assert_eq!(stock.get("apple"), Some(&3));
}
```

`retain` 对每个条目调用一次闭包；闭包返回 `true` 就保留，`false` 就删除。闭包的两个参数是 `&K`（键的只读借用）和 `&mut V`（值的可变借用）。因此可以在判断时顺便修改值，但不能改键——原因还是 8.3 节：键的哈希必须保持稳定。

`retain` 在遍历过程中安全地删除元素，不会出现“边递增索引边删除、结果跳过元素”的问题（第五课讲过那个坑）。

### 9.3 遍历与稳定输出

```rust
use std::collections::HashMap;

fn main() {
    let mut scores: HashMap<String, u32> = HashMap::new();
    scores.insert(String::from("Ada"), 10);
    scores.insert(String::from("Lin"), 20);

    // 借用遍历：拿到的 name 是 &String，score 是 &u32；映射不变。
    for (name, score) in &scores {
        println!("{name}: {score}");
    }

    // 需要稳定顺序时，先收集条目，再排序。
    let mut entries: Vec<(&String, &u32)> = scores.iter().collect();
    entries.sort_unstable();
    for (name, score) in entries {
        println!("{name}: {score}");
    }
}
```

- `for (name, score) in &scores`：借出每个 `(&K, &V)`。映射仍归 `scores` 所有。
- `for (name, score) in &mut scores`：借出 `(&K, &mut V)`，可以改值，不能改键。
- `for (name, score) in scores`：**消费**映射，拿走每个 `(K, V)`。之后 `scores` 失效。
- `scores.iter().collect()`：把借用的条目收集进 `Vec`。第 10 课会正式讲迭代器，这里先当作“把每个条目装进 Vec”的写法。
- `entries.sort_unstable()`：对元组排序。元组按字段依次比较：先比 `name`，名字相同再比 `score`。键在映射里唯一，所以实际只按名字排序。
- `sort_unstable` 不保证相等元素的相对顺序，但这里没有“相等元素”（键唯一），所以完全够用，而且比 `sort` 少一些内存开销。

也可以分别取键和值：

```rust
use std::collections::HashMap;

fn main() {
    let scores = HashMap::from([("Ada", 10), ("Lin", 20)]);

    let mut names: Vec<&str> = scores.keys().copied().collect();
    names.sort_unstable();
    assert_eq!(names, ["Ada", "Lin"]);

    let total: u32 = scores.values().sum();
    assert_eq!(total, 30);
}
```

`keys()`、`values()` 返回迭代器；`sum()` 把一串数字加起来。这些是第 10 课的内容，这里只作为工具使用。

### 9.4 HashMap、BTreeMap、HashSet 与 Vec 怎么选

| 需求 | 常见选择 | 原因 |
|---|---|---|
| 按 key 快速查找，不关心顺序 | `HashMap` | 平均 $O(1)$ 查找，最常用的映射 |
| key 必须有序，或需要范围查询、稳定遍历 | `BTreeMap` | 按 key 排序保存，查找 $O(\log n)$，遍历顺序确定 |
| 只关心“某个值在不在”、需要去重 | `HashSet` | 唯一性由类型保证，支持集合运算 |
| 数据量很小、顺序重要、主要整体遍历 | `Vec` | 连续内存，简单直接，常数开销小 |

`BTreeMap` 和 `HashMap` 的常用方法（`insert`、`get`、`entry`、`remove`）名字基本一致，区别在于前者要求键实现 `Ord`（可排序），并保证按 key 从小到大遍历。不要为了“说不定以后要排序”就默认选 `BTreeMap`；先看语义：**顺序是不是需求的一部分？** 是，就用有序容器；不是，就用 `HashMap`，需要输出时再排序。

## 十、完整示例：词频统计

下面把本课内容串起来：统计一段文本里每个单词出现的次数，并按单词字典序打印。这个例子和主练习不同，可以放心逐行阅读。

```rust
use std::collections::HashMap;

/// 统计每个单词出现的次数。
/// 返回的映射拥有单词（String），不借用输入文本。
fn word_counts(text: &str) -> HashMap<String, u32> {
    let mut counts = HashMap::new();

    for word in text.split_whitespace() {
        *counts.entry(word.to_owned()).or_insert(0) += 1;
    }

    counts
}

fn main() {
    let text = "rust makes ownership explicit rust is fast";
    let counts = word_counts(text);

    // HashMap 的遍历顺序不稳定：先收集条目，再排序，输出才可复现。
    let mut entries: Vec<(&str, u32)> = Vec::new();
    for (word, count) in &counts {
        entries.push((word.as_str(), *count));
    }
    entries.sort_unstable();

    for (word, count) in entries {
        println!("{word}: {count}");
    }
}
```

输出：

```text
explicit: 1
fast: 1
is: 1
makes: 1
ownership: 1
rust: 2
```

数据怎么流动：

1. `text.split_whitespace()` 按空白把文本切成一个个单词，返回一个迭代器；`for` 每次取出一个 `&str`，它借用 `text` 中的一段。
2. `word.to_owned()` 创建一份拥有型 `String`。为什么不让映射借用 `&str`？因为 `counts` 要作为返回值离开 `word_counts`；如果键借用 `text`，返回的映射就必须带着“不能比 `text` 活得久”的限制。选择拥有型键让结果可以独立使用，代价是每个不同的单词分配（向系统申请）一次内存。
3. `counts.entry(...).or_insert(0)` 拿到计数槽位，`*... += 1` 加一。同一个单词第二次出现时，`entry` 找到已有槽位，不会再分配默认值。
4. `&counts` 借出 `(&String, &u32)`。`word.as_str()` 把 `&String` 变成 `&str`，`*count` 复制出 `u32`（`Copy`），装进 `Vec`。
5. `entries.sort_unstable()` 按元组排序。键唯一，所以按单词字典序输出。
6. 如果直接 `for (word, count) in &counts` 打印，顺序可能每次运行都不同——这不是 bug，是哈希表的约定。

对比另一种设计：如果映射只在 `text` 还活着的作用域内使用，可以让键借用文本，省掉每个单词的 `String` 分配：

```rust,ignore
// 片段：借用型键。counts 不能比 text 活得久，也不能直接返回。
let mut counts: HashMap<&str, u32> = HashMap::new();
for word in text.split_whitespace() {
    *counts.entry(word).or_insert(0) += 1;
}
// 这里 counts 的键借用 text；text 必须先于 counts 被清理。
```

两种写法的取舍：

- **拥有型键 `String`**：结果可以返回、可以存进别的结构、可以比输入活得久；每个不同键一次分配。
- **借用型键 `&str`**：没有额外分配，查找也快；但映射的生命周期被输入文本限制住。

生命周期（引用能合法使用多久的规则）第 14 课系统讲。现在的判断标准是：**结果需要离开当前函数或数据吗？** 需要，就用拥有型键；只在局部临时统计，就借用。

### 从键值对收集映射时的重复键

`collect` 可以把一串 `(K, V)` 直接变成 `HashMap`：

```rust
use std::collections::HashMap;

fn main() {
    let pairs = [("a", 1), ("b", 2), ("a", 3)];
    let map: HashMap<&str, i32> = pairs.into_iter().collect();

    // 重复键 "a" 出现两次，后出现的值覆盖前一个。
    assert_eq!(map.get("a"), Some(&3));
    assert_eq!(map.len(), 2);
}
```

如果重复键在业务上是错误，不能依赖 `collect` 静默覆盖；要在插入时检查 `insert` 的返回值，或用 `entry` 区分 `Occupied` / `Vacant`，把错误报告出来。主练习的“完整替换”正是这类需要明确规则的场景。

## 十一、常见编译器诊断与排错

下面都是真实编译器或运行时的输出（行号来自最小示例，放在你的代码里会不同）。

### 11.1 E0382：插入后继续使用被移动的键

```rust,ignore
// 完整程序：插入后继续使用被移动的键。
use std::collections::HashMap;

fn main() {
    let key = String::from("Ada");
    let mut map = HashMap::new();
    map.insert(key, 10);
    println!("{key}");
}
```

真实报错：

```text
error[E0382]: borrow of moved value: `key`
 --> src/main.rs:7:16
  |
4 |     let key = String::from("Ada");
  |         --- move occurs because `key` has type `String`, which does not implement the `Copy` trait
5 |     let mut map = HashMap::new();
6 |     map.insert(key, 10);
  |                --- value moved here
7 |     println!("{key}");
  |                ^^^ value borrowed here after move
  |
help: consider cloning the value if the performance cost is acceptable
  |
6 |     map.insert(key.clone(), 10);
  |                   ++++++++
```

读法：

- 第一行给出错误码和一句话结论：`key` 被移动后又被借用。
- `move occurs because ... does not implement the Copy trait`：`String` 不是 `Copy`，赋值/传参是移动。
- `value moved here`：移动发生在 `insert` 的实参位置。
- `value borrowed here after move`：之后 `println!` 还想用 `key`。
- `help` 给出一个可选修法（`clone`），但先想清楚是不是真的要保留两份文本。

修法按语义选：

1. 不再需要原变量：删掉最后的 `println!`。
2. 需要文本，但映射也要拥有它：`key.clone()`，明确付出复制成本。
3. 只需要按名字查：把映射设计成借用型 `HashMap<&str, _>`，或之后用 `map.get("Ada")` 查询，不要保存旧键。

### 11.2 E0502：借用还没结束就修改 map

```rust,ignore
// 片段：完整程序见 4.5 节。
let count = votes.get("rust");
votes.insert(String::from("cli"), 1);
println!("{count:?}");
```

真实报错：

```text
error[E0502]: cannot borrow `votes` as mutable because it is also borrowed as immutable
 --> src/main.rs:8:5
  |
7 |     let count = votes.get("rust");
  |                 ----- immutable borrow occurs here
8 |     votes.insert(String::from("cli"), 1);
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ mutable borrow occurs here
9 |     println!("{count:?}");
  |                ----- immutable borrow later used here
```

读法：编译器标出了三处——不可变借用在哪、可变借用在哪、旧借用后来在哪被使用。三处共同决定冲突。

修法：

1. 复制出需要的值：`let count = votes.get("rust").copied();`（仅当 `V: Copy`）。
2. 把 `insert` 挪到 `count` 最后一次使用之后。
3. 用 `entry` 把“查 + 改”合并成一次借用：

```rust
use std::collections::HashMap;

fn main() {
    let mut votes: HashMap<String, u32> = HashMap::new();
    votes.insert(String::from("rust"), 1);

    *votes.entry(String::from("rust")).or_insert(0) += 1;

    assert_eq!(votes.get("rust"), Some(&2));
}
```

### 11.3 E0277：浮点数不能当 HashMap 的键

```rust,ignore
// 片段：prices 的类型由插入操作推断为 HashMap<f64, &str>。
use std::collections::HashMap;

fn main() {
    let mut prices = HashMap::new();
    prices.insert(1.5, "coffee");
}
```

真实报错（节选）：

```text
error[E0277]: the trait bound `{float}: Eq` is not satisfied
 --> src/main.rs:5:12
  |
5 |     prices.insert(1.5, "coffee");
  |            ^^^^^^ the trait `Eq` is not implemented for `{float}`
  |
note: required by a bound in `HashMap::<K, V, S, A>::insert`
  |
  |     K: Eq + Hash,
  |        ^^ required by this bound in `HashMap::<K, V, S, A>::insert`

error[E0277]: the trait bound `{float}: Hash` is not satisfied
```

两条错误一起出现，因为 `insert` 要求 `K: Eq + Hash`，而 `f64` 两者都没有。`{float}` 是编译器对“某个浮点类型”的统称。

不要用 `unsafe` 或手工实现去绕过；先确定业务上的相等规则。常见做法是把浮点数按固定精度转换成整数（例如金额用“分”存 `i64`），或者把原始位模式转成 `u64` 后当键，并明确说明“这是位相等，不是数学相等”。

### 11.4 E0277：自定义类型忘了派生 Eq 和 Hash

```rust,ignore
// 片段：UserId 只定义了字段，没有派生比较和哈希。
use std::collections::HashMap;

struct UserId(u64);

fn main() {
    let mut names = HashMap::new();
    names.insert(UserId(7), String::from("Ada"));
}
```

真实报错（节选）：

```text
error[E0277]: the trait bound `UserId: Eq` is not satisfied
 --> src/main.rs:7:11
  |
7 |     names.insert(UserId(7), String::from("Ada"));
  |           ^^^^^^ the trait `Eq` is not implemented for `UserId`
  |
note: required by a bound in `HashMap::<K, V, S, A>::insert`
  |
  |     K: Eq + Hash,
  |        ^^ required by this bound in `HashMap::<K, V, S, A>::insert`
help: consider annotating `UserId` with `#[derive(Eq)]`
  |
3 + #[derive(Eq)]
4 | struct UserId(u64);
  |

error[E0277]: the trait bound `UserId: Hash` is not satisfied
```

编译器会分别报告缺失的 `Eq` 和 `Hash`。修法是给类型加派生（片段：只展示类型定义，省略了 `use` 和 `main`）：

```rust,ignore
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct UserId(u64);
```

如果字段里有 `f64`，派生也会失败，因为 `f64` 本身不满足 `Eq + Hash`；这时要回到业务规则，换一种键的表示。

### 11.5 运行时 panic：用索引查不存在的键

```text
thread 'main' (115684) panicked at src/main.rs:6:25:
no entry found for key
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

`panicked at` 后面是位置，`no entry found for key` 说明方括号索引的键不存在。编译器无法提前发现，因为键的值运行时才确定。来自外部输入的键一律用 `get`；只有内部不变量（始终必须成立的条件）保证存在时才用索引。

### 11.6 测试里的 not yet implemented

完成主练习之前运行测试，会看到：

```text
thread 'exercise::tests::new_index_is_empty' (116320) panicked at lessons/06-hash-collections/src/exercise.rs:38:9:
not yet implemented
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

这不是测试写错了，也不是编译错误。`todo!()` 是“这里还没写”的宏，运行到它就主动 panic，提醒你这里是留给你的任务。实现后这些失败会逐个消失。

区分三类失败：

1. **编译错误**：测试根本没跑起来。先修类型、签名、借用。
2. **`not yet implemented`**：测试跑到了还没实现的 `todo!()`。继续做练习。
3. **断言失败**（`assertion failed` / `left == right`）：实现运行了，但结果与期望不符。这是行为 bug，看断言里的左右值。

### 11.7 测试偶尔失败：依赖了遍历顺序

如果测试写成“把 `map.keys()` 收集成 `Vec` 再和固定数组比较”，它可能这台机器通过、另一台失败，甚至同一次运行的不同 `HashMap` 结果都不同。这是第八节的随机种子导致的，不是实现有随机 bug。修法：比较集合（`HashSet` 与 `HashSet`、`HashMap` 与 `HashMap` 直接 `==`），或先排序再比较。

## 十二、代码预测与解析

先自己判断“能否编译、输出什么”，再展开解析。

### 题目 1

```rust
use std::collections::HashMap;

fn main() {
    let mut map = HashMap::new();
    map.insert(String::from("a"), 1);
    map.insert(String::from("a"), 2);
    println!("{:?}", map.get("a"));
    println!("{}", map.len());
}
```

<details><summary>解析</summary>

输出：

```text
Some(2)
1
```

第二次 `insert` 用同一个键 `"a"`，旧值 `1` 被替换，映射仍然只有一条。`get` 返回 `Option<&i32>`，打印为 `Some(2)`。

</details>

### 题目 2

```rust
use std::collections::HashMap;

fn main() {
    let word = String::from("rust");
    let mut counts: HashMap<String, u32> = HashMap::new();
    *counts.entry(word).or_insert(0) += 1;
    println!("{word}");
}
```

<details><summary>解析</summary>

不能编译，报 E0382（11.1 节）。`entry(word)` 按值接收键，`word` 被移动；即使映射里原本没有 `"rust"`、最终确实插入了，原绑定也已经失效。`or_insert` 的返回值借用的是映射内部的值，不会把键还给你。

</details>

### 题目 3

```rust
use std::collections::HashMap;

fn main() {
    let mut map = HashMap::new();
    map.insert("a", 10);
    let value = map.get("a");
    println!("{value:?}");
    map.insert("b", 20);
    println!("{}", map.len());
}
```

<details><summary>解析</summary>

能编译，输出：

```text
Some(10)
2
```

`value` 的最后一次使用在第一个 `println!`，之后借用结束；第二个 `insert` 才发生，所以没有冲突。如果把 `map.insert("b", 20)` 挪到第一个 `println!` 之前，就会触发 E0502。

</details>

### 题目 4

```rust
use std::collections::HashSet;

fn main() {
    let mut set = HashSet::new();
    println!("{}", set.insert("rust"));
    println!("{}", set.insert("rust"));
    println!("{}", set.len());
}
```

<details><summary>解析</summary>

输出：

```text
true
false
1
```

第一次插入是新元素，返回 `true`；第二次元素已存在，不重复保存，返回 `false`；长度始终是 1。

</details>

### 题目 5

```rust
use std::collections::HashMap;

fn main() {
    let mut map = HashMap::new();
    map.insert("b", 1);
    map.insert("a", 2);
    let keys: Vec<&str> = map.keys().copied().collect();
    println!("{keys:?}");
}
```

<details><summary>解析</summary>

可能打印 `["a", "b"]`，也可能打印 `["b", "a"]`，甚至同一程序不同运行结果不同。`HashMap` 不保证遍历顺序，也不保证按插入顺序。要固定输出，先 `sort_unstable()`；要固定语义上的有序容器，用 `BTreeMap`。

</details>

## 十三、主练习：标签索引

### 13.1 场景与目标

一个笔记程序里，每篇文档有一个名字和一组标签。程序要同时回答两类问题：

- “某个标签被几篇文档使用？”（例如 `rust` 被 3 篇文档使用）
- “某篇文档现在有哪些标签？”

只用一个 `HashMap<String, HashSet<String>>`（文档 → 标签集合）也能回答第一个问题，但每次都要遍历所有文档、检查每篇的标签集合，查询多了很慢。更自然的做法是同时维护两张表。下面是类型定义的片段（省略了文件开头的 `use`、方法实现和测试）：

```rust,ignore
pub struct TagIndex {
    counts: HashMap<String, u32>,                // 标签 → 使用它的文档数
    documents: HashMap<String, HashSet<String>>, // 文档名 → 这篇文档的标签集合
}
```

两张表必须始终一致：`counts` 里标签 `rust` 的值，必须等于 `documents` 中标签集合包含 `rust` 的文档篇数。主练习的核心难点不是“会用 `HashMap`”，而是**在一次操作里正确维护两份互相依赖的数据，并且失败时不留半更新状态**。

两张表各自拥有自己的文本：`counts` 的键拥有一份标签文本，`documents` 的键拥有文档名，值集合里又为每个标签各存一份文本。同一个标签会同时存在于两张表里，这是用额外的空间换取“按标签计数”和“按文档查标签”都能快速完成；代价是任何一次增删都必须同时维护两边。`count(tag: &str)` 不需要构造 `String`，它只是借用传入的标签视图去查 `counts`，这和第 4.3 节讲的借用查找是同一件事。

代码在 `src/exercise.rs`。类型、字段和测试已经写好，你只负责替换各方法体里的 `todo!()`。

### 13.2 行为规格

方法签名与职责：

| 方法 | 输入 | 输出 | 关键规则 |
|---|---|---|---|
| `TagIndex::new()` | 无 | 空索引 | 两张表都为空 |
| `add_document(name, tags)` | 文档名 `String`，标签切片 `&[&str]` | `Result<(), IndexError>` | 规范化、同文档去重、完整替换、失败不改状态 |
| `remove_document(name)` | 文档名 `&str` | `bool` | 先 trim 名字；存在则撤销计数并删除；不存在返回 `false` |
| `count(tag)` | 已规范化的标签 `&str` | `u32` | 不存在返回 0，不 panic |
| `tags_for(name)` | 文档名 `&str` | `Option<&HashSet<String>>` | 先 trim 名字；只看不改；不存在返回 `None` |
| `documents_with(tag)` | 已规范化的标签 `&str` | `Vec<&str>` | 文档名借用内部数据，按字典序排列 |
| `most_common()` | 无 | `Option<(&str, u32)>` | 计数并列时返回字典序较小的标签 |

`IndexError` 是一个枚举，目前只有一个变体（片段：摘自 `src/exercise.rs`，省略了其余代码）：

```rust,ignore
pub enum IndexError {
    EmptyDocumentName,
}
```

规范化规则（`add_document` 的完整约定）：

1. **文档名**去掉两端空白后保存；`tags_for` 和 `remove_document` 查找时也先去掉两端空白。`"  guide  "` 和 `"guide"` 是同一篇文档。
2. 文档名去掉两端空白后为空：返回 `Err(IndexError::EmptyDocumentName)`，**索引完全不变**。
3. **标签**先去掉两端空白，再把 ASCII 大写字母 `A`–`Z` 转成小写，其他字符原样保留。ASCII 是一套用数字表示英文字母、数字和常见符号的编码；本课只要求转换这套编码里的大写字母。例如 `" RÜST "` 规范化成 `"rÜst"`，`Ü` 不在 A–Z 范围内，原样保留。
4. 规范化后为空的标签忽略。
5. 同一篇文档里，规范化后相同的标签只算一次：`["Rust", " rust ", "RUST"]` 只贡献 1 次计数。
6. 如果文档已存在，这次是**完整替换**：先撤销旧标签的计数，再按新标签计数；旧标签集合整个换成新集合。不是合并。
7. 某标签的计数降到 0 时，必须从 `counts` 中整个删除，不能留下计数为 0 的条目。
8. 用空标签列表替换一篇文档是合法的：文档仍然存在，但标签集合为空，所有旧标签计数被撤销。

关于第 3 条：标准库的 `str::to_ascii_lowercase` 正好实现“只把 A–Z 转小写”的规则；`str::to_lowercase` 会按完整 Unicode 规则转换（例如 `İ` 会变成两个字符），不符合本练习要求的“其他字符原样保留”。选择哪个方法会直接影响测试结果。

### 13.3 真实输入与期望输出

以下都是可以直接在测试里表达的调用序列：

```text
let mut index = TagIndex::new();

index.add_document(" guide ".into(), &[" Rust ", "rust", "CLI", "", "   "])
  → Ok(())
count("rust")            → 1
count("cli")             → 1
tags_for("guide")        → Some({"rust", "cli"})
tags_for("  guide  ")    → Some({"rust", "cli"}) // 查找前同样去掉两端空白
documents_with("rust")   → ["guide"]
most_common()            → Some(("cli", 1))    // rust 和 cli 都是 1，字典序 cli 更小

index.add_document("notes".into(), &[])
  → Ok(())
tags_for("notes")        → Some({})            // 文档存在，标签集合为空
count("rust")            → 1                    // notes 不影响其他文档

index.add_document("  ".into(), &["new"])
  → Err(IndexError::EmptyDocumentName)
count("new")             → 0                    // 失败后索引不变
tags_for("  ")           → None                 // trim 后为空，索引里没有这篇文档

index.add_document("guide".into(), &["cli"])    // 完整替换
  → Ok(())
count("rust")            → 0                    // 已从 counts 删除
count("cli")             → 1
tags_for("guide")        → Some({"cli"})
most_common()            → Some(("cli", 1))

index.remove_document("guide")  → true
count("cli")                    → 0
tags_for("guide")               → None
most_common()                   → None
index.remove_document("guide")  → false
```

注意最后 `most_common()` 返回 `None`：不是返回 `Some(("cli", 0))`。计数为 0 的条目不允许留在 `counts` 里，这一点测试会检查。

### 13.4 约束

- 只替换 `src/exercise.rs` 中各方法体里的 `todo!()`；**不要修改**类型定义、字段、方法签名或文件底部的测试。
- 字段保持私有，外部只能通过公开方法访问。
- 不添加依赖；只用标准库。
- 方法内部不要用 `unwrap`、`expect` 或 `panic!`：查询不存在的标签、删除不存在的文档都是正常输入。
- 所有借用返回值（`tags_for`、`documents_with`、`most_common`）必须借用索引内部数据，不要为了省事复制整个集合再返回。
- `add_document` 失败时不能留下部分更新；这是本练习最重要的约束。
- `documents_with` 和 `most_common` 的结果必须按题目规定排序/选择，不能依赖 `HashMap` 的遍历顺序。

### 13.5 推荐完成顺序

1. **`new`**：先让“空索引”测试通过。
2. **`count` 和 `tags_for`**：只读查询，实现简单，但需要先有数据才能测；可以暂时用测试里的调用顺序理解它们。
3. **`add_document` 的第一版**：只处理“新文档”，完成文档名规范化、标签规范化、同文档去重、写入两张表。
4. **`remove_document`**：撤销计数、删除文档、报告是否存在。
5. **`add_document` 的替换逻辑**：文档已存在时先撤销旧计数，再按新标签计数；确保零计数条目被删除。
6. **`documents_with`**：遍历文档表，找出包含该标签的文档，收集名字并排序。
7. **`most_common`**：遍历计数表，找最大值；并列时比较标签的字典序。

每一次都先跑对应测试，不要等全部写完再一起调试。实现顺序与测试的对应关系：

- `new_index_is_empty` 只需要 `new`。
- 规范化、去重、空标签相关测试需要 `add_document` 和查询方法。
- 替换、零计数测试需要 `add_document` 的替换分支。
- `remove_document...` 需要 `remove_document`。
- `documents_with...` 和 `most_common...` 分别需要对应方法。

### 13.6 测试说明与验收命令

自动测试共 13 个，覆盖：

- **正常行为**：新建空索引、添加多标签文档、多文档累加计数、按标签查文档、最常用标签。
- **边界情况**：空标签列表、只含空白的标签、同文档重复标签、大小写与空白规范化、替换后计数归零、替换时旧标签撤销且不影响其他文档的计数、并列最高计数的字典序选择、文档名两端空白（保存、查询和删除都忽略两端空白）。
- **错误行为**：空文档名返回 `EmptyDocumentName`，并且索引完全没有被修改。

测试名称本身就是行为说明：

```text
new_index_is_empty
add_document_normalizes_deduplicates_and_ignores_empty_tags
normalization_only_lowercases_ascii_letters
counts_each_tag_once_per_document
replacing_a_document_withdraws_old_counts_before_adding_new_ones
replacing_one_document_keeps_other_documents_counts
zero_count_entries_do_not_stay_in_the_index
remove_document_updates_counts_and_reports_whether_it_existed
documents_with_returns_sorted_borrowed_names
most_common_uses_lexical_order_to_break_ties
document_names_are_stored_without_surrounding_whitespace
document_name_lookups_ignore_surrounding_whitespace
empty_document_name_fails_without_changing_the_index
```

实现之前，这些测试会以 `not yet implemented` 失败（11.6 节），不是编译错误。如果连测试都无法编译，先检查是不是改了签名或类型。

在课程仓库根目录运行：

```bash
cargo check -p l06_hash_collections --all-targets   # 只编译，不运行
cargo test -p l06_hash_collections --no-run          # 编译测试
cargo test -p l06_hash_collections exercise          # 只跑主练习的 13 个测试
cargo test -p l06_hash_collections                   # 主练习 + 课后巩固
cargo run -p l06_hash_collections                    # 演示程序
```

全部测试通过即完成主练习。卡住时按级别查看 `HINTS.md`，不要直接看第三级。

主练习全部正确时，`cargo run -p l06_hash_collections` 的完整输出是：

```text
rust 被几篇文档使用：2
cli-notes 的标签：["cli", "rust"]
带 rust 标签的文档：["cli-notes", "rust-guide"]
最常用的标签：Some(("cli", 2))
替换后 rust 的计数：1
删除 web-notes：true，cli 的计数：1
```

`cli-notes` 的标签在打印前由演示程序排序，所以顺序固定；`most_common` 在 `cli` 和 `rust` 都是 2 篇时按规则选了字典序更小的 `cli`。如果你的实现让标签计数、替换或删除任何一步出错，这里会看到不同的数字。

## 十四、课后巩固：团购接龙对账

主练习维护两张互相依赖的表；这个任务换一组数据，练三件不同的事：合并重复条目、跨表比较、找所有集合的共同元素。代码在 `src/consolidation.rs`，三个函数都只借用输入，不拥有它们。

场景：邻居们在群里接龙团购，每人报“商品名 + 数量”。你要把明细合并成总量、对照库存找出缺货商品、再找出“每个人都想要”的商品。

### 14.1 目标与行为

- `tally(lines: &[(&str, u32)]) -> HashMap<String, u32>`：把明细合并成“每种商品一共多少份”。同一种商品出现多次要累加；商品名去掉两端空白后为空的行、数量为 0 的行都当作误报忽略；其余商品名按原样保留（`Apple` 和 `apple` 是两种商品）；返回的映射拥有商品名；输入为空返回空映射。
- `find_shortages(demand: &HashMap<String, u32>, stock: &HashMap<String, u32>) -> Vec<String>`：找出需求量**严格大于**库存量的商品。库存表里查不到的商品按库存 0 算；只检查需求表里出现过的商品；结果按字典序排列；不修改两个输入表。
- `everyone_wants(lists: &[&[&str]]) -> Vec<String>`：找出出现在**每一份**心愿单里的商品。同一份心愿单里重复出现的商品只算一次；结果按字典序排列且不重复；没有任何心愿单时返回空 `Vec`。

### 14.2 输入、输出与边界

```text
tally(&[("apple", 2), ("milk", 1), ("apple", 3), ("tea", 4)])
  → {"apple": 5, "milk": 1, "tea": 4}

tally(&[("   ", 5), ("apple", 0), ("", 2)])
  → {}                                  // 空白名和 0 数量都被忽略

tally(&[("Apple", 1), ("apple", 2)])
  → {"Apple": 1, "apple": 2}            // 大小写不同是两种商品

tally(&[]) → {}

find_shortages({"apple": 5, "milk": 2}, {"apple": 3, "milk": 2, "bread": 10})
  → ["apple"]                           // milk 相等不算缺货；bread 没人订

find_shortages({"cake": 1}, {})
  → ["cake"]                            // 查不到库存按 0 算

find_shortages({"pear": 2, "apple": 2, "milk": 1}, {"milk": 5})
  → ["apple", "pear"]                   // 按字典序，不按输入顺序

find_shortages({}, {"apple": 1}) → []

everyone_wants(&[&["apple", "tea"], &["tea", "apple", "milk"], &["milk", "tea", "apple"]])
  → ["apple", "tea"]

everyone_wants(&[&["tea", "tea", "cake"], &["tea"]])
  → ["tea"]                             // 同一份里的重复只算一次

everyone_wants(&[&["b", "a", "b"]])
  → ["a", "b"]                          // 只有一份时，去重后全部返回

everyone_wants(&[&["apple", "tea"], &[]])
  → []                                  // 有人什么都不想要，就没有“每个人都想要”

everyone_wants(&[]) → []                // 没有任何心愿单
```

### 14.3 约束与验收

- 只替换 `src/consolidation.rs` 三个函数体中的 `todo!()`；不修改签名和测试。
- 三个函数都只借用输入，不能修改调用者的数据。
- 返回的 `HashMap` 和 `Vec<String>` 拥有自己的文本；不要把借用输入文本的引用塞进返回值。
- 不使用 `unwrap`、`expect` 或 `panic!`。
- 结果顺序：`Vec<String>` 必须按字典序；`HashMap` 本身没有顺序，测试直接比较映射内容。
- 不添加依赖。

验收命令：

```bash
cargo test -p l06_hash_collections consolidation
```

共 13 个测试，覆盖正常合并、误报过滤、大小写敏感、空输入、库存相等、库存缺失、排序、重复心愿、单份心愿单、空心愿单、没有心愿单等边界。卡住时看 `HINTS.md` 的对应小节。

## 十五、常见误区

- **以为 `HashMap` 保持插入顺序。** 它不保证任何顺序；需要稳定输出就排序，或用 `BTreeMap`。
- **以为 `insert` 会复制键。** 对 `String` 等非 `Copy` 类型，键会被移动；之后原变量失效。
- **先 `get` 拿引用，再 `insert`，还继续用引用。** 这会触发 E0502。复制出需要的值、调整使用顺序，或用 `entry`。
- **用 `map[key]` 处理可能缺失的键。** 缺失时 panic；外部输入用 `get`。
- **在测试里比较 `HashMap` 遍历得到的 `Vec` 顺序。** 这种测试会随机失败；比较映射本身，或先排序。
- **计数降到 0 还留在表里。** 这会让“最常用标签”返回一个实际使用次数为 0 的标签；删除条目，而不是只把值改成 0。
- **用 `Vec` 手工去重。** 每次插入前扫一遍是 $O(n)$，而且容易漏掉某条插入路径；用 `HashSet`。
- **拿 `f64` 或没派生 `Eq + Hash` 的结构体当键。** 编译期就会报 E0277；先定义领域里的相等规则。
- **手工实现 `Eq` 和 `Hash` 却不保持一致。** 相等的值必须哈希相同，否则映射会“找不到”明明存在的键。
- **`entry(word)` 之后继续用 `word`。** 键已被移动，即使最终走的是“已存在”分支。
- **默认值很贵却用 `or_insert(expensive())`。** 参数总会被计算；用 `or_insert_with` 延迟到真正需要时。
- **`collect` 进 `HashMap` 时出现重复键。** 后出现的值静默覆盖前一个；如果重复是错误，要显式检查。
- **同时更新两张表时先改了一张就返回错误。** 失败会留下不一致状态；先完成所有校验和临时计算，再一次性修改。
- **把 `trim` 当成“删除所有空白”。** 它只去掉两端；中间的空白保留，`"a b"` 仍是 `"a b"`。
- **用 `to_lowercase` 代替 ASCII 小写化。** 前者按完整 Unicode 规则转换，可能把一个字符变成多个；本课要求只处理 A–Z。

## 十六、本课总结

1. `HashMap<K, V>` 建立键到值的映射，拥有键和值；同一个键只保留一条，`insert` 返回被替换的旧值。
2. `get` 返回 `Option<&V>`，`get_mut` 返回 `Option<&mut V>`；映射继续拥有值。`HashMap<String, _>` 可以用 `&str` 查找，不需要临时分配。
3. 借用还没结束不能修改映射：`insert` 可能扩容并搬动条目，旧引用会失效。借用检查器在编译期（编译的时候，而不是运行时）阻止这种错误。
4. `entry` 把“键存在”和“键不存在”统一成一个可修改槽位，`or_insert` 返回 `&mut V`；`and_modify`、`or_insert_with`、`or_insert_with_key` 分别处理修改、延迟构造和按 key 构造。
5. 键必须满足 `Eq + Hash`，并且相等的键必须哈希相同；`f64` 不满足，自定义键通常用 `#[derive(PartialEq, Eq, Hash)]`。
6. 哈希表用“哈希定位桶 + 相等确认”实现平均 $O(1)$ 查找；碰撞只影响性能，不改变语义。扩容会重新哈希全部条目。
7. `HashMap` 的遍历顺序不稳定（随机种子），测试和输出不能依赖它；需要顺序时排序或用 `BTreeMap`。
8. `HashSet` 表达唯一性，`insert` 返回“是否新增”，并支持并集、交集、差集、子集等运算。
9. 同时维护两份数据时，先校验、再修改；错误路径不能留下半更新状态。这是主练习的核心工程要求。

## 十七、课后复盘

用自己的话回答，再对照正文检查：

1. 为什么 `get` 返回 `Option<&V>` 而不是 `Option<V>`？如果返回值可能很大、又不能复制，这个设计避免了什么？
2. `HashMap<String, V>` 为什么可以直接用 `&str` 查找？这样做省掉了什么开销？
3. Entry API 相比“先 `contains_key` 再 `insert`”少做了哪一步？为什么这一步在大表上值得省？
4. 为什么拿着 `&V` 时不能对同一个 `HashMap` 做 `insert`？这和 `Vec` 的哪条规则是同一个道理？
5. 为什么 `HashMap` 的遍历顺序不稳定？这对测试写法有什么直接影响？
6. 什么情况下应该选 `BTreeMap` 而不是 `HashMap`？
7. 主练习里，`add_document` 为什么必须先完成所有校验和临时集合的构造，再修改两个字段？如果先改 `documents` 再发现文档名为空，会留下什么问题？
8. 为什么计数降到 0 的标签必须从 `counts` 删除，而不是保留一个 0？`most_common` 会因此出什么错？

## 下一课预告

本课的集合按**键**组织，但所有代码仍挤在 `lib.rs` 和几个文件里。下一课[第七课：模块、crate、package 与可见性](../07-modules-crates/README.md)把类型和函数拆进模块，理解 `pub`（公开可见性标记，决定别处能不能使用这个名字）控制什么、一个 package 如何组织多个 crate（编译单元），以及为什么“能编译”不等于“边界设计清楚”。
