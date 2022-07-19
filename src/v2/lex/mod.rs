pub mod error;

mod position;
mod tokenizer;

pub use error::{LexError, LexErrorKind};
pub use position::Position;
pub use tokenizer::{Keyword, Token, TokenPosition};

pub fn lex(text: &str) -> Result<Vec<TokenPosition>, LexError> {
    Ok(tokenizer::Tokenizer::new().tokenize(text)?.tokens)
}
