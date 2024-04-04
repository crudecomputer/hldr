use crate::v3::Position;
use crate::v3::lexer::Token;

#[derive(Clone, Debug, PartialEq)]
pub enum ParseErrorKind {
    UnexpectedEOF,
    UnexpectedToken(Token),
}

#[derive(Debug, PartialEq)]
pub struct ParseError {
    pub kind: ParseErrorKind,
}

impl ParseError {
    pub fn unexpected(token: Token) -> Self {
        Self { kind: ParseErrorKind::UnexpectedToken(token) }
    }
}
