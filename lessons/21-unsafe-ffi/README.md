# 第二十一课：`unsafe`、裸指针与 FFI

安全 Rust 能表达绝大多数程序。但有些工作必须和机器或别的语言直接打交道：把一个数组拆成两段同时修改、调用操作系统或硬件厂商提供的 C 库、把 Rust 函数交给 C 程序调用。这些场合的共同点是：**编译器掌握的信息不够，无法自动证明某次操作安全**。

`unsafe` 就是为这种情况准备的。它不是“关闭检查的开关”，而是把“编译器无法替你证明的责任”明确交给你，并要求你把这份责任封装在一个安全接口后面。本课学习三件事：

1. `unsafe` 到底允许哪些操作，哪些规则仍然生效；
2. 裸指针、切片构造和内存有效性需要哪些前置条件；
3. 如何跨 C ABI（应用二进制接口）调用和导出函数，并设计安全包装。

## 本课定位

### 先看两个真实问题

**问题一：同一数组的两半要同时修改。** 下面的程序把数组在下标 2 处切开，分别修改两段：

```rust
fn main() {
    let mut data = [1, 2, 3, 4, 5];

    let (left, right) = data.split_at_mut(2);
    left[0] = 10;
    right[0] = 30;

    println!("{data:?}");
}
```

输出：

```text
[10, 2, 30, 4, 5]
```

`split_at_mut` 是标准库提供的安全函数。但如果你试着自己写“返回两个 `&mut` 子切片”的函数，借用检查器（编译器里负责检查引用规则的部分）会拒绝。原因不是标准库有特权，而是它内部使用了 `unsafe`，由作者向编译器证明“两段确实不重叠”。本课的主练习就是亲手完成这部分证明。

**问题二：调用 C 库。** 一个 C 函数可能长这样：

```c
size_t rfz_count_byte(const uint8_t *data, size_t len, uint8_t needle);
```

这行 C 声明的意思是：函数叫 `rfz_count_byte`；第一个参数 `data` 是一个指向只读字节的指针（`*` 表示指针，`const` 表示函数不会通过它修改数据）；第二个参数 `len` 是无符号整数，表示有多少个字节；第三个参数 `needle` 是要统计的字节；返回值也是无符号整数，表示出现次数。

C 没有 Rust 的借用检查、生命周期（引用有效多久的编译期记录）和 `Result` 错误类型。Rust 编译器无法验证这个 C 函数是否真的只读、是否会保存指针、长度是否正确。调用它必须进入 `unsafe`，并由调用者保证这些 C 侧契约。

### 前置知识

本课默认你已经学过：

- **所有权与借用**：每份数据有一个所有者；同一时间要么有多个只读借用，要么只有一个可变借用。见[第三课](../03-ownership-borrowing/README.md)。
- **切片**：切片是对一段连续数据的借用视图，同时记录起始位置和长度。见[第五课](../05-vec-slices/README.md)。
- **`Option` 与 `Result`**：`Option<T>` 表示“可能有值，也可能没有”；`Result<T, E>` 表示“成功值或错误值”。见[第四课](../04-option-result/README.md)。
- **模块与测试**：`mod` 组织代码，`#[test]` 标记测试函数。见[第七课](../07-modules-crates/README.md)与[第八课](../08-testing-quality/README.md)。
- **生命周期**：编译器用生命周期描述引用之间的有效期关系。见[第十四课](../14-lifetimes/README.md)。

需要前课知识时，本课会用一两句话复习，不需要回去重读。

### 本课边界

`unsafe` 还涉及可变静态变量（整个程序共享的全局变量，任何线程都能访问；线程是程序内部可以并行执行的一段执行流）、`union`（联合体：多个字段共用同一块内存的类型）、`unsafe trait`（trait 是描述一组行为的接口；`unsafe trait` 表示实现者要承诺编译器无法检查的不变量）、未初始化内存和手写内存分配器（负责向系统申请和归还内存的组件）。本课会解释它们为什么存在，但练习只覆盖两条最常用、也最能建立直觉的路径：**裸指针拆分切片**和**C ABI 边界**。内联汇编、SIMD（单指令多数据）和分配器实现不在本课范围。

本课不要求你会写 C。出现的 C 语法都会逐项解释。

## 如何学习本课

建议顺序：

1. 先读“术语”和“`unsafe` 到底允许什么”，运行前两个完整示例。
2. 阅读编译器诊断一节，先建立“看到错误知道往哪查”的习惯。
3. 做“代码预测”，写下答案再展开。
4. 完成主练习的两个函数，按“推荐完成顺序”推进。
5. 完成课后巩固，再回来看常见误区与总结。

预留 2～4 小时。每完成一个函数就跑一次对应测试，不要等全部写完再编译。

## 一、术语：先把词说清楚

后面会反复使用这些词，第一次出现时也会就近解释。

- **`unsafe`**：Rust 的一个关键字。它标记一段代码或一个函数，表示“这里存在编译器无法自动证明的安全条件”。它不关闭类型检查、所有权检查或借用检查。
- **未定义行为（undefined behavior，缩写 UB）**：程序违反了语言规则，导致编译器可以假设这件事永不发生。后果不可预测：可能立刻崩溃，可能输出错误结果，也可能看起来正常。**测试通过不能证明没有 UB。**
- **指针（pointer）**：保存内存位置编号的值；可以理解为“某块数据从内存的哪个位置开始”。
- **裸指针（raw pointer）**：只记录这种位置编号的指针类型，写作 `*const T`（只读）或 `*mut T`（可写）。它可为空、可能悬垂（指向已经释放的内存）、可能未对齐，也不携带生命周期信息。
- **对齐（alignment）**：某种类型要求内存位置编号是某个值的整数倍。例如 `i32` 通常要求编号能被 4 整除。把未对齐的位置当成 `i32` 读取属于 UB。
- **不变量（invariant）**：在一段代码执行期间必须始终成立的条件。例如“这个指针非空”“这个长度不超过分配大小”“这块内存已经初始化”。
- **ABI（application binary interface，应用二进制接口）**：函数在机器码（CPU 直接执行的指令）层面的约定，包括参数放在寄存器（CPU 内部的小块高速存储）还是栈（函数调用时自动分配和回收的临时内存区域）上、由谁清理栈、符号（函数在最终程序里的名字）如何命名。两个程序只有遵循同一 ABI 才能互相调用。
- **FFI（foreign function interface，外部函数接口）**：一种语言调用另一种语言函数的机制。本课指 Rust 与 C 互相调用。
- **NUL 字节**：数值为 0 的字节。C 字符串用它表示“文本结束”，而 Rust 字符串用单独的长度字段，内容里可以出现 0。
- **安全抽象（safe abstraction）**：对外暴露普通的安全函数，把 `unsafe` 关在内部，并保证任何合法输入都无法破坏内部不变量。标准库的 `Vec`、`String`、`split_at_mut` 都是安全抽象。

