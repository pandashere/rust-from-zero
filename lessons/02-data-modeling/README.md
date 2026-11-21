# 第二课：用 struct 和 enum 给数据建模

第一课解决的是“怎样计算”：用 `let` 绑定（binding，即给值起的名字）保存数据，用函数封装一段计算，用 `if`、`while`、`loop` 控制执行顺序。但写出来的程序只是一堆散落的数字和变量。

第二课解决一个更接近真实工程的问题：**怎样让数据本身有组织、有含义，让“不合法的数据”在编译阶段就被拒绝。** 为此 Rust 提供两个核心工具：

- **struct（结构体）**：表达“一个东西由哪几部分组成”，例如一个矩形由宽和高组成。
- **enum（枚举）**：表达“一个值只能是几种情况之一”，例如方向只能是北、东、南、西之一。

围绕这两个工具，本课还会学习 `impl`（实现块，把函数挂到类型上）、`match`（匹配，按“是哪种情况”分支并取出里面的数据）以及 `#[derive(...)]`（派生，请编译器自动写出一些常用能力）。

预计学习时间 1～2 小时。

## 本课定位

### 你将学会

- 定义和创建三种结构体：带名字字段的 struct、tuple struct（元组结构体）、unit-like struct（单元结构体）。
- 访问和修改字段，使用字段初始化简写和 struct update syntax（结构体更新语法）。
- 用 `#[derive(...)]` 让编译器自动生成调试打印、比较、复制等能力。
- 在 `impl` 块中定义关联函数（associated function）和方法（method），区分 `self`、`&self`、`&mut self` 和 `Self`。
- 定义不携带数据、携带元组数据、携带命名字段数据的三种 enum 变体（variant，即枚举中的一种具体情况）。
- 用 `match` 穷尽地处理枚举的每种变体，并在匹配时取出变体里携带的数据。
- 使用或模式（`|`）、范围模式、通配符 `_`、match guard（匹配守卫）、`if let` 和 `matches!`。
- 阅读 struct、方法和 `match` 相关的真实编译器报错。

### 前置知识（第一课）

本课会用到第一课的内容，这里简要复习；不熟悉的部分请回到[第一课](../01-basics/README.md)：

- `let` 创建一个绑定，`let mut` 创建可修改的绑定；同名 `let` 是遮蔽（shadowing，新名字盖住旧名字）。
- 函数用 `fn 名字(参数: 类型) -> 返回类型 { ... }` 定义，块中最后一个不带分号的表达式就是返回值。
- `if` / `while` / `loop` 控制执行顺序；整数类型如 `i32`（32 位有符号整数）、`u32`（32 位无符号整数）。
- 元组（tuple）如 `(3, 5)` 把几个值打包成一个值，可以解构取出。

### 本课暂不深入

本课会出现 `&self` 和 `&mut self`，暂时只需要理解成“临时只读地看一看这个值”和“临时独占地修改这个值”。它们背后的所有权（ownership，决定哪段代码负责释放一份数据的规则）与借用（borrowing，把值的访问权临时借出去）规则，会在[第三课](../03-ownership-borrowing/README.md)集中讲解。

本课也不使用 `Option` 和 `Result`（Rust 表达“可能没有值 / 可能失败”的标准类型，[第四课](../04-option-result/README.md)介绍），所以本课练习中的“非法输入”统一采用“忽略这条命令”的策略。

## 如何学习本课

1. 先读懂“为什么需要自定义类型”，理解 struct 和 enum 各自解决什么问题。
2. 概念部分每段代码都动手敲一遍或在脑中推演：这个值是什么类型？是哪种变体？
3. 重点理解 `impl` 块、方法的三种接收者和 `match` 的穷尽性检查。
4. 做“代码预测”，先写下你的判断再展开解析对照。
5. 完成主练习（网格机器人），再做课后巩固（楼层电梯）。
6. 最后用自己的话回答复盘问题：为什么 `match` 不允许漏掉变体？

## 一、场景：一个网格机器人

假设我们要写一个程序，模拟一台在网格上移动的机器人：它有横坐标、纵坐标和一个朝向，能接收“左转”“右转”“前进若干步”三类命令。

### 只用第一课的工具会遇到什么困难

用散落的变量表示机器人：

```rust
let x = 0;
let y = 0;
let direction = 0; // 约定：0 北、1 东、2 南、3 西
```

这段代码能跑，但问题随着程序变大会越来越多：

- **关系靠记忆维持。** `x` 和 `y` 属于同一台机器人，但语言层面它们只是两个互不相干的整数。传给函数时要传两个参数，漏传、传错顺序都不会被编译器发现。
- **含义靠注释维持。** `direction == 0` 是北还是东？只看代码不知道，注释写错或过期了也不会有人发现。
- **非法值挡不住。** `direction` 的类型是整数，什么都不阻止某行代码把它改成 `99`。
- **修改规则靠自觉。** “前进”要求先读朝向再改对应坐标，但没有任何机制保证每个修改位置的函数都遵守这条规则。

这些问题的共同根源是：**数据的组织和约束只存在于程序员的脑子里，编译器一无所知，因此无法帮忙检查。**

### Rust 的引入：把含义写进类型

