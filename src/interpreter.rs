use crate::ast::{BinOp, EventMatcher, Node, Program, SourceError, Span};
use crate::builtins::BuiltinFn;
use rand::seq::SliceRandom;
use std::collections::HashMap;

fn err_span(message: impl Into<String>, span: Option<Span>) -> SourceError {
    SourceError {
        message: message.into(),
        span,
    }
}

/// Result of interpretation: output string and optional trace of source spans that were used.
pub fn interpret(
    program: &Program,
    builtins: &HashMap<String, BuiltinFn>,
    rng: &mut impl rand::RngCore,
    input: Option<&str>,
) -> Result<(String, Vec<Span>), SourceError> {
    let mut env = HashMap::new();
    let mut trace = Vec::new();
    let out = if let Some(s) = input {
        if !program.events.is_empty() {
            for event in &program.events {
                if event_matches(&event.matcher, s) {
                    let out = eval_node(&event.body, program, builtins, &mut env, rng, None, &mut trace)?;
                    return Ok((out, trace));
                }
            }
            return Err(err_span(format!("no event matches input: {:?}", s), None));
        }
        eval_main(program, builtins, &mut env, rng, &mut trace)?
    } else {
        eval_main(program, builtins, &mut env, rng, &mut trace)?
    };
    Ok((out, trace))
}

/// Top-level main: if it's a sequence of branches (multiple Branch children), run each and concatenate; else pick one.
fn eval_main(
    program: &Program,
    builtins: &HashMap<String, BuiltinFn>,
    env: &mut HashMap<String, String>,
    rng: &mut impl rand::RngCore,
    trace: &mut Vec<Span>,
) -> Result<String, SourceError> {
    match &program.main {
        Node::Branch { children, .. } if children.len() > 1
            && children.iter().all(|c| matches!(c, Node::Branch { .. })) =>
        {
            let mut out = String::new();
            for child in children {
                out.push_str(&eval_node(child, program, builtins, env, rng, None, trace)?);
            }
            Ok(out)
        }
        _ => eval_node(&program.main, program, builtins, env, rng, None, trace),
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

fn expand_branch_spreads(
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
                    Node::Branch { children: inner, .. } => out.extend(inner.clone()),
                    other => out.push(other.clone()),
                }
            }
            _ => out.push(c.clone()),
        }
    }
    Ok(out)
}

fn push_span(trace: &mut Vec<Span>, span: Option<Span>) {
    if let Some(s) = span {
        trace.push(s);
    }
}