## 二、`unsafe` 到底允许什么

在 `unsafe` 块中，Rust 额外允许五类操作：

1. 解引用裸指针（读取或写入它指向的位置）；
2. 调用 `unsafe` 函数；
3. 访问或修改可变静态变量（`static mut`）；
4. 实现 `unsafe trait`；
5. 读取 `union` 的字段。

除此之外，普通规则照常执行。类型必须匹配，变量移动后不能再用，借用冲突仍然报错。`unsafe` 不是“我知道我在干什么”的魔法咒语，而是一份需要逐项兑现的证明。

**创建裸指针本身通常是安全的，解引用才需要 `unsafe`。** 下面是一个完整程序：

```rust
fn main() {
    let mut value = 42_i32;

    // &raw const / &raw mut 直接取得地址，不先创建引用。
    let read_only: *const i32 = &raw const value;
    let writable: *mut i32 = &raw mut value;

    // SAFETY: 两个指针都来自仍然存活的局部变量 value，类型和对齐正确，
    // 也没有其他引用在同时访问。
    unsafe {
        println!("通过只读指针读取: {}", *read_only);
        *writable = 50;
    }

    println!("修改后 value = {value}");
}
```

输出：

```text
通过只读指针读取: 42
修改后 value = 50
```

`&raw const value` 和 `&raw mut value` 是 Rust 2024（Rust 的语言版本，不是工具链版本）风格写法：它们直接产生裸指针。旧代码里也能看到 `&value as *const i32`，那会先创建一个普通引用再转换；两种写法在这里结果相同，但 `&raw` 更明确地表达“我现在只要地址”。

注意程序里 `unsafe` 块只包住了真正危险的两行。`println!` 和变量声明仍在安全上下文中，继续受编译器保护。

## 三、`unsafe` 块是责任声明，`unsafe` 函数是契约

### 3.1 块的含义

```rust
unsafe {
    dangerous_operation();
}
```

这段代码应读作：

> 我已经检查过 `dangerous_operation` 文档列出的全部安全前置条件，并确认它们在这里成立。

因此每个 `unsafe` 块旁边都应有 **SAFETY 注释**，说明“依赖了哪些事实”。高质量注释不是重复“这里调用了 unsafe 函数”，而是回答：

- 指针非空吗？对齐吗？
- 指向的内存在使用期间仍然有效吗？
- 读取的范围已经初始化吗？长度在同一块分配内吗？
- 有没有违反“共享或独占”的别名规则（同一块内存能否同时被多个引用访问的规则）？
- 会不会释放两次？
- 外部函数的 ABI、布局和所有权约定是否匹配？

### 3.2 函数层面的契约

`unsafe fn` 表示：**调用者**必须满足函数文档里的条件，否则函数内部的操作可能触发 UB。看一个完整例子：

```rust
/// 读取 `pointer` 指向的 `i32`。
///
/// # Safety
///
/// `pointer` 必须非空、对齐，并指向一个仍然存活的 `i32`。
unsafe fn read_i32(pointer: *const i32) -> i32 {
    // SAFETY: 调用者按上面的契约保证了指针有效。
    unsafe { *pointer }
}

fn main() {
    let value = 7_i32;

    // SAFETY: &value 非空、对齐、有效，并且在调用期间一直存活。
    let result = unsafe { read_i32(&value) };

    println!("{result}");
}
```

输出 `7`。

Rust 2024 里，`unsafe fn` 内部的不安全操作默认会触发警告，要求把不安全操作写进显式的 `unsafe {}` 块。如果省略，编译器会给出类似这样的警告：

```text
warning[E0133]: dereference of raw pointer is unsafe and requires unsafe block
 --> unsafe_fn_op.rs:2:5
  |
2 |     *pointer
  |     ^^^^^^^^ dereference of raw pointer
  |
note: an unsafe function restricts its caller, but its body is safe by default
```

这条规则的价值是：读代码的人能区分“这个函数整体上要求调用者守约”和“真正危险的是这三行”。把 `unsafe` 块缩小，审查范围就缩小。

### 3.3 能用安全代码就不要 `unsafe`

标准库的很多安全 API 内部都用 `unsafe`，但它们已经过长期审查。日常代码优先使用这些现成抽象：

- 取元素用 `values.get(index)`，越界返回 `None`；
- 拆分用 `split_at_mut`、`chunks_mut`；
- 需要共享可变状态时用锁（同一时间只允许一个线程进入的同步工具）或原子类型（多线程下不会读到“改了一半”的值的整数操作），而不是 `static mut`（见[第十七课](../17-shared-concurrency/README.md)）。

只有当安全 API 无法表达需求，或者经过测量证明确实需要底层操作时，才自己写 `unsafe`。本课主练习重新实现 `split_at_mut` 是为了理解机制，不是为了在项目里替代它。

## 四、裸指针与引用有什么不同

