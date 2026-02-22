//! Run and compile commands for CLI.

use branchy::{
  default_registry, deserialize_program, interpret, parse_program, resolve_includes,
  serialize_program,
};
use rand::rngs::StdRng;
use rand::SeedableRng;
use std::fs;
use std::path::Path;

pub fn run(path: &str, input: Option<&str>, seed: Option<u64>) -> Result<(), String> {
  let bytes = fs::read(path).map_err(|e| e.to_string())?;
  let program = if path.ends_with(".branchyc") || (bytes.len() >= 4 && &bytes[0..4] == b"BRCH") {
    deserialize_program(&bytes)?
  } else {
    let src = String::from_utf8(bytes).map_err(|e| e.to_string())?;
    let p = parse_program(&src).map_err(|e| e.to_string())?;
    let base = Path::new(path).parent().unwrap_or(Path::new("."));
    resolve_includes(p, |pth| {
      let full = base.join(pth);
      fs::read_to_string(&full).map_err(|e| e.to_string())
    })?
  };
  let builtins = default_registry();
  let mut rng: StdRng = match seed {
    Some(s) => StdRng::seed_from_u64(s),
    None => StdRng::seed_from_u64(rand::random::<u64>()),
  };
  let (result, _trace) =
    interpret(&program, &builtins, &mut rng, input).map_err(|e| e.to_string())?;
  println!("{}", result);
  Ok(())
}

pub fn compile(input: &str, output: &str) -> Result<(), String> {
  let src = fs::read_to_string(input).map_err(|e| e.to_string())?;
  let program = parse_program(&src).map_err(|e| e.to_string())?;
  let base = Path::new(input).parent().unwrap_or(Path::new("."));
  let program = resolve_includes(program, |pth| {
    let full = base.join(pth);
    fs::read_to_string(&full).map_err(|e| e.to_string())
  })?;
  let bytes = serialize_program(&program)?;
  fs::write(output, bytes).map_err(|e| e.to_string())?;
  Ok(())
}
