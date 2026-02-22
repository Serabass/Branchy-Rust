//! CLI entry: run | compile.

use std::env;

mod run;

fn main() -> Result<(), String> {
  let args: Vec<String> = env::args().collect();
  if args.len() < 2 {
    eprintln!("Usage: branchy run <file.branchy|file.branchyc> [input] [--seed N]");
    eprintln!("       branchy compile <file.branchy> -o <file.branchyc>");
    eprintln!("       branchy fmt [path] [-w|--write] [-c|--check]");
    std::process::exit(1);
  }
  let sub = &args[1];
  match sub.as_str() {
    "run" => {
      if args.len() < 3 {
        return Err("branchy run <file> [input] [--seed N]".into());
      }
      let (input, seed) = parse_run_args(&args[3..])?;
      run::run(&args[2], input, seed)
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
      run::compile(&inp, &out)
    }
    "fmt" | "format" => {
      let mut path = None;
      let mut write = false;
      let mut check = false;
      let mut i = 2;
      while i < args.len() {
        if args[i] == "-w" || args[i] == "--write" {
          write = true;
          i += 1;
        } else if args[i] == "-c" || args[i] == "--check" {
          check = true;
          i += 1;
        } else if path.is_none() {
          path = Some(args[i].as_str());
          i += 1;
        } else {
          return Err("branchy fmt: unexpected argument".into());
        }
      }
      run::fmt(path, write, check)
    }
    _ => run::run(sub, None, None),
  }
}

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
