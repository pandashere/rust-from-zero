# 第十二课：trait、关联类型、`impl Trait` 与 `dyn Trait`

一个程序里经常出现“数据长得不一样，但都能做同一件事”的情况：邮件、短信、控制台消息的字段不同，却都能生成一条摘要；矩形和圆形的数据不同，却都能算面积。trait（中文常译为“特征”，本课统一叫“行为契约”）就是 Rust 用来描述这类共同能力的工具。

本课不只讲“怎么写 trait”，还要回答三个问题：

1. 为什么需要把“能力”从“数据”里单独拿出来？
2. 编译器在什么时候决定调用哪份代码？为什么有时不需要运行时开销，有时又必须查表？
3. 关联类型、孤儿规则、trait object 这些限制分别在保护什么？

## 本课定位

### 你将学会

- 定义 trait，并为具体类型实现它，理解“契约”和“实现”的分离。
- 用默认方法表达共享行为，并在需要时覆盖默认行为。
- 用 trait bound 编写泛型函数，理解静态分发和单态化。
- 用返回位置的 `impl Trait` 隐藏具体类型。
- 用 `Box<dyn Trait>` 把不同类型放进同一个集合，理解 trait object、胖指针和虚函数表。
- 用关联类型表达“一个实现只对应一种关联类型”，并理解它和泛型参数的区别。
- 阅读 supertrait、孤儿规则、blanket impl（批量实现：为满足条件的全部类型统一实现）相关代码。
- 判断一个 trait 能否作为 trait object 使用，并读懂常见编译错误。

### 前置知识

本课默认你已经完成前面的课程。下面用一两句话复习关键点；不熟悉时先回到对应课程：

- **结构体、字段、`impl` 块**（[第二课](../02-data-modeling/README.md)）：`struct Email { to: String }` 定义一份数据，`impl Email { ... }` 为它添加方法。
- **所有权与借用**（[第三课](../03-ownership-borrowing/README.md)）：`&self` 表示“借用这个值来读”，不夺走所有权；`&mut self` 表示“借用它来改”。
- **`String` 与 `&str`**（[第三课](../03-ownership-borrowing/README.md)、[第五课](../05-vec-slices/README.md)）：`String` 是拥有一段可增长文本的值，`&str` 是借来看的文本片段。
- **`Result` 与 `Option`**（[第四课](../04-option-result/README.md)）：`Result<T, E>` 表示“成功得到 `T`，或失败得到原因 `E`”；`Option<T>` 表示“可能有 `T`，也可能没有”。
- **迭代器**（[第十课](../10-iterators/README.md)）：`chars()` 逐个产出字符，`count()` 数个数；本课只用最基础的几种。
- **泛型与 trait bound**（[第十一课](../11-generics/README.md)）：`fn f<T: Trait>(x: T)` 里的 `T` 是编译期类型参数，`T: Trait` 声明“`T` 必须具备 `Trait` 描述的能力”；泛型通常通过单态化（为每个具体类型生成一份专用代码）实现零运行时开销。

### 本课边界

- 不系统讲生命周期标注（[第十四课](../14-lifetimes/README.md)）。本课的引用只使用 `&self`、`&str` 这类省略写法，编译器能自动推断。
- 不展开 `Send`/`Sync` 的并发语义（[第十六课](../16-threads-channels/README.md)、[第十七课](../17-shared-concurrency/README.md)）。本课的 trait object 不跨线程传递。
- 不讲 async trait（[第十九课](../19-async/README.md)）和如何定义宏（[第二十课](../20-macros/README.md)）。
- `Box` 的完整用法在[第十三课](../13-box-recursion/README.md)。本课只需要知道：`Box<T>` 把值放到堆（heap，程序运行时可以动态申请的内存区域）上，自己在栈（stack，程序自动管理的内存区域，局部变量通常放在这里）上保存一个指向它的指针；`Box<dyn Trait>` 因此可以表示“大小在编译期未知的 trait object”。

## 如何学习本课

1. 按顺序读第一到第七节。每读一个例子，先遮住输出，自己说出“调用了哪个实现、结果是什么”，再继续。
2. 读第九节时，不要只看修复后的代码：先读诊断原文，找到编译器指出的那一行，再解释根因。
3. 做第十节的代码预测，再对照折叠答案。
4. 做主练习（第十一节）：先运行一次测试，确认失败都来自 `todo!()`，然后按推荐顺序实现。
5. 最后做课后巩固（第十二节）。卡住时按级别查看 `HINTS.md`，不要直接看第三级。

本课的代码块有两种：第八节的播放列表是完整程序，可以直接运行；其余代码块是聚焦单个概念的片段，可能省略 `fn main`、前面已定义的类型或上下文，不能单独编译。练习脚手架（第十一、十二节）则是完整可编译的 package。

预计用时：概念部分约 2 小时，主练习约 1 小时，课后巩固约 45 分钟。

## 一、为什么需要 trait

### 1.1 先看没有 trait 时的重复

假设程序要为一个通知中心生成摘要。三种通知的数据不同：

```rust
struct Email {
    to: String,
    subject: String,
    body: String,
}

struct Sms {
    phone: u64,
    text: String,
}

struct Console {
    level: String,
    message: String,
}
```

`phone` 用 `u64`（64 位无符号整数，只能表示 0 及以上的整数）保存手机号；手机号不需要负数。`to`、`subject` 等字段用 `String`，因为它们是可能增长、需要程序拥有的文本。

如果每种通知各写一个摘要函数：

```rust
fn email_summary(email: &Email) -> String {
    format!("{}: {}", email.subject, email.body)
}

fn sms_summary(sms: &Sms) -> String {
    format!("短信: {}", sms.text)
}

fn console_summary(console: &Console) -> String {
    format!("{} {}", console.level, console.message)
}
```

三段代码的“形状”一样：接收一个值的引用，拼一段文字，返回 `String`。只有字段名和格式不同。调用方也无法写一个统一的函数，因为参数类型不同。

`format!` 是一个按模板生成 `String` 的宏（macro，一种在编译期展开的代码生成工具）：模板里的 `{}` 会依次被后面的值替换。上面的 `email_summary` 把标题和正文填进 `{}: {}`，得到“标题: 正文”。

### 1.2 用枚举能不能解决

能，而且有些场景枚举更合适：

```rust
enum Notice {
    Email(Email),
    Sms(Sms),
    Console(Console),
}

fn summary(notice: &Notice) -> String {
    match notice {
        Notice::Email(email) => format!("{}: {}", email.subject, email.body),
        Notice::Sms(sms) => format!("短信: {}", sms.text),
        Notice::Console(console) => format!("{} {}", console.level, console.message),
    }
}
```

枚举（enum，一组“只能是其中一种”的变体）把三种数据收进同一个类型，`match` 再按变体分支。它的问题是：

- 每增加一种通知，都要同时修改 `Notice` 和 `summary`。如果 `summary` 在别的模块里，还要找到它。
- `Notice` 必须提前知道所有变体。别的代码库定义的新通知类型，无法接进来。

### 1.3 trait 把“能做什么”单独写出来

trait 只声明一组方法签名，不包含字段，也不规定数据长什么样。这里的方法（method）指绑定在某个值上、可以用 `value.name()` 形式调用的函数；方法签名（signature）指方法名、参数类型和返回类型这一整套约定。trait 是一份**契约**（contract，双方都遵守的约定）：实现者承诺“我能提供这些方法”，调用方则只依赖这些方法，不关心背后是哪种数据。

```rust
trait Notice {
    fn title(&self) -> &str;
    fn body(&self) -> &str;
    fn summary(&self) -> String {
        format!("{}: {}", self.title(), self.body())
    }
}
```

逐项读：

- `trait Notice { ... }`：声明一个名为 `Notice` 的行为契约。
- `fn title(&self) -> &str;`：任何实现者都必须提供一个 `title` 方法。`&self` 是方法的接收者（receiver），表示“借用当前值来读”；`-> &str` 表示返回一段借来的文本。
- `fn summary(&self) -> String { ... }`：带函数体的方法，叫默认方法（default method），下一节详细讲。

注意 trait 里没有 `to`、`phone`、`level` 这些字段。字段属于具体的 `struct`，trait 只关心“能不能取标题、取正文”。

为 `Email` 实现这份契约：

```rust
impl Notice for Email {
    fn title(&self) -> &str {
        &self.subject
    }

    fn body(&self) -> &str {
        &self.body
    }
}
```

