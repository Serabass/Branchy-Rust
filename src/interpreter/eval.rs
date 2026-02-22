use crate::ast::{Node, Program, SourceError};
use crate::builtins::BuiltinFn;
use std::collections::HashMap;

use super::eval_branch;
use super::eval_call;
use super::eval_func;
use super::eval_inline;
use super::eval_leaf;
use super::eval_op;
use super::{err_span_impl, node_span};

pub(super) struct EvalState<'a, 'b, R: rand::RngCore> {
  pub program: &'a Program,
  pub builtins: &'a HashMap<String, BuiltinFn>,
  pub rng: &'b mut R,
  pub trace: &'b mut Vec<crate::ast::Span>,
}

impl<'a, 'b, R: rand::RngCore> EvalState<'a, 'b, R> {
  pub fn new(
    program: &'a Program,
    builtins: &'a HashMap<String, BuiltinFn>,
    rng: &'b mut R,
    trace: &'b mut Vec<crate::ast::Span>,
  ) -> Self {
    Self {
      program,
      builtins,
      rng,
      trace,
    }
  }

  pub fn program(&self) -> &Program {
    self.program
  }

  pub fn eval(
    &mut self,
    node: &Node,
    block_nodes: Option<&HashMap<String, Node>>,
    env: &mut HashMap<String, String>,
  ) -> Result<String, SourceError> {
    match node {
      Node::Branch { children, .. } => eval_branch::eval_branch(self, children, block_nodes, env),
      Node::SpreadParam { .. } | Node::SpreadInclude { .. } => Err(err_span_impl(
        "spread should be expanded (SpreadInclude at load, SpreadParam in branch)",
        node_span(node),
      )),
      Node::CharBlock { ranges, count, span } => eval_leaf::eval_char_block(self, ranges, count, *span),
      Node::Leaf { lit, span } => eval_leaf::eval_leaf(self, lit, *span, env),
      Node::BinaryOp { op, left, right, span } => {
        eval_op::eval_binary_op(self, op.clone(), left, right, *span, env)
      }
      Node::Call {
        name,
        params,
        optional_params,
        block,
        span: call_span,
        ..
      } => eval_call::eval_call(
        self,
        name,
        params,
        optional_params,
        block.as_ref(),
        *call_span,
        env,
      ),
      Node::InlineCall {
        name,
        options,
        span: ic_span,
        ..
      } => eval_inline::eval_inline_call(self, name, options, *ic_span, env),
      Node::FuncCall {
        name,
        args,
        span: fc_span,
        ..
      } => eval_func::eval_func_call(self, name, args, *fc_span, env),
    }
  }
}
