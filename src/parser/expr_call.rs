//! Function call, ident/option/call, block and value parsing.

use crate::ast::{CallBlock, Literal, Node, SourceError};
use crate::lexer::Token;

use super::expr::{parse_branch, parse_expression};
use super::stream::{self, TokenIter};

pub(crate) fn parse_func_call(it: &mut TokenIter) -> Result<Node, SourceError> {
  it.start_span();
  it.next();
  let name = stream::expect_ident(it)?;
  stream::expect(it, Token::LParen)?;
  let mut args = Vec::new();
  loop {
    if matches!(it.peek(), Some(Token::RParen)) {
      break;
    }
    args.push(parse_expression(it)?);
    if !matches!(it.peek(), Some(Token::Comma)) {
      break;
    }
    it.next();
  }
  stream::expect(it, Token::RParen)?;
  Ok(Node::FuncCall {
    name,
    args,
    span: it.get_span(),
  })
}

pub(crate) fn parse_ident_start(it: &mut TokenIter) -> Result<Node, SourceError> {
  it.start_span();
  let name = stream::expect_ident(it)?;
  match it.peek() {
    Some(Token::LAngle) => {
      it.next();
      let mut options = Vec::new();
      loop {
        options.push(parse_expression(it)?);
        if !matches!(it.peek(), Some(Token::Pipe)) {
          break;
        }
        it.next();
      }
      stream::expect(it, Token::RAngle)?;
      Ok(Node::InlineCall {
        name,
        options,
        span: it.get_span(),
      })
    }
    Some(Token::Param(_)) | Some(Token::OptionalParam(_)) | Some(Token::LBrace) => {
      let mut params = Vec::new();
      let mut optional_params = std::collections::HashSet::new();
      while matches!(
        it.peek(),
        Some(Token::Param(_)) | Some(Token::OptionalParam(_))
      ) {
        match it.next() {
          Some(Token::Param(p)) => params.push(p),
          Some(Token::OptionalParam(p)) => {
            params.push(p.clone());
            optional_params.insert(p);
          }
          _ => {}
        }
      }
      let block = if matches!(it.peek(), Some(Token::LBrace)) {
        Some(parse_block(it)?)
      } else {
        None
      };
      Ok(Node::Call {
        name,
        params,
        optional_params,
        block,
        span: it.get_span(),
      })
    }
    _ => Ok(Node::Leaf {
      lit: Literal::Ident(name),
      span: it.get_span(),
    }),
  }
}

pub(crate) fn parse_block(it: &mut TokenIter) -> Result<CallBlock, SourceError> {
  stream::expect(it, Token::LBrace)?;
  let mut bindings = Vec::new();
  loop {
    stream::skip_semicolon(it);
    if matches!(it.peek(), Some(Token::RBrace)) {
      it.next();
      return Ok(CallBlock { bindings });
    }
    let param = match it.next() {
      Some(Token::Param(p)) => p,
      _ => return Err(stream::err_span(it, "expected :param in block")),
    };
    stream::expect(it, Token::Equals)?;
    let value = parse_value(it)?;
    bindings.push((param, value));
    stream::skip_semicolon(it);
  }
}

pub(crate) fn parse_value(it: &mut TokenIter) -> Result<Node, SourceError> {
  match it.peek() {
    Some(Token::LBrack) => parse_branch(it),
    _ => parse_expression(it),
  }
}
