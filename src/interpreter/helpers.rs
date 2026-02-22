use crate::ast::{Node, SourceError, Span};
use std::collections::HashMap;

pub fn err_span(message: impl Into<String>, span: Option<Span>) -> SourceError {
  SourceError {
    message: message.into(),
    span,
  }
}

pub fn expand_branch_spreads(
  children: &[Node],
  block_nodes: Option<&HashMap<String, Node>>,
) -> Result<Vec<Node>, SourceError> {
  let mut out = Vec::new();
  for c in children {
    match c {
      Node::SpreadParam { param: p, .. } => {
        let nodes = block_nodes
          .and_then(|m| m.get(p))
          .ok_or_else(|| err_span(format!("...:{} has no binding in block", p), None))?;
        match nodes {
          Node::Branch {
            children: inner, ..
          } => out.extend(inner.clone()),
          other => out.push(other.clone()),
        }
      }
      _ => out.push(c.clone()),
    }
  }
  Ok(out)
}

pub fn push_span(trace: &mut Vec<Span>, span: Option<Span>) {
  if let Some(s) = span {
    trace.push(s);
  }
}

pub fn node_span(node: &Node) -> Option<Span> {
  match node {
    Node::Branch { span, .. } => *span,
    Node::Leaf { span, .. } => *span,
    Node::BinaryOp { span, .. } => *span,
    Node::Call { span, .. } => *span,
    Node::InlineCall { span, .. } => *span,
    Node::FuncCall { span, .. } => *span,
    Node::SpreadParam { span, .. } => *span,
    Node::SpreadInclude { span, .. } => *span,
    Node::CharBlock { span, .. } => *span,
  }
}

pub fn err_span_impl(message: impl Into<String>, span: Option<Span>) -> SourceError {
  err_span(message, span)
}
