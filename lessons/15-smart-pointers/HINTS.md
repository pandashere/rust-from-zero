# 第十五课提示：主练习与课后巩固

提示分三级，卡到哪一级看哪一级：

- **一级：思考方向**——只提示该想什么，不涉及代码结构。
- **二级：局部分解**——把任务拆成几个判断步骤，仍不写完整代码。
- **三级：接近完整的结构**——给出带空缺的骨架，你需要自己补上关键调用。

先看一级；想清楚了就不要继续往下看。三级提示故意不给出可直接提交的完整代码。

主练习建议按 README 的顺序完成：`new`，`parent`/`child_names`，`path_from_root`，`add_child` 的重复检查与写入，成环检查，最后存活父节点检查。

## 主练习：`Unit::new`（`src/exercise.rs`）

### 一级：思考方向

新部门有三个字段。哪个字段直接来自参数？另外两个字段应该表示“还没有父部门”“还没有子部门”。标准库里有没有创建空 `Vec` 和空 `Weak` 的方法？返回类型要求调用者拿到几个强引用？

### 二级：局部分解

1. `name` 参数是 `impl Into<String>`，先把它转换成拥有型 `String`，再放进字段。
2. 父指针字段是 `RefCell<Weak<Unit>>`，里面放一个不指向任何节点的弱引用。
3. 子列表字段是 `RefCell<Vec<Rc<Unit>>>`，里面放空 `Vec`。
4. 用 `Rc::new(Self { ... })` 包起来返回。

### 三级：接近完整的结构

```rust
pub fn new(name: impl Into<String>) -> Rc<Self> {
    Rc::new(Self {
        name: name./* 转换成 String 的方法 */,
        parent: RefCell::new(/* 空弱引用 */),
        children: RefCell::new(/* 空 Vec */),
    })
}
```

## 主练习：`parent` 与 `child_names`

### 一级：思考方向

`parent` 要处理两种情况：从来没有父部门，以及父部门已经销毁。弱引用的哪个方法能把“可能失效”变成 `Option`？`child_names` 要返回直接子部门的名称副本，不能返回借用，也不能包含孙部门。子列表按什么顺序保存？

### 二级：局部分解

1. `parent()`：先借出 `self.parent` 里的 `Weak`，对它调用 `upgrade()`。`upgrade` 返回 `Option<Rc<Unit>>`，正好就是方法要求的类型。
2. `child_names()`：借出 `self.children`，逐个取出 `Rc<Unit>`，复制它的 `name` 字段，收集成 `Vec<String>`。只遍历这一层，不要递归。
3. 两个方法都只读，不需要 `borrow_mut`。借用凭证在方法返回前销毁。

### 三级：接近完整的结构

```rust
pub fn parent(&self) -> Option<Rc<Unit>> {
    self.parent.borrow()./* 把弱引用变成 Option<Rc<_>> 的方法 */()
}

pub fn child_names(&self) -> Vec<String> {
    self.children
        .borrow()
        .iter()
        .map(|child| child./* 名称字段 */.clone())
        .collect()
}
```

`iter()` 与 `map` 的细节见第十课；这里也可以写成 `for` 循环加 `push`，行为相同。

## 主练习：`path_from_root`

### 一级：思考方向

路径要从“最高的仍存活祖先”排到“当前节点”。但父指针只能向上走，所以你会先收集到“当前节点、父、祖父……”。收集完以后，顺序和题目要求相反。每走到一个节点，要保存的是名称副本还是引用？父链在哪一刻停止？

### 二级：局部分解

1. 先把当前节点的名称放进结果列表。
2. 从 `self.parent` 取得第一个仍存活的父节点（`Option<Rc<Unit>>`）。
3. 只要还有父节点：把它的名称加入列表，再把“当前”更新为它的父节点；父节点失效时循环结束。
4. 最后把列表反转，得到从祖先到当前节点的顺序。
5. 注意每次 `borrow` 都只在一小步内使用，不要把 `Ref` 保存到循环的下一轮。

### 三级：接近完整的结构

```rust
pub fn path_from_root(&self) -> Vec<String> {
    let mut path = vec![self.name.clone()];
    let mut current = self.parent./* 借出并 upgrade */;

    while let Some(node) = current {
        path.push(node./* 名称 */.clone());
        current = node.parent./* 借出并 upgrade */;
    }

    path./* 反转顺序的方法 */();
    path
}
```

先验证两个边界：没有父节点时循环一次也不执行；父节点销毁后 `upgrade` 返回 `None`，循环在该处停止。

