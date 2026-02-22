use crate::ast::SourceError;

use super::cursor::OffsetCursor;
use super::err::err_at;
use super::ident::is_char_block_content;
use super::token::Token;

pub(super) fn handle_lbrack<F>(
  input: &str,
  cur: &mut OffsetCursor<'_>,
  token_start: usize,
  out: &mut Vec<(Token, usize, usize)>,
  recur: F,
) -> Result<(), SourceError>
where
  F: Fn(&str, usize, usize) -> Result<Vec<(Token, usize, usize)>, SourceError>,
{
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
            out.extend(recur(input, content_start, content_end)?);
            out.push((Token::RBrack, off2, off2 + 1));
          }
          break;
        }
      }
      ';' if depth == 1 => {
        out.push((Token::LBrack, token_start, token_start + 1));
        out.extend(recur(input, content_start, content_end)?);
        out.push((Token::Semicolon, off2, off2 + 1));
        break;
      }
      _ => {}
    }
  }
  Ok(())
}
