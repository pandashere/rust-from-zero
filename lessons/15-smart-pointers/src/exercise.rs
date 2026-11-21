//! 主练习：共享组织树。
//!
//! 组织里的每个部门都是一个 `Unit`。程序外部可能持有任意部门的句柄（可以
//! 理解为“指向该部门的一张通行证”），所以一个部门不能只由它的父部门独占。
//! 同时，每个部门要能向上找到自己的父部门，父部门要能按加入顺序列出直接
//! 子部门。
//!
//! 请只替换各方法体里的 `todo!()`，不要修改类型定义、方法签名，也不要改
//! 动本文件底部的测试。每个方法上方的注释就是它的完整行为约定。

use std::cell::RefCell;
use std::rc::{Rc, Weak};

/// 建立父子链接时可能出现的错误。
///
/// 三个错误都表示“这次链接被拒绝”，调用者可以根据具体类别决定如何向
/// 用户解释。失败时不能改动已有的父子链接。
#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq)]
pub enum LinkError {
    /// 子部门已经有另一个仍然存在的父部门。
    AlreadyHasParent,
    /// 这个子部门已经是目标父部门的直接子部门。
    DuplicateChild,
    /// 这次链接会让某个部门成为自己的祖先，形成环。
    WouldCreateCycle,
}

/// 组织树中的一个部门。
///
/// 字段只在方法实现完成后才会被读取，因此先对这个类型单独放行
/// `dead_code` 检查；完成实现后可以删除这个属性。
#[allow(dead_code)]
#[derive(Debug)]
pub struct Unit {
    name: String,
    parent: RefCell<Weak<Unit>>,
    children: RefCell<Vec<Rc<Unit>>>,
}

impl Unit {
    /// 创建一个名为 `name` 的独立部门。
    ///
    /// 行为约定：
    ///
    /// - 返回的 `Rc<Unit>` 是调用者拿到的唯一强引用；
    /// - 新部门没有父部门，也没有子部门；
    /// - `name` 可以是任何能转换成拥有型 `String` 的文本，例如 `&str`。
    pub fn new(name: impl Into<String>) -> Rc<Self> {
        // 完成实现前避免“参数未使用”的提示，写好方法体后可删除。
        let _ = name;
        todo!()
    }

    /// 返回部门的名称（只读借用）。
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 把 `child` 挂到 `parent` 下面，成为它的直接子部门。
    ///
    /// 判断按以下优先级进行，命中第一条就返回对应错误：
    ///
    /// 1. `child` 已经是 `parent` 的直接子部门：`DuplicateChild`；
    /// 2. `parent` 就是 `child` 本身，或者 `parent` 位于 `child` 的子树中：
    ///    `WouldCreateCycle`；
    /// 3. `child` 已经有一个仍然存活的父部门：`AlreadyHasParent`。
    ///
    /// 成功时：
    ///
    /// - `child` 的父指针指向 `parent`（这是一条不保活的弱链接）；
    /// - `child` 被追加到 `parent` 子列表的末尾，已有子部门的顺序不变。
    ///
    /// 失败时不能新增、删除或改写任何已有的父子链接。父部门已经销毁的
    /// 子部门可以重新挂到新父部门下；名称相同但身份不同的部门不算重复。
    pub fn add_child(parent: &Rc<Self>, child: Rc<Self>) -> Result<(), LinkError> {
        // 完成实现前避免“参数未使用”的提示，写好方法体后可删除。
        let _ = (parent, child);
        todo!()
    }

    /// 返回仍然存活的父部门。
    ///
    /// 没有父部门，或者父部门已经被销毁时返回 `None`；否则返回指向父部门
    /// 的新强引用。拿到返回值后，父部门在本次调用期间一定仍然存活。
    pub fn parent(&self) -> Option<Rc<Unit>> {
        todo!()
    }

    /// 按加入顺序返回所有**直接**子部门的名称副本。
    ///
    /// - 不包含更深层的后代；
    /// - 没有子部门时返回空 `Vec`；
    /// - 返回的是拥有型 `String`，调用者可以保留它们，而不会让部门继续
    ///   存活。
    pub fn child_names(&self) -> Vec<String> {
        todo!()
    }