| 性质 | 引用 `&T` / `&mut T` | 裸指针 `*const T` / `*mut T` |
| --- | --- | --- |
| 可以为空 | 不可以 | 可以 |
| 可以悬垂 | 不可以（借用检查器保证） | 可能 |
| 对齐 | 保证 | 不保证 |
| 生命周期 | 编译器跟踪 | 不跟踪 |
| 别名规则 | 编译器检查 | 不自动检查 |
| 解引用 | 安全 | 需要 `unsafe` |
| 地址运算 | 不直接支持 | 支持 `add`、`offset` 等 |

关键区别不是“裸指针更底层所以更厉害”，而是**裸指针把证明责任从编译器转移给了人**。

还有一条容易忽略的规则：**创建无效引用本身就是 UB**，即使你之后没有读取它。把空指针或未对齐地址转换成 `&T`，程序已经违反了规则。相比之下，持有一个空的 `*const T` 是安全的，只要不解引用。

## 五、指针算术：单位是元素，不是字节

裸指针的 `add(n)` 会向后移动 `n` 个**元素**，实际字节数是 `n * size_of::<T>()`。它要求结果仍在同一块分配内，或正好指向末尾的下一个位置（one-past-end）；真正解引用时还必须指向有效元素。

完整示例：

```rust
fn main() {
    let values = [10_i32, 20, 30, 40];
    let base = values.as_ptr();
    let mut index = 0;

    while index < values.len() {
        // SAFETY: index < len，base 来自仍然存活的 values，
        // 所以这次读取仍在同一块分配内。
        let value = unsafe { base.add(index).read() };
        println!("第 {index} 个元素是 {value}");
        index += 1;
    }
}
```

输出：

```text
第 0 个元素是 10
第 1 个元素是 20
第 2 个元素是 30
第 3 个元素是 40
```

这里的 `.read()` 从指针位置复制出一个 `T`。对 `i32` 这种 `Copy`（赋值时直接按位复制的类型）类型没问题；对 `String` 这类拥有堆（运行时按需申请的内存区域）内存的类型，`read` 会复制所有权，原位置不能再析构（值销毁时释放资源的动作），否则会双重释放。本课练习只拆分借用，不做这种所有权搬运。

“同一块分配”还与**指针来源（provenance）**有关。可以把它理解为：指针除了地址，还记录“我来自哪块分配”。只在同一块分配内偏移是合法的；先把指针转成整数、做任意运算、再转回指针，不能用 C 时代“地址就是数字”的直觉推导 Rust 合法性。本课不会使用整数地址往返。

## 六、`from_raw_parts`：用指针和长度造一个切片

### 6.1 它解决什么问题

切片在内存里可以理解为“起始地址 + 长度”。有时你只有这两个值（例如 C 库返回它们），需要临时把它们看成 Rust 切片。标准库提供：

```rust
use std::slice;

fn main() {
    let data = [10_u8, 20, 30];
    let pointer = data.as_ptr();
    let len = data.len();

    // SAFETY: pointer 和 len 来自仍然存活的 data；范围没有越界；
    // 调用期间没有对 data 的可变借用。
    let view = unsafe { slice::from_raw_parts(pointer, len) };

    println!("{view:?}");
}
```

输出：

```text
[10, 20, 30]
```

对应的可写版本是 `from_raw_parts_mut`，要求更严格：返回 `&mut [T]` 期间，不能有其他引用访问同一块内存。

### 6.2 必须逐项满足的契约

`from_raw_parts` 和 `from_raw_parts_mut` 的文档列出了一组前置条件。只要有一条不成立，就是 UB：

1. 指针非空且对齐，**即使长度为 0 也一样**；
2. 整个 `len` 个元素的范围位于同一块有效分配内；
3. `len * size_of::<T>()` 不超过 `isize::MAX`（`isize` 是与机器字长相同的有符号整数类型），且不溢出；
4. 元素已经正确初始化为 `T`；
5. 返回的借用存续期间，内存保持有效；
6. 共享版本不能与可变访问重叠；可变版本必须拥有独占访问；
7. 返回引用的生命周期必须真实反映内存的有效期。

第 1 条常被误解：空切片不是“随便给个空指针”。空切片可以用 `NonNull`（保证非空的指针包装类型）的 `dangling()` 一类“非空且对齐但不指向真实数据”的地址，但更推荐从已有安全切片推导。

### 6.3 为什么不要让调用者随便选生命周期

下面这个签名看起来方便，实际非常危险：

```rust,ignore
unsafe fn view<'a>(pointer: *const u8, len: usize) -> &'a [u8] {
    // 省略：假设这里调用了 from_raw_parts
}
```

`'a` 由调用者选择，调用者可以声称“这个切片活一万年”，编译器无法反驳。正确做法是让生命周期来自一个真实的输入借用：

```rust,ignore
fn view<'a>(data: &'a [u8]) -> &'a [u8] {
    data
}
```

这时返回值的有效期被输入借用约束住，安全调用者无法伪造更长的生命周期。本课主练习的返回值也采用这种思路：两个输出切片的生命周期直接来自输入的 `&mut [T]`。

## 七、安全封装的套路

一个可靠的安全封装通常按这个顺序写：

1. **先验证**：在进入 `unsafe` 之前检查所有能用普通代码检查的条件（长度、范围、空指针、状态）；
2. **缩小危险区**：只把必须用 `unsafe` 的调用包进块里；
3. **写 SAFETY 注释**：逐条对应被调用 API 的前置条件；
4. **让生命周期来自输入**：不要给调用者凭空选择 `'a` 的机会；
5. **不泄露危险句柄**：不要把内部裸指针或未检查的索引交给外部。

标准库的 `split_at_mut` 正是这个结构：先确认 `mid <= len`，再由一个内部 `unsafe` 块构造两段互不重叠的切片。因为检查在安全函数内部完成，外部调用者无论传什么 `mid` 都不会触发 UB（越界会 panic，也就是中止当前线程并报错）。

