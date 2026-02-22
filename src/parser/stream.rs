//! Token stream with span tracking and helpers for parser.

use crate::ast::{Node, SourceError, Span};
use crate::lexer::Token;

pub(crate) fn build_line_index(source: &str) -> Vec<usize> {
  let mut out = vec![0];
  for (i, c) in source.char_indices() {
    if c == '\n' {
      out.push(i + 1);
    }
  }
  out
}

pub(crate) fn offset_to_span(line_index: &[usize], start: usize, end: usize) -> Span {
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

pub(crate) struct SpanStream {
  pub(crate) tokens: Vec<(Token, usize, usize)>,
  line_index: Vec<usize>,
  index: usize,
  first: Option<Span>,
  last: Option<Span>,
}

pub(crate) type TokenIter = SpanStream;

impl SpanStream {
  pub(crate) fn new(tokens: Vec<(Token, usize, usize)>, source: &str) -> Self {
    SpanStream {
      line_index: build_line_index(source),
      tokens,
      index: 0,
      first: None,
      last: None,
    }
  }
  pub(crate) fn start_span(&mut self) {
    self.first = None;
  }
  pub(crate) fn get_span(&self) -> Option<Span> {
    match (self.first, self.last) {
      (Some(s), Some(e)) => Some(Span {
        start_line: s.start_line,
        start_column: s.start_column,
        end_line: e.end_line,
        end_column: e.end_column,
      }),
      _ => None,
    }
  }
  pub(crate) fn next(&mut self) -> Option<Token> {
    let (tok, start, end) = self.tokens.get(self.index)?.clone();
    self.index += 1;
    let span = offset_to_span(&self.line_index, start, end);
    if self.first.is_none() {
      self.first = Some(span);
    }
    self.last = Some(span);
    Some(tok)
  }
  pub(crate) fn peek(&self) -> Option<&Token> {
    self.tokens.get(self.index).map(|(t, _, _)| t)
  }
  pub(crate) fn current_span(&self) -> Option<Span> {
    self.last
  }
  pub(crate) fn peek_span(&self) -> Option<Span> {
    self
      .tokens
      .get(self.index)
      .map(|(_, start, end)| offset_to_span(&self.line_index, *start, *end))
  }
}

pub(crate) fn err_span(it: &TokenIter, message: impl Into<String>) -> SourceError {
  SourceError {
    message: message.into(),
    span: it.current_span().or_else(|| it.peek_span()),
  }
}

pub(crate) fn expect(it: &mut TokenIter, want: Token) -> Result<(), SourceError> {
  match it.next() {
    Some(got) if got == want => Ok(()),
    got => Err(err_span(it, format!("expected {:?}, got {:?}", want, got))),
  }
}

pub(crate) fn expect_ident(it: &mut TokenIter) -> Result<String, SourceError> {
  match it.next() {
    Some(Token::Ident(s)) => Ok(s),
    other => Err(err_span(
      it,
      format!("expected identifier, got {:?}", other),
    )),
  }
}

pub(crate) fn skip_semicolon(it: &mut TokenIter) {
  while matches!(it.peek(), Some(Token::Semicolon)) {
    it.next();
  }
}

pub(crate) fn node_span(n: &Node) -> Option<Span> {
  match n {
    Node::Branch { span, .. }
    | Node::Leaf { span, .. }
    | Node::BinaryOp { span, .. }
    | Node::Call { span, .. }
    | Node::InlineCall { span, .. }
    | Node::FuncCall { span, .. }
    | Node::SpreadParam { span, .. }
    | Node::SpreadInclude { span, .. }
    | Node::CharBlock { span, .. } => *span,
  }
}

pub(crate) fn merge_span(a: Option<Span>, b: Option<Span>) -> Option<Span> {
  match (a, b) {
    (Some(s1), Some(s2)) => {
      let (start_ln, start_col) =
        if (s1.start_line, s1.start_column) <= (s2.start_line, s2.start_column) {
          (s1.start_line, s1.start_column)
        } else {
          (s2.start_line, s2.start_column)
        };
      let (end_ln, end_col) = if (s1.end_line, s1.end_column) >= (s2.end_line, s2.end_column) {
        (s1.end_line, s1.end_column)
      } else {
        (s2.end_line, s2.end_column)
      };
      Some(Span {
        start_line: start_ln,
        start_column: start_col,
        end_line: end_ln,
        end_column: end_col,
      })
    }
    (s, None) | (None, s) => s,
  }
}
