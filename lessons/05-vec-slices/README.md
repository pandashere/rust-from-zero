# 第五课：Vec、slice 与集合所有权

第一课学过**数组（array）**：`[i32; 4]` 装固定数量的同类型值。但“固定数量”在很多时候是个麻烦——记录一天的体温测量、保存购物车里的商品、收集用户陆续输入的名字，这些数据在写程序时**根本不知道会有多少个**。

本课的主角 `Vec<T>` 就是为这个问题准备的：它是一个**长度可以在程序运行中增长和缩短的集合**。同时我们会把第三课学过的 slice（切片）真正用起来，学会给函数参数选择“整只借来看”还是“借来改”的类型，并理解一条重要规则：**Vec 自动扩容时可能把数据搬去新地址，所以借用检查器会阻止你一边握着旧元素的引用、一边往 Vec 里塞东西。**

## 本课定位

### 你将学会

- 区分 `[T; N]`（数组）、`Vec<T>`（动态集合）、`&[T]`（只读切片）、`&mut [T]`（可修改切片）四种类型。
- 理解 Vec 的内存模型：指针、长度、容量。
- 用 `vec![]`、`push`、`pop`、`insert`、`remove` 创建和修改 Vec。
- 区分“索引取值”和 `get` 两种不同的失败方式。
- 给只读的序列参数优先选择 `&[T]`，理解它为什么比 `&Vec<T>` 通用。
- 处理元素的所有权：什么时候能拿走元素，什么时候只能借用或复制。
- 理解扩容为什么会让旧引用失效，以及借用检查器如何提前阻止这种错误。
- 按“是否需要保持顺序”选择 `remove`、`swap_remove` 或 `retain`。

### 前置知识

- 数组 `[i32; 4]`、`for` 循环、区间 `1..4`：见[第一课](../01-basics/README.md)。简单复习：数组的长度是类型的一部分，写程序时就定死了；`for value in 1..4` 让 `value` 依次取 1、2、3。
- struct、方法、`&self` 与 `&mut self`：见[第二课](../02-data-modeling/README.md)。简单复习：`&self` 表示方法只借数据来看，`&mut self` 表示方法要借来修改。
- 所有权、move、`Copy`、共享引用 `&T`、可变引用 `&mut T`、slice 的初步概念：见[第三课](../03-ownership-borrowing/README.md)。简单复习：每份数据只有一个所有者；`&T` 是“借来看”，可以有多个；`&mut T` 是“借来改”，同一时间只能有一个；切片 `&[T]` 是对一段连续数据的借用视图，由“指向第一个元素的指针加上元素个数”组成。
- `Option<T>`、`Some`/`None`、`match`、`if let`：见[第四课](../04-option-result/README.md)。简单复习：`Option` 表示“可能有值，也可能没有”，必须处理两种情况才能取出里面的值。

### 本课边界

本课不深入：泛型（第 11 课，解释 `Vec<T>` 里的 `T` 为什么是“任意类型”的正式写法）、迭代器 `iter`/`map`/`filter` 等方法链（第 10 课）、`VecDeque` 等其他集合（第 6 课会先学 `HashMap`）。闭包（第 9 课）本课只会用到一次，用到时会给出够用的临时解释。本课也不讲排序算法本身；课后巩固里直接使用标准库提供的排序方法。

## 如何学习本课

1. 先读概念部分，重点在纸上画出 Vec 的“长度”和“容量”有什么区别。
2. 每个代码块都先预测结果，再看解释。
3. 「常见编译器诊断」一节不要跳过——能读懂报错比会写代码更省时间。
4. 完成「代码预测」自测后，再做主练习 `Window`，最后做课后巩固。
5. 主练习卡住时按 `HINTS.md` 的三级提示逐级查看，不要直接看第三级。

预计用时：概念部分约 1.5 小时，主练习约 1 小时，课后巩固约 40 分钟。

## 一、为什么需要 Vec：固定数组的三个困难

假设我们要写一个程序，记录温度传感器一整天上报的读数（整数），然后做分析。用第一课学过的数组来写：

```rust
let readings: [i32; 24] = [0; 24]; // 先假设每小时一条，共 24 条
```

马上遇到三个困难：

1. **数量不确定。** 传感器可能一小时报一次，也可能一分钟报一次。数组长度是类型的一部分，编译时就定死了，不能根据运行情况改变。
2. **先占坑再填数很麻烦。** 即使猜一个上限（比如 `[i32; 10000]`），程序还得另外记住“实际填了几条”，并且猜小了存不下、猜大了浪费内存。
3. **无法表达“逐步收集”。** 很多数据是一条一条来的：用户每次输入一个名字、网络每次收到一条消息。数组没有这个“再加一个”的概念。

**集合（collection）**就是装“一组同类型值”的数据结构的统称。Rust 标准库（语言自带、安装后即可直接使用的代码库）中最常用的顺序集合是 `Vec<T>`，读作“vector of T”或“vector”。它的特点是：

- 长度在**运行时可以变化**：随时 `push`（追加）一个元素，随时 `pop`（取出最后一个）。
- 元素在内存中**连续存放**，和数组一样可以用位置（索引）快速访问。
- Vec **拥有**它的元素：Vec 被清理时，里面的元素也一起被清理。