```rust
struct Position {
    x: i32,
    y: i32,
}

enum Direction {
    North,
    East,
    South,
    West,
}
```

- `Position` 把 `x`、`y` 声明为**一个整体**：以后函数可以只接收一个 `Position`，编译器保证里面一定同时有 `x` 和 `y`。
- `Direction` 把方向**限定为四种可能**：`Direction::North` 等是仅有的四个值，写出第五个方向直接编译失败，“方向是 99”这种数据根本无法构造出来。

这就是本课的核心思想：**让类型表达约束，而不是依赖注释和自觉。** 下面分别学习 struct 和 enum 的语法。

## 二、struct（结构体）：把几份数据组合成一个整体

### 2.1 定义类型

```rust
struct Rectangle {
    width: u32,
    height: u32,
}
```

这段代码声明了一个新类型 `Rectangle`。`struct` 是关键字（Rust 语言保留、有特殊含义的单词），后面是类型名，花括号里列出它的字段（field，组成这个类型的每一份数据）：每个 `Rectangle` 值都一定同时拥有一个 `u32` 类型的 `width` 和一个 `u32` 类型的 `height`。

注意这只是**定义类型**，像写出一张“规格表”，还没有创建任何具体的矩形。

### 2.2 创建值

```rust
let rectangle = Rectangle {
    width: 30,
    height: 20,
};
```

创建时给每个字段一个值。字段由名字确定，书写顺序不必和定义一致：

```rust
let rectangle = Rectangle {
    height: 20,
    width: 30,
};
```

必须提供全部字段（除非使用后面介绍的更新语法），漏掉任何一个都是编译错误，编译器会明确告诉你缺哪个——这正是“不合法数据无法构造”的第一层保障。

### 2.3 访问字段

用点号 `.` 访问一个值的字段：

```rust
let area = rectangle.width * rectangle.height;
println!("area = {area}");
```

字段类型在编译期就是已知的：`rectangle.width` 的类型是 `u32`，所以两个字段相乘的结果也是 `u32`。

### 2.4 修改字段

要修改字段，持有这个值的绑定本身必须用 `let mut` 声明为可变：

```rust
let mut rectangle = Rectangle {
    width: 30,
    height: 20,
};

rectangle.width = 40;
```

Rust 不允许只把某一个字段声明为可变。下面这种写法无法通过编译：

```rust,compile_fail
struct Rectangle {
    mut width: u32, // 错误：字段定义里不能写 mut
    height: u32,
}
```

可变性属于“持有这个值的绑定”，而不属于类型定义：同一个类型，用 `let` 绑定的值不能改，用 `let mut` 绑定的值就能改。

### 2.5 字段初始化简写

当局部变量名和字段名恰好相同时，可以省略重复的名字：

```rust
fn make_rectangle(width: u32, height: u32) -> Rectangle {
    Rectangle { width, height }
}
```

它等价于 `Rectangle { width: width, height: height }`。构造函数里这种写法非常常见。

### 2.6 struct update syntax（结构体更新语法）

想创建一个“大部分字段和某个已有值相同”的新值时，可以用 `..已有值` 补齐剩余字段：

```rust
#[derive(Clone, Copy)]
struct Rectangle {
    width: u32,
    height: u32,
}

let first = Rectangle {
    width: 30,
    height: 20,
};

let wider = Rectangle {
    width: 50,
    ..first // height 从 first 那里拿过来
};
```

`wider.height` 的值是 `20`，来自 `first.height`。

上面的 `#[derive(Clone, Copy)]` 让 `Rectangle` 获得了“复制”能力（下一节会解释），所以 `first` 在被取用字段后仍然可以照常使用。如果字段是不能简单复制的类型（例如后面会学到的 `String`），更新语法会把字段“搬走”，`first` 就不能再完整使用了。这个差别属于第三课的所有权内容，现在只需知道有这个区别。

## 三、另外两种 struct：tuple struct 与 unit-like struct

### 3.1 tuple struct（元组结构体）：只有类型名，字段没有名字

```rust
struct Meters(i32);
struct Seconds(i32);
```

创建和访问都像元组，用位置和序号：

```rust
let distance = Meters(100);
let duration = Seconds(9);

println!("{} meters", distance.0);
println!("{} seconds", duration.0);
```

为什么不用两个裸的 `i32`？因为 `Meters` 和 `Seconds` 是**两个不同的类型**，尽管内部都只是一个 `i32`。如果一个函数参数写的是 `Meters`，把 `Seconds` 传进去会直接编译失败——“把 9 秒当成 9 米”这类错误在编译期就被挡住了。用一层薄包装给相同的底层数据加上不同含义，是 tuple struct 最典型的用途。

### 3.2 unit-like struct（单元结构体）：没有任何字段

```rust
struct Shutdown;

let command = Shutdown;
```

它不存任何数据，价值在于“这是一个独立的类型”。等后面学习 trait（特征，一组可以被多个类型共同实现的能力约定）时会看到它的实际用途，现在知道有这种形态即可。

## 四、`#[derive(...)]`：请编译器代写常用能力

自定义类型默认“什么都不会”。例如它不能用 `{:?}` 打印：

```rust,compile_fail
struct Point {
    x: i32,
    y: i32,
}

let point = Point { x: 1, y: 2 };
println!("{point:?}");
```

