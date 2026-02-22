pub(super) fn is_ident_start(c: char) -> bool {
  c.is_alphabetic() || c == '_' || !c.is_ascii()
}

pub(super) fn is_ident_cont(c: char) -> bool {
  c.is_alphanumeric() || c == '_' || !c.is_ascii()
}

pub(super) fn is_char_block_content(s: &str) -> bool {
  s.chars()
    .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == ':' || c == '.' || c.is_whitespace())
    && !s.trim().is_empty()
}