`T` 是“元素类型”的占位符：`Vec<i32>` 是装 `i32` 的 Vec，`Vec<String>` 是装 `String` 的 Vec。同一个 Vec 里所有元素类型必须相同。

## 二、Vec 的内存模型：指针、长度、容量

第三课介绍过：局部变量放在**栈（stack）**上，大小会变化的数据放在**堆（heap）**上。一个 `Vec<i32>` 同时用到两者：

```text
栈上的 Vec 本身（三个字段）          堆上的缓冲区
┌─────────────┐                 ┌────┬────┬────┬────┬───┐
│ pointer  ───┼────────────────▶│ 10 │ 20 │ 30 │ ?? │...│
│ length   = 3│（当前实际装了 3 个）└────┴────┴────┴────┴───┘
│ capacity = 4│（这块空间总共能装 4 个）
└─────────────┘
```

- **指针（pointer）**：一个内存地址，记录堆上那块空间从哪开始。
- **缓冲区（buffer）**：堆上为元素预留的一整块连续空间。
- **长度（length，简称 len）**：当前**实际装好**的元素个数。
- **容量（capacity）**：缓冲区**最多能装**的元素个数，超过去之前不用重新申请空间。

永远满足 `长度 <= 容量`。向 Vec `push` 一个元素时：

- 如果 `长度 < 容量`，直接写进空闲位置，长度加一，非常快。
- 如果 `长度 == 容量`，缓冲区满了：Vec 会去堆上申请一块**更大的**空间（通常约两倍），把旧元素全部搬过去，释放旧空间，再写入新元素。这个过程叫**扩容（reallocation，重新分配）**。

可以用 `capacity()` 方法查看容量：

```rust
let mut values: Vec<i32> = Vec::new(); // 空 Vec
assert_eq!(values.len(), 0);           // len() 返回长度

values.push(10);
values.push(20);
assert_eq!(values.len(), 2);
assert!(values.capacity() >= 2); // 具体容量由 Vec 自行决定
```

有两个要点必须记牢：

1. **容量只是性能细节，不是逻辑内容。** 只有前 `len` 个位置装好了元素；`len` 之后的空间即使存在，里面的内容也是未初始化的“垃圾”，不能读。
2. **扩容会搬动所有元素，元素的新地址和旧地址不同。** 这一点会在第八节引出一条重要的借用规则。

如果事先知道大约要装多少元素，可以用 `Vec::with_capacity` 一次申请好空间，减少扩容次数：

```rust
let mut values: Vec<i32> = Vec::with_capacity(4);
assert_eq!(values.len(), 0);       // 注意：长度仍然是 0！
assert!(values.capacity() >= 4);

values.push(10);
values.push(20);
assert_eq!(values.len(), 2);
```

`with_capacity(4)` 只是“预留 4 个位置”，里面**一个元素也没有**，`values[0]` 会越界。长度永远表示“现在真的有几个”。

## 三、创建和修改 Vec

```rust
let empty: Vec<i32> = Vec::new();     // 空 Vec，元素类型要靠标注或后续使用推断
let mut values = vec![10, 20, 30];    // vec! 宏：用方括号里的内容创建 Vec

values.push(40);                      // 追加到末尾：values 变成 [10, 20, 30, 40]
assert_eq!(values.pop(), Some(40));   // 取出并删除最后一个
assert_eq!(values, [10, 20, 30]);

values.insert(1, 15);                 // 在位置 1 插入，后面的元素右移
assert_eq!(values, [10, 15, 20, 30]);

let removed = values.remove(2);       // 删除位置 2 的元素，后面的元素左移补位
assert_eq!(removed, 20);
assert_eq!(values, [10, 15, 30]);
```

逐点解释：

- `vec![...]` 是宏（macro）。第一课介绍过：名字后面带 `!` 的调用是宏，是“编译时帮你生成代码的代码”。`vec![10, 20, 30]` 大致展开成“创建 Vec，再逐个 push”。
- 修改 Vec 要求绑定可写，所以要写 `let mut values`。
- `push` 把值放进末尾。如果值不是 `Copy` 类型（比如 `String`），它的所有权会 **move** 进 Vec，之后由 Vec 负责清理。
- `pop` 返回 `Option<T>`（第四课）：有元素时返回 `Some(最后一个元素)`，空 Vec 返回 `None`。为什么不让它直接返回 `T`？因为空 Vec 没有“最后一个元素”，硬返回一个假值只会让错误在别处爆发。
- `insert(位置, 值)` 和 `remove(位置)` 都要**搬动后面的元素**：在中间插入或删除时，位置之后的每个元素都要挪一格。元素很多时这是真实的时间开销。`remove` 的位置越界会 panic（程序中止，见第七节）。

## 四、读取元素：索引与 get

**索引（index）**是元素的位置编号，从 0 开始：第一个元素是 `values[0]`。

```rust
let values = vec![10, 20, 30];
let first = values[0];   // 索引取值
assert_eq!(first, 10);
```

