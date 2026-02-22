use crate::ast::{Node, SourceError};
use rand::seq::SliceRandom;
use std::collections::HashMap;

use super::eval::EvalState;
use super::{expand_branch_spreads, err_span_impl};

pub(super) fn eval_branch<R: rand::RngCore>(
  state: &mut EvalState<'_, '_, R>,
  children: &[Node],
  block_nodes: Option<&HashMap<String, Node>>,
  env: &mut HashMap<String, String>,
) -> Result<String, SourceError> {
  let expanded = expand_branch_spreads(children, block_nodes)?;
  let child = expanded
    .choose(state.rng)
    .ok_or_else(|| err_span_impl("empty branch", None))?;
  state.eval(child, block_nodes, env)
}
