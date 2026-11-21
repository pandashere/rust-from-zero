# 第十一课提示：主练习与课后巩固

提示分三级，卡到哪一级看哪一级：

- **一级：思考方向**——只提示该想什么，不涉及具体代码。
- **二级：局部分解**——把方法拆成几个步骤，仍不写完整代码。
- **三级：接近完整的结构**——给出带空缺的骨架，你需要自己补上关键调用。

先看一级；想清楚了就不要继续往下看。三级提示故意不给出可直接提交的完整代码。

主练习的推荐顺序是：`new` 和 `as_tuple` → `swap` → `map` → `map_each` → `max_ref` → `duplicated_left` → `joined` → `FixedWindow` 的四个方法。课后巩固先做 `push`，再做 `pop` 和 `peek`。

## 主练习：`Pair::new` 与 `Pair::as_tuple`

### 一级：思考方向

`new` 的职责是把两个参数按顺序放进两个字段。`as_tuple` 只需要“看一眼”两个字段，不拿走它们；想一想，怎样从 `&self` 得到字段的只读引用，以及元组的两个位置分别对应哪个字段。

### 二级：局部分解

1. `new`：用结构体字面量创建 `Self`，`left` 放第一个参数，`right` 放第二个参数。
2. `as_tuple`：返回一个二元组，第一个元素借用 `left`，第二个元素借用 `right`。
3. 因为方法接收 `&self`，两个返回值都是借用，原 `Pair` 在调用后仍然拥有字段。

### 三级：接近完整的结构

```rust
pub fn new(left: T, right: T) -> Self {
    Self {
        left: /* 第一个参数 */,
        right: /* 第二个参数 */,
    }
}

pub fn as_tuple(&self) -> (&T, &T) {
    (/* 借用 left */, /* 借用 right */)
}
```

## 主练习：`Pair::swap`

### 一级：思考方向

`swap` 接收 `self`，也就是把整个 `Pair` 的所有权拿进来。既然两个字段已经归方法所有，能不能直接把它们放到对方的位置？需要复制吗？需要给 `T` 加 `Clone` 吗？

### 二级：局部分解

1. 创建一个新的 `Pair`。
2. 新 `Pair` 的 `left` 放原来的 `right`，新 `right` 放原来的 `left`。
3. 两个字段都是移动，不是复制；整个方法不需要任何 bound。

### 三级：接近完整的结构

```rust
pub fn swap(self) -> Pair<T> {
    Pair {
        left: self./* 原来的另一侧 */,
        right: self./* 原来的另一侧 */,
    }
}
```

## 主练习：`Pair::map`

### 一级：思考方向

`self` 被消费后，两个字段可以分别移动给闭包。闭包参数类型是 `FnMut`，说明它可能修改捕获状态，所以需要能反复调用。想一想：两个字段各调用一次闭包，结果放进新的 `Pair<U>`；先处理哪一侧？

### 二级：局部分解

1. 新 `Pair` 的 `left` 是 `operation(self.left)`。
2. 新 `Pair` 的 `right` 是 `operation(self.right)`。
3. 两次调用之间闭包可能改变捕获的状态，但 `FnMut` 允许这种调用；不要先 `clone` 字段。
4. 返回值类型是 `Pair<U>`，两个字段类型由闭包返回类型决定。

### 三级：接近完整的结构

```rust
pub fn map<U>(self, mut operation: impl FnMut(T) -> U) -> Pair<U> {
    Pair {
        left: /* 用 operation 处理 self.left */,
        right: /* 用 operation 处理 self.right */,
    }
}
```

## 主练习：`Pair::map_each`

### 一级：思考方向

两个闭包各自只处理自己那一侧。它们是 `FnOnce`，所以可以消费各自捕获的拥有型数据；每个闭包只调用一次。两个返回值必须都是 `U`。

### 二级：局部分解

1. 新 `Pair` 的 `left` 调用 `left_operation`，传入原来的 `self.left`。
2. 新 `Pair` 的 `right` 调用 `right_operation`，传入原来的 `self.right`。
3. 两个闭包分别消耗自己的捕获环境，不需要 `Clone`，也不需要共享状态。

### 三级：接近完整的结构

```rust
pub fn map_each<U, F, G>(self, left_operation: F, right_operation: G) -> Pair<U>
where
    F: FnOnce(T) -> U,
    G: FnOnce(T) -> U,
{
    Pair {
        left: /* 调用哪个闭包，传入哪个字段 */,
        right: /* 调用哪个闭包，传入哪个字段 */,
    }
}
```

## 主练习：`Pair::max_ref`

### 一级：思考方向

只需要比较两个字段，然后返回较大一侧的引用。注意约定：相等时返回 `right` 的引用。方法接收 `&self`，所以返回的是借用，不能移动字段。

### 二级：局部分解

1. 比较 `self.left` 和 `self.right`。
2. 如果左侧严格大于右侧，返回左侧的借用。
3. 否则（包括相等）返回右侧的借用。用“严格大于”而不是“大于等于”，可以让相等时自然落到右侧。

