/// Cursor over a string slice that yields (global_byte_offset, char).
pub(super) struct OffsetCursor<'a> {
  pub s: &'a str,
  pub base: usize,
  pub pos: usize,
}

impl<'a> OffsetCursor<'a> {
  pub fn new(s: &'a str, base: usize) -> Self {
    Self { s, base, pos: 0 }
  }
  pub fn next(&mut self) -> Option<(usize, char)> {
    let c = self.s[self.pos..].chars().next()?;
    let start = self.base + self.pos;
    self.pos += c.len_utf8();
    Some((start, c))
  }
  pub fn peek(&self) -> Option<(usize, char)> {
    let c = self.s[self.pos..].chars().next()?;
    Some((self.base + self.pos, c))
  }
  pub fn position(&self) -> usize {
    self.base + self.pos
  }
}
