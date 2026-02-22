use crate::ast::SourceError;
use rand::seq::SliceRandom;

use super::eval::EvalState;
use super::{err_span_impl, node_span};

pub(super) fn eval_inline_call<R: rand::RngCore>(
  state: &mut EvalState<'_, '_, R>,
  name: &str,
  options: &[crate::ast::Node],
  ic_span: Option<crate::ast::Span>,
  env: &mut std::collections::HashMap<String, String>,
) -> Result<String, SourceError> {
  let opt = options
    .choose(state.rng)
    .ok_or_else(|| err_span_impl("empty inline options", ic_span))?;
  super::push_span(state.trace, node_span(opt));
  let s = state.eval(opt, None, env)?;
  Ok(format!("{} {}", name, s))
}
