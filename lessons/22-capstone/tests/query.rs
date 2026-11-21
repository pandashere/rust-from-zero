use l22_capstone::model::Record;
use l22_capstone::query::{Query, QueryError, SortKey, execute};

fn record(id: u64, category: &str, value: i64, tags: &[&str], note: &str) -> Record {
    Record {
        id,
        category: category.to_string(),
        value,
        tags: tags.iter().map(|tag| (*tag).to_string()).collect(),
        note: note.to_string(),
    }
}

fn sample() -> Vec<Record> {
    vec![
        record(2, "course", 120, &["rust", "video"], "Ownership course"),
        record(1, "book", 89, &["rust", "systems"], "The Rust Book"),
        record(3, "book", -5, &["draft", "rust"], "Correction"),
        record(4, "book", 89, &["cli"], "Second book"),
    ]
}

fn ids(records: &[&Record]) -> Vec<u64> {
    records.iter().map(|record| record.id).collect()
}

#[test]
fn default_query_sorts_by_id_ascending() {
    let records = sample();
    let selected = execute(&records, &Query::default()).unwrap();

    assert_eq!(ids(&selected), vec![1, 2, 3, 4]);
}

#[test]
fn category_filter_matches_only_that_category() {
    let records = sample();
    let query = Query {
        category: Some("book".to_string()),
        ..Query::default()
    };
    let selected = execute(&records, &query).unwrap();

    assert_eq!(ids(&selected), vec![1, 3, 4]);
}

#[test]
fn category_filter_trims_the_query_value() {
    let records = sample();
    let query = Query {
        category: Some("  course ".to_string()),
        ..Query::default()
    };
    let selected = execute(&records, &query).unwrap();

    assert_eq!(ids(&selected), vec![2]);
}

#[test]
fn tag_filter_is_normalized_before_comparison() {
    let records = sample();
    let query = Query {
        tag: Some("  RUST ".to_string()),
        ..Query::default()
    };
    let selected = execute(&records, &query).unwrap();

    assert_eq!(ids(&selected), vec![1, 2, 3]);
}

#[test]
fn min_and_max_include_their_boundaries() {
    let records = sample();
    let bounded = Query {
        min: Some(-5),
        max: Some(89),
        ..Query::default()
    };

    assert_eq!(ids(&execute(&records, &bounded).unwrap()), vec![1, 3, 4]);

    let only_89 = Query {
        min: Some(89),
        max: Some(89),
        ..Query::default()
    };
    assert_eq!(ids(&execute(&records, &only_89).unwrap()), vec![1, 4]);
}

#[test]
fn contains_matches_notes_case_sensitively() {
    let records = sample();
    let lowercase = Query {
        contains: Some("book".to_string()),
        ..Query::default()
    };
    let uppercase = Query {
        contains: Some("Book".to_string()),
        ..Query::default()
    };

    assert_eq!(ids(&execute(&records, &lowercase).unwrap()), vec![4]);
    assert_eq!(ids(&execute(&records, &uppercase).unwrap()), vec![1]);
}

#[test]
fn filters_combine_with_and() {
    let records = sample();
    let query = Query {
        category: Some("book".to_string()),
        tag: Some("cli".to_string()),
        min: Some(0),
        contains: Some("Second".to_string()),
        ..Query::default()
    };

    assert_eq!(ids(&execute(&records, &query).unwrap()), vec![4]);
}

#[test]
fn invalid_range_is_an_error() {
    let records = sample();
    let query = Query {
        min: Some(100),
        max: Some(0),
        ..Query::default()
    };

    assert_eq!(
        execute(&records, &query),
        Err(QueryError::InvalidRange { min: 100, max: 0 })
    );
}

#[test]
fn value_sort_breaks_ties_by_id_ascending() {
    let records = sample();
    let query = Query {
        sort: SortKey::Value,
        ..Query::default()
    };

    assert_eq!(ids(&execute(&records, &query).unwrap()), vec![3, 1, 4, 2]);
}

#[test]
fn descending_reverses_the_primary_key_but_keeps_id_ties_ascending() {
    let records = sample();
    let query = Query {
        sort: SortKey::Value,
        descending: true,
        ..Query::default()
    };

    assert_eq!(ids(&execute(&records, &query).unwrap()), vec![2, 1, 4, 3]);
}

#[test]
fn category_sort_uses_id_as_the_secondary_key() {
    let records = sample();
    let ascending = Query {
        sort: SortKey::Category,
        ..Query::default()
    };
    let descending = Query {
        sort: SortKey::Category,
        descending: true,
        ..Query::default()
    };

    assert_eq!(
        ids(&execute(&records, &ascending).unwrap()),
        vec![1, 3, 4, 2]
    );
    assert_eq!(
        ids(&execute(&records, &descending).unwrap()),
        vec![2, 1, 3, 4]
    );
}

#[test]
fn id_sort_descending_is_supported() {
    let records = sample();
    let query = Query {
        sort: SortKey::Id,
        descending: true,
        ..Query::default()
    };

    assert_eq!(ids(&execute(&records, &query).unwrap()), vec![4, 3, 2, 1]);
}

#[test]
fn limit_is_applied_after_sorting() {
    let records = sample();
    let query = Query {
        sort: SortKey::Value,
        descending: true,
        limit: Some(2),
        ..Query::default()
    };

    assert_eq!(ids(&execute(&records, &query).unwrap()), vec![2, 1]);
}

#[test]
fn zero_limit_returns_no_records() {
    let records = sample();
    let query = Query {
        limit: Some(0),
        ..Query::default()
    };

    assert!(execute(&records, &query).unwrap().is_empty());
}

#[test]
fn query_without_matches_returns_an_empty_vector() {
    let records = sample();
    let query = Query {
        category: Some("missing".to_string()),
        ..Query::default()
    };

    assert!(execute(&records, &query).unwrap().is_empty());
}

#[test]
fn querying_an_empty_slice_returns_an_empty_vector() {
    assert!(execute(&[], &Query::default()).unwrap().is_empty());
}