`impl Notice for Email` 读作“为 `Email` 实现 `Notice`”。`impl` 块里必须逐一兑现契约中所有**没有默认实现**的方法；少一个会编译失败（E0046，见第九节）。

`&self.subject` 的类型是 `&String`，而方法要求返回 `&str`。Rust 会自动把 `&String` 转成 `&str`（这叫解引用强制转换，deref coercion），所以直接写 `&self.subject` 即可。这是第五课讲过的“`String` 可以借成文本切片”的另一个用法。

再为 `Sms` 和 `Console` 实现：

```rust
impl Notice for Sms {
    fn title(&self) -> &str {
        "短信"
    }

    fn body(&self) -> &str {
        &self.text
    }
}

impl Notice for Console {
    fn title(&self) -> &str {
        &self.level
    }

    fn body(&self) -> &str {
        &self.message
    }

    fn summary(&self) -> String {
        format!("{} {}", self.level, self.message)
    }
}
```

`Sms::title` 返回的是字面量 `"短信"`，它的类型是 `&str`，指向程序里固定不变的文本，也能直接作为返回值。`Console` 覆盖了默认的 `summary`，因为控制台消息不需要冒号。

到这里，三种类型都“具备 `Notice` 能力”：

```rust
let email = Email {
    to: String::from("team@example.com"),
    subject: String::from("发布通知"),
    body: String::from("1.0 已上线"),
};
let sms = Sms {
    phone: 138_0000_0000,
    text: String::from("验证码 1234"),
};
let console = Console {
    level: String::from("WARN"),
    message: String::from("磁盘将满"),
};

println!("{}", email.summary());   // 发布通知: 1.0 已上线
println!("{}", sms.summary());     // 短信: 验证码 1234
println!("{}", console.summary()); // WARN 磁盘将满
```

### 1.4 关于 trait 的三条基本事实

1. **一个类型可以实现多个 trait。** `Email` 可以同时实现 `Notice`、`Debug`、`Clone`，彼此独立。
2. **一个 trait 可以被多个类型实现。** 这就是“多态”（polymorphism，同一段调用形式对应多种具体行为）的来源。
3. **trait 只描述行为，不持有数据。** 数据仍在 `struct`/`enum` 里。Rust 不用“基类带字段”的方式共享数据。

`#[derive(Debug)]` 这类写法，本质上是请编译器自动生成一份 trait 实现。`Debug` 是标准库里的 trait，`{:?}` 格式化就依赖它；`#[derive(Debug)]` 与手写 `impl Debug for Email` 地位相同，只是省去机械代码。本课后面会用它让 trait object 也能打印。

## 二、默认方法：把不变的部分写进契约

为聚焦默认方法，本节重新给出了 `Notice` 和 `Console` 的定义；它们与第一节同名同义，实际写代码时只保留一份，不要把两段定义同时粘进同一个文件。

`summary` 的拼接格式对所有通知都一样：`标题: 正文`。如果每个实现都抄一遍，改格式时要改很多处。默认方法把这段不变逻辑放进 trait：

```rust
trait Notice {
    fn title(&self) -> &str;
    fn body(&self) -> &str;

    fn summary(&self) -> String {
        format!("{}: {}", self.title(), self.body())
    }
}
```

实现者只提供 `title` 和 `body`，就自动获得 `summary`：

```rust
struct Article {
    heading: String,
    content: String,
}

impl Notice for Article {
    fn title(&self) -> &str {
        &self.heading
    }

    fn body(&self) -> &str {
        &self.content
    }
}

let article = Article {
    heading: String::from("Rust 1.0 发布"),
    content: String::from("今天发布稳定版"),
};
assert_eq!(article.summary(), "Rust 1.0 发布: 今天发布稳定版");
```

默认方法有两个实际动机：

- **共享行为只写一次。** 真正随类型变化的只有“标题从哪来、正文从哪来”，格式是共同的。
- **让契约可以演化。** 假设一个 trait 已经被很多类型实现，后来想加一个辅助方法。如果新方法没有默认实现，所有实现者立刻编译失败；提供默认实现后，旧实现不用改，愿意定制的实现者再覆盖。标准库的 `Iterator` 就是这样：必须实现的只有 `next`，其余几十个方法都有默认实现。

默认方法有一个重要限制：**它在 trait 定义处检查，只能使用契约里声明的能力**。写默认方法时，编译器还不知道未来会有哪些具体类型，所以它不能访问任何具体类型的字段，也不能调用实现者私有的固有方法（inherent method，直接写在 `impl Type { ... }` 里、不属于任何 trait 的方法）。

覆盖默认方法：

```rust
struct Console {
    level: String,
    message: String,
}

impl Notice for Console {
    fn title(&self) -> &str {
        &self.level
    }

    fn body(&self) -> &str {
        &self.message
    }

    fn summary(&self) -> String {
        format!("{} {}", self.level, self.message)
    }
}
```

`Console` 的实现块里写了 `summary`，调用 `console.summary()` 时用覆盖版本，默认版本不参与。默认方法里调用 `self.title()` 时，如果某个实现覆盖了 `title`，默认方法也会调用到覆盖后的版本——对具体的 `Console` 来说，编译器在编译期就选定了 `Console` 的 `title` 实现（静态分发）。

## 三、trait bound 与静态分发

### 3.1 `T: Notice` 是编译期契约

有了 trait，就能写“对任何具备 `Notice` 能力的类型都成立”的函数：

```rust
fn announce<T: Notice>(notice: &T) -> String {
    format!("【公告】{}", notice.summary())
}
```

`<T: Notice>` 读作：“`T` 是某个实现了 `Notice` 的类型”。函数体里只能用 `Notice` 承诺过的方法；想访问 `notice` 的字段，或者调用 `Notice` 没声明的方法，都会编译失败。

调用时，编译器为每个具体类型分别生成一份代码：

```rust
let email = Email { /* ... */ };
let sms = Sms { /* ... */ };

println!("{}", announce(&email)); // 使用 Email 版本的 announce
println!("{}", announce(&sms));   // 使用 Sms 版本的 announce
```

这个过程叫**单态化**（monomorphization）：泛型函数在编译期被“复制”成具体类型版本。第十一课讲过，它的结果是运行时没有“查表找方法”的开销，代价是代码体积可能变大。

这种“编译期就确定调用哪份代码”的方式叫**静态分发**（static dispatch）。静态分发是 Rust 的默认选择。

### 3.2 `impl Trait` 参数是泛型的简写

下面两个签名在大多数场景下等价：

```rust
fn announce_generic<T: Notice>(notice: &T) -> String {
    format!("【公告】{}", notice.summary())
}

fn announce_impl(notice: &impl Notice) -> String {
    format!("【公告】{}", notice.summary())
}
```

`&impl Notice` 读作“参数是某个实现了 `Notice` 的类型的引用”。它是泛型参数的语法糖（语法糖：不增加新能力、只是让写法更简洁的替代语法），两个函数体一模一样，调用方写法也完全一样。

差别在少数细节上：

- 用 `T` 时可以在调用处显式指定类型：turbofish 写法 `announce_generic::<Email>(&email)`，即在函数名后用 `::<类型>` 指定泛型参数；`impl Trait` 没有名字，不能这样指定。
- 一个函数里出现多个 `impl Trait` 参数，它们是**各自独立**的类型参数；如果要求两个参数是同一类型，必须写成同一个 `T`。

```rust
// a 和 b 可以是不同类型，只要是两个实现了 Notice 的类型
fn pair(a: &impl Notice, b: &impl Notice) -> String {
    format!("{} / {}", a.summary(), b.summary())
}

// a 和 b 必须是同一个具体类型
fn same_pair<T: Notice>(a: &T, b: &T) -> String {
    format!("{} / {}", a.summary(), b.summary())
}
```

这里的选择不是风格问题，而是签名表达能力：需要“两个参数同类型”时，`T` 更准确；只需要“各自具备能力”时，`impl Trait` 更简洁。

### 3.3 bound 不够时会发生什么

如果函数体里想用 `Debug` 格式化，但 bound 只写了 `Notice`：

```rust,compile_fail
fn show<T: Notice>(notice: &T) {
    println!("{notice:?}");
}
```

编译失败，核心诊断是：

```text
error[E0277]: `T` doesn't implement `Debug`
```

`T` 的 bound 只有 `Notice`，编译器不能假设它还能用 `{:?}` 打印。修复方向是补上能力声明：

```rust
fn show<T: Notice + std::fmt::Debug>(notice: &T) {
    println!("{notice:?}");
}
```

或者用 `where` 子句写得更清楚：

