//! Format (unparse) Branchy AST to canonical source text.

use crate::ast::{
  BinOp, CallBlock, CharBlockCount, Event, EventMatcher, FunctionDef, Literal, Node, Program,
};

/// Formatting options (Prettier-like).
#[derive(Debug, Clone)]
pub struct FormatOptions {
  /// Indent string (e.g. two spaces).
  pub indent: String,
  /// "single_line" | "multi_line" | "auto" (single line if total length < max_line_length).
  pub bracket_style: BracketStyle,
  /// Used when bracket_style is Auto.
  pub max_line_length: usize,
  /// One semicolon between branch elements.
  pub semicolon_between_elements: bool,
  /// Trailing semicolon after last element before `]`.
  pub trailing_semicolon: bool,
  /// Spaces around `+` and `*`.
  pub spaces_around_binary: bool,
  /// Empty line after each include.
  pub newline_after_include: bool,
  /// Empty line before main branch.
  pub newline_before_main: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BracketStyle {
  SingleLine,
  MultiLine,
  Auto,
}

impl Default for FormatOptions {
  fn default() -> Self {
    Self {
      indent: "  ".into(),
      bracket_style: BracketStyle::Auto,
      max_line_length: 80,
      semicolon_between_elements: true,
      trailing_semicolon: false,
      spaces_around_binary: true,
      newline_after_include: false,
      newline_before_main: true,
    }
  }
}

/// Escape string for output inside double quotes (same as lexer expects).
fn escape_string(s: &str) -> String {
  let mut out = String::with_capacity(s.len() + 2);
  out.push('"');
  for c in s.chars() {
    match c {
      '\\' => out.push_str("\\\\"),
      '"' => out.push_str("\\\""),
      '\n' => out.push_str("\\n"),
      '\t' => out.push_str("\\t"),
      '\r' => out.push_str("\\r"),
      _ => out.push(c),
    }
  }
  out.push('"');
  out
}

/// Serialize CharBlock to source form: [a-zA-Z], [a-z:5], [a-z:2..5].
fn char_block_to_string(ranges: &[(char, char)], count: &CharBlockCount) -> String {
  let mut set_part = String::new();
  for &(lo, hi) in ranges {
    if lo == hi {
      set_part.push(lo);
    } else {
      set_part.push(lo);
      set_part.push('-');
      set_part.push(hi);
    }
  }
  let count_suffix = match count {
    CharBlockCount::One => String::new(),
    CharBlockCount::Fixed(n) => format!(":{}", n),
    CharBlockCount::Range(lo, hi) => format!(":{}..{}", lo, hi),
  };
  format!("[{}{}]", set_part, count_suffix)
}

/// Format a program to source string.
pub fn format_program(program: &Program, options: &FormatOptions) -> String {
  let mut out = String::new();

  for (i, path) in program.includes.iter().enumerate() {
    if i > 0 && options.newline_after_include {
      out.push('\n');
    }
    out.push_str("include ");
    out.push_str(&escape_string(path));
    out.push_str(";\n");
  }

  for f in &program.functions {
    out.push_str(&format_function_def(f, options));
    out.push('\n');
  }

  for e in &program.events {
    out.push_str(&format_event(e, options));
    out.push('\n');
  }

  if options.newline_before_main && (!program.includes.is_empty() || !program.functions.is_empty() || !program.events.is_empty()) {
    out.push('\n');
  }

  out.push_str(&format_node(&program.main, options, 0));
  out.push('\n');
  out
}

fn format_event(event: &Event, options: &FormatOptions) -> String {
  let matcher = match &event.matcher {
    EventMatcher::ByName(name) => format!("@{}", name),
    EventMatcher::ByStr(s) => escape_string(s),
    EventMatcher::ByRegex(pat) => format!("~{}", escape_string(pat)),
  };
  format!(
    "{} = {};\n",
    matcher,
    format_node(&event.body, options, 0)
  )
}

fn format_function_def(f: &FunctionDef, options: &FormatOptions) -> String {
  let params: Vec<String> = f.params.iter().map(|p| format!(":{}", p)).collect();
  let params_str = params.join(", ");
  format!(
    "!{}({}) = {};\n",
    f.name,
    params_str,
    format_node(&f.body, options, 0)
  )
}

fn format_node(node: &Node, options: &FormatOptions, depth: usize) -> String {
  match node {
    Node::Branch { children, .. } => format_branch(children, options, depth),
    Node::Leaf { lit, .. } => format_literal(lit),
    Node::BinaryOp { op, left, right, .. } => {
      let sep = if options.spaces_around_binary {
        match op {
          BinOp::Plus => " + ",
          BinOp::Star => " * ",
        }
      } else {
        match op {
          BinOp::Plus => "+",
          BinOp::Star => "*",
        }
      };
      format!(
        "{}{}{}",
        format_node(left, options, depth),
        sep,
        format_node(right, options, depth)
      )
    }
    Node::Call {
      name,
      params,
      optional_params,
      block,
      ..
    } => {
      let mut s = name.clone();
      for p in params {
        let prefix = if optional_params.contains(p) { ":?" } else { ":" };
        s.push_str(&format!(" {}{}", prefix, p));
      }
      if let Some(blk) = block {
        s.push_str(" {\n");
        s.push_str(&format_call_block(blk, options, depth));
        s.push_str("\n");
        s.push_str(&options.indent.repeat(depth));
        s.push_str("}");
      }
      s
    }
    Node::InlineCall { name, options: opts, .. } => {
      let parts: Vec<String> = opts.iter().map(|n| format_node(n, options, depth)).collect();
      format!("{} <{}>", name, parts.join("|"))
    }
    Node::FuncCall { name, args, .. } => {
      let args_str: Vec<String> = args.iter().map(|a| format_node(a, options, depth)).collect();
      format!("!{}({})", name, args_str.join(", "))
    }
    Node::SpreadParam { param, .. } => format!("...:{}", param),
    Node::SpreadInclude { path, .. } => format!("...include {}", escape_string(path)),
    Node::CharBlock { ranges, count, .. } => char_block_to_string(ranges, count),
  }
}

fn format_literal(lit: &Literal) -> String {
  match lit {
    Literal::Ident(s) => s.clone(),
    Literal::Param(s) => format!(":{}", s),
    Literal::OptionalParam(s) => format!(":?{}", s),
    Literal::Num(n) => n.to_string(),
    Literal::Range(lo, hi) => format!("{}..{}", lo, hi),
    Literal::Str(s) => escape_string(s),
  }
}

fn format_call_block(block: &CallBlock, options: &FormatOptions, depth: usize) -> String {
  let indent_str = options.indent.repeat(depth);
  let inner_indent = format!("{}{}", indent_str, options.indent);
  let lines: Vec<String> = block
    .bindings
    .iter()
    .map(|(param, node)| {
      format!(
        "{}:{} = {}",
        inner_indent,
        param,
        format_node(node, options, depth + 1)
      )
    })
    .collect();
  let sep = if options.semicolon_between_elements {
    ";\n"
  } else {
    "\n"
  };
  lines.join(sep)
}

fn branch_single_line_len(children: &[Node], options: &FormatOptions) -> usize {
  let sep_len = if options.semicolon_between_elements { 2 } else { 1 };
  let trail = if options.trailing_semicolon { 1 } else { 0 };
  let mut len = 2; // [ ]
  for (i, c) in children.iter().enumerate() {
    if i > 0 {
      len += sep_len;
    }
    len += node_approx_len(c, options);
  }
  len + trail
}

fn node_approx_len(node: &Node, options: &FormatOptions) -> usize {
  match node {
    Node::Branch { children, .. } => branch_single_line_len(children, options),
    Node::Leaf { lit, .. } => literal_approx_len(lit),
    Node::BinaryOp { left, right, .. } => {
      node_approx_len(left, options) + node_approx_len(right, options) + if options.spaces_around_binary { 5 } else { 1 }
    }
    Node::Call { name, params, block, .. } => {
      name.len() + params.iter().map(|p| p.len() + 2).sum::<usize>()
        + block.as_ref().map(|b| 10 + b.bindings.len() * 5).unwrap_or(0)
    }
    Node::InlineCall { name, options: opts, .. } => {
      name.len() + 2 + opts.iter().map(|n| node_approx_len(n, options)).sum::<usize>() + opts.len().saturating_sub(1)
    }
    Node::FuncCall { name, args, .. } => {
      name.len() + 2 + args.iter().map(|a| node_approx_len(a, options)).sum::<usize>()
    }
    Node::SpreadParam { param, .. } => 4 + param.len(),
    Node::SpreadInclude { path, .. } => 12 + path.len(),
    Node::CharBlock { ranges, count, .. } => char_block_to_string(ranges, count).len(),
  }
}

fn literal_approx_len(lit: &Literal) -> usize {
  match lit {
    Literal::Ident(s) => s.len(),
    Literal::Param(s) | Literal::OptionalParam(s) => s.len() + 2,
    Literal::Num(n) => n.to_string().len().max(1),
    Literal::Range(_, _) => 4,
    Literal::Str(s) => s.len() + 4,
  }
}

fn format_branch(children: &[Node], options: &FormatOptions, depth: usize) -> String {
  let use_single = match &options.bracket_style {
    BracketStyle::SingleLine => true,
    BracketStyle::MultiLine => false,
    BracketStyle::Auto => {
      children.len() <= 1
        || branch_single_line_len(children, options) <= options.max_line_length
    }
  };

  if use_single {
    let mut s = String::from("[ ");
    for (i, c) in children.iter().enumerate() {
      if i > 0 {
        if options.semicolon_between_elements {
          s.push_str("; ");
        } else {
          s.push(' ');
        }
      }
      s.push_str(&format_node(c, options, depth));
    }
    if options.trailing_semicolon && !children.is_empty() {
      s.push(';');
    }
    s.push_str(" ]");
    return s;
  }

  let indent_str = options.indent.repeat(depth);
  let inner_indent = format!("{}{}", indent_str, options.indent);
  let mut s = format!("[\n");
  for (i, c) in children.iter().enumerate() {
    s.push_str(&inner_indent);
    s.push_str(&format_node(c, options, depth + 1));
    if i < children.len() - 1 {
      if options.semicolon_between_elements {
        s.push(';');
      }
      s.push('\n');
    } else {
      if options.trailing_semicolon {
        s.push(';');
      }
      s.push('\n');
    }
  }
  s.push_str(&indent_str);
  s.push(']');
  s
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::parser::parse_program;

  fn roundtrip_and_idempotent(src: &str) {
    let opts = FormatOptions::default();
    let p = parse_program(src).unwrap();
    let out1 = format_program(&p, &opts);
    let p2 = parse_program(&out1).unwrap();
    let out2 = format_program(&p2, &opts);
    assert_eq!(out1, out2, "format must be idempotent");
  }

  #[test]
  fn format_hello_roundtrip() {
    let src = r#"[ hello; world; ]"#;
    let p = parse_program(src).unwrap();
    let out = format_program(&p, &FormatOptions::default());
    let p2 = parse_program(&out).unwrap();
    assert_eq!(p.includes, p2.includes);
    assert_eq!(p.functions.len(), p2.functions.len());
    assert_eq!(p.events.len(), p2.events.len());
  }

  #[test]
  fn format_idempotent_hello() {
    roundtrip_and_idempotent(r#"[ hello; world; 123; ]"#);
  }

  #[test]
  fn format_idempotent_math() {
    roundtrip_and_idempotent(r#"[ "1" + "2"; "10" * 3; "ab" + "cd"; ]"#);
  }

  #[test]
  fn format_nested_roundtrip() {
    let src = r#"[
  [
    [ a; b; ];
    [ c; ];
  ];
  [
    [ deep; ];
    [ [ [ leaf; ]; ]; ];
  ];
]"#;
    let opts = FormatOptions::default();
    let p = parse_program(src).unwrap();
    let out = format_program(&p, &opts);
    let p2 = parse_program(&out).unwrap();
    assert_eq!(p.includes, p2.includes);
    assert_eq!(p.functions.len(), p2.functions.len());
    assert_eq!(p.events.len(), p2.events.len());
  }

  #[test]
  fn format_idempotent_api_style() {
    roundtrip_and_idempotent(
      r#"!api(:method, :path) = [ "api "; :method; " "; :path; " "; :?auth; ];
!request(:method, :path) = [ :method; " "; :path; " "; :headers; ];

[
  api :get :users { :auth = "Bearer token"; };
  api :post :login { :auth = "basic"; };
  api :get :health;
  request :GET :users { :headers = [ Accept; :json; ]; };
]"#,
    );
  }

  #[test]
  fn format_idempotent_char_block() {
    roundtrip_and_idempotent(
      r#"[
  [a];
  [a-z];
  [a-zA-Z];
  [a-zA-Z:5];
  [a-zA-Z:2..5];
  [abcdef:3..7];
  [0-9:4];
  "id_" + [a-zA-Z0-9:8];
]"#,
    );
  }

  #[test]
  fn format_idempotent_include() {
    roundtrip_and_idempotent(r#"include "lib.branchy";

[ a; b; ]"#);
  }

  #[test]
  fn escape_string_roundtrip() {
    let s = "a\nb\tc\"d\\e";
    let escaped = escape_string(s);
    assert!(escaped.starts_with('"') && escaped.ends_with('"'));
    let p = parse_program(&format!("[ {}; ]", escaped)).unwrap();
    if let Node::Branch { children, .. } = &p.main {
      if let Some(Node::Leaf { lit: Literal::Str(back), .. }) = children.first() {
        assert_eq!(back, s);
        return;
      }
    }
    panic!("roundtrip failed");
  }
}
