// 第二课主练习：网格机器人状态机
//
// 机器人站在一张无限大的网格上，有坐标 (x, y) 和朝向。
// 坐标约定：北 (North) 是 y 增加的方向，南 (South) 是 y 减少的方向，
// 东 (East) 是 x 增加的方向，西 (West) 是 x 减少的方向。
//
// 你的任务：实现下方 impl 块里的四个方法（方法签名和类型定义不要修改）。
// 每个方法上方的注释描述了它在各种输入下的行为，包括边界和无效输入。
// 全部行为测试通过的方式见 README 的“主练习”一节。

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    North,
    East,
    South,
    West,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Command {
    TurnLeft,
    TurnRight,
    Move(i32),
}

#[derive(Debug, PartialEq, Eq)]
struct Robot {
    x: i32,
    y: i32,
    direction: Direction,
}

impl Robot {
    // 创建一台机器人，把给定的坐标和朝向原样保存下来：
    // 之后 position() 应返回 (x, y)，direction() 应返回创建时给定的方向。
    fn new(x: i32, y: i32, direction: Direction) -> Self {
        let _ = (&x, &y, &direction);
        todo!()
    }

    // 返回机器人当前的坐标，格式是 (x, y)：
    // 第一个元素是 x（东西方向的位置），第二个元素是 y（南北方向的位置）。
    // 这个方法只读取状态，不修改机器人。
    fn position(&self) -> (i32, i32) {
        todo!()
    }

    // 返回机器人当前的朝向。
    // 这个方法只读取状态，不修改机器人。
    fn direction(&self) -> Direction {
        todo!()
    }

    // 执行一条命令，并修改机器人自身的状态：
    // - Command::TurnLeft：朝向按 North -> West -> South -> East -> North
    //   转一格，坐标保持不变。
    // - Command::TurnRight：朝向按 North -> East -> South -> West -> North
    //   转一格，坐标保持不变。
    // - Command::Move(distance)：沿当前朝向前进 distance 步。朝北时 y 增加
    //   distance，朝南时 y 减少 distance，朝东时 x 增加 distance，朝西时
    //   x 减少 distance。distance 为负数时，这条命令无效：坐标和朝向都保持
    //   不变，就像没有收到这条命令一样。
    fn apply(&mut self, command: Command) {
        let _ = &command;
        todo!()
    }
}

fn main() {
    let mut robot = Robot::new(0, 0, Direction::North);

    robot.apply(Command::Move(3));
    robot.apply(Command::TurnRight);
    robot.apply(Command::Move(2));
    robot.apply(Command::TurnLeft);
    robot.apply(Command::Move(1));

    println!("position = {:?}", robot.position());
    println!("direction = {:?}", robot.direction());
}

#[cfg(test)]
mod tests {
    use super::{Command, Direction, Robot};

    #[test]
    fn new_stores_given_position_and_direction() {
        let robot = Robot::new(2, -3, Direction::East);

        assert_eq!(robot.position(), (2, -3));
        assert_eq!(robot.direction(), Direction::East);
    }

    #[test]
    fn turning_right_four_times_returns_to_original_direction() {
        let mut robot = Robot::new(0, 0, Direction::North);

        robot.apply(Command::TurnRight);
        assert_eq!(robot.direction(), Direction::East);
        robot.apply(Command::TurnRight);
        assert_eq!(robot.direction(), Direction::South);
        robot.apply(Command::TurnRight);
        assert_eq!(robot.direction(), Direction::West);
        robot.apply(Command::TurnRight);
        assert_eq!(robot.direction(), Direction::North);
    }

    #[test]
    fn turning_left_four_times_returns_to_original_direction() {
        let mut robot = Robot::new(0, 0, Direction::South);

        robot.apply(Command::TurnLeft);
        assert_eq!(robot.direction(), Direction::East);
        robot.apply(Command::TurnLeft);
        assert_eq!(robot.direction(), Direction::North);
        robot.apply(Command::TurnLeft);
        assert_eq!(robot.direction(), Direction::West);
        robot.apply(Command::TurnLeft);
        assert_eq!(robot.direction(), Direction::South);
    }

    #[test]
    fn turning_does_not_change_position() {
        let mut robot = Robot::new(4, -7, Direction::West);

        robot.apply(Command::TurnLeft);
        robot.apply(Command::TurnRight);
        robot.apply(Command::TurnRight);

        assert_eq!(robot.position(), (4, -7));
    }

    #[test]
    fn moving_changes_only_the_coordinate_along_current_direction() {
        let mut robot = Robot::new(1, 2, Direction::North);

        robot.apply(Command::Move(3));
        assert_eq!(robot.position(), (1, 5));

        robot.apply(Command::TurnRight);
        robot.apply(Command::Move(4));
        assert_eq!(robot.position(), (5, 5));

        robot.apply(Command::TurnRight);
        robot.apply(Command::Move(2));
        assert_eq!(robot.position(), (5, 3));

        robot.apply(Command::TurnRight);
        robot.apply(Command::Move(6));
        assert_eq!(robot.position(), (-1, 3));
    }

    #[test]
    fn move_with_zero_distance_keeps_position() {
        let mut robot = Robot::new(-2, 8, Direction::East);

        robot.apply(Command::Move(0));

        assert_eq!(robot.position(), (-2, 8));
        assert_eq!(robot.direction(), Direction::East);
    }

    #[test]
    fn move_with_negative_distance_is_ignored() {
        let mut robot = Robot::new(3, 3, Direction::South);

        robot.apply(Command::Move(-5));

        assert_eq!(robot.position(), (3, 3));
        assert_eq!(robot.direction(), Direction::South);
    }

    #[test]
    fn move_after_turning_uses_the_new_direction() {
        let mut robot = Robot::new(0, 0, Direction::North);

        robot.apply(Command::TurnLeft);
        robot.apply(Command::Move(2));

        assert_eq!(robot.position(), (-2, 0));
        assert_eq!(robot.direction(), Direction::West);
    }
}
