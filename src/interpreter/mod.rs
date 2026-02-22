//! Interpreter: run Branchy programs with RNG and optional trace.

use crate::ast::{EventMatcher, Node, Program, SourceError, Span};
use crate::builtins::BuiltinFn;
use std::collections::HashMap;

mod eval;
mod eval_branch;
mod eval_call;
mod eval_func;
mod eval_inline;
mod eval_leaf;
mod eval_op;
mod helpers;

use eval::EvalState;
use helpers::err_span;

/// Result of interpretation: output string and optional trace of source spans that were used.
pub fn interpret(
  program: &Program,
  builtins: &HashMap<String, BuiltinFn>,
  rng: &mut impl rand::RngCore,
  input: Option<&str>,
) -> Result<(String, Vec<Span>), SourceError> {
  let mut env = HashMap::new();
  let mut trace = Vec::new();
  let mut state = EvalState::new(program, builtins, rng, &mut trace);
  let out = if let Some(s) = input {
    if !program.events.is_empty() {
      for event in &program.events {
        if event_matches(&event.matcher, s) {
          let out = state.eval(&event.body, None, &mut env)?;
          return Ok((out, trace));
        }
      }
      return Err(err_span(format!("no event matches input: {:?}", s), None));
    }
    eval_main(&mut state, &mut env)?
  } else {
    eval_main(&mut state, &mut env)?
  };
  Ok((out, trace))
}

fn eval_main<R: rand::RngCore>(
  state: &mut EvalState<'_, '_, R>,
  env: &mut HashMap<String, String>,
) -> Result<String, SourceError> {
  let main = state.program().main.clone();
  match &main {
    Node::Branch { children, .. }
      if children.len() > 1 && children.iter().all(|c| matches!(c, Node::Branch { .. })) =>
    {
      let mut out = String::new();
      for child in children {
        out.push_str(&state.eval(child, None, env)?);
      }
      Ok(out)
    }
    _ => state.eval(&main, None, env),
  }
}

fn event_matches(matcher: &EventMatcher, input: &str) -> bool {
  match matcher {
    EventMatcher::ByName(name) => input == name,
    EventMatcher::ByStr(s) => input == s.as_str(),
    EventMatcher::ByRegex(pattern) => {
      regex::Regex::new(pattern).map_or(false, |re| re.is_match(input))
    }
  }
}

pub(super) use helpers::{expand_branch_spreads, err_span_impl, node_span, push_span};
