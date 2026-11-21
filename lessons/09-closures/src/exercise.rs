//! 主练习：事件处理管线。
//!
//! 一个监控程序不断收到事件：每条事件有一个编号和一个整数值。程序先用
//! 过滤器决定“这条事件要不要处理”，再让处理器把要处理的事件变成一行
//! 文本。不同任务需要不同的过滤条件和不同的文本格式，所以这两段逻辑都
//! 由调用者以闭包的形式传进来，而不是写死在函数里。
//!
//! 请只替换函数体里的 `todo!()`。不要修改类型定义和函数签名，也不要
//! 改动本文件底部的测试。每个函数上方的注释就是它的完整行为约定。

/// 一条待处理的事件。
///
/// - `id` 是事件编号，用来在结果或错误里指出“是哪一条”；
/// - `value` 是这条事件的整数值，过滤器通常根据它做判断。
///
/// 两个字段都是公开的，方便 `main` 和测试直接构造事件。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event {
    pub id: u64,
    pub value: i32,
}

/// 用 `operation` 处理 `values` 中的每个整数，并把结果写回原位置。
///
/// 行为约定：
///
/// - 从第一个元素到最后一个元素依次处理；
/// - 每个位置的新值是 `operation(旧值)`；
/// - 直接修改 `values`，不返回新集合；
/// - `values` 为空时什么也不做；
/// - `operation` 可以被调用多次，也可以携带自己的可变状态，例如统计
///   自己被调用了几次。
pub fn apply_all(values: &mut [i32], mut operation: impl FnMut(i32) -> i32) {
    // 完成实现前避免“参数未使用”的提示，写好函数体后可删除这两行。
    let _ = &values;
    let _ = &mut operation;
    todo!()
}

/// 返回第一个满足 `predicate` 的元素：它的下标和值。
///
/// 下标从 0 开始，表示这个元素在 `values` 中的位置。
///
/// 行为约定：
///
/// - 从第一个元素开始逐个检查，顺序不能改变；
/// - 某个元素让 `predicate` 返回 `true` 时，立即返回
///   `Some((下标, 该元素的值))`；
/// - 没有任何元素满足条件时返回 `None`；
/// - `values` 为空时返回 `None`；
/// - 返回的是复制出来的整数，不借用输入切片。
pub fn first_match(values: &[i32], predicate: impl Fn(i32) -> bool) -> Option<(usize, i32)> {
    // 完成实现前避免“参数未使用”的提示，写好函数体后可删除这两行。
    let _ = &values;
    let _ = &predicate;
    todo!()
}

/// 执行 `operation` 一次，并返回它产生的文本。
///
/// 参数只要求“能被调用一次”，所以传入的闭包可以消耗自己捕获的数据：
/// 把捕获的 `String` 作为返回值交出来、按值传给别的函数、或者 `drop`
/// 掉，都是允许的。函数本身只调用它一次，不会尝试再次调用。
pub fn run_once(operation: impl FnOnce() -> String) -> String {
    // 完成实现前避免“参数未使用”的提示，写好函数体后可删除这一行。
    let _ = &operation;
    todo!()
}

/// 按顺序处理事件，收集处理器返回的文本；遇到第一个错误就停止。
///
/// 行为约定：
///
/// - 按 `events` 的原有顺序逐条处理；
/// - 先用 `filter` 判断：返回 `false` 的事件被跳过，**不会**调用
///   `handler`；
/// - `filter` 返回 `true` 时调用 `handler`：返回 `Ok(text)` 就把 `text`
///   追加到结果末尾；返回 `Err(error)` 就立即结束，把该错误原样返回；
/// - 出错时，本次调用中已经收集到的文本会被丢弃，调用者只看到错误；
/// - `events` 为空时，两个闭包都不调用，返回 `Ok(空 Vec)`；
/// - 全部事件都被过滤掉时，也返回 `Ok(空 Vec)`；
/// - 两个闭包都可以携带自己的可变状态，例如分别统计调用次数；
/// - 闭包只借用事件，不需要取得 `Event` 的所有权。
pub fn process_events<E>(
    events: &[Event],
    mut filter: impl FnMut(&Event) -> bool,
    mut handler: impl FnMut(&Event) -> Result<String, E>,
) -> Result<Vec<String>, E> {
    // 完成实现前避免“参数未使用”的提示，写好函数体后可删除这三行。
    let _ = &events;
    let _ = &mut filter;
    let _ = &mut handler;
    todo!()
}

/// 创建一个只保留“数值达到阈值”的事件的过滤器。
///
/// 返回的闭包记住传入的 `threshold`，之后可以反复调用：
///
/// - `event.value` **大于或等于** `threshold` 时返回 `true`，否则返回 `false`。
pub fn make_threshold_filter(threshold: i32) -> impl Fn(&Event) -> bool {
    // 完成实现前避免“参数未使用”的提示；把整个闭包表达式替换成实现。
    let _ = &threshold;
    |_| todo!()
}

#[cfg(test)]
mod tests {
    use super::{Event, apply_all, first_match, make_threshold_filter, process_events, run_once};

    #[derive(Debug, PartialEq, Eq)]
    enum HandlerError {
        Rejected(u64),
    }

    fn sample_events() -> Vec<Event> {
        vec![
            Event { id: 1, value: 5 },
            Event { id: 2, value: 10 },
            Event { id: 3, value: 15 },
        ]
    }

