mod common;

use branchy::{deserialize_program, parse_program, resolve_includes, serialize_program};
use std::collections::HashSet;
use std::process::Command;

use common::{run_with_seed, run_with_seed_and_input};

#[test]
fn binary_roundtrip() {
  let p = parse_program("[ x; y; ]").unwrap();
  let bytes = serialize_program(&p).unwrap();
  assert!(bytes.starts_with(b"BRCH"));
  let p2 = deserialize_program(&bytes).unwrap();
  let out1 = run_with_seed(&p, 1);
  let out2 = run_with_seed(&p2, 1);
  assert_eq!(out1, out2);
}

#[test]
fn binary_roundtrip_with_functions() {
  let p = parse_program(
    r#"
!g(:x) = [ :x; ]
[ !g(hello); ]
"#,
  )
  .unwrap();
  let bytes = serialize_program(&p).unwrap();
  assert!(bytes.starts_with(b"BRCH"));
  let p2 = deserialize_program(&bytes).unwrap();
  assert_eq!(p.functions.len(), p2.functions.len());
  let out2 = run_with_seed(&p2, 0);
  assert_eq!(out2, "hello");
}

#[test]
fn binary_roundtrip_with_events() {
  let p = parse_program(
    r#"
@go = [ ok; ]
[ default; ]
"#,
  )
  .unwrap();
  let bytes = serialize_program(&p).unwrap();
  assert!(bytes.starts_with(b"BRCH"));
  let p2 = deserialize_program(&bytes).unwrap();
  let out_event = run_with_seed_and_input(&p, 0, Some("go")).0;
  let out_event2 = run_with_seed_and_input(&p2, 0, Some("go")).0;
  assert_eq!(out_event, out_event2, "event output");
  let out_main = run_with_seed(&p, 0);
  let out_main2 = run_with_seed(&p2, 0);
  assert_eq!(out_main, out_main2, "main output");
}

#[test]
fn binary_roundtrip_with_spread_param() {
  let p = parse_program(
    r#"
!wrap(:_) = [ a; ...:x; b; ]
[ wrap :_ { :x = [ 1; ]; }; ]
"#,
  )
  .unwrap();
  let program = resolve_includes(p, |_| Err("no includes".into())).unwrap();
  let bytes = serialize_program(&program).unwrap();
  let p2 = deserialize_program(&bytes).unwrap();
  let out1 = run_with_seed(&program, 0);
  let out2 = run_with_seed(&p2, 0);
  let allowed: HashSet<_> = ["a", "1", "b"].into_iter().collect();
  assert!(allowed.contains(out1.as_str()), "got {}", out1);
  assert!(allowed.contains(out2.as_str()), "got {}", out2);
}

#[test]
fn compile_then_run_equals_source_run() {
  let bin = env!("CARGO_BIN_EXE_branchy");
  let temp = std::env::temp_dir().join("branchy_compile_test");
  let _ = std::fs::create_dir_all(&temp);
  let source = temp.join("single.branchy");
  let binary = temp.join("single.branchyc");
  std::fs::write(&source, "[ only; ]").unwrap();
  let compile_ok = Command::new(bin)
    .args([
      "compile",
      source.to_str().unwrap(),
      "-o",
      binary.to_str().unwrap(),
    ])
    .output()
    .unwrap();
  assert!(
    compile_ok.status.success(),
    "compile failed: {:?}",
    String::from_utf8_lossy(&compile_ok.stderr)
  );
  let run_binary = Command::new(bin)
    .args(["run", binary.to_str().unwrap()])
    .output()
    .unwrap();
  assert!(
    run_binary.status.success(),
    "run .branchyc failed: {:?}",
    String::from_utf8_lossy(&run_binary.stderr)
  );
  let out_binary = String::from_utf8_lossy(&run_binary.stdout)
    .trim()
    .to_string();
  let run_source = Command::new(bin)
    .args(["run", source.to_str().unwrap()])
    .output()
    .unwrap();
  assert!(
    run_source.status.success(),
    "run .branchy failed: {:?}",
    String::from_utf8_lossy(&run_source.stderr)
  );
  let out_source = String::from_utf8_lossy(&run_source.stdout)
    .trim()
    .to_string();
  assert_eq!(out_binary, "only", "run .branchyc output");
  assert_eq!(out_source, "only", "run .branchy output");
  assert_eq!(out_binary, out_source, "compile+run must match run source");
}
