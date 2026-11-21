use std::fs;
use std::path::PathBuf;
use std::process::{Command as ProcessCommand, Output};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

const SAMPLE: &str = "\
1|book|89|rust;systems|The Rust Book
2|course|120|rust;video|Ownership course
3|book|-5|draft;rust|Correction
";

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new(label: &str) -> Self {
        let unique = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let path =
            std::env::temp_dir().join(format!("l22-e2e-{label}-{}-{unique}", std::process::id()));
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
fn run(dir: &TempDir, args: &[&str]) -> Output {
    ProcessCommand::new(env!("CARGO_BIN_EXE_datakit"))
        .current_dir(&dir.path)
        .args(args)
        .output()
        .expect("failed to run datakit binary")
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).unwrap()
}

fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).unwrap()
}

#[test]
fn import_writes_the_store_and_reports_counts() {
    let temp = TempDir::new("import");
    fs::write(temp.file("sample.txt"), SAMPLE).unwrap();

    let output = run(&temp, &["import", "sample.txt"]);

    assert!(output.status.success());
    assert_eq!(stdout(&output), "imported 3 records; store has 3 records\n");
    assert_eq!(fs::read_to_string(temp.file("datakit.db")).unwrap(), SAMPLE);
}

#[test]
fn import_appends_to_an_existing_store() {
    let temp = TempDir::new("append");
    fs::write(temp.file("first.txt"), "1|book|10||\n").unwrap();
    fs::write(temp.file("second.txt"), "2|course|20||\n").unwrap();

    assert!(run(&temp, &["import", "first.txt"]).status.success());
    let output = run(&temp, &["import", "second.txt"]);

    assert!(output.status.success());
    assert_eq!(stdout(&output), "imported 1 records; store has 2 records\n");
    assert_eq!(
        fs::read_to_string(temp.file("datakit.db")).unwrap(),
        "1|book|10||\n2|course|20||\n"
    );
}

#[test]
fn list_prints_a_stable_table_sorted_by_id() {
    let temp = TempDir::new("list");
    fs::write(temp.file("sample.txt"), SAMPLE).unwrap();
    assert!(run(&temp, &["import", "sample.txt"]).status.success());

    let output = run(&temp, &["list"]);

    assert!(output.status.success());
    assert_eq!(
        stdout(&output),
        "ID  CATEGORY  VALUE  TAGS          NOTE\n\
         1   book      89     rust,systems  The Rust Book\n\
         2   course    120    rust,video    Ownership course\n\
         3   book      -5     draft,rust    Correction\n"
    );
}

#[test]
fn list_filters_and_sorts_before_printing() {
    let temp = TempDir::new("list-filter");
    fs::write(temp.file("sample.txt"), SAMPLE).unwrap();
    assert!(run(&temp, &["import", "sample.txt"]).status.success());

    let output = run(
        &temp,
        &["list", "--category", "book", "--sort", "value", "--desc"],
    );

    assert!(output.status.success());
    assert_eq!(
        stdout(&output),
        "ID  CATEGORY  VALUE  TAGS          NOTE\n\
         1   book      89     rust,systems  The Rust Book\n\
         3   book      -5     draft,rust    Correction\n"
    );
}

#[test]
fn list_reports_only_the_header_when_nothing_matches() {
    let temp = TempDir::new("list-empty");
    fs::write(temp.file("sample.txt"), SAMPLE).unwrap();
    assert!(run(&temp, &["import", "sample.txt"]).status.success());

    let output = run(&temp, &["list", "--category", "missing"]);

    assert!(output.status.success());
    assert_eq!(stdout(&output), "ID  CATEGORY  VALUE  TAGS  NOTE\n");
}

#[test]
fn stats_prints_the_global_summary() {
    let temp = TempDir::new("stats");
    fs::write(temp.file("sample.txt"), SAMPLE).unwrap();
    assert!(run(&temp, &["import", "sample.txt"]).status.success());

    let output = run(&temp, &["stats"]);

    assert!(output.status.success());
    assert_eq!(
        stdout(&output),
        "count: 3\nsum: 204\nmin: -5\nmax: 120\nmean: 68.00\n"
    );
}

#[test]
fn stats_can_group_by_category() {
    let temp = TempDir::new("stats-group");
    fs::write(temp.file("sample.txt"), SAMPLE).unwrap();
    assert!(run(&temp, &["import", "sample.txt"]).status.success());

    let output = run(&temp, &["stats", "--group", "category"]);

    assert!(output.status.success());
    assert_eq!(
        stdout(&output),
        "CATEGORY  COUNT  SUM  MIN  MAX  MEAN\n\
         book      2      84   -5   89   42.00\n\
         course    1      120  120  120  120.00\n"
    );
}

#[test]
fn check_reports_the_number_of_valid_records() {
    let temp = TempDir::new("check");
    fs::write(temp.file("sample.txt"), SAMPLE).unwrap();
    assert!(run(&temp, &["import", "sample.txt"]).status.success());

    let output = run(&temp, &["check"]);

    assert!(output.status.success());
    assert_eq!(stdout(&output), "ok: 3 records\n");
}

