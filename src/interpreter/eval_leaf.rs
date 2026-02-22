use crate::ast::{CharBlockCount, Literal, SourceError, Span};
use rand::seq::SliceRandom;

use super::eval::EvalState;
use super::{err_span_impl, push_span};

pub(super) fn eval_char_block<R: rand::RngCore>(
  state: &mut EvalState<'_, '_, R>,
  ranges: &[(char, char)],
  count: &CharBlockCount,
  span: Option<Span>,
) -> Result<String, SourceError> {
  push_span(state.trace, span);
  let chars: Vec<char> = ranges
    .iter()
    .flat_map(|&(lo, hi)| (lo..=hi).collect::<Vec<_>>())
    .collect();
  if chars.is_empty() {
    return Ok(String::new());
  }
  let n = match count {
    CharBlockCount::One => 1,
    CharBlockCount::Fixed(k) => (*k).max(0) as usize,
    CharBlockCount::Range(lo, hi) => {
      let (lo, hi) = ((*lo).max(0), (*hi).max(0));
      if hi < lo {
        0
      } else {
        let span_len = (hi - lo + 1) as u32;
        (lo + (state.rng.next_u32() % span_len) as i64) as usize
      }
    }
  };
  let out: String = (0..n).map(|_| *chars.choose(state.rng).unwrap()).collect();
  Ok(out)
}

pub(super) fn eval_leaf<R: rand::RngCore>(
  state: &mut EvalState<'_, '_, R>,
  lit: &Literal,
  span: Option<Span>,
  env: &mut std::collections::HashMap<String, String>,
) -> Result<String, SourceError> {
  push_span(state.trace, span);
  match lit {
    Literal::Param(p) => env
      .get(p)
      .cloned()
      .ok_or_else(|| err_span_impl(format!("undefined param :{}", p), span)),
    Literal::OptionalParam(p) => {
      let value = env.get(p).cloned().unwrap_or_default();
      let show = (state.rng.next_u32() % 2) == 0;
      Ok(if value.is_empty() || !show {
        String::new()
      } else {
        value
      })
    }
    _ => Ok(lit.to_string_value()),
  }
}