索引取值有一个硬规则：**越界就 panic**（程序立刻中止，不像 `Option` 那样给你处理机会）：

```rust,ignore
fn main() {
    let values = vec![10, 20, 30];
    println!("{}", values[10]); // 索引 10 超出长度 3
}
```

运行结果：

```text
thread 'main' (12345) panicked at src/main.rs:3:26:
index out of bounds: the len is 3 but the index is 10
```

（括号里的数字是本次运行的线程编号，每次运行都不同；位置指向 `values[10]` 这个索引表达式。）

翻译：越界了，长度是 3，索引却是 10。

如果“越界”属于正常可能（比如位置来自用户输入），用 `get`，它返回 `Option`：

```rust
let values = vec![10, 20, 30];

assert_eq!(values.get(1), Some(&20)); // Some(第二个元素的引用)
assert_eq!(values.get(10), None);     // 越界时平静地返回 None

if let Some(value) = values.get(10) {
    println!("第 10 个元素是 {value}");
} else {
    println!("没有第 10 个元素");
}
```

选择原则：

- 位置是程序自己算出来的、越界说明程序有 **bug**：用索引，让 panic 尽早暴露问题。
- 位置来自外部输入、越界是**正常情况**：用 `get`，用 `Option` 处理。

注意 `get` 返回的是 `Option<&T>`——**元素的引用**，不是元素本身。Vec 不会因为你“看了一眼”就把元素交出来。

## 五、借用 Vec：slice 作为函数参数

第三课见过切片（slice）：`&[T]` 是对一段连续数据的**只读借用视图**，不拥有元素，由“指向第一个元素的指针 + 元素个数”组成。

### 5.1 为什么参数写 `&[T]` 而不是 `&Vec<T>`

写一个求和函数，两种签名都能工作：

```rust
fn sum(values: &[i32]) -> i32 { /* ... */ }
fn sum_vec(values: &Vec<i32>) -> i32 { /* ... */ }
```

但调用时的差别很大：

```rust
fn sum(values: &[i32]) -> i32 {
    let mut total = 0;
    for value in values {
        total += *value; // value 是 &i32，*value 取出它指向的整数
    }
    total
}

fn main() {
    let vector = vec![1, 2, 3];
    let array = [4, 5, 6];

    assert_eq!(sum(&vector), 6);      // Vec 可以借出整个切片
    assert_eq!(sum(&array), 15);      // 数组也可以
    assert_eq!(sum(&vector[1..]), 5); // 还可以只借一段
}
```

- `&vector`（类型 `&Vec<i32>`）能自动转换成 `&[i32]`，因为 Vec 的数据本来就是连续存放的；
- 数组 `&array` 也能转换成 `&[i32]`；
- `&vector[1..]` 是“从索引 1 到末尾”的子切片。

如果参数写成 `&Vec<i32>`，数组和子区间都传不进来，函数的使用范围白白变窄，调用者还会被迫依赖 Vec 这个具体实现。**因此：只读参数优先写 `&[T]`。**

### 5.2 切片区间

`&values[起点..终点]` 取一段子切片，起点包含、终点不包含；省略一端表示到那一端的尽头：

```rust
let values = vec![10, 20, 30, 40, 50];

assert_eq!(&values[1..3], &[20, 30]); // 索引 1 到 3（不含 3）
assert_eq!(&values[..2], &[10, 20]);  // 从头开始
assert_eq!(&values[3..], &[40, 50]);  // 到末尾结束
assert_eq!(&values[..], &[10, 20, 30, 40, 50]); // 整个
```

区间越界（比如 `&values[3..10]`）或起点大于终点都会 panic。切片只是**视图**：不复制数据，也不拥有数据；原 Vec 还活多久，切片最多就只能用多久。

### 5.3 `&mut [T]`：借来修改

如果要让函数修改“一段元素”但不拥有它们，参数用 `&mut [T]`：

```rust
fn zero_all(values: &mut [i32]) {
    for value in values.iter_mut() {
        *value = 0; // value 是 &mut i32，*value 指向要修改的元素
    }
}

fn main() {
    let mut values = vec![1, 2, 3];
    zero_all(&mut values[1..]); // 只清零后两个
    assert_eq!(values, [1, 0, 0]);
}
```

`iter_mut()` 逐个产出元素的可变引用。注意 `&mut [T]` **不能 push**：切片只是“看向别人缓冲区的一扇窗”，它不知道缓冲区在哪、容量是多少，没有权力替真正的所有者做扩容。想增删元素，必须拿到 Vec 本身。

## 六、元素的所有权

Vec 拥有元素，这带来一个常见问题：**能不能把元素从 Vec 里“拿走”？**

对能整体复制的类型（`i32` 等 `Copy` 类型，第三课学过），索引取值会自动复制，没有麻烦：

```rust
let values = vec![10, 20, 30];
let first = values[0]; // i32 是 Copy，复制一份给 first
```

但对不实现 `Copy` 的类型（比如 `String`），下面这样写**不能编译**：

```rust,ignore
let names = vec![String::from("Ada")];
let first = names[0]; // 试图把 String 从 Vec 里搬出来
```

