use l22_capstone::model::Record;
use l22_capstone::output::{render_groups, render_records, render_summary};
use l22_capstone::stats::Summary;

fn record(id: u64, category: &str, value: i64, tags: &[&str], note: &str) -> Record {
    Record {
        id,
        category: category.to_string(),
        value,
        tags: tags.iter().map(|tag| (*tag).to_string()).collect(),
        note: note.to_string(),
    }
}

#[test]
fn render_records_aligns_columns_and_joins_tags_with_commas() {
    let records = [
        record(1, "book", 89, &["rust", "systems"], "The Rust Book"),
        record(2, "course", 120, &[], ""),
        record(3, "book", -5, &["draft"], "Correction"),
    ];
    let selected: Vec<&Record> = records.iter().collect();

    assert_eq!(
        render_records(&selected),
        "ID  CATEGORY  VALUE  TAGS          NOTE\n\
         1   book      89     rust,systems  The Rust Book\n\
         2   course    120    -             -\n\
         3   book      -5     draft         Correction\n"
    );
}

#[test]
fn render_records_with_no_records_prints_only_the_header() {
    assert_eq!(render_records(&[]), "ID  CATEGORY  VALUE  TAGS  NOTE\n");
}

#[test]
fn render_records_uses_dashes_for_empty_tags_and_note() {
    let records = [record(1, "book", 0, &[], "")];
    let selected: Vec<&Record> = records.iter().collect();

    assert_eq!(
        render_records(&selected),
        "ID  CATEGORY  VALUE  TAGS  NOTE\n1   book      0      -     -\n"
    );
}

#[test]
fn render_summary_formats_numbers_and_empty_extremes() {
    let summary = Summary {
        count: 3,
        sum: 204,
        min: Some(-5),
        max: Some(120),
    };

    assert_eq!(
        render_summary(&summary),
        "count: 3\nsum: 204\nmin: -5\nmax: 120\nmean: 68.00\n"
    );
}

#[test]
fn render_summary_uses_dashes_for_an_empty_set() {
    let summary = Summary {
        count: 0,
        sum: 0,
        min: None,
        max: None,
    };

    assert_eq!(
        render_summary(&summary),
        "count: 0\nsum: 0\nmin: -\nmax: -\nmean: -\n"
    );
}

#[test]
fn render_groups_aligns_columns_and_sorts_nothing_itself() {
    let groups = vec![
        (
            "book".to_string(),
            Summary {
                count: 2,
                sum: 84,
                min: Some(-5),
                max: Some(89),
            },
        ),
        (
            "course".to_string(),
            Summary {
                count: 1,
                sum: 120,
                min: Some(120),
                max: Some(120),
            },
        ),
    ];

    assert_eq!(
        render_groups(&groups),
        "CATEGORY  COUNT  SUM  MIN  MAX  MEAN\n\
         book      2      84   -5   89   42.00\n\
         course    1      120  120  120  120.00\n"
    );
}

#[test]
fn render_groups_with_no_groups_prints_only_the_header() {
    assert_eq!(render_groups(&[]), "CATEGORY  COUNT  SUM  MIN  MAX  MEAN\n");
}