编译器会报错说 `Point` 没有实现 `Debug`（完整报错见“错误实验”一节）。

`Debug` 是一个 trait（特征）：一份“能力的约定”，约定了“怎样把一个值格式化成调试文本”。`#[derive(...)]`（派生）写在类型定义上方，意思是**请编译器照着字段自动生成这份约定的实现**。例如：

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Point {
    x: i32,
    y: i32,
}
```

本课会用到的派生能力：

| 派生 | 含义 |
|---|---|
| `Debug` | 可以用 `{:?}` 或 `{:#?}` 打印，方便调试和测试失败时查看值 |
| `PartialEq` | 可以用 `==`、`!=` 比较两个值是否相等（逐字段比较） |
| `Eq` | 在 `PartialEq` 之上声明“相等关系是完整规范的”，整数和本课的类型都满足，通常和 `PartialEq` 成对出现 |
| `Clone` | 可以显式调用 `.clone()` 得到一个副本 |
| `Copy` | 赋值和传参时自动按位复制，原值不受影响（第一课学到的整数、布尔值就是 `Copy` 的） |

`Copy` 不是任何类型都能派生：类型的**所有字段**都必须是 `Copy` 的。含有 `String` 等“拥有型”字段的结构体不能派生 `Copy`，原因在第三课讲。

也不要图省事给每个类型都堆上全部派生——派生的是这个类型**确实需要**的能力。本课练习中的类型需要打印调试信息、相互比较、按值复制，所以 `Debug, Clone, Copy, PartialEq, Eq` 这一组是合理选择。

## 五、`impl`（实现块）：把行为挂到类型上

回到机器人场景：我们有了 `Position`，接下来希望写“计算两个坐标的曼哈顿距离”“移动一步”这样的功能。当然可以写成普通函数，但 Rust 提供了一种更好的组织方式：把这些函数放进类型的 `impl` 块里，明确声明“这些行为属于这个类型”。

### 5.1 关联函数：通过类型名调用

```rust
struct Rectangle {
    width: u32,
    height: u32,
}

impl Rectangle {
    fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}
```

`impl Rectangle { ... }` 表示“里面是为 `Rectangle` 这个类型准备的函数”。`new` 的第一个参数**不是** `self`，所以它不作用于某个具体矩形，而是用来创建矩形，称为关联函数（associated function）。调用时用类型名加双冒号：

```rust
let rectangle = Rectangle::new(30, 20);
```

`new` 不是关键字，也不是 Rust 自动提供的构造器，只是社区约定俗成的名字——大家看到 `类型::new(...)` 就知道这是在创建一个新值。

### 5.2 `Self`：当前类型的别名

在 `impl Rectangle` 内部，`Self`（首字母大写）就是 `Rectangle` 的别名。`fn new(...) -> Self` 意思是“返回一个 `Rectangle`”。使用 `Self` 可以减少重复：将来类型改名时，`impl` 块里的代码不用跟着改。

### 5.3 方法：通过值调用，用 `&self` 只读

```rust
impl Rectangle {
    fn area(&self) -> u32 {
        self.width * self.height
    }
}
```

第一个参数写成 `&self` 的函数称为方法（method），它作用于一个具体的值，用点号调用：

```rust
let rectangle = Rectangle::new(30, 20);
let area = rectangle.area();
```

`&self` 的含义拆开看：`self`（首字母小写）指“调用方法的这个值本身”，`&` 表示引用（reference，对一份数据的临时访问，不拿走数据本身）。所以 `&self` 就是“临时只读地看一看这个值”——方法可以读字段，但不能修改，方法结束后值仍然完好，像没被碰过一样。

调用时不用自己写 `&`：点号调用会自动完成取引用，写 `rectangle.area()` 即可，不需要写 `(&rectangle).area()`。

### 5.4 用 `&mut self` 修改

```rust
impl Rectangle {
    fn grow(&mut self, amount: u32) {
        self.width += amount;
        self.height += amount;
    }
}
```

`&mut self` 表示“临时独占、可修改地访问这个值”。调用者的绑定必须是可变的：

```rust
let mut rectangle = Rectangle::new(30, 20);
rectangle.grow(5); // rectangle 现在变成 35 x 25
```

“独占”的意思是：在这段修改结束之前，没有别的代码能同时读写同一个值。这条规则是编译器强制检查的，完整内容在第三课；现在只需记住：**要调用 `&mut self` 方法，值必须用 `let mut` 绑定。**

### 5.5 用 `self` 取走整个值

```rust
impl Rectangle {
    fn into_dimensions(self) -> (u32, u32) {
        (self.width, self.height)
    }
}
```

没有 `&` 的 `self` 表示方法接收**整个值本身**。对 `Copy` 类型来说调用时会复制一份；对不能复制的类型，这个值会被“移进”方法，调用后原来就不能再用了。本课练习只用 `&self` 和 `&mut self`，知道存在第三种即可。

### 5.6 方法和字段可以同名

```rust
impl Rectangle {
    fn width(&self) -> u32 {
        self.width
    }
}
```

`rectangle.width` 是访问字段，`rectangle.width()` 是调用方法，括号让两者不会混淆。这种同名 getter（只返回某个字段的方法）很常见，本课练习的 `direction()` 就是这种风格。

## 六、enum（枚举）：一个值只能是几种情况之一

### 6.1 定义与使用

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    North,
    East,
    South,
    West,
}
```

`enum` 是关键字，花括号里列出全部变体（variant）。这个定义向编译器承诺：任何一个 `Direction` 值，在任一时刻**恰好**是这四个变体之一，没有第五种可能。

```rust
let direction = Direction::North;
```

变体通过 `类型名::变体名` 使用。和“用 0、1、2、3 代表方向”相比，枚举的价值在于编译器知道这个承诺：它会在后面的 `match` 中检查你是否处理了全部四种情况（第七节），也会拦住任何试图塞入其他值的行为。

### 6.2 变体可以携带各自的数据

枚举的每种变体还能携带**形状不同**的数据：

```rust
enum Command {
    Stop,
    Move(i32, i32),
    Resize { width: u32, height: u32 },
}
```

三种形态同时出现：

- `Stop`：不携带任何数据。
- `Move(i32, i32)`：携带两个匿名位置的整数（类似元组）。
- `Resize { width: u32, height: u32 }`：携带两个有名字的字段（类似结构体）。

创建值：

```rust
let stop = Command::Stop;
let movement = Command::Move(3, -2);
let resize = Command::Resize {
    width: 800,
    height: 600,
};
```

可以把 `Command::Move` 粗略理解为一个“构造器”：给它两个 `i32`，它就还你一个 `Command` 值，数据被存在这个值里面。怎么把数据取出来，是下一节 `match` 的工作。

### 6.3 为什么不用一个“大而全”的 struct

也可以用 struct 硬塞下所有可能用到的字段：

```rust
struct WeakCommand {
    kind: i32,       // 约定：0 停止、1 移动、2 调整尺寸
    dx: i32,
    dy: i32,
    width: u32,
    height: u32,
}
```

这个类型能表示出海量**无意义**的组合：`kind` 是“停止”却带着移动距离；`kind` 是“移动”却带着尺寸；`kind` 是 `99`，什么都不是。每一处使用 `WeakCommand` 的代码都得自己小心甄别。

enum 从类型层面消灭了这些组合：每种变体只携带自己需要的数据，`Stop` 里根本没有地方塞距离，`Move` 里也没有地方塞尺寸。**无效状态连构造都构造不出来**，就不需要运行时检查了。

## 七、`match`（匹配）：穷尽处理每种情况

### 7.1 基本用法

```rust
fn direction_code(direction: Direction) -> i32 {
    match direction {
        Direction::North => 0,
        Direction::East => 1,
        Direction::South => 2,
        Direction::West => 3,
    }
}
```

`match` 后面跟一个值，花括号里是一条条分支（arm）：左边是模式（pattern，描述“值长什么样子”），右边是结果，中间用 `=>` 连接。`match` 整体是一个表达式，所有分支的结果类型必须一致，整个 `match` 的值就是命中分支的结果——所以这里可以直接作为函数的尾表达式返回。

### 7.2 穷尽性：编译器不许你漏

如果漏写变体：

```rust,compile_fail
match direction {
    Direction::North => 0,
    Direction::East => 1,
}
```

编译直接失败，报错会明确列出 `South` 和 `West` 没有被覆盖（完整报错见“错误实验”一节）。

这是 enum 配 `match` 最有价值的性质：**将来给 enum 新增变体时，所有漏掉它的 `match` 都会立刻变成编译错误**，编译器替你指出每一处需要更新的代码。靠人脑记住“改了枚举要改哪些地方”是一定会出错的，靠编译器则不会。

### 7.3 在模式里取出变体携带的数据

```rust
fn describe(command: Command) {
    match command {
        Command::Stop => println!("stop"),
        Command::Move(dx, dy) => println!("move by {dx}, {dy}"),
        Command::Resize { width, height } => {
            println!("resize to {width} x {height}");
        }
    }
}
```

模式 `Command::Move(dx, dy)` 做两件事：判断这个值是不是 `Move` 变体；如果是，把变体里存的两个值**绑定**到新名字 `dx`、`dy` 上，分支右边就能直接使用。命名字段的变体同理，用 `Command::Resize { width, height }` 取出。

### 7.4 或模式、范围模式与通配符

`match` 不只服务于 enum，整数等类型也可以匹配：

```rust
let number = 3;

