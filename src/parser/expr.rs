//! Expression, element and branch parsing.

use crate::ast::{BinOp, Literal, Node, SourceError};
use crate::lexer::Token;

use super::char_block;
use super::expr_call::{parse_func_call, parse_ident_start};
use super::stream::{self, TokenIter};

pub(crate) use super::expr_call::parse_value;

pub(crate) fn parse_branch(it: &mut TokenIter) -> Result<Node, SourceError> {
  it.start_span();
  stream::expect(it, Token::LBrack)?;
  let mut elements = Vec::new();
  loop {
    if matches!(it.peek(), Some(Token::RBrack)) {
      it.next();
      return Ok(Node::Branch {
        children: elements,
        span: it.get_span(),
      });
    }
    elements.push(parse_expression(it)?);
    stream::skip_semicolon(it);
  }
}

pub(crate) fn parse_expression(it: &mut TokenIter) -> Result<Node, SourceError> {
  let mut left = parse_element(it)?;
  loop {
    let op = match it.peek() {
      Some(Token::Plus) => BinOp::Plus,
      Some(Token::Star) => BinOp::Star,
      _ => break,
    };
    it.next();
    let right = parse_element(it)?;
    let left_span = stream::node_span(&left);
    let right_span = stream::node_span(&right);
    left = Node::BinaryOp {
      op,
      left: Box::new(left),
      right: Box::new(right),
      span: stream::merge_span(left_span, right_span),
    };
  }
  Ok(left)
}

fn parse_element(it: &mut TokenIter) -> Result<Node, SourceError> {
  it.start_span();
  match it.peek() {
    Some(Token::Spread) => {
      it.next();
      match it.peek() {
        Some(Token::Param(p)) => {
          let p = p.clone();
          it.next();
          Ok(Node::SpreadParam {
            param: p,
            span: it.get_span(),
          })
        }
        Some(Token::Include) => {
          it.next();
          let path = match it.next() {
            Some(Token::Str(s)) => s,
            _ => return Err(stream::err_span(it, "expected string after ...include")),
          };
          Ok(Node::SpreadInclude {
            path,
            span: it.get_span(),
          })
        }
        _ => Err(stream::err_span(it, "expected :param or include after ...")),
      }
    }
    Some(Token::Bang) => parse_func_call(it),
    Some(Token::Param(p)) => {
      let p = p.clone();
      it.next();
      Ok(Node::Leaf {
        lit: Literal::Param(p),
        span: it.get_span(),
      })
    }
    Some(Token::OptionalParam(p)) => {
      let p = p.clone();
      it.next();
      Ok(Node::Leaf {
        lit: Literal::OptionalParam(p),
        span: it.get_span(),
      })
    }
    Some(Token::Ident(_)) => parse_ident_start(it),
    Some(Token::Num(lo)) => {
      let lo = *lo;
      it.next();
      if matches!(it.peek(), Some(Token::RangeSep)) {
        it.next();
        match it.next() {
          Some(Token::Num(hi)) => Ok(Node::Leaf {
            lit: Literal::Range(lo, hi),
            span: it.get_span(),
          }),
          _ => Err(stream::err_span(
            it,
            "expected number after .. in range (e.g. 1..3)",
          )),
        }
      } else {
        Ok(Node::Leaf {
          lit: Literal::Num(lo),
          span: it.get_span(),
        })
      }
    }
    Some(Token::Str(s)) => {
      let s = s.clone();
      it.next();
      Ok(Node::Leaf {
        lit: Literal::Str(s),
        span: it.get_span(),
      })
    }
    Some(Token::LBrack) => parse_branch(it),
    Some(Token::CharBlock(s)) => {
      let content = s.clone();
      it.next();
      let (ranges, count) = char_block::parse_char_block_content(&content).map_err(|msg| {
        SourceError {
          message: msg,
          span: it.get_span(),
        }
      })?;
      Ok(Node::CharBlock {
        ranges,
        count,
        span: it.get_span(),
      })
    }
    _ => Err(stream::err_span(it, "expected element")),
  }
}