### 三级：接近完整的结构

```rust
pub fn max_ref(&self) -> &T {
    if self.left /* 严格大于? */ self.right {
        /* 返回左侧借用 */
    } else {
        /* 返回右侧借用 */
    }
}
```

## 主练习：`Pair::duplicated_left`

### 一级：思考方向

结果的两个字段都来自原来的 `left`，但它们必须是两份独立的值，所以各需要一次复制。原来的 `Pair` 只被借用，不能被修改或移动。

### 二级：局部分解

1. 新 `Pair` 的 `left` 是 `self.left` 的副本。
2. 新 `Pair` 的 `right` 也是 `self.left` 的副本。
3. 注意不是“把 left 和 right 各复制一份”；两个位置都要用 `left`。

### 三级：接近完整的结构

```rust
pub fn duplicated_left(&self) -> Pair<T> {
    Pair {
        left: self./* 哪个字段 */./* 复制方法 */(),
        right: self./* 哪个字段 */./* 复制方法 */(),
    }
}
```

## 主练习：`Pair<String>::joined`

### 一级：思考方向

字符串拼接需要生成一个新的 `String`，不能修改原来的字段。顺序是 `left`、分隔符、`right`。想一想标准库的 `format!` 宏怎样把多个文本片段组合成一个新字符串。

### 二级：局部分解

1. 用 `format!` 创建一个新 `String`。
2. 依次放入 `self.left`、参数 `separator`、`self.right`。
3. 三个部分之间不要多加字符；分隔符本身可能为空。

### 三级：接近完整的结构

```rust
pub fn joined(&self, separator: &str) -> String {
    format!(
        "{}{}{}",
        self./* 左侧字段 */,
        /* 分隔符参数 */,
        self./* 右侧字段 */
    )
}
```

## 主练习：`FixedWindow::first`、`last`、`as_slice`、`get`

### 一级：思考方向

这四个方法都只借用内部数组。`first`、`last`、`get` 要处理“没有这个位置”的情况，所以返回 `Option`；`as_slice` 直接给出整段只读视图。空数组是合法输入，不能直接索引。

### 二级：局部分解

1. `first`：数组/slice 有没有现成的“取第一个元素”的安全方法？它返回 `Option<&T>`。
2. `last`：同理，有没有“取最后一个元素”的安全方法？
3. `as_slice`：`&self.values` 的类型是 `&[T; N]`，它可以自动转成 `&[T]`，直接返回即可。
4. `get`：用 slice 的 `get(index)`，它在越界时返回 `None`；不要写 `self.values[index]`，那会 panic。
5. 以上方法对 `N = 0` 都不需要特判，只要使用返回 `Option`/切片的安全操作。

### 三级：接近完整的结构

```rust
pub fn first(&self) -> Option<&T> {
    self.values./* 安全取第一个的方法 */()
}

pub fn last(&self) -> Option<&T> {
    self.values./* 安全取最后一个的方法 */()
}

pub fn as_slice(&self) -> &[T] {
    /* 把整个数组借成切片 */
}

pub fn get(&self, index: usize) -> Option<&T> {
    self.values./* 安全按位置取元素的方法 */(index)
}
```

## 课后巩固：`Stack::push`

### 一级：思考方向

先判断“还能不能放”。容量上限是 const 参数 `N`，当前个数由已提供的方法给出。满了以后不能覆盖旧值，也不能丢掉新值；要把新值原样还给调用者。没满才真正放进 `Vec`。

### 二级：局部分解

1. 如果当前个数等于 `N`，直接 `return Err(value)`；这时 `Vec` 不动。
2. 否则把 `value` 追加到内部 `Vec` 的末尾。
3. 返回 `Ok(())`。`N = 0` 时第一步永远成立，所以自然返回 `Err`。

### 三级：接近完整的结构

```rust
pub fn push(&mut self, value: T) -> Result<(), T> {
    if self.values./* 当前个数 */() == N {
        return Err(/* 哪个值 */);
    }

    self.values./* 追加到末尾 */(value);
    Ok(())
}
```

## 课后巩固：`Stack::pop` 与 `Stack::peek`

### 一级：思考方向

`Vec` 的末尾就是栈顶。`pop` 要移出并返回最后一个值；`peek` 只看最后一个值，不移出。两者在空栈时都返回 `None`，不需要 `Clone`。

### 二级：局部分解

1. `pop`：调用 `Vec` 的 `pop`，它返回 `Option<T>`，正好符合签名。
2. `peek`：调用 `Vec` 的 `last`，它返回 `Option<&T>`，正好符合签名。
3. 两个方法都不需要判断 `N`；空栈时标准库方法自己返回 `None`。

### 三级：接近完整的结构

```rust
pub fn pop(&mut self) -> Option<T> {
    self.values./* 取出最后一个值的方法 */()
}

pub fn peek(&self) -> Option<&T> {
    self.values./* 查看最后一个值的方法 */()
}
```
