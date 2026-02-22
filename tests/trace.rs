//! Tests for execution trace: trace must contain only spans of the chosen path.
//! Use fixed seeds for reproducible results.

mod common;

use branchy::parse_program;
use common::run_with_seed_and_trace;

#[test]
fn trace_literal_single_span() {
  // Top-level is a branch; one option => one span
  let p = parse_program("[ hello; ]").unwrap();
  let (out, trace) = run_with_seed_and_trace(&p, 0);
  assert_eq!(out, "hello");
  assert_eq!(
    trace.len(),
    1,
    "single option branch should produce exactly one span"
  );
  assert_eq!(trace[0].start_line, 1);
  assert_eq!(trace[0].end_line, 1);
}

#[test]
fn trace_branch_single_span() {
  let p = parse_program(
    r#"
[
 one;
 two;
 three;
]
"#,
  )
  .unwrap();
  let seed = 42u64;
  let (out, trace) = run_with_seed_and_trace(&p, seed);
  let allowed = ["one", "two", "three"];
  assert!(allowed.contains(&out.as_str()), "got {:?}", out);
  assert_eq!(
    trace.len(),
    1,
    "branch should produce exactly one span (chosen child only)"
  );
  // Chosen option is on one of lines 2, 3, 4
  assert!(trace[0].start_line >= 2 && trace[0].start_line <= 4);
  assert_eq!(trace[0].start_line, trace[0].end_line, "single line span");
}

#[test]
fn trace_same_seed_same_result_and_trace() {
  let p = parse_program(r#"[ x; y; ]"#).unwrap();
  let seed = 7u64;
  let (out1, trace1) = run_with_seed_and_trace(&p, seed);
  let (out2, trace2) = run_with_seed_and_trace(&p, seed);
  assert_eq!(out1, out2, "same seed must give same output");
  assert_eq!(trace1.len(), trace2.len());
  assert_eq!(trace1.len(), 1);
  assert_eq!(trace1[0].start_line, trace2[0].start_line);
  assert_eq!(trace1[0].start_column, trace2[0].start_column);
  assert_eq!(trace1[0].end_line, trace2[0].end_line);
  assert_eq!(trace1[0].end_column, trace2[0].end_column);
}

#[test]
fn trace_branch_only_chosen_line_highlighted() {
  // Source: line 1 = "\n", line 2 = "[\n", line 3 = " privet;\n", line 4 = " poka;\n", line 5 = " 123;\n", line 6 = "]"
  // We want a seed that picks the second option ("poka") so trace should cover only line 4.
  let p = parse_program(
    r#"
[
 privet;
 poka;
 123;
]
"#,
  )
  .unwrap();
  let mut seed_poka = None;
  for seed in 0..100u64 {
    let (out, _) = run_with_seed_and_trace(&p, seed);
    if out == "poka" {
      seed_poka = Some(seed);
      break;
    }
  }
  let seed = seed_poka.expect("no seed in 0..100 yields poka");
  let (out, trace) = run_with_seed_and_trace(&p, seed);
  assert_eq!(out, "poka");
  assert_eq!(trace.len(), 1);
  assert_eq!(
    trace[0].start_line, 4,
    "trace must be only the chosen line (poka on line 4)"
  );
  assert_eq!(trace[0].end_line, 4);
}

#[test]
fn trace_concat_two_spans() {
  let p = parse_program(r#"[ "a" + "b"; ]"#).unwrap();
  let (out, trace) = run_with_seed_and_trace(&p, 0);
  assert_eq!(out, "ab");
  assert!(trace.len() >= 1, "concat involves multiple nodes");
}
