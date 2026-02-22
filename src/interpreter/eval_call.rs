use crate::ast::{CallBlock, SourceError};
use std::collections::HashMap;

use super::eval::EvalState;
use super::err_span_impl;

pub(super) fn eval_call<R: rand::RngCore>(
  state: &mut EvalState<'_, '_, R>,
  name: &str,
  params: &[String],
  block: Option<&CallBlock>,
  call_span: Option<crate::ast::Span>,
  env: &mut HashMap<String, String>,
) -> Result<String, SourceError> {
  let mut call_env = env.clone();
  let block_nodes_for_body = block.map(|b| {
    b.bindings
      .iter()
      .map(|(k, v)| (k.clone(), v.clone()))
      .collect::<HashMap<_, _>>()
  });
  if let Some(blk) = block {
    for (var, value_node) in &blk.bindings {
      let s = state.eval(value_node, None, env)?;
      call_env.insert(var.clone(), s);
    }
  }
  let resolved: Vec<String> = params
    .iter()
    .map(|p| call_env.get(p).cloned().unwrap_or_else(|| p.clone()))
    .collect();
  if let Some(fd) = state.program.functions.iter().find(|f| f.name == *name) {
    let mut fn_env = HashMap::new();
    for (i, p) in fd.params.iter().enumerate() {
      if let Some(s) = resolved.get(i) {
        fn_env.insert(p.clone(), s.clone());
      }
    }
    for (var, _) in block.iter().flat_map(|b| b.bindings.iter()) {
      if let Some(s) = call_env.get(var) {
        fn_env.insert(var.clone(), s.clone());
      }
    }
    let result = state.eval(&fd.body, block_nodes_for_body.as_ref(), &mut fn_env)?;
    return Ok(result);
  }
  if let Some(blk) = block {
    let param_set: std::collections::HashSet<_> = params.iter().collect();
    let unused: Vec<String> = blk
      .bindings
      .iter()
      .filter(|(k, _)| !param_set.contains(k))
      .map(|(k, _)| format!(":{}", k))
      .collect();
    if !unused.is_empty() {
      return Err(err_span_impl(
        format!(
          "call '{}' has block parameter(s) ({}) but no function definition; block parameters must be used in a template",
          name,
          unused.join(", ")
        ),
        call_span,
      ));
    }
  }
  let parts: Vec<&str> = resolved.iter().map(String::as_str).collect();
  Ok([name]
    .iter()
    .chain(parts.iter())
    .cloned()
    .collect::<Vec<_>>()
    .join(" "))
}
