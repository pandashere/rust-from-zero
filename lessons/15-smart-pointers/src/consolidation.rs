//! 课后巩固：只弱引用订阅者的消息频道。
//!
//! 这是主练习之后的独立任务，场景和边界都不同：主练习管理一棵父子树，
//! 这里管理“一个频道”和“若干订阅者”的多对多关系。频道只记录订阅者的
//! 弱引用，因此订阅者不会因为“还在频道里”而无法销毁。
//!
//! 请只替换各函数体里的 `todo!()`，不要修改类型定义、函数签名，也不要改
//! 动本文件底部的测试。每项上方的注释就是它的完整行为约定。
//!
//! 完成后的验收命令：
//!
//! ```bash
//! cargo test -p l15_smart_pointers consolidation
//! ```

use std::cell::RefCell;
use std::rc::{Rc, Weak};

/// 登记订阅时可能出现的错误。
#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq)]
pub enum SubscribeError {
    /// 同一个订阅者（同一个 `Rc` 分配）已经登记过。
    AlreadySubscribed,
    /// 频道里已经有一个仍然存活、名称相同但身份不同的订阅者。
    DuplicateName,
}

/// 消息频道的订阅者。
///
/// 字段只在函数实现完成后才会被读取，因此先对这个类型单独放行
/// `dead_code` 检查；完成实现后可以删除这个属性。
#[allow(dead_code)]
#[derive(Debug)]
pub struct Subscriber {
    name: String,
    inbox: RefCell<Vec<String>>,
}

impl Subscriber {
    /// 创建名为 `name`、收件箱为空的订阅者。
    ///
    /// 返回的 `Rc<Subscriber>` 是调用者拿到的唯一强引用。
    pub fn new(name: impl Into<String>) -> Rc<Self> {
        // 完成实现前避免“参数未使用”的提示，写好方法体后可删除。
        let _ = name;
        todo!()
    }

    /// 返回订阅者的名称（只读借用）。
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 按收到的先后顺序返回收件箱里所有消息的副本。
    pub fn inbox(&self) -> Vec<String> {
        todo!()
    }
}

/// 一个消息频道。
///
/// 字段只在函数实现完成后才会被读取，因此先对这个类型单独放行
/// `dead_code` 检查；完成实现后可以删除这个属性。
#[allow(dead_code)]
#[derive(Debug)]
pub struct Channel {
    name: String,
    subscribers: RefCell<Vec<Weak<Subscriber>>>,
}

impl Channel {
    /// 创建一个空的频道。
    ///
    /// 返回的 `Rc<Channel>` 是调用者拿到的唯一强引用。
    pub fn new(name: impl Into<String>) -> Rc<Self> {
        // 完成实现前避免“参数未使用”的提示，写好方法体后可删除。
        let _ = name;
        todo!()
    }

    /// 返回频道名称（只读借用）。
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// 把 `subscriber` 登记到 `channel` 的订阅者列表末尾。
///
/// 检查按以下优先级进行，命中第一条就返回对应错误：
///
/// 1. 同一个 `Rc<Subscriber>` 分配已经登记：`AlreadySubscribed`；
/// 2. 已经登记并仍然存活、名称相同但身份不同的订阅者存在：
///    `DuplicateName`。
///
/// 已经销毁的订阅者留下的登记不算“已登记”，也不会阻止同名新订阅者加入。
/// 成功时频道只保存一个指向订阅者的弱引用；失败时频道保持原样。
pub fn subscribe(channel: &Rc<Channel>, subscriber: &Rc<Subscriber>) -> Result<(), SubscribeError> {
    // 完成实现前避免“参数未使用”的提示，写好方法体后可删除。
    let _ = (channel, subscriber);
    todo!()
}

/// 把 `message` 的一份副本放入每个仍然存活的订阅者收件箱。
///
/// - 按登记顺序投递；
/// - 返回实际收到消息的订阅者数量；
/// - 没有任何存活订阅者时返回 `0`，不报错；
/// - 已经销毁的订阅者不会收到消息，也不计入返回值；
/// - 同一条消息在不同订阅者的收件箱里是各自独立的 `String` 副本，
///   投递后修改传入文本不会影响已投递的消息。
pub fn publish(channel: &Rc<Channel>, message: &str) -> usize {
    // 完成实现前避免“参数未使用”的提示，写好方法体后可删除。
    let _ = (channel, message);
    todo!()
}

/// 返回频道中仍然存活的订阅者数量。
///
/// 已经销毁的订阅者不计入，即使它的弱引用登记还在列表中。
pub fn subscriber_count(channel: &Rc<Channel>) -> usize {
    // 完成实现前避免“参数未使用”的提示，写好函数体后可删除。
    let _ = channel;
    todo!()
}

/// 按登记顺序返回仍然存活的订阅者名称副本。
///
/// 已销毁订阅者会被跳过；返回的是拥有型 `String`，不会让订阅者继续存活。
pub fn subscriber_names(channel: &Rc<Channel>) -> Vec<String> {
    // 完成实现前避免“参数未使用”的提示，写好函数体后可删除。
    let _ = channel;
    todo!()
}

#[cfg(test)]
mod tests {
    use super::{
        Channel, SubscribeError, Subscriber, publish, subscribe, subscriber_count, subscriber_names,
    };
    use std::rc::Rc;

    #[test]
    fn new_subscriber_has_empty_inbox() {
        let subscriber = Subscriber::new("Ada");

        assert_eq!(subscriber.name(), "Ada");
        assert_eq!(subscriber.inbox(), Vec::<String>::new());
    }