原因：`names[0]` 只是“透过借用看向那个位置”。如果允许把 `String` 搬走，Vec 的这个位置就只剩下一个空洞，但 Vec 的长度仍然是 1——之后谁去访问这个位置都会读到无效数据。Rust 不允许存在这种“没填满”的 Vec。

合法的“拿走”方式都是**显式**的，按需要选：

```rust
let mut names = vec![String::from("Ada"), String::from("Lin")];

let first = names.remove(0);     // 删除并交出该位置的元素（后面的元素左移）
let last = names.pop();          // 删除并交出最后一个，返回 Option<String>
```

或者只要一份副本，原件留在 Vec 里：

```rust
let names = vec![String::from("Ada")];
let copy = names[0].clone();     // 显式复制一份
let borrow = &names[0];          // 或者只借用，不复制
```

`clone()` 会真的复制数据（第三课讲过它的成本）；`&names[0]` 只是借用，成本低，但借用期间不能修改 Vec。

## 七、迭代 Vec 的三种方式

`for` 循环遍历 Vec 有三种写法，正好对应第三课所有权的三种态度：

```rust
let mut names = vec![String::from("Ada"), String::from("Lin")];

for name in &names {          // 1. 只读借用：name 是 &String
    println!("{name}");
}                             // 循环结束后 names 还能用

for name in &mut names {      // 2. 可变借用：name 是 &mut String
    name.push('!');           // 可以修改每个元素
}                             // 但不能增删元素，长度不变

for name in names {           // 3. 消费：name 是 String（拥有）
    println!("拿走了 {name}");
}                             // 循环结束后 names 已被消耗，不能再用
```

- 第 1 种最常用：看一眼，不改动。
- 第 2 种适合“批量修改每个元素”。
- 第 3 种会把元素**逐个 move 出来**，Vec 本身随之失效。适合“处理完这批数据就不要了”。

这和第 10 课要学的 `iter()` / `iter_mut()` / `into_iter()` 是同一套所有权模型，不是三套互不相干的写法。`for x in &v` 基本等价于 `for x in v.iter()`。

## 八、扩容与引用失效

回顾第二节：缓冲区满了再 push，Vec 会把所有元素**搬到一块新空间**。这带来一个隐患——如果之前保存了某个元素的引用，搬家之后它还指着旧地址：

```rust,ignore
let mut values = vec![1, 2, 3];
let first = &values[0]; // 指向缓冲区里的第一个元素
values.push(4);         // 如果这里扩容，所有元素搬到新地址……
println!("{first}");    // ……first 还指着已经释放的旧空间！
```

旧空间被释放后，`first` 就成了**悬垂引用（dangling reference）**——指向无效内存的引用（第三课见过这个词）。在许多语言里这是真实发生、且极难排查的崩溃来源。Rust 的做法是**在编译期直接拒绝**：

```text
error[E0502]: cannot borrow `values` as mutable because it is also borrowed as immutable
 --> src/main.rs:4:5
  |
3 |     let first = &values[0];
  |                  ------ immutable borrow occurs here
4 |     values.push(4);
  |     ^^^^^^^^^^^^^^ mutable borrow occurs here
5 |     println!("{first}");
  |                ----- immutable borrow later used here
```

逐行读这段诊断（报错信息）：

- 第 3 行发生了一次**不可变借用**（`&values[0]`）；
- 第 4 行 `push` 需要**可变借用**整个 Vec（它可能扩容搬动所有元素）；
- 第 5 行那次不可变借用**还在使用**——两个借用活动时间重叠，违反“多个读者或一个写者”的规则。

第三课学过：借用在它**最后一次使用**之后就结束。所以只要把顺序调成“先用完引用，再修改”，代码就能编译：

```rust
let mut values = vec![1, 2, 3];
let first = &values[0];
println!("{first}"); // first 在这里最后一次使用，借用随之结束
values.push(4);      // 现在可变借用没有冲突
```

如果确实需要先记住第一个元素再修改，就**复制**出来（`i32` 是 `Copy`）：

```rust
let mut values = vec![1, 2, 3];
let first = values[0]; // 复制整数值，不持有引用
values.push(4);
println!("{first}");
```

规则总结成一句话：**只要手里还握着指向 Vec 内容（或切片）的引用，就不能对这个 Vec 做任何可能改动它的操作；先用完，再改。**

## 九、删除元素：remove、swap_remove、retain

删除一个位置上的元素，标准库提供两种策略，选哪个取决于**你在不在乎剩下的元素保持原顺序**：

```rust
let mut values = vec![10, 20, 30, 40];

let removed = values.remove(1);        // 保持顺序：后面的元素左移补位
assert_eq!(removed, 20);
assert_eq!(values, [10, 30, 40]);

let removed = values.swap_remove(1);   // 不保持顺序：用最后一个元素填坑
assert_eq!(removed, 30);
assert_eq!(values, [10, 40]);
```

- `remove(i)` 删除位置 `i` 后，把后面每个元素左移一格。删除位置越靠后越便宜，越靠前越贵；最坏情况要搬动全部元素（操作次数随元素个数线性增长，记作 O(n)）。
- `swap_remove(i)` 把最后一个元素换到位置 `i`，再删掉末尾。无论 Vec 多大，只做固定几步（记作 O(1)），代价是顺序被打乱。
- 顺序有意义（队列、时间线）用 `remove`；顺序无所谓（一堆待处理任务）优先 `swap_remove`。

