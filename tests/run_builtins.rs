mod common;

use branchy::parse_program;
use common::run_with_seed;

#[test]
fn builtin_upper() {
  let p = parse_program("[ !upper(hello); ]").unwrap();
  let out = run_with_seed(&p, 0);
  assert_eq!(out, "HELLO");
}

#[test]
fn builtin_concat() {
  let p = parse_program("[ !concat(a, b); ]").unwrap();
  let out = run_with_seed(&p, 0);
  assert_eq!(out, "ab");
}

#[test]
fn builtin_lower() {
  let p = parse_program("[ !lower(HELLO); ]").unwrap();
  let out = run_with_seed(&p, 0);
  assert_eq!(out, "hello");
}

#[test]
fn builtin_trim() {
  let p = parse_program(r#"[ !trim("  x  "); ]"#).unwrap();
  let out = run_with_seed(&p, 0);
  assert_eq!(out, "x");
}

#[test]
fn builtin_len() {
  let p = parse_program("[ !len(hello); ]").unwrap();
  let out = run_with_seed(&p, 0);
  assert_eq!(out, "5");
}

#[test]
fn builtin_replace() {
  let p = parse_program(r#"[ !replace(aba, a, x); ]"#).unwrap();
  let out = run_with_seed(&p, 0);
  assert_eq!(out, "xbx");
}

#[test]
fn builtin_join() {
  let p = parse_program("[ !join(_, a, b, c); ]").unwrap();
  let out = run_with_seed(&p, 0);
  assert_eq!(out, "a_b_c");
}

#[test]
fn builtin_split() {
  let p = parse_program(r#"[ !split("a,b,c", ","); ]"#).unwrap();
  let out = run_with_seed(&p, 0);
  assert_eq!(out, "a");
}
