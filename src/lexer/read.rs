use crate::ast::SourceError;

use super::cursor::OffsetCursor;
use super::err::err_at;
use super::ident::is_ident_cont;

pub(super) fn read_quoted_offset(
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
      if let Some((next_off, n)) = cur.next() {
        last_end = next_off + n.len_utf8();
        match n {
          'n' => s.push('\n'),
          't' => s.push('\t'),
          'r' => s.push('\r'),
          '\\' => s.push('\\'),
          _ if n == end => s.push(n),
          _ => { s.push(c); s.push(n); }
        }
      }
    } else if c == end {
      return Ok((s, last_end));
    } else {
      s.push(c);
    }
  }
  Err(err_at(input, start, cur.position(), "unterminated string"))
}

pub(super) fn read_number_offset(
  first: char,
  cur: &mut OffsetCursor<'_>,
  start: usize,
) -> (i64, usize) {
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

pub(super) fn read_ident_from_offset(
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
