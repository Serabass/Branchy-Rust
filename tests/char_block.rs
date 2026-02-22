mod common;

use branchy::parse_program;
use common::run_with_seed;
use std::collections::HashSet;

#[test]
fn char_block_one_from_set() {
    let p = parse_program(r#"[ [a]; ]"#).unwrap();
    let out = run_with_seed(&p, 0);
    assert_eq!(out, "a");
}

#[test]
fn char_block_one_from_range() {
    let p = parse_program(r#"[ [a-z]; ]"#).unwrap();
    let out = run_with_seed(&p, 0);
    assert_eq!(out.len(), 1);
    let c = out.chars().next().unwrap();
    assert!(c >= 'a' && c <= 'z', "got {:?}", out);
}

#[test]
fn char_block_fixed_count() {
    let p = parse_program(r#"[ [a-zA-Z:5]; ]"#).unwrap();
    let out = run_with_seed(&p, 0);
    assert_eq!(out.len(), 5);
    assert!(out.chars().all(|c| c.is_ascii_alphabetic()), "got {:?}", out);
}

#[test]
fn char_block_range_count() {
    let p = parse_program(r#"[ [abcdef:3..7]; ]"#).unwrap();
    let allowed_len: HashSet<usize> = [3, 4, 5, 6, 7].into_iter().collect();
    let allowed_chars: HashSet<char> = "abcdef".chars().collect();
    for seed in 0..30u64 {
        let out = run_with_seed(&p, seed);
        assert!(allowed_len.contains(&out.len()), "seed {} len {} got {:?}", seed, out.len(), out);
        assert!(out.chars().all(|c| allowed_chars.contains(&c)), "seed {} got {:?}", seed, out);
    }
}

#[test]
fn char_block_digits() {
    let p = parse_program(r#"[ [0-9:4]; ]"#).unwrap();
    let out = run_with_seed(&p, 42);
    assert_eq!(out.len(), 4);
    assert!(out.chars().all(|c| c.is_ascii_digit()), "got {:?}", out);
}