本课主练习要求你返回 `Option` 而不是 panic：`mid` 越界时返回 `None`。这只是行为选择，不改变安全责任。

## 八、FFI：调用 C 函数

### 8.1 为什么需要 ABI

Rust 和 C 编译后的函数要互相调用，必须约定：

- 参数放在哪些寄存器或栈位置；
- 返回值怎么传；
- 由调用方还是被调方清理栈；
- 函数符号在二进制里叫什么名字。

这套约定叫 ABI。Rust 默认的函数 ABI 不保证和 C 一致，所以跨语言调用要写 `extern "C"`，意思是“使用 C 的调用约定”。

Rust 默认还会对符号名做名称修饰（name mangling），把模块、泛型等信息编进符号。C 只认原始名字，所以导出给 C 的函数要加 `#[unsafe(no_mangle)]`，表示“不要修饰这个名字”。

### 8.2 类型对应关系

| C 类型 | Rust 类型 | 说明 |
| --- | --- | --- |
| `size_t` | `usize` | 无符号整数，通常与指针宽度相同，用来表示大小 |
| `uint8_t` | `u8` | 恰好 8 位的无符号整数 |
| `int32_t` | `i32` | 恰好 32 位的有符号整数 |
| `const uint8_t *` | `*const u8` | 指向只读字节的裸指针 |
| `uint8_t *` | `*mut u8` | 指向可写字节的裸指针 |
| `const char *` | `*const c_char` | 指向只读 C 字符的指针；`c_char` 在多数平台等价于 `i8` |
| `char *` | `*mut c_char` | 指向可写 C 字符的指针 |
| `void`（无返回值） | `()` | 空元组类型 |

不要用 `String`、`Vec`、`Result` 或 Rust 的 `enum`（枚举：由若干命名变体组成的类型）直接跨 C ABI：它们的内存布局（数据在内存里的排列顺序和占用方式）和语义都不是 C 能理解的。

### 8.3 声明外部函数（Rust 2024）

Rust 2024 要求外部声明块本身写成 `unsafe extern`：

```rust,ignore
unsafe extern "C" {
    #[link_name = "rfz_count_byte"]
    fn c_count_byte(data: *const u8, len: usize, needle: u8) -> usize;
}
```

为什么整个块是 `unsafe`？因为编译器无法验证“链接（把编译好的目标文件和库拼成最终程序的步骤）时真的有这个符号”，也无法验证“C 端签名真的和这里写的一致”。签名写错不会在编译期报错，而会在运行时读错参数，直接造成 UB。

如果漏写 `unsafe`，会得到真实诊断：

```text
error: extern blocks must be unsafe
 --> extern_block.rs:1:1
  |
1 | / extern "C" {
2 | |     fn abs(input: i32) -> i32;
3 | | }
  | |_^
```

`#[link_name = "..."]` 用来指定链接时查找的符号名。真实项目通常直接在块上加 `#[link(name = "库名")]` 并让 Rust 名字与 C 名字一致；本课为了让“替身实现”和“外部声明”能放在同一个文件里，才让 Rust 名字与导出符号名不同。

### 8.4 一个完整的两端示例

Rust 默认的结构体布局可能为节省空间而重排字段，字段顺序和间距没有跨语言保证。要与 C 共享结构体，必须用 `#[repr(C)]` 固定布局（下面的代码会看到）。这个程序同时展示“导出给 C”和“从 Rust 调用 C ABI 函数”。为了让示例不依赖 C 编译器也能运行，C 端实现用一个 `#[unsafe(no_mangle)]` 的 Rust 函数代替；从调用方看，经过的仍是同一条 ABI 边界。

```rust
/// 与 C 共享的结构体布局。
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Pair {
    left: i32,
    right: i32,
}

unsafe extern "C" {
    #[link_name = "rfz_pair_sum"]
    fn c_pair_sum(pair: *const Pair) -> i32;
}

/// 模拟 C 库函数：把结构体两个字段相加。
///
/// # Safety
///
/// `pair` 必须非空，并指向一个有效、已初始化的 `Pair`。
#[unsafe(no_mangle)]
unsafe extern "C" fn rfz_pair_sum(pair: *const Pair) -> i32 {
    // SAFETY: 调用者保证 pair 非空、对齐、有效且已初始化。
    let pair = unsafe { &*pair };
    pair.left + pair.right
}

/// 安全包装：调用者只需要提供一个正常的 `&Pair`。
fn pair_sum(pair: &Pair) -> i32 {
    // SAFETY: 引用保证非空、对齐、有效；C 端只在调用期间读取，
    // 不保存指针，也不修改数据。
    unsafe { c_pair_sum(pair) }
}

fn main() {
    let pair = Pair {
        left: 20,
        right: 22,
    };
    println!("pair_sum = {}", pair_sum(&pair));
}
```

输出：

```text
pair_sum = 42
```

这里新增了两个概念：

- **`#[repr(C)]`**：要求结构体按 C 的规则排列字段。Rust 默认布局可能重排字段以省空间，不能直接拿去和 C 共享。
- **`unsafe extern "C" fn`**：函数使用 C 调用约定，并且带有安全契约。`extern "C"` 只规定调用方式，不保证函数体安全。

`repr(C)` 只解决布局，不解决整数宽度（每种整数占多少字节）、指针有效性、字符串编码、所有权和释放方式。这些仍要逐项约定。

### 8.5 本课使用的替身

本课 `src/ffi.rs` 里的 C 端符号声明为：

```c
size_t rfz_count_byte(const uint8_t *data, size_t len, uint8_t needle);
```

对应的 Rust 声明是 `fn c_count_byte(data: *const u8, len: usize, needle: u8) -> usize`。`rfz_count_byte` 是一个已实现的替身，行为与 C 版本一致：按 `len` 扫描 `data`，统计等于 `needle` 的字节。

