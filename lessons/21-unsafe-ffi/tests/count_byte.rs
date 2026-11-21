//! 主练习第二部分的行为测试：C ABI 字节计数的安全包装。

use l21_unsafe_ffi::count_byte;
use l21_unsafe_ffi::ffi::rfz_count_byte;

#[test]
fn counts_repeated_byte() {
    assert_eq!(count_byte(b"banana", b'a'), 3);
}

#[test]
fn empty_slice_has_no_matches() {
    assert_eq!(count_byte(&[], b'x'), 0);
}

#[test]
fn zero_bytes_are_ordinary_data() {
    assert_eq!(count_byte(&[0, 1, 0, 0, 2], 0), 3);
}

#[test]
fn absent_needle_returns_zero() {
    assert_eq!(count_byte(b"hello", b'z'), 0);
}

#[test]
fn every_byte_matching_returns_length() {
    assert_eq!(count_byte(&[9; 32], 9), 32);
}

#[test]
fn agrees_with_iterator_on_large_input() {
    let data: Vec<u8> = (0..100_000).map(|index| (index % 251) as u8).collect();
    let expected = data.iter().filter(|byte| **byte == 200).count();

    assert_eq!(count_byte(&data, 200), expected);
}

#[test]
fn c_fixture_counts_correctly_for_valid_slice() {
    let data = b"mississippi";

    // SAFETY: as_ptr() 与 len() 描述的正是这块仍然存活的字节切片；
    // 替身函数只在调用期间读取，不保存指针。
    let count = unsafe { rfz_count_byte(data.as_ptr(), data.len(), b's') };

    assert_eq!(count, 4);
}
