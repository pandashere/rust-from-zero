//! 第十五课的演示入口：用共享组织树展示 `Rc`、`Weak` 与 `RefCell` 的配合。
//!
//! 在完成 `src/exercise.rs` 中的 `todo!()` 之前，运行本程序会在第一次
//! 创建部门时中止（panic）——这是预期现象。先完成主练习，再回来看输出。

use l15_smart_pointers::exercise::{LinkError, Unit};
use std::rc::Rc;

fn main() {
    let root = Unit::new("Engineering");
    let platform = Unit::new("Platform");
    let ada = Unit::new("Ada");

    Unit::add_child(&root, Rc::clone(&platform)).expect("Platform 应能挂到 Engineering 下");
    Unit::add_child(&platform, Rc::clone(&ada)).expect("Ada 应能挂到 Platform 下");

    println!("根部门: {}", root.name());
    println!("Engineering 的直接子部门: {:?}", root.child_names());
    println!("Platform 的直接子部门: {:?}", platform.child_names());
    println!("Ada 从根到自己的路径: {:?}", ada.path_from_root());

    let duplicate = Unit::add_child(&root, Rc::clone(&platform));
    assert_eq!(duplicate, Err(LinkError::DuplicateChild));
    println!("重复挂载 Platform: {duplicate:?}");

    let research = Unit::new("Research");
    let already_has_parent = Unit::add_child(&research, Rc::clone(&platform));
    assert_eq!(already_has_parent, Err(LinkError::AlreadyHasParent));
    println!("把 Platform 改挂到 Research 下: {already_has_parent:?}");

    let cycle = Unit::add_child(&ada, Rc::clone(&root));
    assert_eq!(cycle, Err(LinkError::WouldCreateCycle));
    println!("把 Engineering 挂到 Ada 下: {cycle:?}");
}
