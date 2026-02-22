//! Top-level program parsing.

use crate::ast::{Node, Program, SourceError};
use crate::lexer::{tokenize_with_offsets, Token};

use super::defs::{parse_event_def, parse_function_def};
use super::expr;
use super::stream;

pub fn parse_program(input: &str) -> Result<Program, SourceError> {
  let tokens = tokenize_with_offsets(input)?;
  let mut it = stream::SpanStream::new(tokens, input);
  let mut includes = Vec::new();
  while matches!(it.peek(), Some(Token::Include)) {
    it.next();
    let path = match it.next() {
      Some(Token::Str(s)) => s,
      _ => return Err(stream::err_span(&it, "expected string path after include")),
    };
    includes.push(path);
    stream::skip_semicolon(&mut it);
  }
  let mut functions = Vec::new();
  let mut events = Vec::new();
  loop {
    if let Some(f) = parse_function_def(&mut it)? {
      functions.push(f);
    } else if let Some(e) = parse_event_def(&mut it)? {
      events.push(e);
    } else {
      break;
    }
  }
  let mut main_branches = Vec::new();
  while matches!(it.peek(), Some(Token::LBrack)) {
    main_branches.push(expr::parse_branch(&mut it)?);
    stream::skip_semicolon(&mut it);
  }
  let main = match main_branches.len() {
    0 => return Err(stream::err_span(&it, "expected at least one main branch")),
    1 => main_branches.into_iter().next().unwrap(),
    _ => Node::Branch {
      children: main_branches,
      span: None,
    },
  };
  if it.next().is_some() {
    return Err(stream::err_span(&it, "unexpected tokens after main branch"));
  }
  Ok(Program {
    includes,
    functions,
    events,
    main,
  })
}
