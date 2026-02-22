//! Parsing of char block content: [a-z], [a-z:5], [a-z:1..3].

use crate::ast::CharBlockCount;

pub(crate) fn parse_char_block_content(
  s: &str,
) -> Result<(Vec<(char, char)>, CharBlockCount), String> {
  let (set_part, count_part) = match s.find(':') {
    Some(i) => {
      let set = s[..i].trim();
      let count_str = s[i + 1..].trim();
      (set, Some(count_str))
    }
    None => (s.trim(), None),
  };
  let ranges = parse_char_set(set_part)?;
  if ranges.is_empty() {
    return Err("char block set is empty".into());
  }
  let count = match count_part {
    None => CharBlockCount::One,
    Some(cs) => {
      if let Some(dot) = cs.find("..") {
        let lo: i64 = cs[..dot]
          .trim()
          .parse()
          .map_err(|_| "invalid range start in char block")?;
        let hi: i64 = cs[dot + 2..]
          .trim()
          .parse()
          .map_err(|_| "invalid range end in char block")?;
        CharBlockCount::Range(lo, hi)
      } else {
        let n: i64 = cs.parse().map_err(|_| "invalid count in char block")?;
        CharBlockCount::Fixed(n)
      }
    }
  };
  Ok((ranges, count))
}

fn parse_char_set(s: &str) -> Result<Vec<(char, char)>, String> {
  let mut ranges = Vec::new();
  let mut it = s.chars();
  let mut prev: Option<char> = None;
  while let Some(c) = it.next() {
    if c == '-' {
      let c1 = prev
        .take()
        .ok_or("char set: '-' at start or after another '-'")?;
      let c2 = it.next().ok_or("char set: expected character after '-'")?;
      if c1 as u32 <= c2 as u32 {
        ranges.push((c1, c2));
      } else {
        ranges.push((c2, c1));
      }
    } else {
      if let Some(p) = prev {
        ranges.push((p, p));
      }
      prev = Some(c);
    }
  }
  if let Some(p) = prev {
    ranges.push((p, p));
  }
  Ok(ranges)
}
