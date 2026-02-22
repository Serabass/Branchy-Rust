//! Resolve includes and flatten SpreadInclude.

mod flatten;

use crate::ast::Program;
use crate::parser::parse_program;
use std::collections::{HashSet, VecDeque};

/// Resolves all `include "path"` directives and `...include "path"` mixins.
pub fn resolve_includes<F>(mut program: Program, resolver: F) -> Result<Program, String>
where
  F: Fn(&str) -> Result<String, String>,
{
  let mut resolved = HashSet::new();
  let mut queue: VecDeque<String> = program.includes.iter().cloned().collect();
  while let Some(path) = queue.pop_front() {
    if resolved.contains(&path) {
      continue;
    }
    resolved.insert(path.clone());
    let src = resolver(&path)?;
    let inc = parse_program(&src).map_err(|e| e.to_string())?;
    for p in inc.includes {
      queue.push_back(p);
    }
    program.functions.extend(inc.functions);
    program.events.extend(inc.events);
  }
  program.includes.clear();

  let mut in_progress = HashSet::new();
  program.main = flatten::flatten_node(program.main.clone(), &resolver, &mut in_progress)?;
  for fd in &mut program.functions {
    fd.body = flatten::flatten_node(fd.body.clone(), &resolver, &mut in_progress)?;
  }
  for ev in &mut program.events {
    ev.body = flatten::flatten_node(ev.body.clone(), &resolver, &mut in_progress)?;
  }
  Ok(program)
}
