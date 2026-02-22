use branchy::parse_program;

#[test]
fn parse_error_unclosed_bracket() {
  let r = parse_program("[ a; b; ");
  assert!(r.is_err());
}

#[test]
fn parse_error_unexpected_token() {
  let r = parse_program("[ a $ b; ]");
  assert!(r.is_err());
}

#[test]
fn lexer_quoted_string() {
  let tokens = branchy::tokenize(r#"[ "hello"; ]"#).unwrap();
  assert!(matches!(tokens.get(1), Some(branchy::lexer::Token::Str(s)) if s == "hello"));
}
