//! 主练习 `Note` 的行为测试。只使用公开 API，描述可观察行为。

use l03_ownership_borrowing::Note;

#[test]
fn new_stores_title_and_body() {
    let note = Note::new(String::from("购物清单"), String::from("牛奶"));

    assert_eq!(note.title(), "购物清单");
    assert_eq!(note.body(), "牛奶");
}

#[test]
fn accessors_borrow_without_consuming_the_note() {
    let note = Note::new(String::from("日记"), String::from("今天学了所有权"));

    // 访问器只是借用，可以反复调用，笔记始终可用
    assert_eq!(note.title(), "日记");
    assert_eq!(note.body(), "今天学了所有权");
    assert_eq!(note.title(), "日记");
    assert_eq!(note.body(), "今天学了所有权");
}

#[test]
fn append_line_to_empty_body_writes_line_without_leading_newline() {
    let mut note = Note::new(String::from("Log"), String::new());

    note.append_line("first line");

    assert_eq!(note.body(), "first line");
}

#[test]
fn append_line_separates_multiple_lines_with_single_newlines() {
    let mut note = Note::new(String::from("Log"), String::from("first line"));

    note.append_line("second line");
    note.append_line("third line");

    assert_eq!(note.body(), "first line\nsecond line\nthird line");
}

#[test]
fn append_line_keeps_title_unchanged() {
    let mut note = Note::new(String::from("周报"), String::new());

    note.append_line("周一：读完第三课");

    assert_eq!(note.title(), "周报");
}

#[test]
fn append_empty_line_to_empty_body_keeps_body_empty() {
    let mut note = Note::new(String::from("Log"), String::new());

    note.append_line("");

    assert_eq!(note.body(), "");
}

#[test]
fn append_empty_line_after_content_adds_trailing_newline() {
    let mut note = Note::new(String::from("Log"), String::from("only line"));

    note.append_line("");

    assert_eq!(note.body(), "only line\n");
}

#[test]
fn into_parts_returns_strings_matching_the_fields() {
    let note = Note::new(String::from("Rust"), String::from("所有权\n借用"));

    let (title, body) = note.into_parts();

    assert_eq!(title, "Rust");
    assert_eq!(body, "所有权\n借用");
}

#[test]
fn into_parts_results_are_owned_and_independently_mutable() {
    let note = Note::new(String::from("Rust"), String::from("Borrowing"));

    let (mut title, mut body) = note.into_parts();
    title.push('!');
    body.push_str(" rules");

    assert_eq!(title, "Rust!");
    assert_eq!(body, "Borrowing rules");
}

#[test]
fn into_parts_handles_empty_title_and_body() {
    let note = Note::new(String::new(), String::new());

    let (title, body) = note.into_parts();

    assert_eq!(title, "");
    assert_eq!(body, "");
}