## 主练习：`add_child` 的三项检查

### 一级：思考方向

三项检查有固定优先级：重复子节点、成环、已有存活父节点。重复检查只比较 `parent` 的**直接**子列表，并且比较身份而不是名称。成环检查要回答：“准备接收子节点的这个父节点，是否就是子节点本身，或者位于子节点的子树里？”沿哪条链走能回答这个问题？已有父节点检查看的是子节点的哪条边？

### 二级：局部分解

1. 重复检查：借出 `parent.children`，用 `Rc::ptr_eq` 逐个比较是否与 `child` 指向同一分配。命中就返回 `DuplicateChild`。
2. 成环检查：从 `parent` 本身开始，沿 `parent` 的父链向上走，每一步比较当前节点是否与 `child` 是同一分配。如果 `parent` 是 `child` 自己，第一步就会命中；如果 `parent` 是 `child` 的后代，走到某个祖先时会遇到 `child`。命中返回 `WouldCreateCycle`。
3. 已有父节点检查：借出 `child.parent`，`upgrade()` 得到 `Option<Rc<Unit>>`。如果是 `Some`，说明还有存活父节点，返回 `AlreadyHasParent`。
4. 三项都通过后再写入：先把 `child` 的父指针改为指向 `parent` 的弱引用，再把 `child` 追加到 `parent.children` 末尾。
5. 所有读取借用在写入前结束，避免同一个 `RefCell` 的借用重叠。

### 三级：接近完整的结构

```rust
pub fn add_child(parent: &Rc<Self>, child: Rc<Self>) -> Result<(), LinkError> {
    let duplicate = parent
        .children
        .borrow()
        .iter()
        .any(|existing| Rc::/* 比较两个 Rc 是否同一分配 */(existing, &child));
    if duplicate {
        return Err(LinkError::DuplicateChild);
    }

    let mut cursor = Some(Rc::clone(parent));
    while let Some(node) = cursor {
        if Rc::/* 比较身份 */(&node, &child) {
            return Err(LinkError::WouldCreateCycle);
        }
        cursor = node.parent.borrow().upgrade();
    }

    if child.parent.borrow().upgrade().is_some() {
        return Err(LinkError::AlreadyHasParent);
    }

    *child.parent.borrow_mut() = Rc::downgrade(parent);
    parent.children.borrow_mut().push(child);
    Ok(())
}
```

这段骨架故意把“检查”和“写入”分开。请自己确认：

- 第三项检查为什么可以放在成环检查之后？
- 如果先写父指针、再发现重复，会留下什么半成品状态？
- `while let` 循环中 `node.parent.borrow()` 的临时凭证什么时候销毁？它会不会和后面的 `borrow_mut` 重叠？

## 课后巩固：`Subscriber` 与 `Channel` 的构造和读取

### 一级：思考方向

`Subscriber` 的收件箱和 `Channel` 的订阅者列表都是“需要通过共享句柄修改”的字段，所以类型已经是 `RefCell`。构造函数只需要创建空的内部值。`inbox()` 要返回消息副本，不能把内部 `Ref` 带出去。

### 二级：局部分解

1. `Subscriber::new`：把名称转换成 `String`，收件箱放空 `Vec`，用 `Rc::new` 返回。
2. `Subscriber::inbox`：借出 `self.inbox`，克隆整个 `Vec<String>` 返回；这样调用者拿到的消息不会让订阅者继续存活。
3. `Channel::new`：名称转换成 `String`，订阅者列表放空 `Vec`。
4. `Channel::name` 已经实现，不需要改。

### 三级：接近完整的结构

```rust
pub fn new(name: impl Into<String>) -> Rc<Self> {
    Rc::new(Self {
        name: name./* 转换成 String */,
        inbox: RefCell::new(/* 空 Vec */),
    })
}

pub fn inbox(&self) -> Vec<String> {
    self.inbox.borrow()./* 复制整个 Vec 的方法 */()
}
```

`Channel::new` 结构相同，只是字段换成 `subscribers: RefCell<Vec<Weak<Subscriber>>>`。

## 课后巩固：`subscribe`

### 一级：思考方向

检查有优先级：先看“同一个 `Rc` 是否已经登记”，再看“是否有仍然存活、同名但身份不同的订阅者”。遍历订阅者列表时，每个元素都是 `Weak<Subscriber>`；先 `upgrade`，失败就跳过。只有全部检查通过，才把新订阅者的弱引用追加到末尾。

### 二级：局部分解