/// Span of a node (for trace: only chosen path).
fn node_span(node: &Node) -> Option<Span> {
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

fn eval_node(
    node: &Node,
    program: &Program,
    builtins: &HashMap<String, BuiltinFn>,
    env: &mut HashMap<String, String>,
    rng: &mut impl rand::RngCore,
    block_nodes: Option<&HashMap<String, Node>>,
    trace: &mut Vec<Span>,
) -> Result<String, SourceError> {
    match node {
        Node::Branch { children, .. } => {
            let expanded = expand_branch_spreads(children, block_nodes)?;
            let child = expanded
                .choose(rng)
                .ok_or_else(|| err_span("empty branch", node_span(node)))?;
            // Do not push here: Leaf/CharBlock/InlineCall push their own span
            eval_node(child, program, builtins, env, rng, block_nodes, trace)
        }
        Node::SpreadParam { .. } | Node::SpreadInclude { .. } => {
            Err(err_span("spread should be expanded (SpreadInclude at load, SpreadParam in branch)", node_span(node)))
        }
        Node::CharBlock { ranges, count, span } => {
            push_span(trace, *span);
            let chars: Vec<char> = ranges
                .iter()
                .flat_map(|&(lo, hi)| (lo..=hi).collect::<Vec<_>>())
                .collect();
            if chars.is_empty() {
                return Ok(String::new());
            }
            let n = match count {
                crate::ast::CharBlockCount::One => 1,
                crate::ast::CharBlockCount::Fixed(k) => (*k).max(0) as usize,
                crate::ast::CharBlockCount::Range(lo, hi) => {
                    let (lo, hi) = ((*lo).max(0), (*hi).max(0));
                    if hi < lo {
                        0
                    } else {
                        let span = (hi - lo + 1) as u32;
                        (lo + (rng.next_u32() % span) as i64) as usize
                    }
                }
            };
            let out: String = (0..n).map(|_| *chars.choose(rng).unwrap()).collect();
            Ok(out)
        }
        Node::Leaf { lit, span } => {
            push_span(trace, *span);
            match lit {
            crate::ast::Literal::Param(p) => env
                .get(p)
                .cloned()
                .ok_or_else(|| err_span(format!("undefined param :{}", p), *span)),
            crate::ast::Literal::OptionalParam(p) => {
                let value = env.get(p).cloned().unwrap_or_default();
                let show = (rng.next_u32() % 2) == 0;
                Ok(if value.is_empty() || !show {
                    String::new()
                } else {
                    value
                })
            }
            _ => Ok(lit.to_string_value()),
            }
        }
        Node::BinaryOp { op, left, right, .. } => {
            match op {
            BinOp::Plus => {
                let a = eval_node(left, program, builtins, env, rng, block_nodes, trace)?;
                let b = eval_node(right, program, builtins, env, rng, block_nodes, trace)?;
                Ok(format!("{}{}", a, b))
            }
            BinOp::Star => {
                let n = match right.as_ref() {
                    crate::ast::Node::Leaf { lit: crate::ast::Literal::Range(lo, hi), .. } => {
                        let (lo, hi) = ((*lo).max(0), (*hi).max(0));
                        if hi < lo {
                            0usize
                        } else {
                            let span = (hi - lo + 1) as u32;
                            (lo + (rng.next_u32() % span) as i64) as usize
                        }
                    }
                    _ => {
                        let b = eval_node(right, program, builtins, env, rng, block_nodes, trace)?;
                        b.parse::<i64>()
                            .map_err(|_| err_span("repeat count must be integer or range (e.g. 1..3)", node_span(right)))?
                            .max(0) as usize
                    }
                };
                let mut out = String::new();
                for _ in 0..n {
                    let a = eval_node(left, program, builtins, env, rng, block_nodes, trace)?;
                    out.push_str(&a);
                }
                Ok(out)
            }
            }
        }
        Node::Call {
            name,
            params,
            block,
            span: call_span,
            ..
        } => {
            let mut call_env = env.clone();
            let block_nodes_for_body = block.as_ref().map(|b| {
                b.bindings
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect::<HashMap<_, _>>()
            });
            if let Some(ref blk) = block {
                for (var, value_node) in &blk.bindings {
                    let s =
                        eval_node(value_node, program, builtins, env, rng, block_nodes, trace)?;
                    call_env.insert(var.clone(), s);
                }
            }
            let resolved: Vec<String> = params
                .iter()
                .map(|p| {
                    call_env
                        .get(p)
                        .cloned()
                        .unwrap_or_else(|| p.clone())
                })
                .collect();
            if let Some(ref fd) = program.functions.iter().find(|f| f.name == *name) {
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
                let result = eval_node(
                    &fd.body,
                    program,
                    builtins,
                    &mut fn_env,
                    rng,
                    block_nodes_for_body.as_ref(),
                    trace,
                )?;
                return Ok(result);
            }
            if let Some(ref blk) = block {
                let param_set: std::collections::HashSet<_> = params.iter().collect();
                let unused: Vec<String> = blk
                    .bindings
                    .iter()
                    .filter(|(k, _)| !param_set.contains(k))
                    .map(|(k, _)| format!(":{}", k))
                    .collect();
                if !unused.is_empty() {
                    return Err(err_span(
                        format!(
                            "call '{}' has block parameter(s) ({}) but no function definition; block parameters must be used in a template",
                            name,
                            unused.join(", ")
                        ),
                        *call_span,
                    ));
                }
            }
            let parts: Vec<&str> = resolved.iter().map(String::as_str).collect();
            Ok([name.as_str()].iter().chain(parts.iter()).cloned().collect::<Vec<_>>().join(" "))
        }
        Node::InlineCall { name, options, span: ic_span, .. } => {
            let opt = options
                .choose(rng)
                .ok_or_else(|| err_span("empty inline options", *ic_span))?;
            push_span(trace, node_span(opt));
            let s = eval_node(opt, program, builtins, env, rng, block_nodes, trace)?;
            Ok(format!("{} {}", name, s))
        }
        Node::FuncCall { name, args, span: fc_span, .. } => {
            if let Some(&builtin) = builtins.get(name) {
                let args: Vec<String> = args
                    .iter()
                    .map(|a| eval_node(a, program, builtins, env, rng, block_nodes, trace))
                    .collect::<Result<Vec<_>, _>>()?;
                return builtin(&args).map_err(|e| err_span(e, *fc_span));
            }
            let fd = program
                .functions
                .iter()
                .find(|f| f.name == *name)
                .ok_or_else(|| err_span(format!("unknown function: {}", name), *fc_span))?;
            if args.len() != fd.params.len() {
                return Err(err_span(
                    format!(
                        "function {} expects {} arguments, got {}",
                        name,
                        fd.params.len(),
                        args.len()
                    ),
                    *fc_span,
                ));
            }
            let mut fn_env = HashMap::new();
            for (i, p) in fd.params.iter().enumerate() {
                let s = eval_node(&args[i], program, builtins, env, rng, block_nodes, trace)?;
                fn_env.insert(p.clone(), s);
            }
            eval_node(&fd.body, program, builtins, &mut fn_env, rng, None, trace)
        }
    }
}
