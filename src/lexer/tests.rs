#[cfg(test)]
mod tests {
  use super::super::{tokenize, tokenize_with_offsets};
  use super::super::Token;

  #[test]
  fn tokenize_spread_param_top_level() {
    let t = tokenize_with_offsets("...:x").unwrap();
    let tokens: Vec<_> = t.into_iter().map(|(tok, _, _)| tok).collect();
    assert_eq!(
      tokens,
      [Token::Spread, Token::Param("x".into())],
      "top-level ...:x"
    );
  }

  #[test]
  fn tokenize_with_offsets_matches_tokenize() {
    for input in [
      "[ a; b; ]",
      "!f(:a) = [ x; ]; [ y; ]",
      r#"[ a; ...:x; b; ]"#,
    ] {
      let a = tokenize(input).unwrap();
      let b: Vec<_> = tokenize_with_offsets(input)
        .unwrap()
        .into_iter()
        .map(|(t, _, _)| t)
        .collect();
      assert_eq!(a, b, "input: {:?}", input);
    }
  }

  #[test]
  fn string_escape_sequences() {
    let t = tokenize(r#"[ "a\nb"; "x\ty"; "\\"; ]"#).unwrap();
    let strs: Vec<_> = t
      .iter()
      .filter_map(|tok| match tok {
        Token::Str(s) => Some(s.clone()),
        _ => None,
      })
      .collect();
    assert_eq!(strs.len(), 3);
    assert_eq!(strs[0], "a\nb");
    assert_eq!(strs[0].as_bytes()[1], b'\n');
    assert_eq!(strs[1], "x\ty");
    assert_eq!(strs[1].as_bytes()[1], b'\t');
    assert_eq!(strs[2], "\\");
  }
}
