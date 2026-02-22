//! Tests for branchy fmt --check.

use branchy::{format_program, parse_program, FormatOptions};
use std::process::Command;

#[test]
fn fmt_check_formatted_file_ok() {
  let bin = env!("CARGO_BIN_EXE_branchy");
  let temp = std::env::temp_dir().join("branchy_fmt_check_ok");
  let _ = std::fs::create_dir_all(&temp);
  let path = temp.join("formatted.branchy");
  let program = parse_program("[ a; b; c; ]").unwrap();
  let canonical = format_program(&program, &FormatOptions::default());
  std::fs::write(&path, &canonical).unwrap();
  let out = Command::new(bin)
    .args(["fmt", path.to_str().unwrap(), "--check"])
    .output()
    .unwrap();
  assert!(
    out.status.success(),
    "fmt --check on formatted file should succeed: stderr={} stdout={}",
    String::from_utf8_lossy(&out.stderr),
    String::from_utf8_lossy(&out.stdout)
  );
}

#[test]
fn fmt_check_unformatted_file_fails() {
  let bin = env!("CARGO_BIN_EXE_branchy");
  let temp = std::env::temp_dir().join("branchy_fmt_check_fail");
  let _ = std::fs::create_dir_all(&temp);
  let path = temp.join("messy.branchy");
  let unformatted = "[a;b;c;]";
  std::fs::write(&path, unformatted).unwrap();
  let out = Command::new(bin)
    .args(["fmt", path.to_str().unwrap(), "--check"])
    .output()
    .unwrap();
  assert!(
    !out.status.success(),
    "fmt --check on unformatted file should fail"
  );
  let stderr = String::from_utf8_lossy(&out.stderr);
  assert!(
    stderr.contains("not formatted") || String::from_utf8_lossy(&out.stdout).contains("not formatted"),
    "expected 'not formatted' in output, got stderr: {}",
    stderr
  );
}
