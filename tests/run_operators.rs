mod common;

use branchy::parse_program;
use common::run_with_seed;

#[test]
fn op_concat() {
  let p = parse_program(r#"[ "hello" + " " + "world"; ]"#).unwrap();
  let out = run_with_seed(&p, 0);
  assert_eq!(out, "hello world");
}

#[test]
fn op_repeat() {
  let p = parse_program(r#"[ "ab" * 3; ]"#).unwrap();
  let out = run_with_seed(&p, 0);
  assert_eq!(out, "ababab");
}

#[test]
fn op_repeat_range() {
  let p = parse_program(r#"[ "x" * 1..3; ]"#).unwrap();
  let allowed: Vec<String> = ["x", "xx", "xxx"].into_iter().map(String::from).collect();
  for seed in 0..40u64 {
    let out = run_with_seed(&p, seed);
    assert!(allowed.contains(&out), "seed {} gave {:?}", seed, out);
  }
}

#[test]
fn op_repeat_branch_recalculated() {
  let p = parse_program(
    r#"
[
  [ "y"; "x"; ] * 5;
]
"#,
  )
  .unwrap();
  let out = run_with_seed(&p, 0);
  assert_eq!(out.len(), 5);
  assert!(out.chars().all(|c| c == 'y' || c == 'x'), "got {:?}", out);
}

#[test]
fn op_repeat_branch_with_range() {
  let p = parse_program(
    r#"
[
  [ "y"; "x"; ] * 1..30;
]
"#,
  )
  .unwrap();
  for seed in 0..20u64 {
    let out = run_with_seed(&p, seed);
    assert!(
      out.len() >= 1 && out.len() <= 30,
      "seed {} len {} got {:?}",
      seed,
      out.len(),
      out
    );
    assert!(
      out.chars().all(|c| c == 'y' || c == 'x'),
      "seed {} got {:?}",
      seed,
      out
    );
  }
}

#[test]
fn op_repeat_inline_recalculated() {
  let p = parse_program(r#"[ x <aa|bb|cc> * 3; ]"#).unwrap();
  let out = run_with_seed(&p, 0);
  assert_eq!(
    out.matches("x ").count(),
    3,
    "each of 3 evals should output 'x <option>', got {:?}",
    out
  );
  assert!(
    out.contains("aa") || out.contains("bb") || out.contains("cc"),
    "got {:?}",
    out
  );
}
