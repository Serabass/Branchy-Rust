//! Lexer: tokenize Branchy source with optional byte offsets.

mod cursor;
mod err;
mod ident;
mod legacy;
mod read;
mod scan;
mod scan_bracket;
mod scan_rest;
mod token;

#[cfg(test)]
mod tests;

pub use legacy::tokenize;
pub use scan::tokenize_with_offsets;
pub use token::Token;
