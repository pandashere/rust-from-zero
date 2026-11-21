use l22_capstone::sessions::{
    Session, SessionError, SessionLineError, busiest_day, parse_sessions, total_minutes_by_project,
};

fn line(line: usize, error: SessionError) -> SessionLineError {
    SessionLineError { line, error }
}

#[test]
fn parses_valid_sessions_and_normalizes_names() {
    let text = "\
2026-11-03| Rust-From-Zero |90
2026-11-03|datakit|45
2026-11-04| Datakit |30
";

    assert_eq!(
        parse_sessions(text),
        Ok(vec![
            Session {
                date: "2026-11-03".to_string(),
                project: "rust-from-zero".to_string(),
                minutes: 90,
            },
            Session {
                date: "2026-11-03".to_string(),
                project: "datakit".to_string(),
                minutes: 45,
            },
            Session {
                date: "2026-11-04".to_string(),
                project: "datakit".to_string(),
                minutes: 30,
            },
        ])
    );
}

#[test]
fn empty_text_is_a_valid_empty_list() {
    assert_eq!(parse_sessions(""), Ok(Vec::new()));
    assert_eq!(parse_sessions("\n  \n\t\n"), Ok(Vec::new()));
}

#[test]
fn wrong_field_count_reports_the_line() {
    assert_eq!(
        parse_sessions("2026-11-03|datakit\n"),
        Err(line(
            1,
            SessionError::FieldCount {
                expected: 3,
                found: 2,
            }
        ))
    );
    assert_eq!(
        parse_sessions("2026-11-03|datakit|30|extra\n"),
        Err(line(
            1,
            SessionError::FieldCount {
                expected: 3,
                found: 4,
            }
        ))
    );
}

#[test]
fn empty_date_is_rejected() {
    assert_eq!(
        parse_sessions("   |datakit|30\n"),
        Err(line(1, SessionError::EmptyDate))
    );
}

#[test]
fn empty_project_is_rejected() {
    assert_eq!(
        parse_sessions("2026-11-03|   |30\n"),
        Err(line(1, SessionError::EmptyProject))
    );
}

#[test]
fn non_numeric_minutes_are_rejected() {
    assert_eq!(
        parse_sessions("2026-11-03|datakit|thirty\n"),
        Err(line(
            1,
            SessionError::InvalidMinutes {
                text: "thirty".to_string(),
            }
        ))
    );
}

#[test]
fn zero_and_more_than_a_day_are_out_of_range() {
    assert_eq!(
        parse_sessions("2026-11-03|datakit|0\n"),
        Err(line(1, SessionError::MinutesOutOfRange { minutes: 0 }))
    );
    assert_eq!(
        parse_sessions("2026-11-03|datakit|1441\n"),
        Err(line(1, SessionError::MinutesOutOfRange { minutes: 1441 }))
    );
}

#[test]
fn one_minute_and_a_full_day_are_accepted() {
    let text = "2026-11-03|datakit|1\n2026-11-04|datakit|1440\n";
    let sessions = parse_sessions(text).unwrap();

    assert_eq!(sessions[0].minutes, 1);
    assert_eq!(sessions[1].minutes, 1440);
}

#[test]
fn duplicate_project_on_the_same_day_reports_both_lines() {
    let text = "2026-11-03|datakit|30\n\n2026-11-03|DATaKit|45\n";

    assert_eq!(
        parse_sessions(text),
        Err(line(
            3,
            SessionError::DuplicateProjectDay {
                date: "2026-11-03".to_string(),
                project: "datakit".to_string(),
                first_line: 1,
            }
        ))
    );
}

#[test]
fn the_same_project_on_different_days_is_allowed() {
    let text = "2026-11-03|datakit|30\n2026-11-04|datakit|45\n";

    assert_eq!(parse_sessions(text).unwrap().len(), 2);
}

#[test]
fn totals_are_summed_per_project_and_sorted_by_name() {
    let sessions = vec![
        Session {
            date: "2026-11-03".to_string(),
            project: "rust".to_string(),
            minutes: 90,
        },
        Session {
            date: "2026-11-03".to_string(),
            project: "datakit".to_string(),
            minutes: 45,
        },
        Session {
            date: "2026-11-04".to_string(),
            project: "rust".to_string(),
            minutes: 30,
        },
    ];

    assert_eq!(
        total_minutes_by_project(&sessions),
        vec![("datakit".to_string(), 45), ("rust".to_string(), 120)]
    );
}

#[test]
fn totals_of_an_empty_list_are_empty() {
    assert!(total_minutes_by_project(&[]).is_empty());
}

#[test]
fn busiest_day_sums_all_projects_on_that_day() {
    let sessions = vec![
        Session {
            date: "2026-11-03".to_string(),
            project: "rust".to_string(),
            minutes: 90,
        },
        Session {
            date: "2026-11-03".to_string(),
            project: "datakit".to_string(),
            minutes: 45,
        },
        Session {
            date: "2026-11-04".to_string(),
            project: "rust".to_string(),
            minutes: 120,
        },
    ];

    assert_eq!(busiest_day(&sessions), Some(("2026-11-03", 135)));
}

#[test]
fn busiest_day_breaks_ties_with_the_earlier_date() {
    let sessions = vec![
        Session {
            date: "2026-11-05".to_string(),
            project: "rust".to_string(),
            minutes: 60,
        },
        Session {
            date: "2026-11-04".to_string(),
            project: "datakit".to_string(),
            minutes: 60,
        },
    ];

    assert_eq!(busiest_day(&sessions), Some(("2026-11-04", 60)));
}

#[test]
fn busiest_day_of_an_empty_list_is_none() {
    assert_eq!(busiest_day(&[]), None);
}