```rust
fn show<T>(notice: &T)
where
    T: Notice + std::fmt::Debug,
{
    println!("{notice:?}");
}
```

`where` 和参数列表里的 bound 含义相同，只是把约束挪到签名下方；约束多时更易读。第十一课已经用过这种写法。

## 四、返回 `impl Trait`

参数位置的 `impl Trait` 是“我接受任何具备该能力的类型”；返回位置的 `impl Trait` 是“我返回某个具备该能力的类型，但我不告诉你具体是哪个”。

```rust
fn default_notice() -> impl Notice {
    Console {
        level: String::from("INFO"),
        message: String::from("系统就绪"),
    }
}

let notice = default_notice();
println!("{}", notice.summary());
```

调用者只能使用 `Notice` 声明的方法，不能把返回值当成 `Console` 去访问 `.level`。函数内部当然知道自己返回的是 `Console`。

两个动机：

- **有些类型写不出名字。** 迭代器链和闭包的类型由编译器生成，源代码里没有可写的名字。返回 `impl Iterator<Item = i32>` 比写出层层嵌套的具体类型现实得多（第十课见过）。
- **保留修改实现的自由。** 签名承诺的是能力，不是具体类型。下个版本把内部的 `Console` 换成别的结构，只要仍实现 `Notice`，调用方不受影响。

规则的另一半：**同一个函数的所有返回路径必须是同一个具体类型**。下面这样不行：

```rust,compile_fail
fn make_notice(flag: bool) -> impl Notice {
    if flag {
        Console { level: String::from("INFO"), message: String::from("a") }
    } else {
        Article { heading: String::from("h"), content: String::from("b") }
    }
}
```

诊断是 E0308：

```text
error[E0308]: `if` and `else` have incompatible types
```

`impl Notice` 不是“任意一种实现了 `Notice` 的类型”，而是“一个确定的、被隐藏的具体类型”。两个分支返回不同类型，编译器无法为返回值选出一个类型。需要真正在运行时二选一时，用下一节的 `Box<dyn Notice>`。

## 五、`dyn Trait`：运行时选择实现

### 5.1 为什么需要 trait object

静态分发适合“编译期知道具体类型”的调用。但有一类需求它解决不了：把不同类型放进同一个 `Vec`。

```rust
let emails: Vec<Email> = Vec::new();
// 这个 Vec 只能装 Email，装不下 Sms 或 Console
```

`Vec<T>` 要求所有元素是同一个类型 `T`。`Email`、`Sms`、`Console` 是三个不同类型，不能直接混装。

把不同具体类型的值放进同一个集合，叫**异构集合**（heterogeneous collection，元素类型可以不同）。普通 `Vec<T>` 做不到，因为它在编译期要求所有元素同类型。

`dyn Trait`（读作“dynamic Trait”，动态的 Trait）表示“某个实现了 `Trait` 的类型，具体是哪个到运行时才知道”。因为具体类型的大小在编译期未知，`dyn Trait` 不能直接当普通值使用，必须放在指针后面：

```rust
let notices: Vec<Box<dyn Notice>> = vec![
    Box::new(Email {
        to: String::from("team@example.com"),
        subject: String::from("发布通知"),
        body: String::from("1.0 已上线"),
    }),
    Box::new(Sms {
        phone: 138_0000_0000,
        text: String::from("验证码 1234"),
    }),
    Box::new(Console {
        level: String::from("WARN"),
        message: String::from("磁盘将满"),
    }),
];

for notice in &notices {
    println!("{}", notice.summary());
}
```

`Box::new(value)` 把值放到堆上，返回一个 `Box<具体类型>`；在需要 `Box<dyn Notice>` 的位置，编译器会自动把它转成 trait object（trait 对象）。

`dyn Trait` 这种类型叫 trait object。trait object 本身大小未知；指向它的指针（`&dyn Trait`、`Box<dyn Trait>`）是**胖指针**（fat pointer，占两个机器字），由两部分组成：

这里的“机器字”（word）指 CPU 一次能处理的数据宽度，64 位平台上通常是 8 字节。

1. 一个指向堆上具体数据的指针；
2. 一个指向**虚函数表**（vtable，一张记录“这个具体类型的每个 trait 方法入口在哪”的表）的指针。

调用 `notice.summary()` 时，程序先通过 vtable 找到 `summary` 的函数地址，再跳过去执行。这叫**动态分发**（dynamic dispatch）：具体调用哪份代码在运行时才确定。

### 5.2 静态分发与动态分发的取舍

| | 静态分发（泛型） | 动态分发（`dyn Trait`） |
| --- | --- | --- |
| 何时确定实现 | 编译期 | 运行时查 vtable |
| 运行时开销 | 通常没有额外开销，可内联 | 一次间接调用，通常不能跨调用内联 |
| 内存开销 | 每个具体类型生成一份代码 | 每个 trait object 多一个 vtable 指针 |
| 能否混装不同类型 | 不能（同一泛型实例只能一种类型） | 能 |
| 代码体积 | 可能膨胀 | 较小 |

默认优先用泛型/静态分发；只有确实需要“运行时才知道类型”或“异构集合”时，才引入 `dyn`。不要为了“看起来灵活”把所有参数都改成 trait object。

### 5.3 为什么不能直接返回 `dyn Trait`

```rust,compile_fail
fn make() -> dyn Notice {
    // ...
}
```

诊断是 E0746：

```text
error[E0746]: return type cannot be a trait object without pointer indirection
```

编译器不知道 `dyn Notice` 的大小，无法在栈上为返回值预留空间。返回值必须是指针形式：

```rust
fn make() -> Box<dyn Notice> {
    Box::new(Console {
        level: String::from("INFO"),
        message: String::from("系统就绪"),
    })
}
```

同理，`let notice: dyn Notice = ...;` 也不合法；`&dyn Notice` 和 `Box<dyn Notice>` 才合法。

### 5.4 supertrait：trait 也可以要求另一种能力

```rust
use std::fmt::Debug;

trait ErasedNotice: Debug {
    fn title(&self) -> &str;
    fn summary(&self) -> String;
}
```

`ErasedNotice: Debug` 里的 `: Debug` 叫 **supertrait**（父 trait）：任何实现 `ErasedNotice` 的类型，必须同时实现 `Debug`。这样在使用 `dyn ErasedNotice` 时，编译器知道它一定能用 `{:?}` 格式化。

supertrait 不会自动帮你实现 `Debug`，它只是增加一条“必须同时具备”的要求。实现者仍要自己写 `impl Debug` 或使用 `#[derive(Debug)]`。

supertrait 的方法在默认方法里也可以调用；这和第 2 节的默认方法限制一致：trait 只能使用自己声明过、或 supertrait 提供的能力。

### 5.5 什么样的 trait 能做 trait object

trait object 的 vtable 必须在编译期为每个方法准备一个固定入口。因此并非所有 trait 都能这样用。一个 trait 能作为 `dyn Trait` 使用，叫 **dyn compatible**（旧称 object safe，对象安全）。主要规则：

- 方法必须有接收者（`self`、`&self`、`&mut self`、`Box<Self>` 等）。没有 `self` 的关联函数无法通过某个具体对象调用。
- 方法不能带自己的泛型参数。`fn parse<T>(&self, input: &str) -> T` 对每个 `T` 都要一份代码，vtable 无法穷举。
- 方法不能按值返回 `Self`。`Self` 是“当前具体类型”，大小未知，vtable 无法为它安排返回值。
- 不能有关联常量（associated const，绑定在 trait 上、由每个实现给出值的常量）。
- 关联类型必须在 `dyn Trait<Assoc = 具体类型>` 中指定。

违反规则时常见诊断是 E0038：

```rust,compile_fail
trait Cloner {
    fn duplicate(&self) -> Self;
}
```

```text
error[E0038]: the trait `Cloner` is not dyn compatible
  ...
  ...because method `duplicate` references the `Self` type in its return type
```

如果某个方法确实不需要通过 trait object 调用，可以用 `where Self: Sized` 把它排除在 vtable 之外：

```rust
trait Parser {
    fn parse(&self, input: &str) -> String;
    fn parse_twice(&self, input: &str) -> (String, String)
    where
        Self: Sized,
    {
        (self.parse(input), self.parse(input))
    }
}
```

`Self: Sized` 读作“当前具体类型的大小在编译期已知”。带这个约束的方法只能对具体类型调用，不能通过 `dyn Parser` 调用；trait 本身仍然可以生成 trait object。

## 六、关联类型：一个实现只对应一种关联数据

