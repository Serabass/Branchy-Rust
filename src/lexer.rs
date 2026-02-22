use crate::ast::{span_from_offsets, SourceError};
use std::iter::Peekable;
use std::str::Chars;

fn err_at(input: &str, start: usize, end: usize, message: impl Into<String>) -> SourceError {
  SourceError {
    message: message.into(),
    span: Some(span_from_offsets(input, start, end)),
  }
}

/// Cursor over a string slice that yields (global_byte_offset, char).
struct OffsetCursor<'a> {
  s: &'a str,
  base: usize,
  pos: usize,
}

impl<'a> OffsetCursor<'a> {
  fn new(s: &'a str, base: usize) -> Self {
    Self { s, base, pos: 0 }
  }
  fn next(&mut self) -> Option<(usize, char)> {
    let c = self.s[self.pos..].chars().next()?;
    let start = self.base + self.pos;
    self.pos += c.len_utf8();
    Some((start, c))
  }
  fn peek(&self) -> Option<(usize, char)> {
    let c = self.s[self.pos..].chars().next()?;
    Some((self.base + self.pos, c))
  }
  fn position(&self) -> usize {
    self.base + self.pos
  }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
  LBrack,
  RBrack,
  Semicolon,
  LBrace,
  RBrace,
  Equals,
  LParen,
  RParen,
  Comma,
  LAngle,
  RAngle,
  Pipe,
  Bang,
  At,
  Tilde,
  Plus,
  Star,
  Spread,
  /// Two dots `..` for range (e.g. 1..3); three dots are Spread
  RangeSep,
  Include,
  Ident(String),
  Param(String),
  /// Optional param :?name — may or may not be output (coin flip)
  OptionalParam(String),
  Num(i64),
  Str(String),
  /// Inline char block: [a-zA-Z], [abc:5], [a-z:2..5]. Content is "set" or "set:n" or "set:lo..hi".
  CharBlock(String),
}

/// Tokenize and return each token with (start_byte, end_byte) in the source.
pub fn tokenize_with_offsets(input: &str) -> Result<Vec<(Token, usize, usize)>, SourceError> {
  tokenize_range(input, 0, input.len())
}

fn tokenize_range(
  input: &str,
  start: usize,
  end: usize,
) -> Result<Vec<(Token, usize, usize)>, SourceError> {
  let s = &input[start..end];
  let mut out = Vec::new();
  let mut cur = OffsetCursor::new(s, start);
  while let Some((off, c)) = cur.next() {
    let token_start = off;
    let token_end = off + c.len_utf8();
    match c {
      ' ' | '\t' | '\n' | '\r' => continue,
      '[' => {
        let content_start = cur.position();
        let mut depth = 1u32;
        let mut content_end = content_start;
        loop {
          let (off2, c2) = match cur.next() {
            Some(p) => p,
            None => {
              return Err(err_at(
                input,
                token_start,
                cur.position(),
                "unclosed [ (expected ] or ;)",
              ))
            }
          };
          content_end = off2;
          match c2 {
            '[' => depth += 1,
            ']' => {
              depth -= 1;
              if depth == 0 {
                let content = &input[content_start..content_end];
                let trimmed = content.trim();
                if trimmed.is_empty() {
                  out.push((Token::LBrack, token_start, token_start + 1));
                  out.push((Token::RBrack, off2, off2 + 1));
                } else if is_char_block_content(trimmed) {
                  let full_end = off2 + 1;
                  out.push((Token::CharBlock(trimmed.to_string()), token_start, full_end));
                } else {
                  out.push((Token::LBrack, token_start, token_start + 1));
                  out.extend(tokenize_range(input, content_start, content_end)?);
                  out.push((Token::RBrack, off2, off2 + 1));
                }
                break;
              }
            }
            ';' if depth == 1 => {
              out.push((Token::LBrack, token_start, token_start + 1));
              out.extend(tokenize_range(input, content_start, content_end)?);
              out.push((Token::Semicolon, off2, off2 + 1));
              break;
            }
            _ => {}
          }
        }
      }
      ']' => out.push((Token::RBrack, token_start, token_end)),
      ';' => out.push((Token::Semicolon, token_start, token_end)),
      '{' => out.push((Token::LBrace, token_start, token_end)),
      '}' => out.push((Token::RBrace, token_start, token_end)),
      '=' => out.push((Token::Equals, token_start, token_end)),
      '(' => out.push((Token::LParen, token_start, token_end)),
      ')' => out.push((Token::RParen, token_start, token_end)),
      ',' => out.push((Token::Comma, token_start, token_end)),
      '<' => out.push((Token::LAngle, token_start, token_end)),
      '>' => out.push((Token::RAngle, token_start, token_end)),
      '|' => out.push((Token::Pipe, token_start, token_end)),
      '!' => out.push((Token::Bang, token_start, token_end)),
      '@' => out.push((Token::At, token_start, token_end)),
      '~' => out.push((Token::Tilde, token_start, token_end)),
      '+' => out.push((Token::Plus, token_start, token_end)),
      '*' => out.push((Token::Star, token_start, token_end)),
      '.' => {
        let (off2, c2) = cur
          .next()
          .ok_or_else(|| err_at(input, token_start, cur.position(), "expected .. or ..."))?;
        if c2 != '.' {
          return Err(err_at(
            input,
            token_start,
            off2 + c2.len_utf8(),
            "expected .. or ...",
          ));
        }
        let (end_off, tok) = match cur.peek() {
          Some((off3, '.')) => {
            let _ = cur.next(); // consume third dot
            (off3 + 1, Token::Spread)
          }
          Some(_) => (off2 + 1, Token::RangeSep),
          None => {
            return Err(err_at(
              input,
              token_start,
              cur.position(),
              "expected .. or ...",
            ))
          }
        };
        out.push((tok, token_start, end_off));
      }
      ':' => {
        // ':' already consumed by the main loop
        let opt_off = cur.peek().map(|(o, _)| o);
        let optional = cur.peek().map(|(_, c)| c) == Some('?');
        if optional {
          cur.next();
        }
        let (name_start, first_c) = cur.next().ok_or_else(|| {
          err_at(
            input,
            token_start,
            token_end,
            "expected identifier after ':'",
          )
        })?;
        if !is_ident_start(first_c) {
          return Err(err_at(
            input,
            name_start,
            name_start + first_c.len_utf8(),
            "invalid param name",
          ));
        }
        let (name, end_off) = read_ident_from_offset(first_c, &mut cur, name_start);
        let from = opt_off.unwrap_or(token_start);
        let tok = if optional {
          Token::OptionalParam(name)
        } else {
          Token::Param(name)
        };
        out.push((tok, from, end_off));
      }
      '"' => {
        let (s, end_off) = read_quoted_offset(&mut cur, '"', token_start, input)?;
        out.push((Token::Str(s), token_start, end_off));
      }
      '\'' => {
        let (s, end_off) = read_quoted_offset(&mut cur, '\'', token_start, input)?;
        out.push((Token::Str(s), token_start, end_off));
      }
      c if c.is_ascii_digit() => {
        let (num, end_off) = read_number_offset(c, &mut cur, token_start);
        out.push((Token::Num(num), token_start, end_off));
      }
      c if is_ident_start(c) => {
        let (ident, end_off) = read_ident_from_offset(c, &mut cur, token_start);
        let tok = if ident == "include" {
          Token::Include
        } else {
          Token::Ident(ident)
        };
        out.push((tok, token_start, end_off));
      }
      _ => {
        return Err(err_at(
          input,
          token_start,
          token_end,
          format!("unexpected character: {}", c),
        ))
      }
    }
  }
  Ok(out)
}

