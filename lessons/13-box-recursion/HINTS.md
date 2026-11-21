# 第十三课提示：主练习与课后巩固

提示分三级，卡到哪一级看哪一级：

- **一级：思考方向**——只提示该想什么，不涉及代码结构。
- **二级：局部分解**——把方法拆成几个判断步骤，仍不写完整代码。
- **三级：接近完整的结构**——给出带空缺的骨架，你需要自己补上关键调用。

先看一级；想清楚了就不要继续往下看。三级提示故意不给出可直接提交的完整代码。

主练习推荐顺序：`eval` 的基本求值 → `eval` 的错误处理 → `depth` → `literals` → `into_literals`。课后巩固推荐顺序：`file_count` → `total_size` → `max_depth` → `find_path`；前三个统计方法的递归形状相似，先做一个能减少陌生感。

所有方法都只改 `src/exercise.rs` 和 `src/consolidation.rs` 里的 `todo!()`，不要修改类型定义、方法签名和测试。

## 主练习：`eval`（`src/exercise.rs`）

### 一级：思考方向

`eval` 的递归结构是：叶子直接给出数字，其他节点先求子表达式的值，再把结果组合起来。六种变体都要覆盖。难点不在递归，而在“算出结果后怎么检查它还能不能放进 `i64`”。标准库里有一组名字以 `checked_` 开头的算术方法，它们不会 panic，而是用 `Option` 表示“成功”或“溢出”。另外，除法的除数要先单独检查是否为 0。

### 二级：局部分解

1. `match self`，为 `Literal`、`Neg` 和四个二元变体各写一个分支。
2. `Literal(value)` 里的 `value` 是 `&i64`；`i64` 是 `Copy`，解引用就能得到值。
3. `Neg(inner)`：先得到子树的 `Result`，有错误就向上返回；成功时用“检查型取负”把 `i64` 变成 `Option<i64>`，`None` 转成 `Err(EvalError::Overflow)`。
4. 二元节点：**先左后右**求值，任何一边出错都立刻返回；两边都成功后再组合。
5. 加减乘用对应的 `checked_` 方法；除法先判断右值是否为 0，再调用检查型除法（它还会覆盖 `i64::MIN / -1` 这种特殊情况）。
6. 把 `Option` 转成 `Result` 时，统一用 `EvalError::Overflow` 作为错误。

### 三级：接近完整的结构

```rust
pub fn eval(&self) -> Result<i64, EvalError> {
    match self {
        Expr::Literal(value) => Ok(*value),
        Expr::Neg(inner) => {
            let value = /* 递归求 inner，错误如何向上传递 */;
            value
                ./* 检查型取负 */
                ./* 把 Option 转成 Result，错误用 Overflow */
        }
        Expr::Add(left, right) => {
            let left = /* 递归求 left，错误如何向上传递 */;
            let right = /* 递归求 right */;
            left
                ./* 检查型加法 */(right)
                ./* 把 Option 转成 Result */
        }
        Expr::Divide(left, right) => {
            let left = /* 先求左边 */;
            let right = /* 再求右边 */;
            if /* 右值是否为 0 */ {
                return Err(EvalError::DivisionByZero);
            }
            left
                ./* 检查型除法 */(right)
                ./* 把 Option 转成 Result */
        }
        // Subtract 和 Multiply 与 Add 形状相同，换成对应运算。
    }
}
```

除法分支里“先求左边、再求右边”的顺序要和注释一致：左边出错时，不应该先报除零。

## 主练习：`depth`

### 一级：思考方向

深度是“从当前节点到最远叶子要经过几个节点”。叶子自己算 1 层；每往下一层就加 1。一元节点只有一棵子树，二元节点要在左右子树里挑更深的那一棵。

### 二级：局部分解

1. `Literal(_)` 直接返回 1。
2. `Neg(inner)` 返回 `1 + inner.depth()`。
3. 四个二元变体可以合并成同一个 `match` 分支（用 `|` 连接模式），返回 `1 + 左深度.max(右深度)`。
4. `usize` 的 `.max(另一个)` 方法返回两者中较大的那个。

### 三级：接近完整的结构