    /// 按从最高可访问祖先到当前部门的顺序返回名称。
    ///
    /// - 完整相连的树从最顶层根部门开始；
    /// - 某个父部门已经销毁时，路径从父链上仍然存活的最靠近根的部门开始；
    /// - 单独一个部门只返回自己的名称。
    pub fn path_from_root(&self) -> Vec<String> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::{LinkError, Unit};
    use std::rc::Rc;

    fn linked_tree() -> (Rc<Unit>, Rc<Unit>, Rc<Unit>) {
        let root = Unit::new("Engineering");
        let platform = Unit::new("Platform");
        let ada = Unit::new("Ada");

        Unit::add_child(&root, Rc::clone(&platform)).unwrap();
        Unit::add_child(&platform, Rc::clone(&ada)).unwrap();

        (root, platform, ada)
    }

    #[test]
    fn rc_clone_shares_one_allocation() {
        let original = Rc::new(String::from("shared"));
        let copy = Rc::clone(&original);

        assert_eq!(Rc::strong_count(&original), 2);
        assert!(Rc::ptr_eq(&original, &copy));
        assert_eq!(&*copy, "shared");
    }

    #[test]
    fn weak_reference_does_not_keep_value_alive() {
        let owner = Rc::new(7);
        let weak = Rc::downgrade(&owner);

        assert_eq!(weak.upgrade().map(|value| *value), Some(7));

        drop(owner);

        assert!(weak.upgrade().is_none());
    }

    #[test]
    fn new_unit_has_no_parent_and_no_children() {
        let unit = Unit::new("Solo");

        assert_eq!(unit.name(), "Solo");
        assert!(unit.parent().is_none());
        assert_eq!(unit.child_names(), Vec::<String>::new());
        assert_eq!(unit.path_from_root(), ["Solo"]);
    }

    #[test]
    fn add_child_sets_both_directions() {
        let root = Unit::new("Engineering");
        let child = Unit::new("Platform");

        assert_eq!(Unit::add_child(&root, Rc::clone(&child)), Ok(()));
        assert_eq!(
            child.parent().map(|parent| parent.name().to_owned()),
            Some(String::from("Engineering"))
        );
        assert_eq!(root.child_names(), ["Platform"]);
        assert_eq!(child.path_from_root(), ["Engineering", "Platform"]);
    }

    #[test]
    fn child_names_keep_insertion_order() {
        let root = Unit::new("Engineering");
        let beta = Unit::new("Beta");
        let alpha = Unit::new("Alpha");
        let gamma = Unit::new("Gamma");

        Unit::add_child(&root, Rc::clone(&beta)).unwrap();
        Unit::add_child(&root, Rc::clone(&alpha)).unwrap();
        Unit::add_child(&root, Rc::clone(&gamma)).unwrap();

        assert_eq!(root.child_names(), ["Beta", "Alpha", "Gamma"]);
    }

    #[test]
    fn path_from_root_lists_each_ancestor() {
        let (_root, _platform, ada) = linked_tree();

        assert_eq!(ada.path_from_root(), ["Engineering", "Platform", "Ada"]);
    }

    #[test]
    fn parent_owns_child_while_external_handle_exists() {
        let (root, platform, ada) = linked_tree();

        // Platform 的强引用来自：外部变量 platform、root 的子列表。
        assert_eq!(Rc::strong_count(&platform), 2);
        // Platform 的弱引用来自：Ada 的父指针。
        assert_eq!(Rc::weak_count(&platform), 1);
        // Engineering 的弱引用来自：Platform 的父指针。
        assert_eq!(Rc::weak_count(&root), 1);
        // Ada 的强引用来自：外部变量 ada、platform 的子列表。
        assert_eq!(Rc::strong_count(&ada), 2);
    }