你不需要安装 C 编译器，也不需要新增依赖。真实项目若要用 C 代码，通常会在 `build.rs`（Cargo 在构建项目前自动运行的 Rust 脚本）里用 `cc` 一类构建依赖编译 C 文件；本课刻意避免这条路径，让练习只聚焦 ABI 边界本身。

### 8.6 包装是否安全，取决于 C 的完整契约

把 `&[u8]` 的指针和长度传给 C，只能保证：

- 指针非空、对齐（`u8` 对齐要求为 1）；
- 长度范围内的内存已初始化且对调用期间有效；
- 没有其他可变借用同时存在。

它**不能**保证 C 函数不保存指针、不在另一个线程访问、不越界读写、不释放这块内存。安全包装必须把这些条件写进文档，并由 C 端遵守。如果 C 保存了指针供以后使用，那么“短期借用”的包装就是不安全的；此时应改为复制数据、转移所有权，或返回一个能保持数据存活的 guard（离开作用域时自动执行清理的包装类型）类型。

## 九、从 Rust 导出给 C

导出函数的基本形式是：

```rust,ignore
#[unsafe(no_mangle)]
pub extern "C" fn add(left: i32, right: i32) -> i32 {
    left + right
}
```

在 Rust 2024 中，`no_mangle` 是 unsafe 属性，因为“保留这个名字”可能造成符号冲突，也可能让外部代码用错误签名调用。漏写 `unsafe(...)` 会得到：

```text
error: unsafe attribute used without unsafe
 --> no_mangle.rs:1:3
  |
1 | #[no_mangle]
  |   ^^^^^^^^^ usage of unsafe attribute
  |
help: wrap the attribute in `unsafe(...)`
  |
1 | #[unsafe(no_mangle)]
  |   +++++++         +
```

导出函数还要遵守两条纪律：

1. **先验证输入**：C 调用者可能传空指针、过大的长度、悬垂指针或无效编码；不要假设它守规矩。
2. **不要让 panic 穿过 FFI 边界**：C 没有 Rust 的栈展开（panic 时沿调用栈逐层清理局部变量的过程）机制。Rust 不保证 panic 能安全穿过 C ABI，默认行为还可能随编译选项变化。常见做法是用 `std::panic::catch_unwind`（捕获 panic 并转成普通返回值）捕获，并把失败转换成错误码；或者让函数体保证不 panic。

一个简化结构（片段，省略了具体业务）：

```rust,ignore
#[unsafe(no_mangle)]
pub extern "C" fn entry(input: i32) -> i32 {
    match std::panic::catch_unwind(|| work(input)) {
        Ok(value) => value,
        Err(_) => -1,
    }
}
```

`catch_unwind` 只能捕获会展开的 panic；如果程序配置为 `panic = "abort"`（panic 时直接终止整个进程，不逐层清理），panic 会直接终止进程。它也不能让 C 端的内存错误变得安全。

## 十、C 字符串

### 10.1 两种字符串的内存布局不同

C 字符串是一段字节，最后一个字节是 NUL（数值 0），没有单独的长度字段。Rust 的 `String` 和 `str` 记录长度，并且内容中可以出现 NUL。

这意味着：

- C 字符串不能包含内部 NUL，否则会被当成提前结束；
- Rust 字符串转 C 字符串时，要检查内部 NUL；
- 从 C 读入时，要一直扫描到 NUL，且要验证 UTF-8（一种把字符编码成字节的通用规则）。

### 10.2 `CString`：拥有 NUL 结尾缓冲区的 Rust 类型

```rust
use std::ffi::CString;

fn main() {
    let name = CString::new("rust").expect("文本中没有 NUL 字节");
    println!("字节: {:?}", name.as_bytes_with_nul());

    let pointer = name.as_ptr();
    println!("指针非空: {}", !pointer.is_null());
}
```

输出（指针的具体地址每次可能不同，所以这里只展示字节和“是否非空”）：

```text
字节: [114, 117, 115, 116, 0]
指针非空: true
```

`CString::new` 返回 `Result`：如果输入包含内部 NUL，会得到 `NulError`（表示文本包含内部 NUL 的错误类型）。`as_bytes_with_nul()` 把结尾的 0 也显示出来。`as_ptr()` **只是借用**缓冲区，不转移所有权；指针只在 `name` 存活且未被销毁期间有效。

### 10.3 `CStr`：借来的 C 字符串视图

`CStr` 是对“以 NUL 结尾的一段字节”的借用视图。它可以从 `&CStr` 安全地创建，也可以用 `CStr::from_ptr` 从裸指针创建。后者是 `unsafe`，因为：

- 指针可能为空；
- 指针可能悬垂或未对齐；
- 内容可能没有 NUL 结尾，函数会越过缓冲区继续读，直到碰巧遇到数值 0，造成 UB；
- 编译器无法知道这块内存有多长。

拿到 `CStr` 后，转成 Rust 文本有两种选择：

- `to_str()`：要求内容是合法 UTF-8，失败返回错误，可以恢复处理；
- `to_string_lossy()`：把非法序列替换成替换字符，可能改变内容。

**不要**把 `String::as_ptr()` 的结果传给 `CStr::from_ptr`：Rust 字符串不保证以 NUL 结尾，函数会越过缓冲区继续读，造成 UB。需要 C 字符串时先用 `CString`。

课后巩固会让你实现“C 指针 → 拥有型 `String`”的读取函数，练习空指针、空字符串和非法 UTF-8 三种情况。

## 十一、所有权、错误和未初始化内存

### 11.1 跨边界所有权

最危险的问题之一是“谁负责释放”。原则：

- 分配方通常提供配对的释放函数；
- 不要用 Rust 的分配器释放 C 分配的内存，反之亦然；
- 文档要说明指针是**借用**（borrowed）、**转移**（transferred）还是**返回所有权**（returned-owned）；
- 指针、长度、容量和元素初始化状态要一起传递；
- 同一份所有权只能转换回来一次。

