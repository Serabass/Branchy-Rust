#[derive(Debug, Clone, PartialEq)]
pub enum Token {
  LBrack,
  RBrack,
  Semicolon,
  LBrace,
  RBrace,
  Equals,
  LParen,
  RParen,
  Comma,
  LAngle,
  RAngle,
  Pipe,
  Bang,
  At,
  Tilde,
  Plus,
  Star,
  Spread,
  /// Two dots `..` for range (e.g. 1..3); three dots are Spread
  RangeSep,
  Include,
  Ident(String),
  Param(String),
  /// Optional param :?name — may or may not be output (coin flip)
  OptionalParam(String),
  Num(i64),
  Str(String),
  /// Inline char block: [a-zA-Z], [abc:5], [a-z:2..5]. Content is "set" or "set:n" or "set:lo..hi".
  CharBlock(String),
}
