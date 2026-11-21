use l22_capstone::model::{ParseRecordError, Record, ValidationError};

fn forbidden(field: &'static str, character: char) -> ValidationError {
    ValidationError::ForbiddenCharacter { field, character }
}

#[test]
fn try_new_trims_category_and_preserves_note() {
    let record = Record::try_new(
        7,
        "  Book  ".to_string(),
        -3,
        vec!["  Rust ".to_string(), "SYSTEMS".to_string()],
        "  keep  ".to_string(),
    )
    .unwrap();

    assert_eq!(record.id, 7);
    assert_eq!(record.category, "Book");
    assert_eq!(record.value, -3);
    assert_eq!(record.tags, vec!["rust".to_string(), "systems".to_string()]);
    assert_eq!(record.note, "  keep  ");
}

#[test]
fn try_new_rejects_category_that_is_blank() {
    assert_eq!(
        Record::try_new(1, "   ".to_string(), 0, Vec::new(), String::new()),
        Err(ValidationError::EmptyCategory)
    );
}

#[test]
fn try_new_normalizes_tags_by_trimming_lowercasing_deduplicating_and_sorting() {
    let record = Record::try_new(
        1,
        "book".to_string(),
        0,
        vec![
            "b".to_string(),
            " A ".to_string(),
            "a".to_string(),
            String::new(),
            "C".to_string(),
        ],
        String::new(),
    )
    .unwrap();

    assert_eq!(
        record.tags,
        vec!["a".to_string(), "b".to_string(), "c".to_string()]
    );
}

#[test]
fn try_new_rejects_pipe_in_category() {
    assert_eq!(
        Record::try_new(1, "a|b".to_string(), 0, Vec::new(), String::new()),
        Err(forbidden("category", '|'))
    );
}

#[test]
fn try_new_rejects_line_breaks_and_tabs_in_note() {
    assert_eq!(
        Record::try_new(1, "book".to_string(), 0, Vec::new(), "a\nb".to_string()),
        Err(forbidden("note", '\n'))
    );
    assert_eq!(
        Record::try_new(1, "book".to_string(), 0, Vec::new(), "a\tb".to_string()),
        Err(forbidden("note", '\t'))
    );
    assert_eq!(
        Record::try_new(1, "book".to_string(), 0, Vec::new(), "a\rb".to_string()),
        Err(forbidden("note", '\r'))
    );
}

#[test]
fn try_new_rejects_separators_inside_tags() {
    assert_eq!(
        Record::try_new(
            1,
            "book".to_string(),
            0,
            vec!["a;b".to_string()],
            String::new()
        ),
        Err(forbidden("tag", ';'))
    );
    assert_eq!(
        Record::try_new(
            1,
            "book".to_string(),
            0,
            vec!["a|b".to_string()],
            String::new()
        ),
        Err(forbidden("tag", '|'))
    );
}

#[test]
fn try_new_accepts_zero_and_negative_values() {
    let zero = Record::try_new(0, "book".to_string(), 0, Vec::new(), String::new()).unwrap();
    let negative = Record::try_new(1, "book".to_string(), -9, Vec::new(), String::new()).unwrap();

    assert_eq!(zero.value, 0);
    assert_eq!(negative.value, -9);
}

#[test]
fn parse_line_reads_every_field() {
    let record = Record::parse_line("9|book|-3|rust;systems| hi ").unwrap();

    assert_eq!(
        record,
        Record {
            id: 9,
            category: "book".to_string(),
            value: -3,
            tags: vec!["rust".to_string(), "systems".to_string()],
            note: " hi ".to_string(),
        }
    );
}

#[test]
fn parse_line_accepts_empty_tags_and_note() {
    let record = Record::parse_line("1|book|0||").unwrap();

    assert!(record.tags.is_empty());
    assert_eq!(record.note, "");
}

#[test]
fn parse_line_rejects_wrong_field_count() {
    assert_eq!(
        Record::parse_line("1|book|0|rust"),
        Err(ParseRecordError::FieldCount { found: 4 })
    );
    assert_eq!(
        Record::parse_line("1|book|0|rust|note|extra"),
        Err(ParseRecordError::FieldCount { found: 6 })
    );
}

#[test]
fn parse_line_rejects_empty_line() {
    assert_eq!(
        Record::parse_line(""),
        Err(ParseRecordError::FieldCount { found: 1 })
    );
}

#[test]
fn parse_line_rejects_invalid_id() {
    assert_eq!(
        Record::parse_line("-1|book|0||"),
        Err(ParseRecordError::InvalidId {
            text: "-1".to_string()
        })
    );
    assert_eq!(
        Record::parse_line("abc|book|0||"),
        Err(ParseRecordError::InvalidId {
            text: "abc".to_string()
        })
    );
}

#[test]
fn parse_line_rejects_invalid_value() {
    assert_eq!(
        Record::parse_line("1|book|1.5||"),
        Err(ParseRecordError::InvalidValue {
            text: "1.5".to_string()
        })
    );
    assert_eq!(
        Record::parse_line("1|book|999999999999999999999999999999||"),
        Err(ParseRecordError::InvalidValue {
            text: "999999999999999999999999999999".to_string()
        })
    );
}

#[test]
fn parse_line_reports_domain_validation_errors() {
    assert_eq!(
        Record::parse_line("1|   |5||x"),
        Err(ParseRecordError::Validation(ValidationError::EmptyCategory))
    );
}

#[test]
fn to_line_round_trips_a_record() {
    let record = Record::parse_line("4|course|120|rust;video|Ownership course").unwrap();

    assert_eq!(record.to_line(), "4|course|120|rust;video|Ownership course");
}

#[test]
fn to_line_writes_empty_tags_as_an_empty_field() {
    let record = Record {
        id: 1,
        category: "book".to_string(),
        value: 0,
        tags: Vec::new(),
        note: String::new(),
    };

    assert_eq!(record.to_line(), "1|book|0||");
}

#[test]
fn parse_line_keeps_the_largest_id() {
    let record = Record::parse_line("18446744073709551615|book|0||").unwrap();

    assert_eq!(record.id, u64::MAX);
    assert_eq!(record.to_line(), "18446744073709551615|book|0||");
}