let category = match number {
    0 => "zero",
    1 | 2 => "small",
    3..=9 => "single digit",
    _ => "other",
};
```

- `1 | 2`：或模式，匹配其中任意一个。
- `3..=9`：范围模式，匹配 3 到 9（含两端）。
- `_`：通配符（wildcard），匹配所有前面没列到的情况，但不把值绑定到任何名字。

整数的取值是无穷的，不可能逐一枚举，所以整数匹配最后必须有 `_` 之类的兜底分支。

### 7.5 通配符不是默认首选

对 enum 来说，通配符要慎用。下面的写法能编译：

```rust
match direction {
    Direction::North => println!("north"),
    _ => println!("not north"),
}
```

但它把“东、南、西”和**未来新增的任何方向**一起吞掉了。将来给 `Direction` 加上第五个变体，这个 `match` 依然编译通过，新增的变体被悄悄归进“not north”，没有任何警告。

反过来，如果明确写出全部四个变体，新增变体时编译器会立刻报错提醒。所以经验法则是：**业务上真的只关心“某一种对其余全部”时才用 `_`；每个变体各有含义时，逐一枚举。**

### 7.6 match guard（匹配守卫）：模式之后再加条件

```rust
match command {
    Command::Move(dx, dy) if dx == 0 && dy == 0 => {
        println!("no movement");
    }
    Command::Move(dx, dy) => {
        println!("move by {dx}, {dy}");
    }
    _ => println!("another command"),
}
```

`if ...` 写在模式之后，称为匹配守卫：模式先匹配，命中后再检查这个布尔条件，条件不满足就继续往下尝试其他分支。注意分支是**从上到下**依次尝试的，所以更具体的带守卫分支要写在前面。守卫适合简单的附加条件；条件一旦复杂，应该提取成函数，或者回头重新审视类型设计。

（`&&` 表示“并且”：两个条件都成立时，整个条件才成立。第一课第七节介绍过 `&&` 和 `||`。）

## 八、`if let` 与 `matches!`：只关心一种情况时的简写

### 8.1 `if let`

```rust
let command = Command::Move(3, 4);