```rust
pub fn depth(&self) -> usize {
    match self {
        Expr::Literal(_) => 1,
        Expr::Neg(inner) => 1 + inner./* 递归统计深度 */ (),
        Expr::Add(left, right)
        | Expr::Subtract(left, right)
        | Expr::Multiply(left, right)
        | Expr::Divide(left, right) => {
            1 + left./* 递归 */ ()./* 与右子树深度取较大者 */(right./* 递归 */ ())
        }
    }
}
```

`left` 和 `right` 的类型都是 `&Box<Expr>`，调用 `Expr` 的方法时编译器会自动解引用。

## 主练习：`literals`

### 一级：思考方向

结果要按“先左后右”的顺序排列。只借用树，所以不能把节点移走；最简单的做法是在方法内部准备一个 `Vec<i64>`，再写一个负责递归的辅助函数，把遇到的叶子依次追加进去。辅助函数可以定义在 `literals` 的方法体里面。

### 二级：局部分解

1. 在 `literals` 方法体内部定义一个普通函数 `fn walk(expr: &Expr, out: &mut Vec<i64>)`。
2. `walk` 的 `match` 里，`Literal` 把 `*value` push 进 `out`。
3. `Neg` 只递归进入它的子树。
4. 四个二元变体可以合并：先对左子树递归，再对右子树递归。
5. `literals` 自己创建 `Vec`，调用 `walk(self, &mut values)`，返回 `values`。

### 三级：接近完整的结构

```rust
pub fn literals(&self) -> Vec<i64> {
    fn walk(expr: &Expr, out: &mut Vec<i64>) {
        match expr {
            Expr::Literal(value) => out./* 追加 */(*value),
            Expr::Neg(inner) => /* 对 inner 递归 */,
            Expr::Add(left, right)
            | Expr::Subtract(left, right)
            | Expr::Multiply(left, right)
            | Expr::Divide(left, right) => {
                /* 先处理 left */;
                /* 再处理 right */;
            }
        }
    }

    let mut values = Vec::new();
    walk(self, &mut values);
    values
}
```

`walk` 的 `expr` 是借用，模式里的 `left`、`right` 自然也是借用，所以调用 `walk(left, out)` 不需要写 `&*left`。

## 主练习：`into_literals`

### 一级：思考方向

这个方法和 `literals` 的结果一样，但接收者是 `self`，整棵树归它所有。因此可以在模式匹配时把 `Box` 里的 `Expr` 移出来，再递归处理。不要调用 `self.literals()` 后直接返回：那样虽然结果对，但没有练习“消费树、逐层移出数据”这个目标。

### 二级：局部分解

1. 方法体内部定义 `fn walk(expr: Expr, out: &mut Vec<i64>)`，注意参数是 `Expr`，不是 `&Expr`。
2. `Literal(value)` 直接把 `value` push 进 `out`。
3. `Neg(inner)` 里 `inner` 是 `Box<Expr>`，用 `*inner` 把里面的 `Expr` 移出来，交给 `walk`。
4. 二元变体先处理 `*left`，再处理 `*right`，顺序与 `literals` 一致。
5. `into_literals` 创建 `Vec`，调用 `walk(self, &mut values)`，返回结果。

### 三级：接近完整的结构

```rust
pub fn into_literals(self) -> Vec<i64> {
    fn walk(expr: Expr, out: &mut Vec<i64>) {
        match expr {
            Expr::Literal(value) => out./* 追加 value */,
            Expr::Neg(inner) => /* 把 *inner 交给 walk */,
            Expr::Add(left, right)
            | Expr::Subtract(left, right)
            | Expr::Multiply(left, right)
            | Expr::Divide(left, right) => {
                /* 先把 *left 交给 walk */;
                /* 再把 *right 交给 walk */;
            }
        }
    }

    let mut values = Vec::new();
    walk(self, &mut values);
    values
}
```

`*inner`、`*left`、`*right` 能从 `Box` 里移出值，是因为 `Box` 自己拥有内容；如果 `walk` 接收的是 `&Expr`，这些移动就会变成 `E0507`。

## 课后巩固：`file_count`（`src/consolidation.rs`）

### 一级：思考方向

文件算 1 个，目录本身不算文件，只把子项的结果加起来。空目录没有子项，累加自然得到 0。先匹配 `EntryKind`，再决定是否遍历 `children`。

### 二级：局部分解

