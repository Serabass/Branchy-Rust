//! Parser tests.

#[cfg(test)]
mod tests {
  use crate::ast::{BinOp, Literal, Node};
  use crate::parser::parse_program;

  #[test]
  fn parse_simple_branch() {
    let p = parse_program("[ a; b; c; ]").unwrap();
    assert!(p.functions.is_empty());
    let Node::Branch { children, .. } = &p.main else {
      panic!("expected branch")
    };
    assert_eq!(children.len(), 3);
  }

  #[test]
  fn parse_literals() {
    let p = parse_program("[ hello; 42; ]").unwrap();
    let Node::Branch { children, .. } = &p.main else {
      panic!("branch")
    };
    assert!(matches!(&children[0], Node::Leaf { lit: Literal::Ident(s), .. } if s == "hello"));
    assert!(matches!(
      &children[1],
      Node::Leaf {
        lit: Literal::Num(42),
        ..
      }
    ));
  }

  #[test]
  fn parse_nested_branch() {
    let p = parse_program("[ [ a; b ]; c; ]").unwrap();
    let Node::Branch { children, .. } = &p.main else {
      panic!("branch")
    };
    assert_eq!(children.len(), 2);
    let Node::Branch {
      children: inner, ..
    } = &children[0]
    else {
      panic!("inner branch")
    };
    assert_eq!(inner.len(), 2);
  }

  #[test]
  fn parse_function_def() {
    let p = parse_program("!f(:a) = [ x; ]; [ y; ]").unwrap();
    assert_eq!(p.functions.len(), 1);
    assert_eq!(p.functions[0].name, "f");
    assert_eq!(p.functions[0].params, ["a"]);
  }

  #[test]
  fn parse_inline_call() {
    let p = parse_program("[ hello <a|b|c> ]").unwrap();
    let Node::Branch { children, .. } = &p.main else {
      panic!("branch")
    };
    let Node::InlineCall { name, options, .. } = &children[0] else {
      panic!("inline")
    };
    assert_eq!(name, "hello");
    assert_eq!(options.len(), 3);
  }

  #[test]
  fn parse_func_call() {
    let p = parse_program("[ !f(мир); ]").unwrap();
    let Node::Branch { children, .. } = &p.main else {
      panic!("branch")
    };
    let Node::FuncCall { name, args, .. } = &children[0] else {
      panic!("funccall")
    };
    assert_eq!(name, "f");
    assert_eq!(args.len(), 1);
  }

  #[test]
  fn parse_binary_op_concat() {
    let p = parse_program(r#"[ "a" + "b"; ]"#).unwrap();
    let Node::Branch { children, .. } = &p.main else {
      panic!("branch")
    };
    let Node::BinaryOp {
      op: BinOp::Plus,
      left,
      right,
      ..
    } = &children[0]
    else {
      panic!("binary op")
    };
    assert!(matches!(left.as_ref(), Node::Leaf { lit: Literal::Str(s), .. } if s == "a"));
    assert!(matches!(right.as_ref(), Node::Leaf { lit: Literal::Str(s), .. } if s == "b"));
  }

  #[test]
  fn parse_binary_op_repeat() {
    let p = parse_program(r#"[ "x" * 2; ]"#).unwrap();
    let Node::Branch { children, .. } = &p.main else {
      panic!("branch")
    };
    let Node::BinaryOp {
      op: BinOp::Star,
      left,
      right,
      ..
    } = &children[0]
    else {
      panic!("binary op")
    };
    assert!(matches!(left.as_ref(), Node::Leaf { lit: Literal::Str(s), .. } if s == "x"));
    assert!(matches!(
      right.as_ref(),
      Node::Leaf {
        lit: Literal::Num(2),
        ..
      }
    ));
  }

  #[test]
  fn parse_include() {
    let p = parse_program(r#"include "lib.branchy"; [ x; ]"#).unwrap();
    assert_eq!(p.includes, ["lib.branchy"]);
    let Node::Branch { children, .. } = &p.main else {
      panic!("branch")
    };
    assert_eq!(children.len(), 1);
  }

  #[test]
  fn parse_spread_param() {
    let p = parse_program(r#"[ a; ...:x; b; ]"#).unwrap();
    let Node::Branch { children, .. } = &p.main else {
      panic!("branch")
    };
    assert_eq!(children.len(), 3);
    assert!(matches!(&children[0], Node::Leaf { lit: Literal::Ident(s), .. } if s == "a"));
    assert!(matches!(&children[1], Node::SpreadParam { param: s, .. } if s == "x"));
    assert!(matches!(&children[2], Node::Leaf { lit: Literal::Ident(s), .. } if s == "b"));
  }

  #[test]
  fn parse_spread_include() {
    let p = parse_program(r#"[ ...include "mix.branchy"; ]"#).unwrap();
    let Node::Branch { children, .. } = &p.main else {
      panic!("branch")
    };
    assert_eq!(children.len(), 1);
    assert!(matches!(&children[0], Node::SpreadInclude { path: s, .. } if s == "mix.branchy"));
  }

  #[test]
  fn parse_two_adjacent_branches() {
    let p = parse_program("[ a; b; ] [ a; b; ];").unwrap();
    let Node::Branch { children, .. } = &p.main else {
      panic!("expected outer branch")
    };
    assert_eq!(children.len(), 2);
    let Node::Branch { children: c1, .. } = &children[0] else {
      panic!("first child branch")
    };
    let Node::Branch { children: c2, .. } = &children[1] else {
      panic!("second child branch")
    };
    assert_eq!(c1.len(), 2);
    assert_eq!(c2.len(), 2);
  }
}