if let Command::Move(dx, dy) = command {
    println!("move by {dx}, {dy}");
}
```

读作：“如果 `command` 能匹配模式 `Command::Move(dx, dy)`，就执行花括号里的代码，其中 `dx`、`dy` 已经绑好。”它等价于只写一个分支、其余情况全部忽略的 `match`，适合“只关心一种情况”的局部逻辑。

反过来，如果每种变体都有不同的处理逻辑，请老老实实用 `match`，不要拆成一串互不相干的 `if let`——那样既丢掉了穷尽性检查，也让代码难读。

### 8.2 `matches!`

```rust
let direction = Direction::North;
let is_vertical = matches!(direction, Direction::North | Direction::South);
```

`matches!(值, 模式)` 是一个宏（macro，以 `!` 结尾、在编译期展开成普通代码的工具），结果是 `bool`：值匹配模式就是 `true`，否则是 `false`。它替代“整个 `match` 只返回 true/false”的啰嗦写法。

## 九、模式不只属于 `match`

模式是 Rust 里一套统一的“按形状拆解值”的语法，`match` 只是它的一个使用场景。

元组解构（第一课已经见过）：

```rust
let point = (3, 5);
let (x, y) = point;
```

结构体解构：

```rust
struct Point {
    x: i32,
    y: i32,
}

let point = Point { x: 3, y: 5 };
let Point { x, y } = point; // x 是 3，y 是 5
```

解构时还可以给字段起新名字：

```rust
let Point {
    x: horizontal,
    y: vertical,
} = point;
```

后面的课程还会在函数参数、循环以及 `Option`、`Result` 上反复用到模式，它是值得熟练掌握的通用工具。

## 十、完整示例：交通灯状态机

下面是一个完整可运行的程序，把本课的 struct、enum、`impl`、`match`、`matches!` 组合在一起。它和练习中的机器人是**不同的例子**，但组织方式相同，可以对照着读。

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Light {
    Red,
    Green,
    Yellow,
}

#[derive(Debug, PartialEq, Eq)]
struct TrafficLight {
    current: Light,
    changes: u32,
}

impl TrafficLight {
    fn new() -> Self {
        Self {
            current: Light::Red,
            changes: 0,
        }
    }

    fn current(&self) -> Light {
        self.current
    }

    fn advance(&mut self) {
        self.current = match self.current {
            Light::Red => Light::Green,
            Light::Green => Light::Yellow,
            Light::Yellow => Light::Red,
        };
        self.changes += 1;
    }

    fn can_go(&self) -> bool {
        matches!(self.current, Light::Green)
    }
}

fn main() {
    let mut light = TrafficLight::new();

    println!("{:?}, can go = {}", light.current(), light.can_go());
    light.advance();
    println!("{:?}, can go = {}", light.current(), light.can_go());
}
```

运行输出：

```text
Red, can go = false
Green, can go = true
```

逐步分析：

1. `Light` 用枚举把灯的状态限定为红、绿、黄三种，不存在第四种值。
2. `TrafficLight` 用结构体把“当前状态”和“切换次数”两份数据组合成一个整体。
3. `new` 是关联函数（没有 `self` 参数），通过 `TrafficLight::new()` 调用，负责创建初始状态。
4. `current` 和 `can_go` 只读取状态，所以接收 `&self`。
5. `advance` 要修改字段，所以接收 `&mut self`，调用它的 `light` 必须用 `let mut` 绑定。
6. `advance` 里的 `match` 根据当前状态算出下一个状态。三种状态逐一枚举，编译器保证没有遗漏。
7. `can_go` 用 `matches!` 把“当前是否为绿灯”压成一行。

最后一点值得停下来体会：如果将来给 `Light` 新增一个 `Flashing`（闪烁）变体，`advance` 里的 `match` 会**立刻编译失败**，逼你回答“闪烁之后应该变成什么”。类型设计把“将来容易漏改的地方”变成了“编译器自动检查的地方”。

## 十一、错误实验：阅读真实编译器报错

