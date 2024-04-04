use std::error::Error;
use std::fmt;
use crate::lexer::tokens::Token;

#[derive(Clone, Debug, PartialEq)]
pub enum ParseErrorKind {
    UnexpectedEOF, // TODO: Replace with more specific expectations
    UnexpectedToken(Token),
}

impl fmt::Display for ParseErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseErrorKind::UnexpectedEOF => {
                write!(f, "unexpected end of file")
            }
            ParseErrorKind::UnexpectedToken(t) => {
                write!(f, "unexpected token '{}'", t.kind)
            }
        }
    }
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

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.kind)
    }
}

impl Error for ParseError {}
