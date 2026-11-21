// 第二课课后巩固：楼层电梯
//
// 一部电梯在一栋楼里运行。楼层的下限和上限在建电梯时确定（例如
// 地下 2 层到地上 10 层，即 min_floor = -2，max_floor = 10）。
// 电梯有一扇门，出于安全规定：门开着的时候，电梯不能移动。
//
// 这是课后巩固任务，和主练习（网格机器人）使用同一批概念，但数据
// 和边界不同。你的任务：实现下方 impl 块里的四个方法（方法签名和
// 类型定义不要修改）。每个方法上方的注释描述了它在各种输入下的
// 行为，包括边界和无效输入。验收方式见 README 的“课后巩固”一节。

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DoorState {
    Open,
    Closed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ElevatorCommand {
    GoUp(i32),
    GoDown(i32),
    OpenDoor,
    CloseDoor,
}

#[derive(Debug, PartialEq, Eq)]
struct Elevator {
    floor: i32,
    min_floor: i32,
    max_floor: i32,
    door: DoorState,
}

impl Elevator {
    // 创建一部电梯，楼层范围是 min_floor 到 max_floor（含两端）。
    // 电梯初始停靠在 min_floor，门是关闭的。
    // 之后 floor() 应返回 min_floor，door() 应返回 DoorState::Closed。
    fn new(min_floor: i32, max_floor: i32) -> Self {
        let _ = (&min_floor, &max_floor);
        todo!()
    }

    // 返回电梯当前所在的楼层。只读取状态，不修改电梯。
    fn floor(&self) -> i32 {
        todo!()
    }

    // 返回电梯门当前的状态（DoorState::Open 或 DoorState::Closed）。
    // 只读取状态，不修改电梯。
    fn door(&self) -> DoorState {
        todo!()
    }

    // 执行一条命令，并修改电梯自身的状态：
    // - ElevatorCommand::OpenDoor：门变为 DoorState::Open；
    //   如果门本来就是开的，状态不变。
    // - ElevatorCommand::CloseDoor：门变为 DoorState::Closed；
    //   如果门本来就是关的，状态不变。
    // - ElevatorCommand::GoUp(n)：电梯上升 n 层。如果门是开着的，
    //   整条命令无效，楼层不变。如果上升后会超过 max_floor，
    //   电梯停在 max_floor。n 为负数时整条命令无效，楼层不变。
    // - ElevatorCommand::GoDown(n)：电梯下降 n 层。如果门是开着的，
    //   整条命令无效，楼层不变。如果下降后会低于 min_floor，
    //   电梯停在 min_floor。n 为负数时整条命令无效，楼层不变。
    fn apply(&mut self, command: ElevatorCommand) {
        let _ = &command;
        todo!()
    }
}

fn main() {
    let mut elevator = Elevator::new(-2, 10);

    elevator.apply(ElevatorCommand::GoUp(5));
    elevator.apply(ElevatorCommand::OpenDoor);
    elevator.apply(ElevatorCommand::GoUp(1));
    elevator.apply(ElevatorCommand::CloseDoor);
    elevator.apply(ElevatorCommand::GoDown(3));

    println!("floor = {}", elevator.floor());
    println!("door = {:?}", elevator.door());
}

#[cfg(test)]
mod tests {
    use super::{DoorState, Elevator, ElevatorCommand};

    #[test]
    fn new_elevator_starts_at_min_floor_with_door_closed() {
        let elevator = Elevator::new(-2, 10);

        assert_eq!(elevator.floor(), -2);
        assert_eq!(elevator.door(), DoorState::Closed);
    }

    #[test]
    fn go_up_and_go_down_move_the_requested_floors() {
        let mut elevator = Elevator::new(1, 10);

        elevator.apply(ElevatorCommand::GoUp(4));
        assert_eq!(elevator.floor(), 5);

        elevator.apply(ElevatorCommand::GoDown(2));
        assert_eq!(elevator.floor(), 3);
    }

    #[test]
    fn go_up_stops_at_max_floor() {
        let mut elevator = Elevator::new(1, 6);

        elevator.apply(ElevatorCommand::GoUp(100));

        assert_eq!(elevator.floor(), 6);
    }

    #[test]
    fn go_down_stops_at_min_floor() {
        let mut elevator = Elevator::new(-2, 10);

        elevator.apply(ElevatorCommand::GoDown(100));

        assert_eq!(elevator.floor(), -2);
    }

    #[test]
    fn move_commands_are_ignored_while_door_is_open() {
        let mut elevator = Elevator::new(1, 10);

        elevator.apply(ElevatorCommand::OpenDoor);
        elevator.apply(ElevatorCommand::GoUp(3));
        elevator.apply(ElevatorCommand::GoDown(1));

        assert_eq!(elevator.floor(), 1);
        assert_eq!(elevator.door(), DoorState::Open);
    }

    #[test]
    fn elevator_moves_again_after_door_closes() {
        let mut elevator = Elevator::new(1, 10);

        elevator.apply(ElevatorCommand::OpenDoor);
        elevator.apply(ElevatorCommand::GoUp(3));
        elevator.apply(ElevatorCommand::CloseDoor);
        elevator.apply(ElevatorCommand::GoUp(3));

        assert_eq!(elevator.floor(), 4);
        assert_eq!(elevator.door(), DoorState::Closed);
    }

    #[test]
    fn move_with_negative_distance_is_ignored() {
        let mut elevator = Elevator::new(1, 10);

        elevator.apply(ElevatorCommand::GoUp(2));
        elevator.apply(ElevatorCommand::GoUp(-7));
        elevator.apply(ElevatorCommand::GoDown(-7));

        assert_eq!(elevator.floor(), 3);
    }

    #[test]
    fn opening_and_closing_the_door_only_changes_door_state() {
        let mut elevator = Elevator::new(1, 10);

        elevator.apply(ElevatorCommand::GoUp(2));
        elevator.apply(ElevatorCommand::OpenDoor);
        elevator.apply(ElevatorCommand::OpenDoor);
        assert_eq!(elevator.door(), DoorState::Open);
        assert_eq!(elevator.floor(), 3);

        elevator.apply(ElevatorCommand::CloseDoor);
        elevator.apply(ElevatorCommand::CloseDoor);
        assert_eq!(elevator.door(), DoorState::Closed);
        assert_eq!(elevator.floor(), 3);
    }
}
