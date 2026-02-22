use crate::ast::SourceError;

use super::cursor::OffsetCursor;
use super::err::err_at;
use super::ident::is_ident_start;
use super::read::{read_ident_from_offset, read_number_offset, read_quoted_offset};
use super::token::Token;

pub(super) fn handle_dots(
  input: &str,
  cur: &mut OffsetCursor<'_>,
  token_start: usize,
  out: &mut Vec<(Token, usize, usize)>,
) -> Result<(), SourceError> {
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
      let _ = cur.next();
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
  Ok(())
}

pub(super) fn handle_colon(
  input: &str,
  cur: &mut OffsetCursor<'_>,
  token_start: usize,
  token_end: usize,
  out: &mut Vec<(Token, usize, usize)>,
) -> Result<(), SourceError> {
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
  let (name, end_off) = read_ident_from_offset(first_c, cur, name_start);
  let from = opt_off.unwrap_or(token_start);
  let tok = if optional {
    Token::OptionalParam(name)
  } else {
    Token::Param(name)
  };
  out.push((tok, from, end_off));
  Ok(())
}

pub(super) fn handle_quote(
  input: &str,
  cur: &mut OffsetCursor<'_>,
  token_start: usize,
  end: char,
  out: &mut Vec<(Token, usize, usize)>,
) -> Result<(), SourceError> {
  let (s, end_off) = read_quoted_offset(cur, end, token_start, input)?;
  out.push((Token::Str(s), token_start, end_off));
  Ok(())
}

pub(super) fn handle_digit(
  first: char,
  cur: &mut OffsetCursor<'_>,
  token_start: usize,
  out: &mut Vec<(Token, usize, usize)>,
) {
  let (num, end_off) = read_number_offset(first, cur, token_start);
  out.push((Token::Num(num), token_start, end_off));
}

pub(super) fn handle_ident(
  first: char,
  cur: &mut OffsetCursor<'_>,
  token_start: usize,
  out: &mut Vec<(Token, usize, usize)>,
) {
  let (ident, end_off) = read_ident_from_offset(first, cur, token_start);
  let tok = if ident == "include" {
    Token::Include
  } else {
    Token::Ident(ident)
  };
  out.push((tok, token_start, end_off));
}