本节为了加入“目的地”，把通知契约扩展成带关联类型的版本。它与前面同名，但内容不同；如果在同一个文件里试验，请用这一版替换前面的 `trait Notice` 和三个 `impl Notice for ...`，不要同时保留两份同名定义。5.4 节的 `ErasedNotice` 只是用来说明 supertrait，本节 6.3 会给出完整版本，同样需要替换。

### 6.1 问题：目的地类型随通知类型变化

邮件的目的地是邮箱地址（`String`），短信的目的地是手机号（`u64`），控制台没有目的地（可以用空元组 `()` 表示“没有值”）。如果用泛型参数写：

```rust
trait Notice<Target> {
    fn title(&self) -> &str;
    fn target(&self) -> &Target;
}
```

那么同一个 `Email` 可以同时实现 `Notice<String>` 和 `Notice<u64>`。这在语义上不合理：一封邮件只有一种目的地。更麻烦的是调用处：`email.target()` 返回的是 `&String` 还是 `&u64`？编译器无法从 `&self` 推断，调用者必须写完全限定形式消歧（完全限定形式：显式写出 `Trait::method(&value)`，而不是 `value.method()`）。

关联类型（associated type）把“这个实现绑定的类型”写进 trait 内部：

```rust
trait Notice {
    type Target;

    fn title(&self) -> &str;
    fn target(&self) -> &Self::Target;
    fn body(&self) -> &str;

    fn summary(&self) -> String {
        format!("{}: {}", self.title(), self.body())
    }
}
```

- `type Target;` 声明一个关联类型：每个实现必须给出它的具体类型。
- `Self::Target` 表示“当前实现所指定的那个 `Target`”。

实现：

```rust
impl Notice for Email {
    type Target = String;

    fn title(&self) -> &str { &self.subject }
    fn target(&self) -> &Self::Target { &self.to }
    fn body(&self) -> &str { &self.body }
}

impl Notice for Sms {
    type Target = u64;

    fn title(&self) -> &str { "短信" }
    fn target(&self) -> &Self::Target { &self.phone }
    fn body(&self) -> &str { &self.text }
}

impl Notice for Console {
    type Target = ();

    fn title(&self) -> &str { &self.level }
    fn target(&self) -> &Self::Target { &() }
    fn body(&self) -> &str { &self.message }
}
```

`()` 是“空元组”，只有一个值 `()`；它不携带信息，本身也不占数据空间，适合表示“没有目的地”。`&()` 是一个指向这个空值的引用。

在泛型函数里引用关联类型：

```rust
fn target_of<T: Notice>(notice: &T) -> &T::Target {
    notice.target()
}
```

`T::Target` 是 `<T as Notice>::Target` 的简写，读作“`T` 作为 `Notice` 实现时的 `Target`”。当 `T` 同时实现多个带同名关联类型的 trait 时，必须写完整形式消歧。

### 6.2 关联类型与泛型参数的区别

| | 关联类型 `trait Notice { type Target; }` | 泛型参数 `trait Notice<Target>` |
| --- | --- | --- |
| 一个类型能有几份实现 | 每个 trait 一份 | 可以有 `Notice<String>`、`Notice<u64>` 多份 |
| 调用处能否直接推断 | 能，`Target` 由实现唯一确定 | 不能，常需 turbofish 或完全限定 |
| 表达的关系 | “一对一”：通知类型决定目的地类型 | “一对多”或“多对多” |
| 适合场景 | 实现天然绑定一种关联数据 | 同一个类型需要按不同参数实现多次 |

反过来说，关联类型版本里，同一个类型不能对 `Notice` 写两份实现；第二份会报 E0119（conflicting implementations，实现冲突）。这正是“一对一”关系的强制效果。

选择原则：如果“这个类型实现该 trait 时，关联数据只有一种”，用关联类型；如果确实需要同一类型针对不同参数多次实现，用泛型参数。

关联类型带来一个直接后果：`dyn Notice` 无法单独成为一个类型。诊断是 E0191：

```rust,compile_fail
let notices: Vec<Box<dyn Notice>> = Vec::new();
```

```text
error[E0191]: the value of the associated type `Target` in `Notice` must be specified
  ...
  help: specify the associated type: `Notice<Target = Type>`
```

因为 `dyn Notice` 必须知道 `Target` 是什么，才能为 `target()` 安排返回值。写 `Box<dyn Notice<Target = String>>` 可以，但它只能装 `Target = String` 的实现（比如 `Email`），装不下 `Sms` 和 `Console`。三种通知的 `Target` 不同，无法放进同一个集合。

### 6.3 类型擦除：恢复异构集合

要把三种通知放进同一个队列，需要把“关联类型”从 trait object 的签名里抹掉。做法是再定义一个 trait，把所有结果统一成 `String`：

```rust
use std::fmt::Debug;

trait TargetText {
    fn to_text(&self) -> String;
}

impl TargetText for String {
    fn to_text(&self) -> String {
        self.clone()
    }
}

impl TargetText for u64 {
    fn to_text(&self) -> String {
        self.to_string()
    }
}

impl TargetText for () {
    fn to_text(&self) -> String {
        String::from("console")
    }
}

trait ErasedNotice: Debug {
    fn title(&self) -> &str;
    fn target_text(&self) -> String;
    fn summary(&self) -> String;
}
```

注意 `ErasedNotice: Debug` 要求实现者也实现 `Debug`。如果照抄第一节的三个 `struct` 定义，需要分别在 `struct Email`、`struct Sms`、`struct Console` 上方补一行 `#[derive(Debug)]`；否则把 `Box::new(email)` 转成 `Box<dyn ErasedNotice>` 时会报 “`Email` doesn't implement `Debug`”。

`ErasedNotice` 没有任何关联类型，所以 `dyn ErasedNotice` 是合法的单一类型。接下来用 **blanket impl**（批量实现：为所有满足条件的类型统一写一份实现）把 `Notice` 的能力搬过来：

```rust
impl<T: Notice + Debug> ErasedNotice for T
where
    T::Target: TargetText,
{
    fn title(&self) -> &str {
        Notice::title(self)
    }

    fn target_text(&self) -> String {
        self.target().to_text()
    }

    fn summary(&self) -> String {
        Notice::summary(self)
    }
}
```

逐行读：

- `impl<T: Notice + Debug> ErasedNotice for T`：为**每一个**同时实现 `Notice` 和 `Debug` 的类型 `T` 实现 `ErasedNotice`。
- `where T::Target: TargetText`：额外要求 `T` 的关联类型 `Target` 实现 `TargetText`。没有这条，`self.target().to_text()` 不合法。
- `Notice::title(self)`：用完全限定形式调用 `Notice` 的 `title`，避免和 `ErasedNotice::title` 同名混淆。

现在可以构造异构队列：

```rust
let queue: Vec<Box<dyn ErasedNotice>> = vec![
    Box::new(email),
    Box::new(sms),
    Box::new(console),
];

for item in &queue {
    println!("{} -> {}", item.title(), item.target_text());
}
```

`Box::new(email)` 的类型是 `Box<Email>`。因为 `Email: Notice + Debug` 且 `Email::Target = String: TargetText`，blanket impl 让 `Email` 自动实现了 `ErasedNotice`，所以它能转成 `Box<dyn ErasedNotice>`。

这里有两层设计，各自解决不同问题：

- **静态层**（`Notice`）：保留精确的关联类型。`target_of(&email)` 返回 `&String`，调用方拿到的是原始数据，不是文本。
- **动态层**（`ErasedNotice`）：把关联类型统一成 `String`，换取“不同类型放进同一集合”的能力。代价是原始类型信息丢失，只剩文本。

不要因为动态层方便就删掉静态层；两者服务不同场景。主练习会同时要求这两层。

### 6.4 为什么 `TargetText` 要自己定义

`String` 和 `u64` 已经实现了标准库的 `Display`（能用 `{}` 打印），但 `()` 没有实现 `Display`，而且 `Display` 是外部 trait、`()` 是外部类型，孤儿规则（orphan rule；下一节详细讲，简单说：trait 和类型至少有一个由当前 crate 定义）不允许我们在自己的 crate（Rust 的编译单元；一个 Cargo package 通常对应一个 crate）里补这份实现。定义本地的 `TargetText`，再为三种类型分别实现，就绕开了这个限制：trait 是本地的，规则允许。

## 七、supertrait、孤儿规则与 newtype

### 7.1 孤儿规则