按**条件**批量删除时，用 `retain`，保留满足条件的元素、删除其余的，顺序不变：

```rust
let mut values = vec![1, 2, 3, 4, 5];
values.retain(|value| value % 2 == 1); // 只保留奇数
assert_eq!(values, [1, 3, 5]);
```

`|value| value % 2 == 1` 是一个**闭包（closure）**——写在使用现场的一次性小函数，`|value|` 是它的参数，后面是返回值。`retain` 会对每个元素调用一次这个小函数，返回 `true` 就保留。闭包的完整语法在第 9 课学习，现在照这个形状写即可。

千万别用“索引从 0 递增、边循环边 `remove`”的写法：删掉一个元素后，后面的元素都左移了一格，下一个要检查的元素正好被跳过——这是新手最常写的隐蔽 bug。

另外两个相关方法：

```rust
let mut values = vec![1, 2, 3, 4, 5];

values.truncate(2);          // 只保留前 2 个
assert_eq!(values, [1, 2]);

values.clear();              // 删除全部元素
assert!(values.is_empty());  // is_empty() 判断长度是否为 0
```

`clear` 只清空内容，**容量通常保留**：缓冲区还在，下次 push 不用重新申请空间。如果你打算立刻再装满一批数据，这比新建 Vec 省一次申请。

## 十、容量管理（可选的优化手段）

前面说过扩容要搬动全部元素。如果事先知道元素数量级，可以提前预留：

```rust
let mut values = Vec::new();
values.reserve(100); // 确保还能再装至少 100 个而不触发扩容
```

- `reserve(额外数量)`：保证 `容量 >= 长度 + 额外数量`。
- `shrink_to_fit()`：请求释放多余容量（只是请求，不保证精确收缩）。

重要的心态：**容量是性能优化，不是正确性问题。** 不预留容量，程序照样正确，只是可能多做几次搬家。不要给每个小 Vec 都机械地 `reserve`——只有数据量大、且数量可以提前估算时才值得。

## 十一、完整示例：购物车

下面这个完整例子把本课的东西串起来：创建、push、借用切片、`&mut` 修改、`remove`、按条件 `retain`、消费。它和主练习不同，可以放心细读。

场景：一个命令行购物车，商品用名字（`String`）表示，支持加购、删除一件商品、清空打折区的失效商品、最后结账时把清单交给快递系统。

```rust
/// 购物车：拥有所有商品名。
#[derive(Debug)]
struct Cart {
    items: Vec<String>,
}

impl Cart {
    /// 创建一辆空购物车。
    fn new() -> Self {
        Cart { items: Vec::new() }
    }

    /// 加购一件商品。
    fn add(&mut self, name: &str) {
        self.items.push(String::from(name));
    }

    /// 删除第一件名字匹配的商品；不存在时返回 false。
    fn remove_one(&mut self, name: &str) -> bool {
        for index in 0..self.items.len() {
            if self.items[index] == name {
                self.items.remove(index); // remove 会返回该元素，这里直接丢弃
                return true;
            }
        }
        false
    }

    /// 删除所有标记为“已售罄”的商品（名字以 "（售罄）" 结尾）。
    fn remove_sold_out(&mut self) {
        self.items.retain(|item| !item.ends_with("（售罄）"));
    }

    /// 查看当前所有商品，只读。
    fn items(&self) -> &[String] {
        &self.items
    }

    /// 结账：交出整辆车的商品清单，购物车本身被消耗。
    fn checkout(self) -> Vec<String> {
        self.items
    }
}

fn main() {
    let mut cart = Cart::new();
    cart.add("苹果");
    cart.add("牛奶（售罄）");
    cart.add("面包");
    cart.add("苹果"); // 允许重复

    println!("加购后: {:?}", cart.items());
    // 加购后: ["苹果", "牛奶（售罄）", "面包", "苹果"]

    cart.remove_one("苹果"); // 只删掉第一个“苹果”
    println!("删一件后: {:?}", cart.items());
    // 删一件后: ["牛奶（售罄）", "面包", "苹果"]

    cart.remove_sold_out();
    println!("清理后: {:?}", cart.items());
    // 清理后: ["面包", "苹果"]

    let shipped = cart.checkout();
    println!("发货清单: {shipped:?}");
    // 发货清单: ["面包", "苹果"]
    // 此处之后不能再使用 cart：所有权已经随 checkout 移出
}
```

（`#[derive(Debug)]` 是第二课学过的派生：它让编译器为 `Cart` 生成调试打印能力，例子里的 `{:?}` 才能工作。）

数据怎么流动：