    #[test]
    fn apply_all_replaces_each_value_in_place() {
        let mut values = [1, 2, 3];

        apply_all(&mut values, |value| value * 10);

        assert_eq!(values, [10, 20, 30]);
    }

    #[test]
    fn apply_all_allows_the_operation_to_keep_state() {
        let mut values = [1, 2, 3];
        let mut calls = 0;

        apply_all(&mut values, |value| {
            calls += 1;
            value + calls
        });

        assert_eq!(values, [2, 4, 6]);
        assert_eq!(calls, 3);
    }

    #[test]
    fn apply_all_accepts_an_empty_slice() {
        let mut values: [i32; 0] = [];
        let mut calls = 0;

        apply_all(&mut values, |value| {
            calls += 1;
            value
        });

        assert_eq!(values, []);
        assert_eq!(calls, 0);
    }

    #[test]
    fn first_match_returns_the_position_and_value_of_the_first_match() {
        let threshold = 8;

        assert_eq!(
            first_match(&[3, 9, 10], |value| value > threshold),
            Some((1, 9))
        );
    }

    #[test]
    fn first_match_returns_none_when_nothing_matches() {
        assert_eq!(first_match(&[1, 3, 5], |value| value % 2 == 0), None);
    }

    #[test]
    fn first_match_returns_none_for_empty_input() {
        assert_eq!(first_match(&[], |_| true), None);
    }

    #[test]
    fn run_once_accepts_a_closure_that_consumes_owned_data() {
        let payload = String::from("finished");

        let returned = run_once(move || payload);

        assert_eq!(returned, "finished");
    }

    #[test]
    fn process_events_preserves_order_and_skips_filtered_events() {
        let events = sample_events();
        let mut handled_ids = Vec::new();

        let result: Result<Vec<String>, HandlerError> = process_events(
            &events,
            |event| event.value >= 10,
            |event| {
                handled_ids.push(event.id);
                Ok(format!("{}:{}", event.id, event.value))
            },
        );

        assert_eq!(result, Ok(vec![String::from("2:10"), String::from("3:15")]));
        assert_eq!(handled_ids, [2, 3]);
    }

    #[test]
    fn process_events_empty_input_calls_no_callbacks() {
        let mut filter_calls = 0;
        let mut handler_calls = 0;

        let result: Result<Vec<String>, HandlerError> = process_events(
            &[],
            |_| {
                filter_calls += 1;
                true
            },
            |_| {
                handler_calls += 1;
                Ok(String::new())
            },
        );

        assert_eq!(result, Ok(Vec::new()));
        assert_eq!(filter_calls, 0);
        assert_eq!(handler_calls, 0);
    }

    #[test]
    fn process_events_returns_empty_output_when_everything_is_filtered() {
        let events = sample_events();
        let mut handler_calls = 0;

        let result: Result<Vec<String>, HandlerError> = process_events(
            &events,
            |_| false,
            |_| {
                handler_calls += 1;
                Ok(String::from("unreachable"))
            },
        );

        assert_eq!(result, Ok(Vec::new()));
        assert_eq!(handler_calls, 0);
    }

    #[test]
    fn process_events_stops_at_the_first_handler_error() {
        let events = sample_events();
        let mut handled_ids = Vec::new();

        let result: Result<Vec<String>, HandlerError> = process_events(
            &events,
            |_| true,
            |event| {
                handled_ids.push(event.id);
                if event.id == 2 {
                    Err(HandlerError::Rejected(event.id))
                } else {
                    Ok(format!("accepted-{}", event.id))
                }
            },
        );

        assert_eq!(result, Err(HandlerError::Rejected(2)));
        assert_eq!(handled_ids, [1, 2]);
    }

    #[test]
    fn process_events_lets_both_callbacks_keep_state() {
        let events = sample_events();
        let mut filter_calls = 0;
        let mut handler_calls = 0;

        let result: Result<Vec<String>, HandlerError> = process_events(
            &events,
            |event| {
                filter_calls += 1;
                event.value > 0
            },
            |event| {
                handler_calls += 1;
                Ok(format!("event-{}", event.id))
            },
        );

        assert_eq!(
            result,
            Ok(vec![
                String::from("event-1"),
                String::from("event-2"),
                String::from("event-3"),
            ])
        );
        assert_eq!(filter_calls, 3);
        assert_eq!(handler_calls, 3);
    }

    #[test]
    fn threshold_filter_is_inclusive() {
        let filter = make_threshold_filter(10);

        assert!(filter(&Event { id: 1, value: 10 }));
        assert!(!filter(&Event { id: 2, value: 9 }));
        assert!(filter(&Event { id: 3, value: 11 }));
    }

    #[test]
    fn threshold_filter_can_be_called_repeatedly() {
        let filter = make_threshold_filter(0);
        let values = [-1, 0, 1];

        let accepted: Vec<bool> = values
            .iter()
            .map(|value| {
                filter(&Event {
                    id: 0,
                    value: *value,
                })
            })
            .collect();

        assert_eq!(accepted, [false, true, true]);
    }

    #[test]
    fn move_closure_that_only_reads_owned_text_can_run_twice() {
        let text = String::from("owned");
        let length = move || text.len();

        assert_eq!(length(), 5);
        assert_eq!(length(), 5);
    }
}