Rust 把 `Box`（[第十五课](../15-smart-pointers/README.md)讲过的、把值放在堆上的智能指针）交给 C：

```rust,ignore
let raw = Box::into_raw(Box::new(value));
```

取回时只能做一次：

```rust,ignore
let boxed = unsafe { Box::from_raw(raw) };
drop(boxed);
```

如果 C 还保存着 `raw`，Rust 取回并释放后，C 手里的指针就悬垂了。

### 11.2 用错误码跨 C ABI

C 不能直接返回 Rust 的 `Result<T, E>`。常见设计是：

- 返回一个整数状态码（0 表示成功，非 0 表示不同错误）；
- 结果写入一个输出指针；
- Rust 安全包装立即把状态码转换成 `Result`，让调用者回到 Rust 的错误处理习惯。

包装层要检查空指针、未知错误码、输出是否已初始化，以及 C 端是否线程安全。

### 11.3 未初始化内存

Rust 假设一个 `T` 类型的值始终是有效的。直接伪造“未初始化的 `T`”可能立刻是 UB。标准库用 `MaybeUninit<T>` 表示“这块内存可能还没初始化”：

```rust
use std::mem::MaybeUninit;

fn main() {
    let mut slot = MaybeUninit::<u32>::uninit();
    slot.write(42);

    // SAFETY: 上一行已经完整写入一个 u32。
    let value = unsafe { slot.assume_init() };

    println!("{value}");
}
```

输出 `42`。

除非你在实现底层容器或对接外部 API，否则不要自己管理未初始化内存。`Vec`、`String` 等安全类型已经处理好这些细节。

## 十二、编译器与测试诊断

### 12.1 想返回两个可变子切片，被借用检查器拒绝

有人会先尝试这样写：

```rust,ignore
fn split(values: &mut [i32], mid: usize) -> (&mut [i32], &mut [i32]) {
    (&mut values[..mid], &mut values[mid..])
}
```

真实诊断：

```text
error[E0499]: cannot borrow `*values` as mutable more than once at a time
 --> split_borrow.rs:2:31
  |
1 | fn split(values: &mut [i32], mid: usize) -> (&mut [i32], &mut [i32]) {
  |                  - let's call the lifetime of this reference `'1`
2 |     (&mut values[..mid], &mut values[mid..])
  |     --------------------------^^^^^^--------
  |     |     |                   |
  |     |     |                   second mutable borrow occurs here
  |     |     first mutable borrow occurs here
  |     returning this value requires that `*values` is borrowed for `'1`
  |
  = help: use `.split_at_mut(position)` to obtain two mutable non-overlapping sub-slices
```

诊断里的 `'1` 是编译器给输入引用起的生命周期名字，用来描述“返回值必须活多久”。错误的核心是：编译器只看到两次 `&mut values[...]`，无法从运行时的 `mid` 推出两个范围不重叠。它不知道第一次借用的范围是 `0..mid`，第二次是 `mid..len`。

修复方向不是“把 `mid` 改成常量”，而是：

- 日常代码直接用 `.split_at_mut(mid)`；
- 需要自己实现时，用裸指针构造两段，并在 `unsafe` 注释里证明不重叠。

### 12.2 在安全上下文解引用裸指针

```text
error[E0133]: dereference of raw pointer is unsafe and requires unsafe block
 --> raw_deref.rs:4:16
  |
4 |     let copy = *pointer;
  |                ^^^^^^^^ dereference of raw pointer
  |
  = note: raw pointers may be null, dangling or unaligned; they can violate aliasing rules and cause data races: all of these are undefined behavior
```

诊断直接列出了裸指针的四种风险：空、悬垂、未对齐、违反别名规则。修复不是“给整段程序加 unsafe”，而是先确认这些风险都不存在，再把解引用包进最小的块。

### 12.3 外部声明块漏写 `unsafe`

```text
error: extern blocks must be unsafe
 --> extern_block.rs:1:1
  |
1 | / extern "C" {
2 | |     fn abs(input: i32) -> i32;
3 | | }
  | |_^
```

Rust 2024 的规则。把 `extern "C"` 改成 `unsafe extern "C"`。

### 12.4 导出函数漏写 `unsafe(no_mangle)`

```text
error: unsafe attribute used without unsafe
 --> no_mangle.rs:1:3
  |
1 | #[no_mangle]
  |   ^^^^^^^^^ usage of unsafe attribute
  |
help: wrap the attribute in `unsafe(...)`
  |
1 | #[unsafe(no_mangle)]
  |   +++++++         +
```

`no_mangle` 会影响链接结果，所以属于 unsafe 属性。

### 12.5 脚手架尚未实现时的测试失败

完成练习前运行测试，会看到类似：

