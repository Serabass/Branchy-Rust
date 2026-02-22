use crate::ast::{BinOp, Literal, Node, SourceError};

use super::eval::EvalState;
use super::{err_span_impl, node_span};

pub(super) fn eval_binary_op<R: rand::RngCore>(
  state: &mut EvalState<'_, '_, R>,
  op: BinOp,
  left: &Node,
  right: &Node,
  _span: Option<crate::ast::Span>,
  env: &mut std::collections::HashMap<String, String>,
) -> Result<String, SourceError> {
  match op {
    BinOp::Plus => {
      let a = state.eval(left, None, env)?;
      let b = state.eval(right, None, env)?;
      Ok(format!("{}{}", a, b))
    }
    BinOp::Star => {
      let n = match right {
        Node::Leaf {
          lit: Literal::Range(lo, hi),
          ..
        } => {
          let (lo, hi) = ((*lo).max(0), (*hi).max(0));
          if hi < lo {
            0usize
          } else {
            let span_len = (hi - lo + 1) as u32;
            (lo + (state.rng.next_u32() % span_len) as i64) as usize
          }
        }
        _ => {
          let b = state.eval(right, None, env)?;
          b.parse::<i64>()
            .map_err(|_| {
              err_span_impl(
                "repeat count must be integer or range (e.g. 1..3)",
                node_span(right),
              )
            })?
            .max(0) as usize
        }
      };
      let mut out = String::new();
      for _ in 0..n {
        let a = state.eval(left, None, env)?;
        out.push_str(&a);
      }
      Ok(out)
    }
  }
}