#[test]
fn importing_the_same_file_twice_fails_without_changing_the_store() {
    let temp = TempDir::new("duplicate-existing");
    fs::write(temp.file("sample.txt"), SAMPLE).unwrap();
    assert!(run(&temp, &["import", "sample.txt"]).status.success());

    let output = run(&temp, &["import", "sample.txt"]);

    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).contains("duplicate id 1"));
    let check = run(&temp, &["check"]);
    assert_eq!(stdout(&check), "ok: 3 records\n");
}

#[test]
fn duplicate_ids_inside_one_input_file_are_rejected_atomically() {
    let temp = TempDir::new("duplicate-input");
    fs::write(temp.file("bad.txt"), "1|book|1||\n1|course|2||\n").unwrap();

    let output = run(&temp, &["import", "bad.txt"]);

    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).contains("duplicate id 1"));
    assert!(stderr(&output).contains("input line 2"));
    assert!(!temp.file("datakit.db").exists());
}

#[test]
fn a_malformed_input_line_reports_the_line_and_writes_nothing() {
    let temp = TempDir::new("malformed-input");
    fs::write(temp.file("bad.txt"), "1|book|1||\n2|book|oops||\n").unwrap();

    let output = run(&temp, &["import", "bad.txt"]);

    assert_eq!(output.status.code(), Some(1));
    let message = stderr(&output);
    assert!(message.contains("line 2"), "stderr was: {message}");
    assert!(message.contains("invalid value"), "stderr was: {message}");
    assert!(!temp.file("datakit.db").exists());
}

#[test]
fn replace_discards_the_previous_store() {
    let temp = TempDir::new("replace");
    fs::write(temp.file("sample.txt"), SAMPLE).unwrap();
    fs::write(temp.file("one.txt"), "9|new|1||\n").unwrap();
    assert!(run(&temp, &["import", "sample.txt"]).status.success());

    let output = run(&temp, &["import", "one.txt", "--replace"]);

    assert!(output.status.success());
    assert_eq!(stdout(&output), "imported 1 records; store replaced\n");
    let check = run(&temp, &["check"]);
    assert_eq!(stdout(&check), "ok: 1 records\n");
    assert_eq!(
        fs::read_to_string(temp.file("datakit.db")).unwrap(),
        "9|new|1||\n"
    );
}

#[test]
fn empty_import_is_valid_and_creates_an_empty_store() {
    let temp = TempDir::new("empty-import");
    fs::write(temp.file("empty.txt"), "").unwrap();

    let output = run(&temp, &["import", "empty.txt"]);

    assert!(output.status.success());
    assert_eq!(stdout(&output), "imported 0 records; store has 0 records\n");
    assert_eq!(fs::read_to_string(temp.file("datakit.db")).unwrap(), "");
    assert_eq!(stdout(&run(&temp, &["check"])), "ok: 0 records\n");
}

#[test]
fn check_reports_the_line_number_of_a_broken_store() {
    let temp = TempDir::new("broken-store");
    fs::write(
        temp.file("datakit.db"),
        "1|book|1||\n2|book|not-a-number||\n",
    )
    .unwrap();

    let output = run(&temp, &["check"]);

    assert_eq!(output.status.code(), Some(1));
    let message = stderr(&output);
    assert!(message.contains("line 2"), "stderr was: {message}");
    assert!(message.contains("invalid value"), "stderr was: {message}");
}

#[test]
fn list_also_propagates_a_broken_store() {
    let temp = TempDir::new("broken-list");
    fs::write(temp.file("datakit.db"), "1|book|1||\nnot valid\n").unwrap();

    let output = run(&temp, &["list"]);

    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).contains("line 2"));
}

#[test]
fn a_missing_import_file_is_a_runtime_error() {
    let temp = TempDir::new("missing-import");

    let output = run(&temp, &["import", "absent.txt"]);

    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).contains("cannot read import file"));
    assert!(!temp.file("datakit.db").exists());
}

#[test]
fn invalid_range_is_a_usage_error_with_exit_code_two() {
    let temp = TempDir::new("invalid-range");

    let output = run(&temp, &["list", "--min", "5", "--max", "2"]);

    assert_eq!(output.status.code(), Some(2));
    let message = stderr(&output);
    assert!(message.contains("min (5)"), "stderr was: {message}");
    assert!(message.contains("usage: datakit"), "stderr was: {message}");
}

#[test]
fn a_global_store_path_is_used_by_every_command() {
    let temp = TempDir::new("global-store");
    fs::create_dir(temp.file("data")).unwrap();
    fs::write(temp.file("sample.txt"), SAMPLE).unwrap();

    let import = run(
        &temp,
        &["--store", "data/custom.db", "import", "sample.txt"],
    );
    assert!(import.status.success());
    let check = run(&temp, &["--store", "data/custom.db", "check"]);

    assert_eq!(stdout(&check), "ok: 3 records\n");
    assert!(!temp.file("datakit.db").exists());
}