1. 借出 `channel.subscribers`，遍历每个 `weak`。
2. 对每个 `weak` 调用 `upgrade()`；返回 `None` 的登记直接忽略，它既不算重复，也不算同名。
3. 对存活订阅者，先用 `Rc::ptr_eq` 与传入的 `subscriber` 比较身份；命中返回 `AlreadySubscribed`。
4. 再比较 `name` 是否相同；命中返回 `DuplicateName`。
5. 循环结束后，把 `Rc::downgrade(subscriber)` 追加到列表末尾，返回 `Ok(())`。
6. 检查期间的读借用要在追加之前结束，避免同一个 `RefCell` 读写重叠。

### 三级：接近完整的结构

```rust
pub fn subscribe(
    channel: &Rc<Channel>,
    subscriber: &Rc<Subscriber>,
) -> Result<(), SubscribeError> {
    for weak in channel.subscribers.borrow().iter() {
        let Some(existing) = weak./* 尝试升级 */() else {
            continue;
        };
        if Rc::/* 身份比较 */(&existing, subscriber) {
            return Err(SubscribeError::AlreadySubscribed);
        }
        if existing.name == subscriber.name {
            return Err(SubscribeError::DuplicateName);
        }
    }

    channel
        .subscribers
        ./* 可变借用 */
        .push(Rc::/* 创建弱引用 */(subscriber));
    Ok(())
}
```

想一想：如果先 `borrow_mut` 再在循环里 `upgrade`，会不会和同一个 `RefCell` 的读借用冲突？把借用拆成“读阶段”和“写阶段”更容易保证不 panic。

## 课后巩固：`publish`

### 一级：思考方向

发布要按登记顺序遍历，对每个仍存活的订阅者追加一条消息，并统计成功投递的人数。已销毁的登记要跳过，不计入返回值。消息要变成每个订阅者各自拥有的 `String`，不能把同一个借用传进去后失效。

### 二级：局部分解

1. 先准备一个计数器 `delivered = 0`。
2. 借出 `channel.subscribers`，按顺序遍历。
3. 对每个 `weak` 调用 `upgrade()`；`None` 就跳过。
4. `Some(subscriber)` 时，借出它的 `inbox`，把 `message` 转换成拥有型 `String` 后追加，计数器加 1。
5. 循环结束后返回计数器。
6. 注意：频道的 `RefCell` 和订阅者的 `RefCell` 是两个不同的格子，可以分别借用；但不要把频道的可变借用和订阅者操作混在同一个长表达式里，先让代码容易读。

### 三级：接近完整的结构

```rust
pub fn publish(channel: &Rc<Channel>, message: &str) -> usize {
    let mut delivered = 0;

    for weak in channel.subscribers.borrow().iter() {
        if let Some(subscriber) = weak./* 升级 */() {
            subscriber
                .inbox
                ./* 可变借用 */
                .push(message./* 变成拥有型 String */());
            delivered += 1;
        }
    }

    delivered
}
```

如果你希望在发布时顺便清理失效登记，可以在遍历后对列表做一次保留操作；但测试只检查可观察行为，清理不是必须的。若这样做，务必避免在遍历时同时持有列表的可变借用。

## 课后巩固：`subscriber_count` 与 `subscriber_names`

### 一级：思考方向

两个函数都只统计“仍然存活”的订阅者。数量可以直接数 `upgrade()` 成功的次数；名称要按登记顺序返回副本。它们在语义上是查询，不应该 panic，也不应该让订阅者继续存活。

### 二级：局部分解

1. `subscriber_count`：借出列表，遍历，对每个 `weak` 调用 `upgrade()`，统计 `Some` 的数量。
2. `subscriber_names`：借出列表，遍历，`upgrade()` 成功就复制名称加入结果；失败跳过。
3. 返回的 `Vec<String>` 是拥有型数据，函数返回后不借用频道。

### 三级：接近完整的结构

```rust
pub fn subscriber_count(channel: &Rc<Channel>) -> usize {
    channel
        .subscribers
        .borrow()
        .iter()
        .filter(|weak| weak./* 能否升级 */().is_some())
        .count()
}

pub fn subscriber_names(channel: &Rc<Channel>) -> Vec<String> {
    let mut names = Vec::new();
    for weak in channel.subscribers.borrow().iter() {
        if let Some(subscriber) = weak./* 升级 */() {
            names.push(subscriber./* 名称 */.clone());
        }
    }
    names
}
```

验证边界：空频道返回 0 和空 `Vec`；所有订阅者都销毁后也一样；不要对 `upgrade()` 的结果直接 `unwrap`。
