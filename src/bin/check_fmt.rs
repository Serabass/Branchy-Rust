//! Check that all examples/*.branchy are formatted (same as branchy fmt --check per file).
//! Exit 1 if any file would be reformatted; skip parse errors and report at end.

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
  let mut not_formatted = Vec::new();
  let mut parse_errors = Vec::new();
  for name in names {
    let path = examples_dir.join(format!("{}.branchy", name));
    let src = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let program = match parse_program(&src) {
      Ok(p) => p,
      Err(e) => {
        parse_errors.push((path.display().to_string(), e.to_string()));
        continue;
      }
    };
    let formatted = format_program(&program, &opts);
    if formatted != src {
      not_formatted.push(path.display().to_string());
    }
  }
  for (path, err) in &parse_errors {
    eprintln!("skip {}: {}", path, err);
  }
  if !not_formatted.is_empty() {
    for p in &not_formatted {
      eprintln!("not formatted: {}", p);
    }
    eprintln!(
      "{} file(s) need formatting (run: docker-compose run --rm --entrypoint cargo app run --bin fmt_all)",
      not_formatted.len()
    );
    std::process::exit(1);
  }
  if !parse_errors.is_empty() {
    eprintln!("{} file(s) skipped (parse error)", parse_errors.len());
  }
  Ok(())
}
