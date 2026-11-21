use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use l22_capstone::model::{ParseRecordError, Record, ValidationError};
use l22_capstone::store::{self, StoreError};

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new(label: &str) -> Self {
        let unique = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let path =
            std::env::temp_dir().join(format!("l22-store-{label}-{}-{unique}", std::process::id()));
        fs::create_dir_all(&path).unwrap();
        Self { path }
    }

    fn file(&self, name: &str) -> PathBuf {
        self.path.join(name)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn record(id: u64, category: &str, value: i64, tags: &[&str], note: &str) -> Record {
    Record {
        id,
        category: category.to_string(),
        value,
        tags: tags.iter().map(|tag| (*tag).to_string()).collect(),
        note: note.to_string(),
    }
}

fn write(path: &Path, text: &str) {
    fs::write(path, text).unwrap();
}

#[test]
fn missing_store_loads_as_empty() {
    let temp = TempDir::new("missing");
    let records = store::load(&temp.file("does-not-exist.db")).unwrap();

    assert!(records.is_empty());
}

#[test]
fn load_ignores_blank_lines_but_keeps_real_line_numbers() {
    let temp = TempDir::new("blank");
    let path = temp.file("data.db");
    write(&path, "\n1|book|1||\n\n\n2|course|2||\n");

    let records = store::load(&path).unwrap();

    assert_eq!(records.len(), 2);
    assert_eq!(records[0].id, 1);
    assert_eq!(records[1].id, 2);
}

#[test]
fn load_accepts_windows_line_endings() {
    let temp = TempDir::new("crlf");
    let path = temp.file("data.db");
    write(&path, "1|book|1||\r\n2|course|2||\r\n");

    let records = store::load(&path).unwrap();

    assert_eq!(records.len(), 2);
    assert_eq!(records[1].category, "course");
}

#[test]
fn load_reports_the_line_number_of_a_malformed_record() {
    let temp = TempDir::new("malformed");
    let path = temp.file("data.db");
    write(&path, "1|book|1||\n2|book|not-a-number||\n");

    match store::load(&path) {
        Err(StoreError::Line { line, source, .. }) => {
            assert_eq!(line, 2);
            assert_eq!(
                source,
                ParseRecordError::InvalidValue {
                    text: "not-a-number".to_string()
                }
            );
        }
        other => panic!("unexpected result: {other:?}"),
    }
}

#[test]
fn load_reports_domain_validation_with_its_line() {
    let temp = TempDir::new("invalid-domain");
    let path = temp.file("data.db");
    write(&path, "1|book|1||\n2|   |1||\n");

    match store::load(&path) {
        Err(StoreError::Line { line, source, .. }) => {
            assert_eq!(line, 2);
            assert_eq!(
                source,
                ParseRecordError::Validation(ValidationError::EmptyCategory)
            );
        }
        other => panic!("unexpected result: {other:?}"),
    }
}

#[test]
fn load_rejects_duplicate_ids_and_reports_both_lines() {
    let temp = TempDir::new("duplicate");
    let path = temp.file("data.db");
    write(&path, "1|book|1||\n\n1|course|2||\n");

    match store::load(&path) {
        Err(StoreError::DuplicateId {
            id,
            first_line,
            second_line,
            ..
        }) => {
            assert_eq!(id, 1);
            assert_eq!(first_line, 1);
            assert_eq!(second_line, 3);
        }
        other => panic!("unexpected result: {other:?}"),
    }
}

#[test]
fn save_and_load_round_trip_records() {
    let temp = TempDir::new("round-trip");
    let path = temp.file("data.db");
    let records = vec![
        record(1, "book", 89, &["rust", "systems"], "The Rust Book"),
        record(2, "course", -5, &[], ""),
    ];

    store::save(&path, &records).unwrap();
    let loaded = store::load(&path).unwrap();

    assert_eq!(loaded, records);
    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "1|book|89|rust;systems|The Rust Book\n2|course|-5||\n"
    );
}

#[test]
fn saving_an_empty_slice_creates_an_empty_store() {
    let temp = TempDir::new("empty");
    let path = temp.file("empty.db");

    store::save(&path, &[]).unwrap();

    assert!(path.is_file());
    assert_eq!(fs::read_to_string(&path).unwrap(), "");
    assert!(store::load(&path).unwrap().is_empty());
}

#[test]
fn save_replaces_the_previous_contents() {
    let temp = TempDir::new("replace");
    let path = temp.file("data.db");

    store::save(&path, &[record(1, "book", 1, &[], "")]).unwrap();
    store::save(&path, &[record(2, "course", 2, &[], "")]).unwrap();

    let loaded = store::load(&path).unwrap();
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].id, 2);
}

#[test]
fn load_reads_a_last_line_without_a_trailing_newline() {
    let temp = TempDir::new("no-final-newline");
    let path = temp.file("data.db");
    write(&path, "1|book|1||\n2|course|2||");

    let records = store::load(&path).unwrap();

    assert_eq!(records.len(), 2);
    assert_eq!(records[1].id, 2);
}

#[test]
fn failed_save_does_not_leave_a_temporary_file() {
    let temp = TempDir::new("failed-save");
    let directory_target = temp.file("target-directory");
    fs::create_dir(&directory_target).unwrap();

    let result = store::save(&directory_target, &[]);

    assert!(matches!(result, Err(StoreError::Write { .. })));
    assert!(directory_target.is_dir());
    assert!(!store::temporary_path(&directory_target).exists());
}
