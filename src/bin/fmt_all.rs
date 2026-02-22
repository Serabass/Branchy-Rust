//! One-off: format all examples/*.branchy with default options.
//! Skips files that fail to parse and reports them.

use branchy::{format_program, parse_program, FormatOptions};
use std::fs;
use std::path::Path;

fn main() -> Result<(), String> {
  let examples_dir = Path::new("examples");
  let mut names: Vec<String> = fs::read_dir(examples_dir)
    .map_err(|e| e.to_string())?
    .filter_map(|e| e.ok())
    .map(|e| e.path())
    .filter(|p| p.extension().map_or(false, |e| e == "branchy"))
    .filter_map(|p| p.file_stem().map(|s| s.to_string_lossy().into_owned()))
    .collect();
  names.sort();
  let opts = FormatOptions::default();
  let mut failed = Vec::new();
  for name in names {
    let path = examples_dir.join(format!("{}.branchy", name));
    let src = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let program = match parse_program(&src) {
      Ok(p) => p,
      Err(e) => {
        eprintln!("skip {}: {}", path.display(), e);
        failed.push((path.display().to_string(), e.to_string()));
        continue;
      }
    };
    let formatted = format_program(&program, &opts);
    fs::write(&path, formatted).map_err(|e| e.to_string())?;
    println!("formatted {}", path.display());
  }
  if !failed.is_empty() {
    eprintln!("{} file(s) skipped (parse error)", failed.len());
  }
  Ok(())
}
