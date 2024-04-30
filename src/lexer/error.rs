use crate::Position;
use std::error::Error;
use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub enum LexErrorKind {
    InvalidNumericLiteral(String),
    UnclosedQuotedIdentifier,
    UnclosedString,
    UnexpectedEOF,
    UnexpectedCharacter(char),
}

impl fmt::Display for LexErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use LexErrorKind::*;

        match self {
            InvalidNumericLiteral(n) => {
                write!(f, "invalid numeric literal `{}`", n)
            }
            UnclosedQuotedIdentifier => {
                write!(f, "unclosed quoted identifier starting")
            }
            UnclosedString => {
                write!(f, "unclosed string starting")
            }
            UnexpectedEOF => {
                write!(f, "unexpected end of file")
            }
            UnexpectedCharacter(c) => {
                write!(f, "unexpected character `{}`", c)
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LexError {
    pub kind: LexErrorKind,
    pub position: Position,
}

impl LexError {
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} at {}", self.kind, self.position)
    }
}

impl Error for LexError {}
