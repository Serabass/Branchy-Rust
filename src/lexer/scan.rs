use crate::ast::SourceError;

use super::cursor::OffsetCursor;
use super::err::err_at;
use super::ident::is_ident_start;
use super::scan_bracket;
use super::scan_rest;
use super::token::Token;

pub fn tokenize_with_offsets(input: &str) -> Result<Vec<(Token, usize, usize)>, SourceError> {
  fn rec(
    input: &str,
    start: usize,
    end: usize,
  ) -> Result<Vec<(Token, usize, usize)>, SourceError> {
    tokenize_range_rec(input, start, end, &rec)
  }
  rec(input, 0, input.len())
}

fn tokenize_range_rec<F>(
  input: &str,
  start: usize,
  end: usize,
  recur: &F,
) -> Result<Vec<(Token, usize, usize)>, SourceError>
where
  F: Fn(&str, usize, usize) -> Result<Vec<(Token, usize, usize)>, SourceError>,
{
  let s = &input[start..end];
  let mut out = Vec::new();
  let mut cur = OffsetCursor::new(s, start);
  while let Some((off, c)) = cur.next() {
    let token_start = off;
    let token_end = off + c.len_utf8();
    match c {
      ' ' | '\t' | '\n' | '\r' => continue,
      '[' => scan_bracket::handle_lbrack(input, &mut cur, token_start, &mut out, recur)?,
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
      '.' => scan_rest::handle_dots(input, &mut cur, token_start, &mut out)?,
      ':' => scan_rest::handle_colon(input, &mut cur, token_start, token_end, &mut out)?,
      '"' => scan_rest::handle_quote(input, &mut cur, token_start, '"', &mut out)?,
      '\'' => scan_rest::handle_quote(input, &mut cur, token_start, '\'', &mut out)?,
      c if c.is_ascii_digit() => scan_rest::handle_digit(c, &mut cur, token_start, &mut out),
      c if is_ident_start(c) => scan_rest::handle_ident(c, &mut cur, token_start, &mut out),
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