1. `add` 接收 `&str`（借用），内部复制成 `String` 后 push 进 Vec——Vec 从此拥有这份文本。
2. `remove_one` 用索引循环找到第一个匹配项，用 `remove` 把它拿出来。这里我们不需要拿出来的值，所以没接收返回值；它被自动丢弃清理。
3. `remove_sold_out` 用 `retain` 按条件批量删除，`ends_with` 判断文本结尾。
4. `items()` 返回 `&[String]`：调用方（`main` 里的 `println!`）能看，但不能改、不能删。
5. `checkout(self)` 的参数是 `self` 而不是 `&self`：它**拿走整辆车的所有权**，把 Vec 原样交出去，没有任何复制。调用后 `cart` 失效，编译器会阻止你再用它。

注意 `items()` 返回 `&[String]` 而不是 `&Vec<String>`，原因正是第五节的结论：调用者只需要“一段连续的商品名”，不需要知道也不应依赖它是不是 Vec。哪天内部换成别的集合，这个签名不用变。

## 十二、常见编译器诊断与排错

### 12.1 E0502：借用还在使用时就修改 Vec

第八节已经完整读过这个报错。排错步骤：

1. 找到报错里指出的两处借用位置（“immutable borrow occurs here” 和 “mutable borrow occurs here”）。
2. 找到旧引用的**最后一次使用**（“later used here”）。
3. 三选一：把修改挪到最后一次使用之后；复制出需要的值（`Copy` 类型直接赋值，或 `clone()`）；改用索引（`usize`）代替保存引用，用到时重新取。

### 12.2 E0507：不能通过索引搬走非 Copy 元素

```rust,ignore
let names = vec![String::from("Ada")];
let first = names[0];
```

真实报错：

```text
error[E0507]: cannot move out of index of `Vec<String>`
 --> src/main.rs:3:17
  |
3 |     let first = names[0];
  |                 ^^^^^^^^ move occurs because value has type `String`, which does not implement the `Copy` trait
  |
help: consider borrowing here
  |
3 |     let first = &names[0];
  |                 +
help: consider cloning the value if the performance cost is acceptable
  |
3 |     let first = names[0].clone();
  |                         ++++++++
```

编译器给出了两条修复建议：借用（`&names[0]`）或复制（`clone()`）。如果语义是“拿走这个元素”，用 `remove(0)` 或 `pop()`；只是“看看”就用借用。

### 12.3 越界 panic

```text
thread 'main' (12345) panicked at src/main.rs:3:26:
index out of bounds: the len is 3 but the index is 10
```

`panicked at` 后面是出错位置。这种错误在**运行时**才暴露，编译器帮不上忙，因为索引值运行时才确定。排查时先打印长度和索引，再确认索引来源。来自外部输入的位置，一开始就该用 `get`。

### 12.4 测试里的 `not yet implemented`

在学习仓库里做练习时，你会先看到这种测试失败：

```text
thread 'exercise::tests::new_window_is_empty' (12345) panicked at lessons/05-vec-slices/src/exercise.rs:31:9:
not yet implemented
```

（线程名由模块路径加测试名组成；括号里是线程编号，每次运行不同；路径按仓库根目录显示。）

这不是测试写错了，而是脚手架里的 `todo!()` 宏在运行到它时主动 panic，提醒你“这里是留给你的作业”。实现完成后，这些失败会逐个消失。

## 十三、代码预测与解析

先自己判断每段代码能否编译、输出什么，再展开解析。

### 题目 1

```rust
let mut values = vec![1, 2, 3];
let first = values.first().copied();
values.push(4);
println!("{first:?}");
```

能编译吗？输出什么？

<details>
<summary>查看解析</summary>

能编译，输出 `Some(1)`。`first()` 返回 `Option<&i32>`（第一个元素的引用），`copied()` 把里面的 `i32` **复制**出来，得到 `Option<i32>`。此后不再持有任何指向 Vec 的引用，所以 `push` 没有借用冲突。

</details>

### 题目 2

```rust
let mut values: Vec<i32> = Vec::new();
println!("{:?}", values.pop());
```

输出什么？

<details>
<summary>查看解析</summary>

输出 `None`。空 Vec 没有最后一个元素，`pop` 用 `None` 表示“没有”，而不是 panic。

</details>

### 题目 3

```rust
let mut values = vec![10, 20, 30, 40];
let tail = &values[2..];
println!("{tail:?}");
values.push(50);
println!("{values:?}");
```

能编译吗？

<details>
<summary>查看解析</summary>

能编译。切片 `tail` 在第二行**最后一次使用**，借用随之结束；之后的 `push` 没有冲突。如果把两行 `println!` 交换顺序，就会得到 E0502，因为 `tail` 的使用跨越了 `push`——而 `push` 可能扩容，让所有元素搬家。

</details>

### 题目 4

```rust
let mut values = vec![1, 2, 3, 4];
let second = values.swap_remove(1);
println!("{second}, {values:?}");
```

输出什么？

<details>
<summary>查看解析</summary>

输出 `2, [1, 4, 3]`。`swap_remove(1)` 删除位置 1 的元素 `2`，并用最后一个元素 `4` 填坑。对比：`remove(1)` 会得到 `[1, 3, 4]`，保持顺序但要搬动后续元素。

</details>

### 题目 5

```rust
let mut values: Vec<i32> = Vec::with_capacity(100);
println!("{}", values.len());
println!("{:?}", values.get(50));
```

输出什么？

<details>
<summary>查看解析</summary>

