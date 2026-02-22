//! Parser for Branchy source: program, expressions, branches, calls.

mod char_block;
mod expr;
mod expr_call;
mod program;
mod stream;

#[cfg(test)]
mod tests;

pub use program::parse_program;