下面的报错都是在 stable 工具链上真实产生的（节选关键行）。学习读报错比背语法更重要：报错会告诉你问题在哪一行、编译器期待什么。

### 11.1 创建 struct 时漏了字段（E0063）

```rust,compile_fail
struct Point {
    x: i32,
    y: i32,
}

let point = Point { x: 3 };
```

```text
error[E0063]: missing field `y` in initializer of `Point`
 --> src/main.rs:7:17
  |
7 |     let point = Point { x: 3 };
  |                 ^^^^^ missing `y`
```

修复：补上 `y`，或者用合法的更新语法从别的值补齐。报错开头的 `E0063` 是错误编号，可以用 `rustc --explain E0063` 查看官方解释。

### 11.2 修改不可变绑定的字段（E0594）

```rust,compile_fail
let point = Point { x: 3, y: 5 };
point.x = 10;
```

```text
error[E0594]: cannot assign to `point.x`, as `point` is not declared as mutable
 --> src/main.rs:8:5
  |
8 |     point.x = 10;
  |     ^^^^^^^^^^^^ cannot assign
  |
help: consider changing this to be mutable
  |
7 |     let mut point = Point { x: 3, y: 5 };
  |         +++
```

注意编译器给出的 `help` 建议：修复方法是让**绑定**可变（`let mut point`），而不是去改类型定义。调用 `&mut self` 方法时报的也是这一类错误。

### 11.3 没有派生 `Debug` 就用 `{:?}` 打印（E0277）

```rust,compile_fail
struct Point {
    x: i32,
    y: i32,
}

let point = Point { x: 1, y: 2 };
println!("{point:?}");
```

```text
error[E0277]: `Point` doesn't implement `Debug`
 --> src/main.rs:8:15
  |
8 |     println!("{point:?}");
  |               ^^^^^^^^^ `Point` cannot be formatted using `{:?}` because it doesn't implement `Debug`
  = help: the trait `Debug` is not implemented for `Point`
help: consider annotating `Point` with `#[derive(Debug)]`
```

修复就是建议里的那行：在类型定义上方加 `#[derive(Debug)]`。

### 11.4 `match` 没有穷尽（E0004）

```rust,compile_fail
match direction {
    Direction::North => 0,
    Direction::East => 1,
}
```

```text
error[E0004]: non-exhaustive patterns: `Direction::South` and `Direction::West` not covered
 --> src/main.rs:9:11
  |
9 |     match direction {
  |           ^^^^^^^^^ patterns `Direction::South` and `Direction::West` not covered
```

修复方向是**把真正遗漏的变体补上**。编译器有时还会建议加一个 `_` 分支，但那只是把问题藏起来：请回到 7.5 节想清楚该不该用通配符。

### 11.5 各分支结果类型不一致（E0308）

```rust,compile_fail
let result = match direction {
    Direction::North => 0,
    Direction::East => "east",
    Direction::South => 2,
    Direction::West => 3,
};
```

```text
error[E0308]: `match` arms have incompatible types
  --> src/main.rs:12:28
   |
11 |         Direction::North => 0,
   |                             - this is found to be of type `{integer}`
12 |         Direction::East => "east",
   |                            ^^^^^^ expected integer, found `&str`
```

`match` 是表达式，所有分支的结果必须能当成同一种类型使用。要么统一返回类型，要么让各个分支做不产生返回值的事（比如各自 `println!`）。

## 十二、代码预测与解析

先自己写下答案，再展开对照。

### 题目 1：`::` 还是 `.`

```rust
let rectangle = Rectangle::new(10, 20);
let area = rectangle.area();
```

为什么一个用 `::` 调用，另一个用 `.` 调用？

<details>
<summary>查看解析</summary>

`new` 的第一个参数不是 `self`，它是挂在类型上的关联函数，所以用类型名加 `::` 调用：`Rectangle::new(10, 20)`。`area` 的第一个参数是 `&self`，它作用于一个具体的值，所以用值加点号调用：`rectangle.area()`。

</details>

### 题目 2：可变性

```rust
let rectangle = Rectangle::new(10, 20);
rectangle.grow(5);
```

如果 `grow` 接收 `&mut self`，这段代码能编译吗？

<details>
<summary>查看解析</summary>

不能。调用 `&mut self` 方法需要对值做可变访问，绑定必须写成 `let mut rectangle`。这与 11.2 节修改字段的报错是同一类问题。

</details>

### 题目 3：枚举里的数据从哪来

```rust
let command = Command::Move(3, -2);

match command {
    Command::Move(dx, dy) => println!("{dx}, {dy}"),
    _ => println!("other"),
}
```

`dx` 和 `dy` 是从哪里冒出来的？

<details>
<summary>查看解析</summary>

创建 `Command::Move(3, -2)` 时，两个整数被存进了这个值。模式 `Command::Move(dx, dy)` 匹配成功后，把存着的两个值绑定到新的局部名字 `dx` 和 `dy` 上，所以打印结果是 `3, -2`。

</details>

### 题目 4：穷尽性与当前值无关

```rust
enum State {
    Ready,
    Running,
    Finished,
}

let state = State::Ready;

let text = match state {
    State::Ready => "ready",
    State::Running => "running",
};
```

`state` 明明就是 `Ready`，为什么编译器仍然拒绝这段代码？

