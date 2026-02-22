use std::collections::HashMap;

pub type BuiltinFn = fn(&[String]) -> Result<String, String>;

pub fn default_registry() -> HashMap<String, BuiltinFn> {
  let mut m = HashMap::new();
  m.insert("upper".into(), upper as BuiltinFn);
  m.insert("lower".into(), lower as BuiltinFn);
  m.insert("trim".into(), trim as BuiltinFn);
  m.insert("concat".into(), concat as BuiltinFn);
  m.insert("join".into(), join as BuiltinFn);
  m.insert("len".into(), len as BuiltinFn);
  m.insert("replace".into(), replace as BuiltinFn);
  m.insert("split".into(), split_one as BuiltinFn);
  m
}

pub fn upper(args: &[String]) -> Result<String, String> {
  exact_args(1, args)?;
  Ok(args[0].to_uppercase())
}

pub fn lower(args: &[String]) -> Result<String, String> {
  exact_args(1, args)?;
  Ok(args[0].to_lowercase())
}

pub fn trim(args: &[String]) -> Result<String, String> {
  exact_args(1, args)?;
  Ok(args[0].trim().to_string())
}

pub fn concat(args: &[String]) -> Result<String, String> {
  if args.len() < 2 {
    return Err("concat expects at least 2 arguments".into());
  }
  Ok(args.join(""))
}

pub fn join(args: &[String]) -> Result<String, String> {
  if args.len() < 2 {
    return Err("join(sep, ...) expects at least sep and one part".into());
  }
  let sep = &args[0];
  Ok(args[1..].join(sep))
}

pub fn len(args: &[String]) -> Result<String, String> {
  exact_args(1, args)?;
  Ok(args[0].len().to_string())
}

pub fn replace(args: &[String]) -> Result<String, String> {
  exact_args(3, args)?;
  Ok(args[0].replace(&args[1], &args[2]))
}

pub fn split_one(args: &[String]) -> Result<String, String> {
  exact_args(2, args)?;
  let parts: Vec<&str> = args[0].split(&args[1]).collect();
  let idx = 0;
  Ok(parts.get(idx).map(|s| s.to_string()).unwrap_or_default())
}

fn exact_args(n: usize, args: &[String]) -> Result<(), String> {
  if args.len() != n {
    Err(format!("expected {} arguments, got {}", n, args.len()))
  } else {
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_upper() {
    assert_eq!(upper(&["hello".into()]).unwrap(), "HELLO");
  }

  #[test]
  fn test_lower() {
    assert_eq!(lower(&["HELLO".into()]).unwrap(), "hello");
  }

  #[test]
  fn test_trim() {
    assert_eq!(trim(&["  a b  ".into()]).unwrap(), "a b");
  }

  #[test]
  fn test_concat() {
    assert_eq!(concat(&["a".into(), "b".into()]).unwrap(), "ab");
  }

  #[test]
  fn test_join() {
    assert_eq!(
      join(&[", ".into(), "a".into(), "b".into()]).unwrap(),
      "a, b"
    );
  }

  #[test]
  fn test_len() {
    assert_eq!(len(&["hello".into()]).unwrap(), "5");
  }

  #[test]
  fn test_replace() {
    assert_eq!(
      replace(&["hello".into(), "l".into(), "x".into()]).unwrap(),
      "hexxo"
    );
  }

  #[test]
  fn test_split_one() {
    assert_eq!(split_one(&["a,b,c".into(), ",".into()]).unwrap(), "a");
  }
}
