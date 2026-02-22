use serde::{Deserialize, Serialize};
use std::fmt;

/// Source span for trace (Monaco: 1-based line, 1-based column; end exclusive for column).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span {
  pub start_line: u32,
  pub start_column: u32,
  pub end_line: u32,
  pub end_column: u32,
}

/// Error with optional source location (line, column).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceError {
  pub message: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub span: Option<Span>,
}

impl SourceError {
  pub fn with_span(mut self, span: Option<Span>) -> Self {
    self.span = span;
    self
  }
}

impl fmt::Display for SourceError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    if let Some(s) = &self.span {
      write!(
        f,
        "line {}, column {}: {}",
        s.start_line, s.start_column, self.message
      )?;
      if s.start_line != s.end_line || s.start_column != s.end_column {
        write!(f, " (through line {}, column {})", s.end_line, s.end_column)?;
      }
      Ok(())
    } else {
      write!(f, "{}", self.message)
    }
  }
}

impl std::error::Error for SourceError {}

/// Build a Span from source string and byte offsets (0-based). Useful for error reporting.
pub fn span_from_offsets(source: &str, start: usize, end: usize) -> Span {
  let mut line_index = vec![0];
  for (i, c) in source.char_indices() {
    if c == '\n' {
      line_index.push(i + 1);
    }
  }
  let start_line = (0..line_index.len())
    .rposition(|i| line_index[i] <= start)
    .unwrap_or(0);
  let end_line = (0..line_index.len())
    .rposition(|i| line_index[i] <= end.saturating_sub(1))
    .unwrap_or(0);
  Span {
    start_line: (start_line + 1) as u32,
    start_column: (start - line_index[start_line] + 1) as u32,
    end_line: (end_line + 1) as u32,
    end_column: (end - line_index[end_line] + 1) as u32,
  }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Program {
  #[serde(default)]
  pub includes: Vec<String>,
  pub functions: Vec<FunctionDef>,
  #[serde(default)]
  pub events: Vec<Event>,
  pub main: Node,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EventMatcher {
  ByName(String),
  ByStr(String),
  ByRegex(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Event {
  pub matcher: EventMatcher,
  pub body: Node,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionDef {
  pub name: String,
  pub params: Vec<String>,
  pub body: Node,
}

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
  /// Mixin: spread branch from block param `...:var`
  SpreadParam {
    param: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    span: Option<Span>,
  },
  /// Mixin: spread branch from file (resolved at load time)
  SpreadInclude {
    path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    span: Option<Span>,
  },
  /// Inline char block: [a-zA-Z], [abc:5], [a-z:2..5]. One or more random chars from set.
  CharBlock {
    /// Inclusive ranges (e.g. (a,z), (0,9))
    ranges: Vec<(char, char)>,
    /// None = 1, Some(n) = n, Some((lo,hi)) = random in [lo, hi]
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
  /// Optional param :?name — 50% output value, 50% empty
  OptionalParam(String),
  Num(i64),
  /// Range for * operator: repeat count in [lo, hi] inclusive (e.g. 1..3)
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
