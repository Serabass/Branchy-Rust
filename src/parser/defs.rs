//! Event and function definition parsing.

use crate::ast::{Event, EventMatcher, FunctionDef, SourceError};
use crate::lexer::Token;

use super::expr;
use super::stream::{self, TokenIter};

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
