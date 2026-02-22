use crate::ast::{Node, SourceError};
use std::collections::HashMap;

use super::eval::EvalState;
use super::err_span_impl;

pub(super) fn eval_func_call<R: rand::RngCore>(
  state: &mut EvalState<'_, '_, R>,
  name: &str,
  args: &[Node],
  fc_span: Option<crate::ast::Span>,
  env: &mut HashMap<String, String>,
) -> Result<String, SourceError> {
  if let Some(builtin) = state.builtins.get(name) {
    let evaled: Vec<String> = args
      .iter()
      .map(|a| state.eval(a, None, env))
      .collect::<Result<Vec<_>, _>>()?;
    return builtin(&evaled).map_err(|e| err_span_impl(e, fc_span));
  }
  let fd = state
    .program
    .functions
    .iter()
    .find(|f| f.name == *name)
    .ok_or_else(|| err_span_impl(format!("unknown function: {}", name), fc_span))?;
  if args.len() != fd.params.len() {
    return Err(err_span_impl(
      format!(
        "function {} expects {} arguments, got {}",
        name,
        fd.params.len(),
        args.len()
      ),
      fc_span,
    ));
  }
  let mut fn_env = HashMap::new();
  for (i, p) in fd.params.iter().enumerate() {
    let s = state.eval(&args[i], None, env)?;
    fn_env.insert(p.clone(), s);
  }
  state.eval(&fd.body, None, &mut fn_env)
}