1. `match &self.kind`：`File { .. }` 直接返回 1。
2. `Directory { children }` 里准备一个计数器，初始为 0。
3. 用 `for child in children` 逐个加上 `child.file_count()`。
4. 返回计数器。

### 三级：接近完整的结构

```rust
pub fn file_count(&self) -> usize {
    match &self.kind {
        EntryKind::File { .. } => 1,
        EntryKind::Directory { children } => {
            let mut count = 0;
            for child in children {
                count += /* 子项自己的文件数 */;
            }
            count
        }
    }
}
```

## 课后巩固：`total_size`

### 一级：思考方向

和 `file_count` 几乎一样，只是文件贡献的是 `size`，不是 1。目录把所有子项的大小加起来，空目录得到 0。注意 `size` 是 `u64`，不要和 `usize` 的计数混在一起。

### 二级：局部分解

1. `match &self.kind`。
2. `File { size }` 返回 `*size`（`size` 是 `&u64`，`u64` 是 `Copy`）。
3. `Directory { children }` 用一个 `u64` 累加器，逐个加上 `child.total_size()`。
4. 返回总和。

### 三级：接近完整的结构

```rust
pub fn total_size(&self) -> u64 {
    match &self.kind {
        EntryKind::File { size } => *size,
        EntryKind::Directory { children } => {
            let mut total = 0;
            for child in children {
                total += /* 子项自己的总大小 */;
            }
            total
        }
    }
}
```

## 课后巩固：`max_depth`

### 一级：思考方向

文件没有子项，深度是 1；空目录也没有子项，深度同样是 1，不是 0。非空目录要先找到所有子项里最大的深度，再加 1。难点是“空目录不能去取最大值”，否则没有东西可取。

### 二级：局部分解

1. `File { .. }` 返回 1。
2. `Directory { children }` 先判断 `children.is_empty()`：为空直接返回 1。
3. 非空时，把 `deepest` 初始化为 0，遍历 `children`，用 `child.max_depth()` 不断更新最大值。
4. 最后返回 `1 + deepest`。

### 三级：接近完整的结构

```rust
pub fn max_depth(&self) -> usize {
    match &self.kind {
        EntryKind::File { .. } => 1,
        EntryKind::Directory { children } => {
            if /* children 是否为空 */ {
                return 1;
            }
            let mut deepest = 0;
            for child in children {
                deepest = deepest./* 与 child 的深度取较大者 */;
            }
            1 + deepest
        }
    }
}
```

也可以不提前返回，用 `.max().unwrap_or(0)` 处理空目录；两种写法都要保证空目录得到 1。

## 课后巩固：`find_path`

### 一级：思考方向

查找分两步：先看“当前条目自己”是不是目标；不是的话，按 `children` 的顺序逐个进入子树找。子项找到后，返回的路径还缺当前这一层，需要把当前名字放到路径最前面。找不到就返回 `None`，不要把空 `Vec` 当成“找到了”。

### 二级：局部分解

1. 先比较 `self.name == target`；相等就返回只含 `self.name.clone()` 的 `Some(Vec)`。
2. 如果当前是文件且名字不匹配，返回 `None`。
3. 如果是目录，遍历 `children`，对每个子项调用 `child.find_path(target)`。
4. 某个子项返回 `Some(mut path)` 时，用 `path.insert(0, self.name.clone())` 把当前层补到最前面，再返回 `Some(path)`。
5. 所有子项都返回 `None` 时，返回 `None`。
6. 注意顺序：先比较当前条目，再进入子树；这样根节点和空目录也能被找到，同名时也会返回存储顺序靠前的那一个。

### 三级：接近完整的结构

```rust
pub fn find_path(&self, target: &str) -> Option<Vec<String>> {
    if /* 当前名字等于 target */ {
        return Some(vec![/* 当前名字的副本 */]);
    }

    match &self.kind {
        EntryKind::File { .. } => None,
        EntryKind::Directory { children } => {
            for child in children {
                if let Some(mut path) = /* 在子树里查找 */ {
                    path./* 把当前名字插到最前面 */(0, /* 当前名字的副本 */);
                    return Some(path);
                }
            }
            None
        }
    }
}
```

`find_path` 只借用树，返回的路径是复制出来的 `String`，所以原树在查找后仍然可以继续使用。路径拼接也可以换成“递归时携带前缀”的写法，但不要为了省一次插入而引入借用冲突。
