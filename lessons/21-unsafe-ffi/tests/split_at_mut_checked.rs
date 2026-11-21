//! 主练习第一部分的行为测试：受检拆分可变切片。
//!
//! 测试只通过公开函数观察结果，不检查内部实现方式。

use l21_unsafe_ffi::split_at_mut_checked;

#[test]
fn splits_in_the_middle() {
    let mut values = [1, 2, 3, 4, 5];

    let (left, right) = split_at_mut_checked(&mut values, 2).expect("2 在长度 5 的范围内");

    assert_eq!(left, &[1, 2]);
    assert_eq!(right, &[3, 4, 5]);
}

#[test]
fn split_at_zero_gives_empty_left_and_full_right() {
    let mut values = [7, 8, 9];

    let (left, right) = split_at_mut_checked(&mut values, 0).expect("0 是合法切点");

    assert!(left.is_empty());
    assert_eq!(right, &[7, 8, 9]);
}

#[test]
fn split_at_length_gives_full_left_and_empty_right() {
    let mut values = [7, 8, 9];

    let (left, right) = split_at_mut_checked(&mut values, 3).expect("长度本身是合法切点");

    assert_eq!(left, &[7, 8, 9]);
    assert!(right.is_empty());
}

#[test]
fn empty_slice_splits_into_two_empty_slices() {
    let mut values: [i32; 0] = [];

    let (left, right) = split_at_mut_checked(&mut values, 0).expect("空切片只在 0 处可拆");

    assert!(left.is_empty());
    assert!(right.is_empty());
}

#[test]
fn index_beyond_length_returns_none() {
    let mut values = [1, 2, 3];

    assert!(split_at_mut_checked(&mut values, 4).is_none());
    assert!(split_at_mut_checked(&mut values, usize::MAX).is_none());
    // 失败的拆分不能改变原数据。
    assert_eq!(values, [1, 2, 3]);
}

#[test]
fn both_halves_can_be_modified_independently() {
    let mut values = [1, 2, 3, 4];

    {
        let (left, right) = split_at_mut_checked(&mut values, 2).expect("2 在范围内");
        left[0] = 10;
        right[1] = 40;
    }

    assert_eq!(values, [10, 2, 3, 40]);
}

#[test]
fn matches_standard_library_for_every_split_point() {
    let original = [10, 20, 30, 40, 50];

    for mid in 0..=original.len() {
        let mut mine = original;
        let (mine_left, mine_right) =
            split_at_mut_checked(&mut mine, mid).expect("0..=len 都是合法切点");

        let mut reference = original;
        let (reference_left, reference_right) = reference.split_at_mut(mid);

        assert_eq!(mine_left, reference_left, "mid = {mid} 的前段");
        assert_eq!(mine_right, reference_right, "mid = {mid} 的后段");
    }
}

#[test]
fn works_with_non_copy_elements() {
    let mut values = vec![String::from("a"), String::from("b"), String::from("c")];

    {
        let (left, right) = split_at_mut_checked(&mut values, 1).expect("1 在范围内");
        left[0].push('!');
        right[0] = String::from("B");
    }

    assert_eq!(
        values,
        [String::from("a!"), String::from("B"), String::from("c")]
    );
}
