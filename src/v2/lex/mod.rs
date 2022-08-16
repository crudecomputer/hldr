pub mod error;

mod position;
mod tokenizer;

pub use error::{LexError, LexErrorKind};
pub use position::Position;
pub use tokenizer::{Keyword, Number, Symbol, Token, TokenPosition, Whitespace};

pub fn lex(text: &str) -> Result<Vec<TokenPosition>, LexError> {
    Ok(tokenizer::Tokenizer::new().tokenize(text)?.tokens)
}
