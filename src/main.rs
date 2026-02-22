use branchy::{
  default_registry, deserialize_program, interpret, parse_program, resolve_includes,
  serialize_program,
};
use rand::rngs::StdRng;
use rand::SeedableRng;
use std::env;
use std::fs;
use std::path::Path;

fn main() -> Result<(), String> {
  let args: Vec<String> = env::args().collect();
  if args.len() < 2 {
    eprintln!("Usage: branchy run <file.branchy|file.branchyc> [input] [--seed N]");
    eprintln!("       branchy compile <file.branchy> -o <file.branchyc>");
    std::process::exit(1);
  }
  let sub = &args[1];
  match sub.as_str() {
    "run" => {
      if args.len() < 3 {
        return Err("branchy run <file> [input] [--seed N]".into());
      }
      let (input, seed) = parse_run_args(&args[3..])?;
      run(&args[2], input, seed)
    }
    "compile" => {
      let mut input = None;
      let mut output = None;
      let mut i = 2;
      while i < args.len() {
        if args[i] == "-o" {
          i += 1;
          if i < args.len() {
            output = Some(args[i].clone());
            i += 1;
          }
        } else {
          input = Some(args[i].clone());
          i += 1;
        }
      }
      let inp = input.ok_or("branchy compile <input.branchy> -o <output.branchyc>")?;
      let out = output.ok_or("branchy compile <input.branchy> -o <output.branchyc>")?;
      compile(&inp, &out)
    }
    _ => run(sub, None, None),
  }
}

/// Parse run args: [input] [--seed N]. Returns (input, seed: None = random).
fn parse_run_args(args: &[String]) -> Result<(Option<&str>, Option<u64>), String> {
  let mut input = None;
  let mut seed = None;
  let mut i = 0;
  while i < args.len() {
    if args[i] == "--seed" || args[i] == "-s" {
      i += 1;
      let s = args.get(i).ok_or("--seed requires a number")?;
      let n: u64 = s.parse().map_err(|_| format!("invalid seed: {}", s))?;
      seed = Some(n);
      i += 1;
    } else if input.is_none() {
      input = Some(args[i].as_str());
      i += 1;
    } else {
      return Err("unexpected argument".into());
    }
  }
  Ok((input, seed))
}

fn run(path: &str, input: Option<&str>, seed: Option<u64>) -> Result<(), String> {
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

fn compile(input: &str, output: &str) -> Result<(), String> {
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