fn read_quoted_offset(
  cur: &mut OffsetCursor<'_>,
  end: char,
  start: usize,
  input: &str,
) -> Result<(String, usize), SourceError> {
  let mut s = String::new();
  let mut last_end = start + 1;
  while let Some((off, c)) = cur.next() {
    last_end = off + c.len_utf8();
    if c == '\\' {
      if let Some((_, n)) = cur.next() {
        s.push(if n == end || n == '\\' { n } else { c });
      }
    } else if c == end {
      return Ok((s, last_end));
    } else {
      s.push(c);
    }
  }
  Err(err_at(input, start, cur.position(), "unterminated string"))
}

fn read_number_offset(first: char, cur: &mut OffsetCursor<'_>, start: usize) -> (i64, usize) {
  let mut s = String::from(first);
  let mut last_end = start + first.len_utf8();
  while let Some((off, c)) = cur.peek() {
    if c.is_ascii_digit() {
      cur.next();
      s.push(c);
      last_end = off + c.len_utf8();
    } else {
      break;
    }
  }
  (s.parse().unwrap_or(0), last_end)
}

fn read_ident_from_offset(
  first: char,
  cur: &mut OffsetCursor<'_>,
  start: usize,
) -> (String, usize) {
  let mut s = String::from(first);
  let mut last_end = start + first.len_utf8();
  while let Some((off, c)) = cur.peek() {
    if is_ident_cont(c) {
      cur.next();
      s.push(c);
      last_end = off + c.len_utf8();
    } else {
      break;
    }
  }
  (s, last_end)
}

