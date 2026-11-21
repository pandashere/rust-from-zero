# 第二课提示：主练习与课后巩固

本文件为主练习（网格机器人）和课后巩固（楼层电梯）各准备三级提示。
卡住时先看第 1 级，继续卡住再看下一级，不要直接跳到第 3 级。
提示不直接给出完整答案；第 3 级也只给出接近完整的结构，留有空缺需要你补全。

## 主练习：网格机器人（src/main.rs）

### 第 1 级：思考方向

- `new`、`position`、`direction` 三个方法都不需要任何分支逻辑。想一想：
  struct 的字段存在哪里？方法里怎样读到“自己这个值”的字段？
- `apply` 才是本练习的核心。它收到的 `command` 有三种可能，每种可能的
  处理方式完全不同——回想 README 里交通灯的 `advance` 是怎样根据当前
  状态分情况处理的。
- `Move(distance)` 里“沿当前方向移动”意味着：同一个命令，在四种朝向下
  修改的坐标不同。这需要再做一次“按方向分情况”。
- 负距离要求“整条命令无效”。把这条检查放在修改任何字段之前。

### 第 2 级：局部分解

- `new`：用字段初始化简写，把三个参数原样装进 `Self { ... }`。
- `position`：把两个字段组合成一个元组返回，顺序是先 x 后 y。
- `direction`：返回同名字段的值。
- `apply`：最外层对 `command` 分三种情况：
  - `TurnLeft`：把当前方向替换成“逆时针转一格”后的方向。四个方向都要
    给出明确的新方向，不要用 `_` 兜底。
  - `TurnRight`：同上，方向相反（顺时针）。
  - `Move(distance)`：先处理“distance 为负则什么都不做”；否则按当前
    方向分四种情况，每种情况只修改一个坐标：北加 y、南减 y、东加 x、
    西减 x。

### 第 3 级：接近完整的结构

```rust
fn apply(&mut self, command: Command) {
    match command {
        Command::TurnLeft => {
            self.direction = match self.direction {
                Direction::North => Direction::West,
                Direction::West => todo!("逆时针再转一格是哪个方向？"),
                // 其余两个方向按同样规律各写一行
                Direction::South => todo!(),
                Direction::East => todo!(),
            };
        }
        Command::TurnRight => {
            // 与 TurnLeft 结构相同，只是每个方向映射到顺时针的下一个
            todo!()
        }
        Command::Move(distance) => {
            if distance >= 0 {
                match self.direction {
                    Direction::North => self.y += distance,
                    Direction::East => todo!("只修改 x"),
                    // 其余两个方向各修改一个坐标
                    Direction::South => todo!(),
                    Direction::West => todo!(),
                }
            }
        }
    }
}
```

骨架里的每个 `todo!()` 对应一个需要你补全的方向或分支，替换后编译器会
重新做穷尽性检查。如果你发现自己想用 `_` 一次性代替剩下的方向，回到
README 的“通配符不是默认首选”一节再想一次。

## 课后巩固：楼层电梯（src/bin/l02_elevator.rs）

### 第 1 级：思考方向

- 四条命令其实只有两类：开门/关门只改 `door` 字段；上楼/下楼改 `floor`
  字段，但有两条安全规定要先检查。
- “门开着时移动命令无效”和“距离为负时命令无效”都是**先检查、不满足就
  直接结束**，电梯状态一点不动。
- “停在边界楼层”不是“命令无效”：电梯要尽量移动，只是不能越过边界。

### 第 2 级：局部分解

- `new`：四个字段分别是——起始楼层是 `min_floor`（不是 0），上下限照存，
  门初始为 `DoorState::Closed`。
- `floor` 与 `door`：各返回一个字段。
- `apply` 对 `command` 分四种情况：
  - `OpenDoor` / `CloseDoor`：直接给 `self.door` 赋对应值，重复执行无害。
  - `GoUp(n)`：满足以下任一条件就什么都不做——门当前是开的；`n` 是负数。
    否则给 `self.floor` 加上 `n`；加完若超过 `self.max_floor`，就把它
    改回 `self.max_floor`。
  - `GoDown(n)`：与 `GoUp` 对称，边界是 `self.min_floor`。

### 第 3 级：接近完整的结构

```rust
fn apply(&mut self, command: ElevatorCommand) {
    match command {
        ElevatorCommand::OpenDoor => self.door = DoorState::Open,
        ElevatorCommand::CloseDoor => /* 与上一行对称 */,
        ElevatorCommand::GoUp(n) => {
            // 两个“命令无效”的条件：门开着，或 n 为负
            if self.door == DoorState::Closed && n >= 0 {
                self.floor += n;
                if self.floor > self.max_floor {
                    // 越界了：电梯应该停在哪一层？
                    todo!()
                }
            }
        }
        ElevatorCommand::GoDown(n) => {
            // 与 GoUp 结构相同：同样的两个无效条件，
            // 减去 n 之后检查的是 min_floor
            todo!()
        }
    }
}
```

判断“门是否关着”用 `==` 比较（`DoorState` 已派生 `PartialEq`），也可以
改用 `matches!`。两种写法都正确，选你能读懂的。
