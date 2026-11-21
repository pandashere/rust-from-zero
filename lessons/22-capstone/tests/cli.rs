use std::path::PathBuf;

use l22_capstone::cli::{Cli, Command, DEFAULT_STORE, UsageError, parse_args};
use l22_capstone::query::{Query, SortKey};

fn parse(items: &[&str]) -> Result<Cli, UsageError> {
    parse_args(items.iter().map(|item| (*item).to_string()))
}

#[test]
fn defaults_to_check_with_the_default_store() {
    assert_eq!(
        parse(&["check"]),
        Ok(Cli {
            store: PathBuf::from(DEFAULT_STORE),
            command: Command::Check,
        })
    );
}

#[test]
fn global_store_option_changes_the_path() {
    assert_eq!(
        parse(&["--store", "data/records.db", "check"]),
        Ok(Cli {
            store: PathBuf::from("data/records.db"),
            command: Command::Check,
        })
    );
}

#[test]
fn import_parses_path_and_replace_flag() {
    assert_eq!(
        parse(&["--store", "s.db", "import", "input.txt", "--replace"]),
        Ok(Cli {
            store: PathBuf::from("s.db"),
            command: Command::Import {
                path: PathBuf::from("input.txt"),
                replace: true,
            },
        })
    );
}

#[test]
fn import_defaults_to_append() {
    assert_eq!(
        parse(&["import", "input.txt"]),
        Ok(Cli {
            store: PathBuf::from(DEFAULT_STORE),
            command: Command::Import {
                path: PathBuf::from("input.txt"),
                replace: false,
            },
        })
    );
}

#[test]
fn list_parses_every_supported_option() {
    let cli = parse(&[
        "list",
        "--category",
        "book",
        "--tag",
        "rust",
        "--min",
        "-5",
        "--max",
        "10",
        "--contains",
        "note",
        "--sort",
        "value",
        "--desc",
        "--limit",
        "3",
    ])
    .unwrap();

    assert_eq!(
        cli.command,
        Command::List(Query {
            category: Some("book".to_string()),
            tag: Some("rust".to_string()),
            min: Some(-5),
            max: Some(10),
            contains: Some("note".to_string()),
            sort: SortKey::Value,
            descending: true,
            limit: Some(3),
        })
    );
}

#[test]
fn list_defaults_to_id_sort_without_a_limit() {
    let cli = parse(&["list"]).unwrap();

    assert_eq!(cli.command, Command::List(Query::default()));
}

#[test]
fn stats_parses_group_category() {
    let cli = parse(&["stats", "--group", "category"]).unwrap();

    assert_eq!(cli.command, Command::Stats { grouped: true });
}

#[test]
fn stats_without_group_is_global() {
    assert_eq!(
        parse(&["stats"]).unwrap().command,
        Command::Stats { grouped: false }
    );
}

#[test]
fn missing_command_is_an_error() {
    assert_eq!(parse(&[]), Err(UsageError::MissingCommand));
}

#[test]
fn unknown_command_is_an_error() {
    assert_eq!(
        parse(&["remove"]),
        Err(UsageError::UnknownCommand("remove".to_string()))
    );
}

#[test]
fn unknown_option_is_an_error() {
    assert_eq!(
        parse(&["list", "--unknown"]),
        Err(UsageError::UnknownOption("--unknown".to_string()))
    );
}

#[test]
fn option_without_a_value_is_an_error() {
    assert_eq!(
        parse(&["list", "--limit"]),
        Err(UsageError::MissingOptionValue("--limit".to_string()))
    );
}

#[test]
fn invalid_limit_is_an_error() {
    assert_eq!(
        parse(&["list", "--limit", "-1"]),
        Err(UsageError::InvalidOptionValue {
            option: "--limit".to_string(),
            value: "-1".to_string(),
        })
    );
}

#[test]
fn invalid_sort_key_is_an_error() {
    assert_eq!(
        parse(&["list", "--sort", "note"]),
        Err(UsageError::InvalidOptionValue {
            option: "--sort".to_string(),
            value: "note".to_string(),
        })
    );
}

#[test]
fn duplicate_option_is_an_error() {
    assert_eq!(
        parse(&["list", "--limit", "1", "--limit", "2"]),
        Err(UsageError::DuplicateOption("--limit".to_string()))
    );
}

#[test]
fn duplicate_store_option_is_an_error() {
    assert_eq!(
        parse(&["--store", "a.db", "--store", "b.db", "check"]),
        Err(UsageError::DuplicateOption("--store".to_string()))
    );
}

#[test]
fn import_requires_a_path() {
    assert_eq!(
        parse(&["import"]),
        Err(UsageError::MissingArgument("import <file>"))
    );
}

#[test]
fn import_rejects_a_second_path() {
    assert_eq!(
        parse(&["import", "a.txt", "b.txt"]),
        Err(UsageError::UnexpectedArgument("b.txt".to_string()))
    );
}

#[test]
fn invalid_group_value_is_an_error() {
    assert_eq!(
        parse(&["stats", "--group", "tag"]),
        Err(UsageError::InvalidOptionValue {
            option: "--group".to_string(),
            value: "tag".to_string(),
        })
    );
}

#[test]
fn check_rejects_extra_arguments() {
    assert_eq!(
        parse(&["check", "extra"]),
        Err(UsageError::UnexpectedArgument("extra".to_string()))
    );
}

#[test]
fn min_greater_than_max_is_rejected_during_parsing() {
    assert_eq!(
        parse(&["list", "--min", "5", "--max", "2"]),
        Err(UsageError::InvalidRange { min: 5, max: 2 })
    );
}

#[test]
fn store_option_must_come_before_the_command() {
    assert_eq!(
        parse(&["list", "--store", "a.db"]),
        Err(UsageError::UnknownOption("--store".to_string()))
    );
}