<details>
<summary>查看解析</summary>

`match` 的穷尽性检查看的是**类型**：`state` 的类型是 `State`，这个类型的值可能是三个变体中的任何一个，而 `Finished` 没有对应分支。编译器不会根据“这一处刚好赋了哪个值”放宽检查——这样将来改动赋值代码时，`match` 的正确性才有保障。

</details>

### 题目 5：`Copy` 枚举

```rust
#[derive(Debug, Clone, Copy)]
enum Direction {
    North,
    South,
}

let first = Direction::North;
let second = first;
println!("{first:?}");
```

最后一行能正常打印吗？

<details>
<summary>查看解析</summary>

能。`Direction` 派生了 `Copy`，赋值 `let second = first;` 会复制一份，`first` 不受影响；`Debug` 则是 `{:?}` 能打印的前提（第四节讲过）。本课的类型都只含可复制的数据，所以可以自由地赋值和传参；等数据里出现 `String` 这类不可复制的类型时，规则会变化，那是第三课的主题。
</details>

## 十三、主练习：网格机器人状态机

练习文件是 `src/main.rs`（本课 package 的默认程序），类型定义、方法签名和测试已经写好，四个方法体是 `todo!()`，由你实现。

### 13.1 数据模型（已给出，不要修改）

```rust
enum Direction {
    North,
    East,
    South,
    West,
}

enum Command {
    TurnLeft,
    TurnRight,
    Move(i32),
}

struct Robot {
    x: i32,
    y: i32,
    direction: Direction,
}
```

坐标约定：北是 y 增大的方向，南是 y 减小，东是 x 增大，西是 x 减小。坐标可以取任意整数，包括负数——网格无限大。

```text
          North (+y)
              ^
              |
West (-x) <---+---> East (+x)
              |
              v
          South (-y)
```

### 13.2 要实现的方法

```rust
impl Robot {
    fn new(x: i32, y: i32, direction: Direction) -> Self;
    fn position(&self) -> (i32, i32);
    fn direction(&self) -> Direction;
    fn apply(&mut self, command: Command);
}
```

每个方法上方的注释已经按“这个方法自己的数据”描述了行为要求，读注释即可开工，不需要翻测试反推规格。

### 13.3 行为示例

下面这组连续调用展示了期望行为（每个箭头右边是该调用结束后读到的状态），包括正常、边界和无效输入：

| 调用 | position() | direction() |
|---|---|---|
| `Robot::new(0, 0, Direction::North)` | `(0, 0)` | `North` |
| `apply(Command::Move(3))` | `(0, 3)` | `North` |
| `apply(Command::TurnRight)` | `(0, 3)` | `East` |
| `apply(Command::Move(2))` | `(2, 3)` | `East` |
| `apply(Command::Move(-1))`（负距离，整条无效） | `(2, 3)` | `East` |
| `apply(Command::TurnLeft)` | `(2, 3)` | `North` |

另外两个值得自己推演的边界情况：

- 连续 `TurnRight` 四次后，朝向应回到原来的方向（`TurnLeft` 同理）。
- `Move(0)` 是合法命令：坐标不变。

### 13.4 约束

- 不修改 enum、struct 的定义和方法签名，不删除或弱化测试。
- `apply` 必须用 `match` 处理 `Command` 的全部三个变体。
- 转向和移动中的方向判断必须穷尽列出全部四个 `Direction` 变体，不用 `_` 兜底（理由见 7.5 节）。
- 不用整数代表方向，不使用外部 crate（Rust 对“可复用的代码包”的称呼）、全局可变状态或字符串解析。

### 13.5 推荐实现顺序

1. 先实现 `new`，用上字段初始化简写。
2. 实现 `position` 和 `direction`，体会 `&self` 的只读访问。
3. 在 `apply` 里最外层对 `Command` 做 `match`。
4. 完成 `TurnRight` 的四方向转换，运行下面 13.6 的单项测试验证。
5. 完成 `TurnLeft`。
6. 在 `Move(distance)` 分支里先处理负距离，再对当前方向做 `match`，修改对应坐标。
7. 跑全部测试。

卡住时按 `HINTS.md` 的级别逐级查看提示，不要直接跳到最后一级。

### 13.6 验收命令

在**仓库根目录**运行（也可以在本课目录里运行，效果相同）：

```bash
cargo run -p l02_data_modeling        # 运行示例程序
cargo test -p l02_data_modeling       # 运行全部测试（主练习 + 课后巩固）
```

只跑主练习、或只跑某一个测试：

```bash
cargo test -p l02_data_modeling --bin l02_data_modeling
cargo test -p l02_data_modeling turning_right
```

（第二条命令的 `turning_right` 是测试名的一部分，Cargo 只运行名字里含这段文字的测试。）

实现前，行为测试会因为 `todo!()` 而失败，这是正常的起点。只跑主练习时，全部通过即完成主练习；`cargo test -p l02_data_modeling` 会连课后巩固一起运行，巩固任务未完成前它仍会失败。主练习测试覆盖：构造保存状态、左右转各四次回到原向、转向不改坐标、四个方向移动只改对应坐标、`Move(0)`、负距离忽略、转向后移动使用新方向。

## 十四、课后巩固：楼层电梯