Rust 有一条**孤儿规则**（orphan rule）：实现 trait 时，trait 和类型至少要有一个定义在当前 crate 里。也就是：

- 本地 trait + 本地类型：允许。
- 本地 trait + 外部类型：允许（本课的 `TargetText for u64`）。
- 外部 trait + 本地类型：允许（本课的 `Display for LineList`）。
- 外部 trait + 外部类型：禁止。

最后一条正是“孤儿”这个名字的来源：这个实现既不属于 trait 的 crate，也不属于类型的 crate，没有“家”。

尝试给 `Vec<String>` 实现标准库的 `Display`：

```rust,compile_fail
use std::fmt::{self, Display};

impl Display for Vec<String> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.join("\n"))
    }
}
```

诊断是 E0117：

```text
error[E0117]: only traits defined in the current crate can be implemented for types defined outside of the crate
  ...
  = note: define and implement a trait or new type instead
```

为什么要有这条规则？因为 Rust 要求“对任意类型调用任意 trait 方法，全世界只有一份实现”。如果允许外部 trait + 外部类型，两个互不认识的 crate 都可能给 `Vec<String>` 实现 `Display`，一旦同时被引入，编译器无法选择，链接后的程序也会出现两份冲突实现。孤儿规则把这种冲突挡在编译期。

### 7.2 newtype：用一层薄包装获得本地类型

不能直接给 `Vec<String>` 实现 `Display`，但可以定义一个新类型包住它：

```rust
use std::fmt::{self, Display};

struct LineList(Vec<String>);

impl Display for LineList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.join("\n"))
    }
}

let list = LineList(vec![
    String::from("alpha"),
    String::from("beta"),
]);

println!("{list}");
```

输出：

```text
alpha
beta
```

`struct LineList(Vec<String>);` 是元组结构体（tuple struct）：字段没有名字，用 `self.0` 访问第一个字段。`LineList` 是本地类型，`Display` 是外部 trait，本地类型 + 外部 trait 合法。

`fmt` 是 `Display` 要求实现的方法；`f` 是格式化器（formatter），负责接收要输出的文本；`fmt::Result` 表示写入是否成功。`write!` 把格式化后的内容写进 `f`；`join("\n")` 用换行符把多个 `String` 连接成一个 `String`。

这种“只包一层、用来获得本地身份”的写法叫 **newtype**（新类型）。它不会额外复制内部数据：`LineList` 只是把 `Vec` 的所有权接过来。需要访问内部 `Vec` 时，通过 `self.0` 即可；如果希望外部也能取回，可以实现 `Into<Vec<String>>` 或提供方法，但本课不需要。

newtype 也常用于给外部类型附加语义，例如 `struct Meters(f64)` 和 `struct Seconds(f64)` 防止单位混用。它通常不增加运行时开销，只在类型层面增加区分。

### 7.3 blanket impl 的一致性代价

blanket impl 很强：一行 `impl<T: ...> ErasedNotice for T` 就覆盖了未来所有满足条件的类型。但它也占住了这块“实现空间”：如果之后想给某个具体类型写一份特殊的 `ErasedNotice` 实现，就会与 blanket impl 冲突（E0119， conflicting implementations）。设计库 API 时，blanket impl 要谨慎，因为它把扩展点收走了。

本课的 blanket impl 是安全的：`ErasedNotice` 是本地 trait，且我们确实希望所有 `Notice + Debug` 类型都走同一套擦除逻辑，不需要例外。

## 八、完整示例：播放列表

下面这个例子和两个练习都不同，用来把本课的概念串起来：默认方法、覆盖、静态分发、异构集合、`dyn Trait`。代码可以直接放进一个 `main.rs` 运行。

```rust
trait Track {
    fn title(&self) -> &str;
    fn seconds(&self) -> u32;

    fn label(&self) -> String {
        format!(
            "{} ({}:{:02})",
            self.title(),
            self.seconds() / 60,
            self.seconds() % 60
        )
    }
}

struct Song {
    title: String,
    seconds: u32,
}

struct Audiobook {
    title: String,
    minutes: u32,
}

struct Podcast {
    title: String,
    seconds: u32,
    episode: u32,
}

impl Track for Song {
    fn title(&self) -> &str {
        &self.title
    }

    fn seconds(&self) -> u32 {
        self.seconds
    }
}

impl Track for Audiobook {
    fn title(&self) -> &str {
        &self.title
    }

    fn seconds(&self) -> u32 {
        self.minutes * 60
    }
}

impl Track for Podcast {
    fn title(&self) -> &str {
        &self.title
    }

    fn seconds(&self) -> u32 {
        self.seconds
    }

    fn label(&self) -> String {
        format!(
            "{} #{} ({}:{:02})",
            self.title,
            self.episode,
            self.seconds / 60,
            self.seconds % 60
        )
    }
}

fn first_label<T: Track>(track: &T) -> String {
    track.label()
}

fn queue_seconds(queue: &[Box<dyn Track>]) -> u32 {
    queue.iter().map(|track| track.seconds()).sum()
}

fn main() {
    let queue: Vec<Box<dyn Track>> = vec![
        Box::new(Song {
            title: String::from("Rust 之歌"),
            seconds: 125,
        }),
        Box::new(Audiobook {
            title: String::from("所有权入门"),
            minutes: 3,
        }),
        Box::new(Podcast {
            title: String::from("每周 Rust"),
            seconds: 95,
            episode: 7,
        }),
    ];

    for track in &queue {
        println!("{}", track.label());
    }

    println!("总时长: {} 秒", queue_seconds(&queue));
    println!("第一项: {}", first_label(&Song {
        title: String::from("编译通过"),
        seconds: 61,
    }));
}
```

输出：

```text
Rust 之歌 (2:05)
所有权入门 (3:00)
每周 Rust #7 (1:35)
总时长: 400 秒
第一项: 编译通过 (1:01)
```

数据如何流动：

1. 三个具体类型分别实现 `Track`。`Song` 和 `Audiobook` 用默认 `label`，`Podcast` 覆盖它，额外显示集数。
2. `first_label` 是泛型函数，`T` 在调用处确定为 `Song`，编译期生成 `Song` 专用版本，属于静态分发。
3. `queue` 是 `Vec<Box<dyn Track>>`，三个 `Box` 各自保存具体数据和 vtable 入口。`queue_seconds` 遍历时通过 vtable 调用各自的 `seconds`，属于动态分发。
4. `Audiobook::seconds` 把分钟换算成秒（`minutes * 60`），所以总时长是 `125 + 180 + 95 = 400`。

注意 `first_label` 只接受同一具体类型的引用；它不能接收 `&dyn Track`。如果需要通过 trait object 调用，参数要写成 `track: &dyn Track`。这两种签名表达不同的接受范围。

## 九、编译错误与排错

本节的诊断来自实际编译器输出（Rust 1.94）。不同版本措辞可能略有变化，错误码和根因不变。

### 9.1 E0277：缺少 trait bound

```text
error[E0277]: `T` doesn't implement `Debug`
 --> src/main.rs:3:15
  |
3 |     println!("{item:?}");
  |               ^^^^^^^^ `T` cannot be formatted using `{:?}` because it doesn't implement `Debug`
  |
  = note: required for `&T` to implement `Debug`
help: consider further restricting type parameter `T` with trait `Debug`
```

**根因**：泛型函数体只允许使用 bound 声明的能力。`T: Summary` 没有承诺 `Debug`。

**修复方向**：在 bound 里补 `Debug`（`T: Summary + Debug`），或者用 `where` 子句追加。不要用 `{:?}` 去打印一个没有声明 `Debug` 的泛型值。

### 9.2 E0046：trait 方法没有全部实现

```text
error[E0046]: not all trait items implemented, missing: `bye`
 --> src/main.rs:6:1
  |
3 |     fn bye(&self) -> String;
  |     ------------------------ `bye` from trait
...
6 | impl Greet for Bot {
  | ^^^^^^^^^^^^^^^^^^ missing `bye` in implementation
```

**根因**：trait 声明了 `hello` 和 `bye`，其中 `bye` 没有默认实现，实现块只写了 `hello`。

**修复方向**：补上缺失的方法，或者给 trait 里的方法提供默认实现（如果你有权修改 trait）。诊断会直接点名缺少哪个方法。

### 9.3 E0117：孤儿规则

```text
error[E0117]: only traits defined in the current crate can be implemented for types defined outside of the crate
 --> src/main.rs:2:1
  |
2 | impl Display for Vec<String> {
  | ^^^^^^^^^^^^^^^^^-----------
  |                  |
  |                  `Vec` is not defined in the current crate
  |
  = note: define and implement a trait or new type instead
```