输出 `0` 然后 `None`。`with_capacity(100)` 只预留空间，长度仍为 0，第 50 个位置根本不存在初始化好的元素。容量永远不等于内容。

</details>

## 十四、主练习：有界样本窗口

传感器每秒上报一个整数读数，分析程序只关心**最近的一小段**。你要实现一个 `Window`（窗口）：容量固定，放满之后每进入一个新样本就丢弃最旧的一个（先进先出）。

### 14.1 目标与行为

本练习的代码在本课 package `l05_vec_slices` 的 `src/exercise.rs` 中。类型已经定义好：

```rust
pub struct Window {
    values: Vec<i32>,
    capacity: usize,
}
```

需要实现的方法（方法上方的注释就是完整行为约定）：

- `Window::new(capacity)`：创建空窗口。
- `push(value)`：放入一个样本；满了先淘汰最旧的；容量为 0 时忽略。
- `values() -> &[i32]`：按从旧到新返回全部样本的只读视图。
- `latest() -> Option<i32>`：返回最新的样本，空窗口返回 `None`。
- `clear_negative()`：删除所有负数样本，保留其余样本的相对顺序。
- `extend(values)`：把一批样本按顺序逐个放入，效果等同连续 `push`。
- `split_recent(count) -> (&[i32], &[i32])`：切成“较旧部分”和“最多 count 个最近样本”两段只读视图。
- `capacity() -> usize`：返回容量。

### 14.2 输入、输出与边界

以下都是真实调用序列及其期望结果：

```text
Window::new(3)
  → values() == []，latest() == None，capacity() == 3

Window::new(3)，依次 push(10) push(20) push(30)
  → values() == [10, 20, 30]，latest() == Some(30)

接上继续 push(40) push(50)
  → values() == [30, 40, 50]   （最旧的 10、20 被依次淘汰）

Window::new(0)，push(7)，再 extend([2, 3, 4])
  → values() == []，latest() == None   （容量为 0，丢弃一切输入）

Window::new(3)，extend([1, 2, 3, 4, 5])
  → values() == [3, 4, 5]   （超出的部分同样先进先出）

Window::new(6)，extend([-3, 2, -1, 0, 5, -8])，然后 clear_negative()
  → values() == [2, 0, 5]，capacity() 仍为 6

Window::new(4)，extend([10, 20, 30, 40])，然后：
  split_recent(0)  → ([10, 20, 30, 40], [])
  split_recent(2)  → ([10, 20], [30, 40])
  split_recent(4)  → ([], [10, 20, 30, 40])
  split_recent(10) → ([], [10, 20, 30, 40])   （count 大于长度时取全部）
```

### 14.3 脚手架与约束

- 只替换 `src/exercise.rs` 各方法体中的 `todo!()`；不修改类型定义、方法签名和文件底部的测试。
- 两个字段保持私有：窗口外部只能通过公开方法接触数据。
- `values()` 和 `split_recent()` 返回借用，不允许为了“方便”而 `clone` 整个 Vec 再返回——那样既慢，也失去了借用带来的编译期保护。
- `extend` 的行为必须等同逐个 `push`（超量输入只保留最后的样本）。
- 不使用 `unwrap`、`expect` 或 `panic!`。
- 不添加任何第三方依赖。

### 14.4 推荐完成顺序

1. `new` 和 `capacity`：把两个字段准备好，让窗口能被创建、容量能读出。
2. `values`、`push`（先只管“未满时追加”的情况）和 `latest`：第一个测试同时检查容量、空视图和最新样本，做到这一步它才会通过。
3. 补上 `push` 的“满时淘汰最旧样本”和“容量为 0 忽略”两个分支。
4. `extend`：用 `for` 循环复用 `push`。
5. `clear_negative`：回想第九节哪个方法专门做“按条件删除”。
6. `split_recent`：先想清楚分界点索引怎么算（`count` 大于长度时不能算出负数——`usize` 没有负数），再按 `HINTS.md` 的提示选用切片切分的方法。

### 14.5 测试说明与验收命令

实现之前运行测试，每个测试都会在对应的 `todo!()` 处以 `not yet implemented` 失败——这是学习流程的一部分，不是测试写错了（见 12.4 节）。

在课程仓库根目录运行：

```bash
cargo run -p l05_vec_slices        # 演示程序（完成练习后才有完整输出）
cargo test -p l05_vec_slices exercise  # 只运行主练习的 9 个测试
cargo fmt -p l05_vec_slices -- --check
```

主练习的 9 个测试全部通过即完成主练习。`cargo test -p l05_vec_slices` 会连课后巩固一起运行，巩固任务未完成前它仍会失败。卡住时按级别查看 `HINTS.md`。

## 十五、课后巩固：分数分析

换一个场景检验你是否真的理解了“借用切片”和“拥有 Vec”的分工。这次的数据是一组考试分数（`u32`，0 到 100），代码在 `src/consolidation.rs`，三个函数都**不拥有**集合——这正是切片参数的用武之地。

### 15.1 目标与行为