主练习用了“方向 + 坐标”，巩固任务换一组数据和边界，但用的是同一批概念。练习文件是 `src/bin/l02_elevator.rs`，同样已经写好类型、注释和测试，方法体由你实现。

### 14.1 需求

一部电梯在楼层下限 `min_floor` 和上限 `max_floor` 之间运行（创建电梯时确定，例如 `-2` 到 `10`）。电梯有一扇门。命令有四种：`GoUp(n)`、`GoDown(n)`、`OpenDoor`、`CloseDoor`。

行为规则（方法注释里有同样的描述）：

- 电梯初始停在 `min_floor`，门关闭。
- 门开着的时候，任何移动命令都无效（安全规定），楼层不变。
- 移动距离为负数时，该命令无效。
- 移动会越过边界时，电梯停在边界楼层（不是命令无效，而是“尽量走，撞到边界停下”）。
- 开关门命令只改变门的状态，重复执行没有额外效果。

### 14.2 行为示例

| 调用 | floor() | door() |
|---|---|---|
| `Elevator::new(-2, 10)` | `-2` | `Closed` |
| `apply(GoUp(5))` | `3` | `Closed` |
| `apply(GoUp(100))`（越界，停在顶层） | `10` | `Closed` |
| `apply(OpenDoor)` | `10` | `Open` |
| `apply(GoDown(2))`（门开着，无效） | `10` | `Open` |
| `apply(CloseDoor)` | `10` | `Closed` |
| `apply(GoDown(20))`（越界，停在底层） | `-2` | `Closed` |

### 14.3 验收命令

```bash
cargo run -p l02_data_modeling --bin l02_elevator
cargo test -p l02_data_modeling --bin l02_elevator
```

`HINTS.md` 里同样为这个任务准备了三级提示。

## 十五、常见误区

### 误区 1：用整数代替 enum

`0`、`1`、`2`、`3` 在类型层面只是普通整数，拦不住 `99`，也让每个读代码的人去背约定。enum 让编译器参与检查，成本几乎为零。

### 误区 2：给 enum 的 match 随手加 `_`

对有限状态（方向、灯、门）来说，逐一枚举才是常态。`_` 会吞掉将来新增的变体，让“新增状态时编译器提醒所有遗漏点”这个最重要的保障失效。

### 误区 3：所有函数都写成自由函数

自由函数没有错，但当一段逻辑天然读写某个类型的内部状态时，放进 `impl` 成为方法，调用方式（`robot.position()`）和 API 边界都更清晰。

### 误区 4：混淆 `Self` 和 `self`

- `Self`（大写）是**类型**，在 `impl Robot` 里就是 `Robot`。
- `self`（小写）是**调用方法的那个具体的值**。
- `&self`、`&mut self` 描述方法以什么方式访问这个值：只读看一看，还是独占修改。

### 误区 5：以为 `derive` 只是注释

`#[derive(PartialEq)]` 会让编译器真的生成逐字段比较的实现。删掉它，测试里的 `assert_eq!` 就会因为这个类型不能比较而报错。

### 误区 6：在不可变绑定上调用 `&mut self` 方法

报错信息指向的是**绑定**的可变性，修复是 `let mut`，不是去改字段或方法的定义。

## 十六、本课总结

1. struct 表达“一个值由哪几份数据组成”；enum 表达“一个值只能是几种情况之一”，且每种情况可以携带自己的数据。
2. `impl` 块把与类型紧密相关的函数组织在一起：没有 `self` 参数的是关联函数（`Type::new()` 调用），有 `self` 参数的是方法（`value.method()` 调用）。
3. `&self` 临时只读，`&mut self` 临时独占修改，后者要求调用者的绑定是 `let mut`。
4. `match` 既做分支选择，也用模式把变体里的数据解出来；它要求穷尽所有可能，这正是 enum 价值的兑现方式。
5. `_` 通配符留给“真的只关心一种情况”的场合；有限状态请逐一枚举。
6. `#[derive(...)]` 请编译器生成调试、比较、复制等能力，按类型的真实需要选择。
7. 类型设计的目标不是少打字，而是**让无效状态无法构造、让遗漏分支变成编译错误**。

## 十七、课后复盘

完成练习和巩固后，用自己的话回答（可以记到 `notes/concepts.md`）：

1. struct 和 enum 分别适合表达什么关系？
2. `Robot::new(...)` 和 `robot.position()` 为什么用不同的调用符号？
3. 现阶段可以怎样理解 `&self` 和 `&mut self`？
4. 机器人转向为什么不该用 `_` 分支？
5. `Command::Move(3)` 里的 `3` 存在哪里？怎样通过模式把它取出来？
6. 如果给 `Command` 新增一个 `Reset` 变体，哪些代码会被编译器立刻指出需要修改？

## 下一课预告

本课刻意只使用整数和 `Copy` 枚举组成的数据，赋值、传参都很随意。[第三课](../03-ownership-borrowing/README.md) 会进入 Rust 最核心的机制：所有权、移动（move）、借用，以及 `String`、`&str` 和切片（slice）。到时你会看到，一旦数据里出现“拥有资源”的类型，本课里自由的赋值和方法调用会多出一套编译期规则——而那套规则正是 Rust 内存安全的来源。
