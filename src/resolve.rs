use crate::ast::{Node, Program};
use crate::parser::parse_program;
use std::collections::{HashSet, VecDeque};

/// Resolves all `include "path"` directives and `...include "path"` mixins.
/// - Top-level includes: merge functions/events from those files.
/// - SpreadInclude in branches: inline that file's main branch into the current branch (at load time).
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
    program.main = flatten_node(program.main.clone(), &resolver, &mut in_progress)?;
    for fd in &mut program.functions {
        fd.body = flatten_node(fd.body.clone(), &resolver, &mut in_progress)?;
    }
    for ev in &mut program.events {
        ev.body = flatten_node(ev.body.clone(), &resolver, &mut in_progress)?;
    }
    Ok(program)
}

fn flatten_node<F>(
    node: Node,
    resolver: &F,
    in_progress: &mut HashSet<String>,
) -> Result<Node, String>
where
    F: Fn(&str) -> Result<String, String>,
{
    use crate::ast::Node::*;
    match node {
        Branch { children, span } => {
            let mut out = Vec::new();
            for c in children {
                match c {
                    SpreadInclude { path, .. } => {
                        if in_progress.contains(&path) {
                            return Err(format!("circular ...include: {}", path));
                        }
                        in_progress.insert(path.clone());
                        let src = resolver(&path)?;
                        let inc = parse_program(&src).map_err(|e| e.to_string())?;
                        let main = flatten_node(inc.main, resolver, in_progress)?;
                        in_progress.remove(&path);
                        match main {
                            Branch { children: nodes, .. } => out.extend(nodes),
                            other => out.push(other),
                        }
                    }
                    other => out.push(flatten_node(other, resolver, in_progress)?),
                }
            }
            Ok(Branch { children: out, span })
        }
        BinaryOp { op, left, right, span } => Ok(BinaryOp {
            op,
            left: Box::new(flatten_node(*left, resolver, in_progress)?),
            right: Box::new(flatten_node(*right, resolver, in_progress)?),
            span,
        }),
        Call {
            name,
            params,
            block,
            span,
        } => {
            let block = block
                .map(|b| {
                    b.bindings
                        .into_iter()
                        .map(|(k, v)| flatten_node(v, resolver, in_progress).map(|n| (k, n)))
                        .collect::<Result<Vec<_>, _>>()
                        .map(|bindings| crate::ast::CallBlock { bindings })
                })
                .transpose()?;
            Ok(Call {
                name,
                params,
                block,
                span,
            })
        }
        InlineCall { name, options, span } => Ok(InlineCall {
            name,
            options: options
                .into_iter()
                .map(|n| flatten_node(n, resolver, in_progress))
                .collect::<Result<Vec<_>, _>>()?,
            span,
        }),
        FuncCall { name, args, span } => Ok(FuncCall {
            name,
            args: args
                .into_iter()
                .map(|n| flatten_node(n, resolver, in_progress))
                .collect::<Result<Vec<_>, _>>()?,
            span,
        }),
        Leaf { .. } | SpreadParam { .. } | CharBlock { .. } => Ok(node),
        SpreadInclude { path, .. } => {
            if in_progress.contains(&path) {
                return Err(format!("circular ...include: {}", path));
            }
            in_progress.insert(path.clone());
            let src = resolver(&path)?;
            let inc = parse_program(&src).map_err(|e| e.to_string())?;
            let main = flatten_node(inc.main, resolver, in_progress)?;
            in_progress.remove(&path);
            Ok(main)
        }
    }
}