**根因**：`Display` 和 `Vec` 都来自标准库，两个都不是本地定义。

**修复方向**：用 newtype 包一层（第 7.2 节），或者改为实现本地 trait。不要试图用 `#[derive]` 绕过：`derive` 也只能生成合法的实现。

### 9.4 E0191：trait object 没有指定关联类型

```text
error[E0191]: the value of the associated type `Target` in `Notice` must be specified
 --> src/main.rs:3:29
  |
1 | trait Notice { type Target; fn target(&self) -> &Self::Target; }
  |                ----------- `Target` defined here
3 |     let _items: Vec<Box<dyn Notice>> = Vec::new();
  |                             ^^^^^^ help: specify the associated type: `Notice<Target = Type>`
```

**根因**：`dyn Notice` 必须知道 `Target` 的具体类型，才能确定 `target()` 的返回类型。

**修复方向**：如果只需要一种目的地，写 `dyn Notice<Target = String>`；如果要混装不同目的地，定义擦除层（第 6.3 节），让 trait object 的方法直接返回 `String`。

### 9.5 E0038：trait 不能作为 trait object

```text
error[E0038]: the trait `Cloner` is not dyn compatible
 --> src/main.rs:5:29
  |
5 |     let _items: Vec<Box<dyn Cloner>> = Vec::new();
  |                             ^^^^^^ `Cloner` is not dyn compatible
  |
note: for a trait to be dyn compatible it needs to allow building a vtable
  |
1 | trait Cloner { fn duplicate(&self) -> Self; }
  |       ------                          ^^^^ ...because method `duplicate` references the `Self` type in its return type
```

**根因**：`duplicate` 按值返回 `Self`，而 `dyn Cloner` 不知道 `Self` 的大小，vtable 无法安排这样的返回。

**修复方向**：改为返回 `Box<dyn Cloner>`；或者把该方法标成 `where Self: Sized`，让它只对具体类型可用；或者把方法拆到另一个不用于 trait object 的 trait 里。

### 9.6 E0034：多个 trait 提供同名方法

```text
error[E0034]: multiple applicable items in scope
 --> src/main.rs:8:18
  |
8 |     let _ = item.name();
  |                  ^^^^ multiple `name` found
  |
note: candidate #1 is defined in an impl of the trait `A` for the type `Item`
note: candidate #2 is defined in an impl of the trait `B` for the type `Item`
help: disambiguate the method for candidate #1
  |
8 -     let _ = item.name();
8 +     let _ = A::name(&item);
```

**根因**：`Item` 同时通过 trait `A` 和 trait `B` 提供了 `name`，方法调用的点号写法无法决定用哪一个。

**修复方向**：用完全限定形式 `A::name(&item)` 或 `B::name(&item)` 指定。若其中一个方法是类型自带的固有方法（`impl Item { fn name(&self) ... }`），固有方法优先，点号调用**不会**报 E0034，而是静默选择固有方法；这时想调用 trait 版本，仍要写 `A::name(&item)`。这是很常见的踩坑点：两个同名方法都存在时，先确认实际调用的是哪个。

### 9.7 E0308 / E0746：返回类型不一致或缺少指针

`impl Trait` 的两个分支返回不同类型：

```text
error[E0308]: `if` and `else` have incompatible types
  |
7 |     if flag { Article } else { Report }
  |               -------          ^^^^^^ expected `Article`, found `Report`
  |
help: you could change the return type to be a boxed trait object
```

按值返回 `dyn Trait`：

```text
error[E0746]: return type cannot be a trait object without pointer indirection
  |
2 | fn make() -> dyn Summary {
  |              ^^^^^^^^^^^ doesn't have a size known at compile-time
  |
help: consider returning an `impl Trait` instead of a `dyn Trait`
help: alternatively, box the return type, and wrap all of the returned values in `Box::new`
```

**根因与修复**：`impl Trait` 承诺“一个隐藏但唯一的具体类型”，不能在不同分支换成另一个类型；需要运行时二选一时改用 `Box<dyn Trait>`。`dyn Trait` 本身大小未知，必须放在 `Box` 或引用后面。

## 十、代码预测与解析

先写下答案，再展开对照。每题的答案都包含“为什么”。

### 题目 1：同名方法

```rust
trait A {
    fn name(&self) -> &str {
        "A"
    }
}

trait B {
    fn name(&self) -> &str {
        "B"
    }
}

struct Item;

impl A for Item {}
impl B for Item {}

fn main() {
    let item = Item;
    println!("{}", item.name());
}
```

这段代码能编译吗？如果不能，怎样分别调用两个版本？

<details>
<summary>答案与解析</summary>

不能编译，错误是 E0034：`multiple applicable items in scope`。`Item` 通过两个 trait 都提供了 `name`，点号写法无法选择。

分别调用：

```rust
println!("{}", A::name(&item)); // A
println!("{}", B::name(&item)); // B
```

如果 `Item` 自己还有一个固有方法 `fn name(&self)`，点号写法会优先选固有方法，不报错；但 trait 版本仍要显式限定。

</details>

### 题目 2：默认方法与覆盖

```rust
trait Greet {
    fn name(&self) -> &str;

    fn hello(&self) -> String {
        format!("hello, {}", self.name())
    }
}

struct Bot;

impl Greet for Bot {
    fn name(&self) -> &str {
        "bot"
    }

    fn hello(&self) -> String {
        format!("HELLO, {}", self.name())
    }
}

fn main() {
    println!("{}", Bot.hello());
}
```

输出是什么？如果删掉 `Bot` 的 `hello`，输出又是什么？

<details>
<summary>答案与解析</summary>

第一问输出 `HELLO, bot`：`Bot` 覆盖了默认 `hello`，调用的是覆盖版本。

第二问输出 `hello, bot`：默认 `hello` 调用 `self.name()`，而 `name` 由 `Bot` 实现，返回 `"bot"`。默认方法不必知道具体类型，它只依赖契约里的方法。

</details>

### 题目 3：`impl Trait` 返回两个分支

```rust
trait Summary {
    fn summary(&self) -> String;
}

struct Article;
struct Report;

impl Summary for Article {
    fn summary(&self) -> String {
        String::from("article")
    }
}

impl Summary for Report {
    fn summary(&self) -> String {
        String::from("report")
    }
}

fn make(flag: bool) -> impl Summary {
    if flag { Article } else { Report }
}
```

为什么编译失败？两种修法分别是什么？

<details>
<summary>答案与解析</summary>

`impl Summary` 只能代表一个确定的隐藏类型。`if` 分支返回 `Article`，`else` 分支返回 `Report`，编译器无法选出一个类型，报 E0308。

修法一：返回 `Box<dyn Summary>`，两个分支分别 `Box::new(Article)` 和 `Box::new(Report)`。这是运行时二选一。

修法二：如果调用方其实只需要其中一种类型，就让函数只返回那一种，或者把两个分支的行为统一到一个本地类型里（例如一个枚举，并为枚举实现 `Summary`）。不要为了保留 `impl Trait` 而强行让两个不同类型“变成”一个。

</details>

### 题目 4：关联类型与 trait object

```rust
trait Container {
    type Item;

    fn first(&self) -> Option<&Self::Item>;
}

struct Numbers(Vec<i32>);
struct Words(Vec<String>);

impl Container for Numbers {
    type Item = i32;

    fn first(&self) -> Option<&Self::Item> {
        self.0.first()
    }
}

impl Container for Words {
    type Item = String;

    fn first(&self) -> Option<&Self::Item> {
        self.0.first()
    }
}
```

能把 `Numbers` 和 `Words` 一起放进 `Vec<Box<dyn Container>>` 吗？如果不能，有哪两种常见处理方式？

<details>
<summary>答案与解析</summary>

不能。`dyn Container` 必须指定 `Item`（E0191）；写 `dyn Container<Item = i32>` 只能装 `Numbers`，写 `Item = String` 只能装 `Words`。两个实现的关联类型不同，无法共用一个 trait object 类型。

处理方式：

1. **类型擦除**：定义一个不含关联类型的新 trait，把 `first` 的结果统一转成 `String`（或枚举），再为所有 `Container` 实现它。这是第 6.3 节的做法。
2. **枚举**：定义 `enum AnyContainer { Numbers(Numbers), Words(Words) }`，用 `match` 分派。变体固定时枚举更简单。

如果调用方其实不需要混装，保留泛型 `T: Container` 的静态分发即可，不必引入 trait object。