    #[test]
    fn subscribe_makes_subscriber_visible() {
        let channel = Channel::new("releases");
        let subscriber = Subscriber::new("Ada");

        assert_eq!(subscribe(&channel, &subscriber), Ok(()));
        assert_eq!(channel.name(), "releases");
        assert_eq!(subscriber_count(&channel), 1);
        assert_eq!(subscriber_names(&channel), ["Ada"]);
    }

    #[test]
    fn subscribing_the_same_rc_twice_is_rejected() {
        let channel = Channel::new("releases");
        let subscriber = Subscriber::new("Ada");

        subscribe(&channel, &subscriber).unwrap();

        assert_eq!(
            subscribe(&channel, &subscriber),
            Err(SubscribeError::AlreadySubscribed)
        );
        assert_eq!(subscriber_count(&channel), 1);
        assert_eq!(subscriber_names(&channel), ["Ada"]);
    }

    #[test]
    fn different_subscriber_with_same_name_is_rejected() {
        let channel = Channel::new("releases");
        let first = Subscriber::new("Ada");
        let second = Subscriber::new("Ada");

        assert!(!Rc::ptr_eq(&first, &second));
        subscribe(&channel, &first).unwrap();

        assert_eq!(
            subscribe(&channel, &second),
            Err(SubscribeError::DuplicateName)
        );
        assert_eq!(subscriber_count(&channel), 1);
        assert_eq!(subscriber_names(&channel), ["Ada"]);
        assert_eq!(publish(&channel, "release"), 1);
        assert_eq!(second.inbox(), Vec::<String>::new());
    }

    #[test]
    fn publish_delivers_to_all_live_subscribers_in_order() {
        let channel = Channel::new("releases");
        let ada = Subscriber::new("Ada");
        let lin = Subscriber::new("Lin");

        subscribe(&channel, &ada).unwrap();
        subscribe(&channel, &lin).unwrap();

        assert_eq!(publish(&channel, "release"), 2);
        assert_eq!(ada.inbox(), ["release"]);
        assert_eq!(lin.inbox(), ["release"]);
    }

    #[test]
    fn publish_appends_successive_messages() {
        let channel = Channel::new("releases");
        let ada = Subscriber::new("Ada");

        subscribe(&channel, &ada).unwrap();
        publish(&channel, "first");
        publish(&channel, "second");

        assert_eq!(ada.inbox(), ["first", "second"]);
    }

    #[test]
    fn publish_ignores_and_does_not_count_dropped_subscribers() {
        let channel = Channel::new("releases");
        let ada = Subscriber::new("Ada");
        let lin = Subscriber::new("Lin");

        subscribe(&channel, &ada).unwrap();
        subscribe(&channel, &lin).unwrap();
        drop(lin);

        assert_eq!(subscriber_count(&channel), 1);
        assert_eq!(subscriber_names(&channel), ["Ada"]);
        assert_eq!(publish(&channel, "still here"), 1);
        assert_eq!(ada.inbox(), ["still here"]);
    }

    #[test]
    fn publish_to_empty_channel_delivers_nothing() {
        let channel = Channel::new("releases");

        assert_eq!(subscriber_count(&channel), 0);
        assert_eq!(subscriber_names(&channel), Vec::<String>::new());
        assert_eq!(publish(&channel, "nobody"), 0);
    }

    #[test]
    fn same_name_can_join_after_previous_holder_is_dropped() {
        let channel = Channel::new("releases");
        let first = Subscriber::new("Ada");
        subscribe(&channel, &first).unwrap();
        drop(first);

        let second = Subscriber::new("Ada");
        assert_eq!(subscribe(&channel, &second), Ok(()));
        assert_eq!(subscriber_count(&channel), 1);
        assert_eq!(subscriber_names(&channel), ["Ada"]);
        assert_eq!(publish(&channel, "welcome"), 1);
        assert_eq!(second.inbox(), ["welcome"]);
    }

    #[test]
    fn publish_copies_the_message_for_each_subscriber() {
        let channel = Channel::new("releases");
        let ada = Subscriber::new("Ada");
        let lin = Subscriber::new("Lin");
        let mut message = String::from("hello");

        subscribe(&channel, &ada).unwrap();
        subscribe(&channel, &lin).unwrap();
        assert_eq!(publish(&channel, &message), 2);

        message.push('!');

        assert_eq!(ada.inbox(), ["hello"]);
        assert_eq!(lin.inbox(), ["hello"]);
    }

    #[test]
    fn channel_does_not_keep_subscriber_alive() {
        let channel = Channel::new("releases");
        let subscriber = Subscriber::new("Ada");

        subscribe(&channel, &subscriber).unwrap();

        // 频道只保存弱引用，所以强引用仍然只有外部变量这一个。
        assert_eq!(Rc::strong_count(&subscriber), 1);
        assert_eq!(Rc::weak_count(&subscriber), 1);

        drop(subscriber);

        assert_eq!(subscriber_count(&channel), 0);
        assert_eq!(subscriber_names(&channel), Vec::<String>::new());
    }

    #[test]
    fn failed_subscription_leaves_channel_unchanged() {
        let channel = Channel::new("releases");
        let first = Subscriber::new("Ada");
        let second = Subscriber::new("Ada");

        subscribe(&channel, &first).unwrap();
        let _ = subscribe(&channel, &second);

        assert_eq!(subscriber_count(&channel), 1);
        assert_eq!(subscriber_names(&channel), ["Ada"]);
        assert_eq!(publish(&channel, "hello"), 1);
        assert_eq!(first.inbox(), ["hello"]);
        assert_eq!(second.inbox(), Vec::<String>::new());
    }
}