- `summarize(scores: &[u32]) -> Option<Summary>`：统计份数、最低分、最高分、总分。一个分数也没有时返回 `None`（此时最低分、最高分不存在），否则返回 `Some(Summary)`。`Summary` 的 `total` 字段是 `u64`，避免大量分数相加溢出 `u32`。
- `top_scores(scores: &[u32], count: usize) -> Vec<u32>`：选出最高的 `count` 个分数，从高到低返回。并列分数各占一个位置；`count` 超过总数时返回全部；`count` 为 0 或输入为空时返回空 Vec。返回的 Vec 归调用者所有。
- `apply_bonus(scores: &mut [u32], bonus: u32)`：给每个分数加 `bonus` 分，最高不超过 100，直接修改传入的切片。空切片什么都不做。

### 15.2 输入、输出与边界

```text
summarize(&[90, 60, 90, 45])
  → Some(Summary { count: 4, min: 45, max: 90, total: 285 })
summarize(&[75])
  → Some(Summary { count: 1, min: 75, max: 75, total: 75 })
summarize(&[]) → None

top_scores(&[58, 92, 74, 92, 61], 3) → [92, 92, 74]
top_scores(&[30, 10, 20], 0)         → []
top_scores(&[30, 10, 20], 10)        → [30, 20, 10]
top_scores(&[58, 92, 74], 2) 之后，原数组仍是 [58, 92, 74]（借用不会被修改）

apply_bonus(&mut [95, 80, 100], 10) 之后 → [100, 90, 100]
apply_bonus(&mut [], 10) 正常结束，什么都不做
```

### 15.3 约束与验收

- 只替换 `src/consolidation.rs` 三个函数体中的 `todo!()`；不修改类型、签名和测试。
- `top_scores` 不能修改传入的切片（它本来就是只读借用；如果你发现“想排
  序但切片不让改”，说明你需要先复制出一个自己拥有的 Vec——这正是本课
  要练的判断）。
- 不使用 `unwrap`、`expect` 或 `panic!`。
- 排序直接使用标准库 `Vec` 自带的 `sort()` 方法（升序排列）；算法细节不属于本课范围。

验收命令：

```bash
cargo test -p l05_vec_slices consolidation
```

## 十六、常见误区

- **把容量当长度。** `with_capacity(100)` 之后立刻读第 50 项是越界。只有 `len()` 之内的元素存在。
- **握着引用改 Vec。** 保存了 `&values[i]` 或切片之后，先想清楚它最后一次使用在哪一行，再决定能否 push/remove。
- **一边递增索引一边 `remove`。** 删除后元素左移，会跳过待检查的项。按条件删除用 `retain`。
- **参数写 `&Vec<T>`。** 这让数组和子区间都传不进来。只读参数用 `&[T]`，要改元素用 `&mut [T]`。
- **对小 Vec 过度优化容量。** `reserve` 只在“量大且数量可预估”时有意义；正确性从来不依赖容量。
- **用 `get` 处理一切，或用索引处理一切。** 越界是 bug 就用索引（让 panic 暴露问题），越界是正常输入就用 `get`（返回 `Option`）。
- **字符串切片套用数组切片的经验。** 本课的 slice 是 `[T]` 的切片，按元素个数切。第三课讲过：对 `str` 切片按**字节**计数，还必须落在 UTF-8 字符边界上，两套规则不要混用。

## 十七、本课总结

1. `Vec<T>` 拥有可增长的连续缓冲区；`[T; N]` 长度固定在类型里；`&[T]` 和 `&mut [T]` 只是借用视图。
2. Vec 有长度和容量两个属性：长度是“装了几个”，容量是“不搬家最多能装几个”。容量是性能细节。
3. `push` 在缓冲区满时会扩容：申请更大的空间并搬动全部元素。因此握着元素引用时不能修改 Vec，借用检查器在编译期强制这一点。
4. 索引越界 panic；`get` 返回 `Option`。按“越界是 bug 还是正常输入”选择。
5. 删除按需求选择：保持顺序用 `remove`，追求速度且不在乎顺序用 `swap_remove`，按条件批量删除用 `retain`。
6. 函数参数表达意图：只读用 `&[T]`，改元素用 `&mut [T]`，需要增删才接收 Vec 本身。
7. 迭代对应所有权：`&v` 借来看，`&mut v` 借来改，`v` 直接消费掉。

## 十八、课后复盘

用自己的话回答，再对照正文检查：

1. 为什么 `&[T]` 比 `&Vec<T>` 更适合做只读参数？
2. 长度和容量有什么区别？为什么 `Vec::with_capacity(100)` 之后不能读第 50 项？
3. 为什么拿着 `&values[0]` 时 push 会被拒绝？编译器在防什么？
4. `pop` 为什么返回 `Option` 而不是直接返回值？
5. 什么时候 `swap_remove` 比 `remove` 更合适？
6. “一边递增索引一边 remove”会出什么问题？正确做法是什么？

## 下一课预告

本课的集合按**位置**组织：第几个元素。下一课[第六课：HashMap、HashSet 与 Entry API](../06-hash-collections/README.md)学习按**键（key）**组织的集合：用名字查分数、统计单词出现次数、自动去重。你会看到所有权规则如何同样作用在键和值上。