</details>

### 题目 5：blanket impl 的覆盖范围

```rust
use std::fmt::Debug;

trait Check {
    fn ok(&self) -> bool;
}

trait Erased: Debug {
    fn ok_text(&self) -> String;
}

impl<T: Check + Debug> Erased for T {
    fn ok_text(&self) -> String {
        if self.ok() {
            String::from("ok")
        } else {
            String::from("bad")
        }
    }
}

struct Plain;
impl Check for Plain {
    fn ok(&self) -> bool {
        true
    }
}
```

`Plain` 能放进 `Box<dyn Erased>` 吗？

<details>
<summary>答案与解析</summary>

不能。blanket impl 要求 `T: Check + Debug`，而 `Plain` 只实现了 `Check`，没有实现 `Debug`。`Plain` 不满足 `Erased` 的 supertrait 要求，也就没有自动获得 `Erased` 实现。

修复：给 `Plain` 加 `#[derive(Debug)]`，或手写 `impl Debug for Plain`。这也说明 supertrait 是硬性要求，blanket impl 不会替你补上缺失的能力。

</details>

### 题目 6：默认方法能访问什么

```rust
trait Named {
    fn name(&self) -> &str;

    fn greeting(&self) -> String {
        format!("hello, {}", self.display_name())
    }
}

struct User {
    name: String,
}

impl User {
    fn display_name(&self) -> &str {
        &self.name
    }
}
```

如果只给 `User` 实现 `Named::name`，`greeting` 能通过编译吗？

<details>
<summary>答案与解析</summary>

不能。默认方法在 trait 定义处检查，编译器只知道 `Named` 契约里的方法，不知道未来某个实现者会有 `display_name` 这个固有方法。错误大意是“no method named `display_name` found for type parameter `Self`”。

修复方向：把 `display_name` 也放进 trait 契约（可以给默认实现），或者让默认方法只调用 `self.name()`。默认方法不能依赖具体类型的私有 API。

</details>

## 十一、主练习：表单检查流水线

### 11.1 目标与行为

一个表单提交前要经过多项检查：

- 输入不能为空；
- 输入不能超过长度上限；
- 输入必须全是数字，并能解析成整数。

不同检查产出的值类型不同：空检查和长度检查产出字符数（`usize`），数字检查产出数值（`u64`）。程序还要能把它们放进同一条流水线，逐项报告成功结果或失败原因。
`usize` 是表示长度、数量这类非负整数的类型，大小随平台变化；`u64` 是 64 位无符号整数，范围是 0 到 18446744073709551615。

练习代码在本课 package `l12_traits` 的 `src/exercise.rs`。类型和 trait 已经定义好，你要实现标有 `todo!()` 的方法：

| 位置 | 需要实现的行为 |
| --- | --- |
| `NonEmpty::check` | 去掉首尾空白后不能为空；通过时返回剩余字符数 |
| `MaxChars::check` | 字符数不能超过 `limit`；通过时返回实际字符数 |
| `Digits::check` | 空输入、非数字、超出 `u64` 分别返回对应错误；通过时返回数值 |
| `Digits::describe` | 覆盖默认说明，固定为 `纯数字（只接受 0-9）` |
| `ValueText for usize` / `u64` | 返回十进制文本 |
| `run_one` | 运行一项检查，成功时组装 `Outcome`，失败时原样返回错误 |
| `run_all` | 依次运行 trait object 队列，保持顺序，某一项失败不影响后续 |

`Check` trait、blanket impl `impl<T: Check + Debug> ErasedCheck for T`、各类型的 `label` 已经给出，不需要修改。先读懂它们，再实现 `todo!()`。

### 11.2 输入、输出与边界

以下是真实调用及期望结果：

下面例子里的 “ASCII 数字” 指字符 `0` 到 `9` 这十个半角字符；全角数字、空格、正负号都不算。

```text
NonEmpty：
  "  rust  " -> Ok(4)          （去掉首尾空白，还剩 4 个字符）
  "你好"     -> Ok(2)          （按字符计数，不是字节）
  ""         -> Err(Empty)
  " \t\n "   -> Err(Empty)     （只有空白也算空）

MaxChars { limit: 3 }：
  "abc"      -> Ok(3)          （正好等于上限，通过）
  "abcd"     -> Err(TooLong { limit: 3, actual: 4 })
  ""         -> Ok(0)

MaxChars { limit: 0 }：
  ""         -> Ok(0)
  "a"        -> Err(TooLong { limit: 0, actual: 1 })

Digits：
  "00123"    -> Ok(123)        （允许前导零）
  ""         -> Err(Empty)
  "12a"      -> Err(NotDigits)
  " 12"      -> Err(NotDigits) （空格不是数字）
  "１２"      -> Err(NotDigits) （全角数字不是 ASCII 数字）
  "18446744073709551616" -> Err(TooLarge)  （u64 最大值加一）

run_one(&NonEmpty, "  rust  ")：
  -> Ok(Outcome { label: "非空", value: 4, description: "[非空]" })

run_all([NonEmpty, MaxChars { limit: 3 }, Digits], "1234")：
  -> [Ok("4"),
      Err(TooLong { limit: 3, actual: 4 }),
      Ok("1234")]              （第二项失败，第三项仍然运行）
```

边界和错误都要覆盖：

- 空输入、只有空白的输入；
- 长度正好等于上限、上限为 0；
- 前导零、非 ASCII 数字、负数形式、空输入；
- 超出 `u64` 的数值；
- 空队列、队列中间某一项失败。

### 11.3 脚手架与约束

- 只替换 `src/exercise.rs` 中的 `todo!()`；不要修改类型定义、方法签名、`Check` trait、`ErasedCheck` trait、blanket impl 和文件底部的测试。
- `NonEmpty`、`MaxChars`、`Digits` 的 `label` 已经给出；`Digits::describe` 需要你覆盖。
- `run_one` 必须返回 `Outcome { label, value, description }`，其中 `label` 是检查项名称的 `String` 副本，`description` 来自该检查项的 `describe()`。
- `run_all` 对空切片返回空 `Vec`；对每一项调用 `result_text`，不要提前 `return`，也不要跳过失败项。
- 按字符计数时，中文一个字符算 1，不能按字节算成 3。
- 不使用 `unwrap`、`expect` 或 `panic!`；错误必须通过 `Result` 返回。
- 不添加第三方依赖。`ValueText` 是本 crate 的 trait，因此为 `usize`、`u64` 实现它不违反孤儿规则；不要试图给 `Vec` 或 `()` 实现外部 trait。
- 占位参数不要用移动的方式“消警”；如果暂时没用到某个参数，用 `let _ = &parameter;`。

### 11.4 推荐完成顺序

1. `NonEmpty::check`：先处理空和纯空白，再数字符。
2. `MaxChars::check`：先数字符，再和 `limit` 比较；注意 `actual` 字段要填实际值。
3. `Digits::check`：按“空 → 非数字 → 解析溢出”的顺序判断；`check` 通过后再写 `Digits::describe`。
4. `ValueText` 的两个实现：把数字转成十进制文本。
5. `run_one`：先调用 `check`，用 `?` 让错误直接返回，再组装 `Outcome`。
6. `run_all`：遍历队列，逐项调用 `result_text`，收集结果。
7. 运行测试，按失败信息回到对应方法。

这个顺序让每一步都有测试可验证：先让单项检查通过，再接通静态封装，最后接通动态队列。

### 11.5 测试说明与验收命令

实现之前运行测试，绝大多数行为测试会在对应的 `todo!()` 处以 `not yet implemented` 失败。这是学习流程的一部分，不是测试写错。`associated_value_types_are_distinct` 和部分默认 `describe` 测试不依赖你的实现，会先通过。

在课程仓库根目录运行：

```bash
cargo test -p l12_traits
```

只跑主练习：

```bash
cargo test -p l12_traits exercise
```

演示程序（完成练习后才有完整输出）：

```bash
cargo run -p l12_traits
```

代码风格与静态检查：

```bash
cargo fmt -p l12_traits -- --check
cargo clippy -p l12_traits --all-targets -- -D warnings
```

全部通过即完成主练习。卡住时按级别查看 `HINTS.md`。

## 十二、课后巩固：图形面积报表

主练习处理的是“同一种输入、不同检查”，巩固任务换成“不同数据、同一种报表动作”：矩形和圆形字段不同，却都能算面积、缩放、生成说明。代码在 `src/consolidation.rs`。

### 12.1 目标与行为

