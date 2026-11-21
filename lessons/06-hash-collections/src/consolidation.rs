//! 课后巩固：团购接龙对账。
//!
//! 主练习的标签索引维护“同时拥有”的两张表；这个巩固任务换一组数据和
//! 边界：输入是邻居们在群里接龙的明细行，你要用 HashMap 合并数量、核对
//! 库存，再用 HashSet 找出“每个人都想要”的商品。
//!
//! 请只替换三个函数体里的 `todo!()`，不要修改函数签名，也不要改动本文
//! 件底部的测试。每个函数上方的注释就是它的完整行为约定。
//!
//! 完成后的验收命令：
//!
//! ```bash
//! cargo test -p l06_hash_collections consolidation
//! ```

use std::collections::HashMap;

/// 把接龙明细合并成“每种商品一共订了多少份”。
///
/// 明细的每一行是 `(商品名, 数量)`：同一种商品可能被报了多次（先报
/// 2 份苹果，后来又追加 1 份），同一商品的数量要累加。
///
/// 行为约定：
///
/// - 商品名去掉两端空白后没有任何字符的行、以及数量为 0 的行，都视为
///   误报，直接忽略，不进结果；
/// - 其余行按商品名累加数量；商品名按原样保留，不做大小写转换，`Apple`
///   和 `apple` 是两种商品；
/// - 返回的 HashMap 拥有商品名（key 是 `String`），不再借用输入文本；
/// - 输入为空时返回空的 HashMap。
pub fn tally(lines: &[(&str, u32)]) -> HashMap<String, u32> {
    // 完成实现前避免“参数未使用”的提示，写好函数体后可删除。
    let _ = &lines;
    todo!()
}

/// 核对库存：找出需求总量超过库存的商品。
///
/// `demand` 是合并后的需求表，`stock` 是仓库库存表。库存表里查不到的
/// 商品库存按 0 算——也就是说只要有人订，它就一定缺货。
///
/// 行为约定：
///
/// - 需求量**严格大于**库存量的商品才算缺货，刚好相等不算；
/// - 只检查需求表里的商品：库存有剩但没人订的商品不进结果；
/// - 结果按字典序（字符串的默认大小比较）从小到大排列；
/// - 两个表都只是借用，函数不能修改它们；
/// - 需求表为空时返回空的 `Vec`。
pub fn find_shortages(demand: &HashMap<String, u32>, stock: &HashMap<String, u32>) -> Vec<String> {
    // 完成实现前避免“参数未使用”的提示，写好函数体后可删除。
    let _ = &demand;
    let _ = &stock;
    todo!()
}

/// 找出“每个人都想要”的商品：出现在每一份心愿单里的商品。
///
/// `lists` 是每位邻居的心愿单，每份心愿单是一组商品名。
///
/// 行为约定：
///
/// - 一个商品要出现在**每一份**心愿单里才进结果，只被部分人想要的不算；
/// - 同一份心愿单里重复出现的商品只算一次；
/// - 结果按字典序从小到大排列，且不含重复；
/// - 只要有一份心愿单为空，就没有商品能同时出现在每一份里，结果为空
///   `Vec`；
/// - 一位邻居也没有（空切片）时返回空 `Vec`：“每个人都想要”在没有人时
///   没有意义，这里约定为空。
pub fn everyone_wants(lists: &[&[&str]]) -> Vec<String> {
    // 完成实现前避免“参数未使用”的提示，写好函数体后可删除。
    let _ = &lists;
    todo!()
}

#[cfg(test)]
mod tests {
    use super::{HashMap, everyone_wants, find_shortages, tally};

    fn table(pairs: &[(&str, u32)]) -> HashMap<String, u32> {
        pairs
            .iter()
            .map(|(name, count)| ((*name).to_owned(), *count))
            .collect()
    }

    #[test]
    fn tally_sums_repeated_items() {
        let lines = [("apple", 2), ("milk", 1), ("apple", 3), ("tea", 4)];

        assert_eq!(
            tally(&lines),
            table(&[("apple", 5), ("milk", 1), ("tea", 4)])
        );
    }

    #[test]
    fn tally_ignores_blank_names_and_zero_quantities() {
        let lines = [("   ", 5), ("apple", 0), ("", 2)];

        assert_eq!(tally(&lines), HashMap::new());
    }

    #[test]
    fn tally_keeps_names_exactly_as_written() {
        let lines = [("Apple", 1), ("apple", 2)];

        assert_eq!(tally(&lines), table(&[("Apple", 1), ("apple", 2)]));
    }

    #[test]
    fn tally_accepts_empty_input() {
        assert_eq!(tally(&[]), HashMap::new());
    }

    #[test]
    fn find_shortages_reports_items_with_demand_above_stock() {
        let demand = table(&[("apple", 5), ("milk", 2)]);
        let stock = table(&[("apple", 3), ("milk", 2), ("bread", 10)]);

        // milk 需求与库存相等，不算缺货；bread 没人订，不进结果。
        assert_eq!(find_shortages(&demand, &stock), ["apple".to_owned()]);
    }

    #[test]
    fn find_shortages_treats_missing_stock_as_zero() {
        let demand = table(&[("cake", 1)]);
        let stock = HashMap::new();

        assert_eq!(find_shortages(&demand, &stock), ["cake".to_owned()]);
    }

    #[test]
    fn find_shortages_returns_sorted_names() {
        let demand = table(&[("pear", 2), ("apple", 2), ("milk", 1)]);
        let stock = table(&[("milk", 5)]);

        assert_eq!(
            find_shortages(&demand, &stock),
            ["apple".to_owned(), "pear".to_owned()]
        );
    }

    #[test]
    fn find_shortages_accepts_empty_demand() {
        let stock = table(&[("apple", 1)]);

        assert_eq!(
            find_shortages(&HashMap::new(), &stock),
            Vec::<String>::new()
        );
    }

    #[test]
    fn everyone_wants_returns_items_present_in_every_list() {
        let lists: &[&[&str]] = &[
            &["apple", "tea"],
            &["tea", "apple", "milk"],
            &["milk", "tea", "apple"],
        ];

        assert_eq!(
            everyone_wants(lists),
            ["apple".to_owned(), "tea".to_owned()]
        );
    }

    #[test]
    fn everyone_wants_counts_duplicates_within_one_list_once() {
        let lists: &[&[&str]] = &[&["tea", "tea", "cake"], &["tea"]];

        assert_eq!(everyone_wants(lists), ["tea".to_owned()]);
    }

    #[test]
    fn everyone_wants_with_a_single_list_returns_its_deduplicated_items() {
        let lists: &[&[&str]] = &[&["b", "a", "b"]];

        assert_eq!(everyone_wants(lists), ["a".to_owned(), "b".to_owned()]);
    }

    #[test]
    fn everyone_wants_returns_nothing_when_a_list_is_empty() {
        let lists: &[&[&str]] = &[&["apple", "tea"], &[]];

        assert_eq!(everyone_wants(lists), Vec::<String>::new());
    }

    #[test]
    fn everyone_wants_accepts_no_lists_at_all() {
        assert_eq!(everyone_wants(&[]), Vec::<String>::new());
    }
}
