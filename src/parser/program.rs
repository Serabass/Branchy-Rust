//! Top-level program, event and function definition parsing.

use crate::ast::{Event, EventMatcher, FunctionDef, Node, Program, SourceError};
use crate::lexer::{tokenize_with_offsets, Token};

use super::expr;
use super::stream::{self, TokenIter};

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

pub(crate) fn parse_event_def(it: &mut TokenIter) -> Result<Option<Event>, SourceError> {
  let (matcher, body) = match it.peek() {
    Some(Token::At) => {
      it.next();
      let name = stream::expect_ident(it)?;
      stream::expect(it, Token::Equals)?;
      let body = expr::parse_branch(it)?;
      (EventMatcher::ByName(name), body)
    }
    Some(Token::Str(s)) => {
      let s = s.clone();
      it.next();
      stream::expect(it, Token::Equals)?;
      let body = expr::parse_branch(it)?;
      (EventMatcher::ByStr(s), body)
    }
    Some(Token::Tilde) => {
      it.next();
      let pattern = match it.next() {
        Some(Token::Str(s)) => s,
        other => {
          return Err(stream::err_span(
            it,
            format!("expected regex string after ~, got {:?}", other),
          ))
        }
      };
      stream::expect(it, Token::Equals)?;
      let body = expr::parse_branch(it)?;
      (EventMatcher::ByRegex(pattern), body)
    }
    _ => return Ok(None),
  };
  stream::skip_semicolon(it);
  Ok(Some(Event { matcher, body }))
}

pub(crate) fn parse_function_def(it: &mut TokenIter) -> Result<Option<FunctionDef>, SourceError> {
  match it.peek() {
    Some(Token::Bang) => {}
    _ => return Ok(None),
  }
  it.next();
  let name = stream::expect_ident(it)?;
  stream::expect(it, Token::LParen)?;
  let mut params = Vec::new();
  loop {
    if !matches!(it.peek(), Some(Token::Param(_))) {
      break;
    }
    if let Some(Token::Param(p)) = it.next() {
      params.push(p);
    }
    if !matches!(it.peek(), Some(Token::Comma)) {
      break;
    }
    it.next();
  }
  stream::expect(it, Token::RParen)?;
  stream::expect(it, Token::Equals)?;
  let body = expr::parse_value(it)?;
  stream::skip_semicolon(it);
  Ok(Some(FunctionDef { name, params, body }))
}