    #[test]
    fn duplicate_child_is_rejected_without_changing_graph() {
        let (root, platform, _ada) = linked_tree();

        assert_eq!(
            Unit::add_child(&root, Rc::clone(&platform)),
            Err(LinkError::DuplicateChild)
        );
        assert_eq!(root.child_names(), ["Platform"]);
        assert_eq!(
            platform.parent().map(|parent| parent.name().to_owned()),
            Some(String::from("Engineering"))
        );
    }

    #[test]
    fn child_with_live_parent_is_rejected() {
        let (_root, platform, _ada) = linked_tree();
        let research = Unit::new("Research");

        assert_eq!(
            Unit::add_child(&research, Rc::clone(&platform)),
            Err(LinkError::AlreadyHasParent)
        );
        assert_eq!(research.child_names(), Vec::<String>::new());
        assert_eq!(
            platform.parent().map(|parent| parent.name().to_owned()),
            Some(String::from("Engineering"))
        );
    }

    #[test]
    fn adding_ancestor_below_descendant_is_rejected() {
        let (root, _platform, ada) = linked_tree();

        assert_eq!(
            Unit::add_child(&ada, Rc::clone(&root)),
            Err(LinkError::WouldCreateCycle)
        );
        assert!(root.parent().is_none());
        assert_eq!(ada.child_names(), Vec::<String>::new());
        assert_eq!(root.path_from_root(), ["Engineering"]);
    }

    #[test]
    fn adding_node_to_itself_is_rejected() {
        let (_root, _platform, ada) = linked_tree();

        assert_eq!(
            Unit::add_child(&ada, Rc::clone(&ada)),
            Err(LinkError::WouldCreateCycle)
        );
        assert_eq!(ada.child_names(), Vec::<String>::new());
        assert_eq!(ada.path_from_root(), ["Engineering", "Platform", "Ada"]);
    }

    #[test]
    fn same_name_different_identity_is_not_duplicate() {
        let root = Unit::new("Engineering");
        let first = Unit::new("Ada");
        let second = Unit::new("Ada");

        assert!(!Rc::ptr_eq(&first, &second));
        assert_eq!(Unit::add_child(&root, Rc::clone(&first)), Ok(()));
        assert_eq!(Unit::add_child(&root, Rc::clone(&second)), Ok(()));
        assert_eq!(root.child_names(), ["Ada", "Ada"]);
    }

    #[test]
    fn dropping_root_turns_parent_link_into_none() {
        let (root, platform, ada) = linked_tree();

        drop(root);

        assert!(platform.parent().is_none());
        assert_eq!(platform.path_from_root(), ["Platform"]);
        assert_eq!(ada.path_from_root(), ["Platform", "Ada"]);
    }

    #[test]
    fn dropping_intermediate_node_stops_path() {
        let (root, platform, ada) = linked_tree();

        drop(root);
        drop(platform);

        assert!(ada.parent().is_none());
        assert_eq!(ada.path_from_root(), ["Ada"]);
    }

    #[test]
    fn child_can_be_linked_again_after_old_parent_is_dropped() {
        let old_root = Unit::new("Old");
        let child = Unit::new("Platform");
        Unit::add_child(&old_root, Rc::clone(&child)).unwrap();

        drop(old_root);

        let new_root = Unit::new("New");
        assert_eq!(Unit::add_child(&new_root, Rc::clone(&child)), Ok(()));
        assert_eq!(new_root.child_names(), ["Platform"]);
        assert_eq!(child.path_from_root(), ["New", "Platform"]);
    }

    #[test]
    fn deep_chain_cycle_is_rejected() {
        let root = Unit::new("A");
        let second = Unit::new("B");
        let third = Unit::new("C");
        let fourth = Unit::new("D");

        Unit::add_child(&root, Rc::clone(&second)).unwrap();
        Unit::add_child(&second, Rc::clone(&third)).unwrap();
        Unit::add_child(&third, Rc::clone(&fourth)).unwrap();

        assert_eq!(
            Unit::add_child(&fourth, Rc::clone(&root)),
            Err(LinkError::WouldCreateCycle)
        );
        assert!(root.parent().is_none());
        assert_eq!(fourth.child_names(), Vec::<String>::new());
    }
}
