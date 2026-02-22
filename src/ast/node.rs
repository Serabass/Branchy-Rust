//! AST node types: Node, Literal, BinOp, CallBlock, CharBlockCount.

use serde::{Deserialize, Serialize};

use super::span::Span;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BinOp {
  Plus,
  Star,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Node {
  Branch {
    children: Vec<Node>,
    #[serde(skip_serializing_if = "Option::is_none")]
    span: Option<Span>,
  },
  Leaf {
    lit: Literal,
    #[serde(skip_serializing_if = "Option::is_none")]
    span: Option<Span>,
  },
  BinaryOp {
    op: BinOp,
    left: Box<Node>,
    right: Box<Node>,
    #[serde(skip_serializing_if = "Option::is_none")]
    span: Option<Span>,
  },
  Call {
    name: String,
    params: Vec<String>,
    #[serde(default)]
    optional_params: std::collections::HashSet<String>,
    block: Option<CallBlock>,
    #[serde(skip_serializing_if = "Option::is_none")]
    span: Option<Span>,
  },
  InlineCall {
    name: String,
    options: Vec<Node>,
    #[serde(skip_serializing_if = "Option::is_none")]
    span: Option<Span>,
  },
  FuncCall {
    name: String,
    args: Vec<Node>,
    #[serde(skip_serializing_if = "Option::is_none")]
    span: Option<Span>,
  },
  SpreadParam {
    param: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    span: Option<Span>,
  },
  SpreadInclude {
    path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    span: Option<Span>,
  },
  CharBlock {
    ranges: Vec<(char, char)>,
    count: CharBlockCount,
    #[serde(skip_serializing_if = "Option::is_none")]
    span: Option<Span>,
  },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CharBlockCount {
  One,
  Fixed(i64),
  Range(i64, i64),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CallBlock {
  pub bindings: Vec<(String, Node)>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Literal {
  Ident(String),
  Param(String),
  OptionalParam(String),
  Num(i64),
  Range(i64, i64),
  Str(String),
}

impl Literal {
  pub fn to_string_value(&self) -> String {
    match self {
      Literal::Ident(s) => s.clone(),
      Literal::Param(s) | Literal::OptionalParam(s) => s.clone(),
      Literal::Num(n) => n.to_string(),
      Literal::Range(lo, hi) => format!("{}..{}", lo, hi),
      Literal::Str(s) => s.clone(),
    }
  }
}
