use l22_capstone::model::Record;
use l22_capstone::stats::{Summary, group_by_category};

fn record(id: u64, category: &str, value: i64) -> Record {
    Record {
        id,
        category: category.to_string(),
        value,
        tags: Vec::new(),
        note: String::new(),
    }
}

#[test]
fn empty_summary_has_zero_count_and_no_extremes() {
    let summary = Summary::summarize(&[]);

    assert_eq!(
        summary,
        Summary {
            count: 0,
            sum: 0,
            min: None,
            max: None,
        }
    );
    assert_eq!(summary.mean(), None);
}

#[test]
fn single_record_summary_repeats_the_value() {
    let records = [record(1, "book", -7)];
    let summary = Summary::summarize(&records);

    assert_eq!(summary.count, 1);
    assert_eq!(summary.sum, -7);
    assert_eq!(summary.min, Some(-7));
    assert_eq!(summary.max, Some(-7));
    assert_eq!(summary.mean(), Some(-7.0));
}

#[test]
fn mixed_summary_tracks_negative_zero_and_positive_values() {
    let records = [
        record(1, "book", -5),
        record(2, "book", 0),
        record(3, "course", 120),
    ];
    let summary = Summary::summarize(&records);

    assert_eq!(summary.count, 3);
    assert_eq!(summary.sum, 115);
    assert_eq!(summary.min, Some(-5));
    assert_eq!(summary.max, Some(120));
    assert!((summary.mean().unwrap() - 115.0 / 3.0).abs() < 1e-12);
}

#[test]
fn sum_uses_i128_for_large_i64_values() {
    let records = [record(1, "book", i64::MAX), record(2, "book", i64::MAX)];
    let summary = Summary::summarize(&records);

    assert_eq!(summary.sum, i128::from(i64::MAX) * 2);
    assert_eq!(summary.min, Some(i64::MAX));
    assert_eq!(summary.max, Some(i64::MAX));
}

#[test]
fn mean_of_positive_values_is_exact_for_even_sums() {
    let records = [
        record(1, "book", 10),
        record(2, "book", 20),
        record(3, "book", 30),
    ];
    let summary = Summary::summarize(&records);

    assert_eq!(summary.mean(), Some(20.0));
}

#[test]
fn group_by_category_on_empty_input_is_empty() {
    assert!(group_by_category(&[]).is_empty());
}

#[test]
fn group_by_category_sorts_groups_and_aggregates_each_one() {
    let records = [
        record(1, "book", 89),
        record(2, "course", 120),
        record(3, "book", -5),
    ];
    let groups = group_by_category(&records);

    assert_eq!(groups.len(), 2);
    assert_eq!(groups[0].0, "book");
    assert_eq!(
        groups[0].1,
        Summary {
            count: 2,
            sum: 84,
            min: Some(-5),
            max: Some(89),
        }
    );
    assert_eq!(groups[1].0, "course");
    assert_eq!(
        groups[1].1,
        Summary {
            count: 1,
            sum: 120,
            min: Some(120),
            max: Some(120),
        }
    );
}

#[test]
fn every_record_is_counted_exactly_once_across_groups() {
    let records = [
        record(1, "a", 1),
        record(2, "a", 2),
        record(3, "b", 3),
        record(4, "c", 4),
    ];
    let groups = group_by_category(&records);
    let total: u64 = groups.iter().map(|(_, summary)| summary.count).sum();

    assert_eq!(total, records.len() as u64);
    let names: Vec<&str> = groups.iter().map(|(name, _)| name.as_str()).collect();
    assert_eq!(names, vec!["a", "b", "c"]);
}

#[test]
fn group_by_category_keeps_zero_summaries_distinct_from_empty() {
    let records = [record(1, "book", 0), record(2, "book", 0)];
    let groups = group_by_category(&records);

    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].1.count, 2);
    assert_eq!(groups[0].1.sum, 0);
    assert_eq!(groups[0].1.mean(), Some(0.0));
}