- `Shape` trait 已给出，包含 `name`、`area`、`scale` 和默认的 `describe`。

所有面积都用 `f64`（双精度浮点数，能表示小数）计算。

- `Rectangle::area`：返回 `width * height`。
- `Rectangle::scale`：宽和高同时乘以 `factor`；`factor` 小于等于 `0` 时返回 `Err(NonPositiveFactor)`，且不修改自身。
- `Circle::area`：返回 `π * radius * radius`，`π` 用 `std::f64::consts::PI`。
- `Circle::scale`：半径乘以 `factor`；错误条件同上。
- `Circle::describe`：覆盖默认说明，格式为 `圆形（半径 X.XX）`，半径保留两位小数。
- `total_area`：所有图形面积之和；空切片返回 `0.0`。
- `largest_name`：面积最大的图形名称；空切片返回 `None`；并列最大时返回最先出现的；返回 `String` 副本。
- `scale_all`：先校验 `factor`，不合格时返回错误且不修改任何图形；合格时逐个缩放。
- `build_report`：为每个图形生成一行 `describe()`，装进 `Report`。
- `Report` 的 `Display`：每行一个元素，用换行分隔，末尾不加多余换行；空报告输出空字符串。

### 12.2 输入、输出与边界

```text
Rectangle { width: 3.0, height: 4.0 }.area()        -> 12.0
Circle { radius: 2.0 }.area()                       -> 4π ≈ 12.566370614
Rectangle { width: 2.0, height: 3.0 }.describe()    -> "矩形: 6.00"
Circle { radius: 2.0 }.describe()                   -> "圆形（半径 2.00）"

total_area(&[])                                     -> 0.0
total_area(&[矩形 2×3, 圆 r=1])                     -> 6.0 + π

largest_name(&[])                                   -> None
largest_name(&[矩形 2×3, 圆 r=2, 矩形 1×1])          -> Some("圆形")
largest_name(&[矩形 π×1, 圆 r=1])                    -> Some("矩形")  （并列，取先出现的）

Rectangle { 2×3 }.scale(2.0)                        -> Ok(()), 面积变为 24.0
Circle { r=2 }.scale(0.5)                           -> Ok(()), 半径变为 1.0
Rectangle { 2×3 }.scale(0.0)                        -> Err(NonPositiveFactor), 面积仍为 6.0
scale_all(&mut [矩形 2×3, 圆 r=1], 0.0)             -> Err(NonPositiveFactor), 两者都不变
scale_all(&mut [矩形 2×3, 圆 r=1], 2.0)             -> Ok(()), 面积分别为 24.0 和 4π

build_report(&[矩形 2×3, 圆 r=2]) ->
  Report(["矩形: 6.00", "圆形（半径 2.00）"])
format!("{}", 上述报告) ->
  "矩形: 6.00\n圆形（半径 2.00）"
format!("{}", build_report(&[])) -> ""
```

错误行为的关键是“先校验、后修改”：`scale_all` 收到非法因子时，不能已经把前面的图形改了一半。

### 12.3 约束与验收

- 只替换 `src/consolidation.rs` 的 `todo!()`；不修改类型定义、方法签名和测试。
- 浮点比较在测试里用近似判断，不要直接写 `==`；实现本身正常计算即可。
- `scale` 和 `scale_all` 的失败路径不能留下部分修改。
- 不使用 `unwrap`、`expect` 或 `panic!`；不添加依赖。
- `Report` 的字段是公开的，测试会直接构造 `Report(vec![...])` 比较；`Display` 只负责拼接。

只跑巩固任务：

```bash
cargo test -p l12_traits consolidation
```

全部通过即完成课后巩固。提示同样在 `HINTS.md` 中，按级别查看。

## 十三、常见误区

- **把 trait 当成“带字段的基类”。** trait 不持有数据，字段必须在 `struct`/`enum` 里。默认方法也不能访问实现者的字段；它只能调用契约里声明的方法。
- **以为 `impl Trait` 返回值可以按分支返回不同类型。** 它承诺“一个隐藏但唯一的具体类型”。需要运行时二选一时用 `Box<dyn Trait>`。
- **把 `dyn Trait` 当普通类型使用。** `dyn Trait` 大小未知，不能按值放进变量、参数或返回值；必须用 `&dyn Trait` 或 `Box<dyn Trait>`。
- **以为 trait object 没有成本。** 每次调用要走 vtable 间接跳转，通常不能内联，还要多存一个指针。能静态分发时优先静态分发。
- **用泛型参数替代关联类型。** 如果语义上“一个类型只有一种关联数据”，关联类型更准确，也不会让调用处产生歧义。反过来，确实需要同一类型多份实现时才用泛型参数。
- **以为 `Box<dyn Trait>` 能自动擦除关联类型。** 不能。带关联类型的 trait 必须先指定关联类型，或者另写一个返回统一类型（如 `String`）的擦除 trait。
- **以为 blanket impl 随时可以再补特例。** blanket impl 覆盖了所有满足条件的类型，再写具体实现会冲突（E0119）。设计时要确认不需要例外。
- **试图给外部类型实现外部 trait。** 孤儿规则禁止。用 newtype 包一层，或定义本地 trait。
- **同名方法不确认调用的是哪一个。** 固有方法优先于点号调用的 trait 方法；两个 trait 同名则报 E0034。需要明确时用 `Trait::method(&value)`。
- **给 trait 加一个返回 `Self` 的方法后继续用 `dyn Trait`。** 这会让 trait 失去 dyn compatibility（E0038）。要么返回 `Box<dyn Trait>`，要么给方法加 `where Self: Sized`。

## 十四、本课总结

1. trait 是一份行为契约：只声明“能做什么”，不持有数据。`impl Trait for Type` 兑现契约。
2. 默认方法把共享行为写进契约，实现者可以覆盖；默认方法只能使用契约和 supertrait 声明的能力。
3. `T: Trait` 是编译期能力声明。泛型函数体只能用 bound 承诺的操作，调用方只能传满足 bound 的类型。
4. 泛型通常通过单态化实现静态分发：每个具体类型生成一份代码，运行时没有查表开销，代价是代码体积。
5. 返回位置的 `impl Trait` 隐藏一个唯一的具体类型；所有返回路径必须是同一类型。
6. `dyn Trait` 是 trait object，必须放在指针后面。它由数据指针和 vtable 指针组成，运行时通过 vtable 动态分发，适合异构集合和运行时选择实现。
7. 关联类型表达“一个实现绑定一种关联数据”。它让调用处无需消歧，但也让 `dyn Trait` 必须指定关联类型；要混装不同关联类型，需要类型擦除层。
8. blanket impl 为所有满足条件的类型统一实现 trait；它强大，但会占住实现空间。
9. supertrait 要求实现者同时具备另一种能力；孤儿规则要求 trait 和类型至少有一个是本地定义，newtype 是常用的合法通道。
10. 选择顺序：能用具体类型就用具体类型；需要复用逻辑用泛型；需要异构集合或运行时选择才用 `dyn Trait`。

## 十五、课后复盘

用自己的话回答，再对照正文检查：

1. trait 和 `struct` 各自负责什么？为什么默认方法不能访问实现者的字段？
2. 静态分发和动态分发分别在什么时候确定调用哪份代码？各自的开销是什么？
3. 为什么 `impl Trait` 返回值不能在不同分支返回不同类型？这时应该改用什么？
4. 关联类型和泛型参数在“一个类型能有几份实现”上有什么区别？各自适合什么语义？
5. `Box<dyn Notice>` 为什么不能直接装带有不同关联类型的实现？擦除层做了什么？
6. 孤儿规则防止什么问题？newtype 为什么合法？
7. 什么样的方法会让 trait 失去 dyn compatibility？`where Self: Sized` 起什么作用？
8. 看到 E0277、E0046、E0117、E0191、E0038 时，你分别先检查什么？

## 下一课预告

本课多次出现 `Box<dyn Trait>`，但只把 `Box` 当作“把值放到堆上”的工具。下一课[第十三课：Box、递归数据结构与动态大小类型](../13-box-recursion/README.md)会解释：

- 为什么递归的 `enum` 必须通过 `Box` 断开无限大小；
- `Box<T>` 的所有权、解引用和与 `&T` 的区别；
- `dyn Trait` 为什么是“动态大小类型”（DST，大小在编译期未知、必须通过指针使用的类型）。

本课打下的 trait 基础会在下一课直接用到：递归数据结构里的节点，经常需要把“子节点”表示成 `Box<Node>`，而遍历行为可以用 trait 抽象出来。