```text
running 1 test
test splits_in_the_middle ... FAILED

failures:

---- splits_in_the_middle stdout ----

thread 'splits_in_the_middle' (136149) panicked at lessons/21-unsafe-ffi/src/exercise.rs:26:5:
not yet implemented
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

`not yet implemented` 来自 `todo!()`，表示“这里还没写”，不是编译器或测试本身坏了。完成对应函数后，这个测试才会真正检查行为。

## 十三、完整示例：三个可以独立运行的程序

下面三个程序都在前文给出了完整代码和实际输出，与主练习的问题不同，只用于理解机制。

**示例一：安全的标准库拆分。** 完整代码见“本课定位”的问题一，输出 `[10, 2, 30, 4, 5]`。它说明需求本身是合理的，只是实现需要额外的证明。

**示例二：用指针遍历数组。** 完整代码见“指针算术”一节，输出四个元素。它说明 `add(n)` 的单位是元素，且必须留在同一块分配内。

**示例三：把指针和长度还原成切片。** 完整代码见“`from_raw_parts`”一节，输出 `[10, 20, 30]`。它说明切片只是“地址 + 长度”的视图，但构造它需要满足一整套前置条件。

三个例子都没有使用 `transmute`（强制重新解释类型的操作）、整数地址往返或手工内存分配。这些高级手段在本课没有必要。

## 十四、代码预测

先写下答案，再展开。

**问题 1：创建裸指针需要 `unsafe` 吗？解引用呢？**

<details><summary>解析</summary>

创建通常不需要：`&raw const value` 和 `&raw mut value` 都是安全表达式，`Box::into_raw` 也是安全函数。解引用、把裸指针转成引用、用裸指针读写才需要 `unsafe`。

原因是“持有地址”本身不违反内存规则；危险发生在“通过地址访问内存”的时候。

</details>

**问题 2：`from_raw_parts_mut(std::ptr::null_mut(), 0)` 合法吗？**

<details><summary>解析</summary>

不合法。`from_raw_parts_mut` 要求指针非空且对齐，即使长度为 0 也一样。空切片可以用 `NonNull::dangling()` 这类满足对齐的非空地址，但更推荐直接从已有安全切片推导。

</details>

**问题 3：对一个 `Box<i32>` 的裸指针调用一次 `Box::from_raw`，值销毁后还能再调用一次吗？**

<details><summary>解析</summary>

不能。第一次 `from_raw` 恢复了唯一所有权，`Box` 销毁时那块分配已经被释放。第二次会对悬垂指针构造 `Box`，最终导致双重释放，属于 UB。所有权只能转换回来一次。

</details>

**问题 4：`split_at_mut_checked(&mut values, len + 1)` 应该 panic 还是返回 `None`？**

<details><summary>解析</summary>

按本课约定返回 `None`，不 panic。越界是可恢复的调用错误，用 `Option` 表达；函数也不应该修改或移动元素。测试会同时检查“返回 `None`”和“原数据不变”。

</details>

**问题 5：C 函数保存了传入的指针，Rust 包装返回后切片被释放，C 以后再用这个指针会怎样？**

<details><summary>解析</summary>

会访问已释放内存，属于 UB。`&[u8]` 只保证“调用期间”有效，不能支持“保存到以后”。如果 C 需要长期保存，包装必须复制数据、转移一块拥有型缓冲区的所有权，或返回一个在 C 使用期间保持数据存活的 guard。仅仅写一个 `unsafe` 块不能解决这个问题。

</details>

## 十五、主练习：受检拆分与 C 字节计数

### 15.1 目标

完成 `lessons/21-unsafe-ffi` 中的两个函数：

1. `src/exercise.rs` 的 `split_at_mut_checked`：用裸指针实现可变切片拆分；
2. `src/ffi.rs` 的 `count_byte`：为 C ABI 函数写安全包装。

不要修改函数签名、类型定义和测试。每个函数上方的注释就是行为规格。

### 15.2 输入、输出与边界

**`split_at_mut_checked(&mut [T], usize) -> Option<(&mut [T], &mut [T])>`**

| 输入 | 期望输出 |
| --- | --- |
| `[1, 2, 3, 4, 5]`，`mid = 2` | `Some(([1, 2], [3, 4, 5]))` |
| `[7, 8, 9]`，`mid = 0` | `Some(([], [7, 8, 9]))` |
| `[7, 8, 9]`，`mid = 3` | `Some(([7, 8, 9], []))` |
| `[]`，`mid = 0` | `Some(([], []))` |
| `[1, 2, 3]`，`mid = 4` | `None`，原数组不变 |
| `[1, 2, 3]`，`mid = usize::MAX` | `None`，不 panic |

正常情况（中间拆分）、边界情况（0、长度、空切片）和错误情况（越界）都要覆盖。测试还会比较每个合法切点与标准库 `split_at_mut` 的结果，并用 `String` 这类非 `Copy` 元素确认没有错误的复制假设。

**`count_byte(&[u8], u8) -> usize`**

| 输入 | 期望输出 |
| --- | --- |
| `b"banana"`，`b'a'` | `3` |
| `&[]`，`b'x'` | `0` |
| `[0, 1, 0, 0, 2]`，`0` | `3` |
| `b"hello"`，`b'z'` | `0` |
| `[9; 32]`，`9` | `32` |

字节 0 是普通数据，不是结束标记；空切片合法；长度很大的输入也要与普通迭代器结果一致。

### 15.3 约束

- `split_at_mut_checked` 必须用裸指针（`as_mut_ptr`）和 `slice::from_raw_parts_mut` 完成拆分；
- 危险操作只放在一个尽可能小的 `unsafe` 块里，并写 SAFETY 注释逐项证明前置条件；
- 不使用 `transmute`、整数地址往返、手工分配器或第三方依赖；
- 返回切片的生命周期必须来自输入的 `&mut [T]`；
- `count_byte` 只在调用 C ABI 函数处使用 `unsafe`，不得改变 C 端契约；
- 本课不需要新增任何依赖。

### 15.4 推荐完成顺序

1. 先运行一次 `cargo test -p l21_unsafe_ffi --no-run`，确认脚手架能编译。
2. 读 `tests/split_at_mut_checked.rs`，先实现“正常中间拆分”，跑 `cargo test -p l21_unsafe_ffi --test split_at_mut_checked splits_in_the_middle`。
3. 补上越界检查，让 `index_beyond_length_returns_none` 通过。
4. 检查 0、长度、空切片三个边界，再跑整个 `split_at_mut_checked` 测试文件。
5. 读 `src/ffi.rs` 的 C 端声明，实现 `count_byte`，先让 `counts_repeated_byte` 通过。
6. 跑空切片、NUL 字节和大输入测试，最后跑完整测试与 Clippy。

### 15.5 验收命令

```bash
cargo test -p l21_unsafe_ffi
cargo clippy -p l21_unsafe_ffi --all-targets -- -D warnings
cargo fmt -p l21_unsafe_ffi -- --check
```

也可以在仓库根目录运行课程结构检查：

```bash
python3 tools/check_course.py
```

在实现前，21 个测试中有 20 个会因 `todo!()` 失败；唯一通过的是 `c_fixture_counts_correctly_for_valid_slice`，它验证的是已提供的 C 替身，不是你的实现。主练习完成后，拆分与计数的 15 个测试应全部通过；`tests/consolidation.rs` 的 6 个要等第十六节完成后才会通过。

## 十六、课后巩固：读取 C 字符串

### 16.1 目标

完成 `src/consolidation.rs` 的：

```rust,ignore
pub unsafe fn read_c_name(pointer: *const c_char) -> Result<String, NameError>
```

它把以 NUL 结尾的 C 字符串复制成拥有型 Rust 文本。

### 16.2 输入、输出与边界

| 输入 | 期望输出 |
| --- | --- |
| 指向 `"ferris"` 的 C 字符串 | `Ok(String::from("ferris"))` |
| 指向 `""` 的 C 字符串 | `Ok(String::new())` |
| 指向 `"海豚"` 的合法 UTF-8 | `Ok(String::from("海豚"))` |
| 空指针 | `Err(NameError::NullPointer)` |
| 字节 `[0x66, 0x80, 0x6f]` | `Err(NameError::NotUtf8)` |

另外要保证返回的 `String` 拥有自己的内存：原始 `CString` 销毁后，文本仍然可用。

### 16.3 约束

- 先检查空指针，确认安全后再读内存；
- 使用 `CStr` 的借用视图，不要把 C 缓冲区直接变成 Rust 的 `&str` 长期返回；
- 非法 UTF-8 是可恢复错误，必须返回 `Err(NotUtf8)`，不能 panic；
- 函数本身是 `unsafe fn`：调用者要保证非空指针有效且以 NUL 结尾，这个责任无法由函数内部检查；
- 不新增依赖，不修改 `NameError` 和签名。

### 16.4 验收命令

```bash
cargo test -p l21_unsafe_ffi --test consolidation
```

实现前 6 个测试都会因 `todo!()` 失败；实现后应全部通过。

## 十七、常见误区

**误区 1：`unsafe` 块会关闭所有检查。**

不会。类型错误、变量移动后再用、借用冲突仍然报错。`unsafe` 只额外开放少数操作。

**误区 2：把整个函数包进 `unsafe` 更省事。**

这样会让审查范围扩大到所有普通代码，也容易掩盖真正危险的那一行。应把 `unsafe` 缩到最小，并让其余部分继续受安全规则保护。

**误区 3：SAFETY 注释只写“这里调用了 unsafe 函数”。**

这种注释没有提供任何证明。应逐条说明：哪个 API、它要求什么、代码中的哪个事实保证了它。

**误区 4：空切片可以随便传空指针。**

`from_raw_parts` 和 `from_raw_parts_mut` 即使长度为 0 也要求指针非空且对齐。空切片应使用满足条件的安全来源。

**误区 5：`CString::as_ptr()` 会把所有权交给 C。**

它只是借用。`CString` 销毁后指针失效；如果 C 需要长期保存，必须复制或转移一块独立的拥有型缓冲区。

**误区 6：`repr(C)` 一加，结构体就完全兼容 C 了。**

它只固定字段布局。整数宽度、指针有效性、字符串编码、所有权和释放函数仍需单独约定。

**误区 7：测试全部通过就说明没有 UB。**

测试只覆盖它实际执行的输入。未定义行为可能在小数据上不显现，也可能被优化改变。还需要审查不变量，必要时使用 Miri（Rust 官方的未定义行为检查工具）、sanitizer（在运行时检测内存和线程错误的工具）或 fuzzing（用大量随机输入寻找崩溃）等工具；这些工具也只能覆盖一部分问题。

**误区 8：Rust 导出的函数 panic 了，C 端接住就行。**

C 没有 Rust 的栈展开机制。panic 不允许随意穿过 C ABI。应在导出边界捕获 panic 或保证不 panic，并把失败转换为 C 能理解的错误码。

## 十八、本课总结

- `unsafe` 只开放五类额外操作，不关闭类型、移动和借用检查。
- 每个 `unsafe` 块都应是一份可核查的证明，而不是“我愿意冒险”的声明。
- 裸指针可以为空、悬垂、未对齐，也不携带生命周期；引用则有强保证。
- 指针算术以元素为单位，必须留在同一块分配内。
- `from_raw_parts(_mut)` 有一整套前置条件，空切片也不例外。
- 安全封装的结构是：先验证，再缩小危险区，写 SAFETY 注释，让生命周期来自输入。
- FFI 需要同时对齐 ABI、类型宽度、结构体布局、字符串编码、错误表示、panic 行为和所有权协议。
- 分配方与释放方必须配对；跨边界指针是借用还是转移必须写清楚。
- C 字符串以 NUL 结尾，Rust 字符串记录长度；两者不能直接互换。
- 测试和工具帮助发现问题，但不能替代对不变量的证明。

## 十九、课后复盘

用自己的话回答：

1. 为什么“创建裸指针”可以是安全的，而“解引用裸指针”必须用 `unsafe`？
2. 标准库的 `split_at_mut` 内部为什么需要 `unsafe`？它对外为什么仍然是安全函数？
3. 一个 `unsafe` 块的 SAFETY 注释应包含哪些信息？
4. `&[u8]` 能向 C 保证哪些事，不能保证哪些事？
5. 为什么 `CStr::from_ptr` 不能直接接收 `String::as_ptr()` 的结果？
6. 如果 C 函数要保存指针供以后使用，Rust 侧的安全包装应怎样改变？

## 下一课

下一课是综合项目。你会把数据建模、错误处理、集合、迭代器、trait、模块、测试和文件读写组合成一个完整的命令行数据工具，并继续练习“只在确有收益时引入复杂方案”。见[第二十二课](../22-capstone/README.md)。