pub fn tokenize(input: &str) -> Result<Vec<Token>, String> {
  let mut tokens = Vec::new();
  let mut it = input.chars().peekable();
  while let Some(c) = it.next() {
    match c {
      '[' => {
        let mut buf = String::new();
        let mut depth = 1u32;
        loop {
          match it.next() {
            Some('[') => {
              depth += 1;
              buf.push('[');
            }
            Some(']') => {
              depth -= 1;
              if depth == 0 {
                let s = buf.trim().to_string();
                if s.is_empty() {
                  tokens.push(Token::LBrack);
                  tokens.push(Token::RBrack);
                } else if is_char_block_content(&s) {
                  tokens.push(Token::CharBlock(s));
                } else {
                  tokens.push(Token::LBrack);
                  tokens.extend(tokenize(&buf)?);
                  tokens.push(Token::RBrack);
                }
                break;
              }
              buf.push(']');
            }
            Some(';') if depth == 1 => {
              tokens.push(Token::LBrack);
              tokens.extend(tokenize(&buf)?);
              tokens.push(Token::Semicolon);
              break;
            }
            Some(c) => buf.push(c),
            None => return Err("unclosed [ (expected ] or ;)".into()),
          }
        }
      }
      ']' => tokens.push(Token::RBrack),
      ';' => tokens.push(Token::Semicolon),
      '{' => tokens.push(Token::LBrace),
      '}' => tokens.push(Token::RBrace),
      '=' => tokens.push(Token::Equals),
      '(' => tokens.push(Token::LParen),
      ')' => tokens.push(Token::RParen),
      ',' => tokens.push(Token::Comma),
      '<' => tokens.push(Token::LAngle),
      '>' => tokens.push(Token::RAngle),
      '|' => tokens.push(Token::Pipe),
      '!' => tokens.push(Token::Bang),
      '@' => tokens.push(Token::At),
      '~' => tokens.push(Token::Tilde),
      '+' => tokens.push(Token::Plus),
      '*' => tokens.push(Token::Star),
      '.' => match it.next() {
        Some('.') => {
          if it.peek() == Some(&'.') {
            it.next();
            tokens.push(Token::Spread);
          } else {
            tokens.push(Token::RangeSep);
          }
        }
        _ => return Err("expected .. or ...".into()),
      },
      ':' => {
        let optional = it.peek() == Some(&'?');
        if optional {
          it.next();
        }
        let name = read_ident(&mut it)?;
        if optional {
          tokens.push(Token::OptionalParam(name));
        } else {
          tokens.push(Token::Param(name));
        }
      }
      '"' => {
        let s = read_quoted(&mut it, '"')?;
        tokens.push(Token::Str(s));
      }
      '\'' => {
        let s = read_quoted(&mut it, '\'')?;
        tokens.push(Token::Str(s));
      }
      c if c.is_whitespace() => {}
      c if c.is_ascii_digit() => {
        let num = read_number(c, &mut it);
        tokens.push(Token::Num(num));
      }
      c if is_ident_start(c) => {
        let s = read_ident_from(c, &mut it);
        if s == "include" {
          tokens.push(Token::Include);
        } else {
          tokens.push(Token::Ident(s));
        }
      }
      _ => return Err(format!("unexpected character: {}", c)),
    }
  }
  Ok(tokens)
}

fn read_quoted(it: &mut Peekable<Chars>, end: char) -> Result<String, String> {
  let mut s = String::new();
  while let Some(c) = it.next() {
    if c == '\\' {
      if let Some(n) = it.next() {
        s.push(if n == end || n == '\\' { n } else { c });
      }
    } else if c == end {
      return Ok(s);
    } else {
      s.push(c);
    }
  }
  Err("unterminated string".into())
}

fn read_number(first: char, it: &mut Peekable<Chars>) -> i64 {
  let mut s = String::from(first);
  while let Some(&c) = it.peek() {
    if c.is_ascii_digit() {
      s.push(it.next().unwrap());
    } else {
      break;
    }
  }
  s.parse().unwrap_or(0)
}

fn is_ident_start(c: char) -> bool {
  c.is_alphabetic() || c == '_' || !c.is_ascii()
}

fn is_ident_cont(c: char) -> bool {
  c.is_alphanumeric() || c == '_' || !c.is_ascii()
}

fn read_ident(it: &mut Peekable<Chars>) -> Result<String, String> {
  let c = it.next().ok_or("expected identifier after ':'")?;
  if !is_ident_start(c) {
    return Err("invalid param name".into());
  }
  Ok(read_ident_from(c, it))
}

fn is_char_block_content(s: &str) -> bool {
  s.chars()
    .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == ':' || c == '.' || c.is_whitespace())
    && !s.trim().is_empty()
}

fn read_ident_from(first: char, it: &mut Peekable<Chars>) -> String {
  let mut s = String::from(first);
  while let Some(&c) = it.peek() {
    if is_ident_cont(c) {
      s.push(it.next().unwrap());
    } else {
      break;
    }
  }
  s
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn tokenize_spread_param_top_level() {
    let t = tokenize_with_offsets("...:x").unwrap();
    let tokens: Vec<_> = t.into_iter().map(|(tok, _, _)| tok).collect();
    assert_eq!(
      tokens,
      [Token::Spread, Token::Param("x".into())],
      "top-level ...:x"
    );
  }

  #[test]
  fn tokenize_with_offsets_matches_tokenize() {
    for input in [
      "[ a; b; ]",
      "!f(:a) = [ x; ]; [ y; ]",
      r#"[ a; ...:x; b; ]"#,
    ] {
      let a = tokenize(input).unwrap();
      let b: Vec<_> = tokenize_with_offsets(input)
        .unwrap()
        .into_iter()
        .map(|(t, _, _)| t)
        .collect();
      assert_eq!(a, b, "input: {:?}", input);
    }
  }
}
