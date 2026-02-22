use std::iter::Peekable;
use std::str::Chars;

use super::ident::{is_ident_cont, is_ident_start};
use super::token::Token;

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
                } else if super::ident::is_char_block_content(&s) {
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

fn read_ident(it: &mut Peekable<Chars>) -> Result<String, String> {
  let c = it.next().ok_or("expected identifier after ':'")?;
  if !is_ident_start(c) {
    return Err("invalid param name".into());
  }
  Ok(read_ident_from(c, it))
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
