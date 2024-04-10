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
    pub(crate) fn bad_char(c: char, position: Position) -> Self {
        Self {
            kind: LexErrorKind::UnexpectedCharacter(c),
            position,
        }
    }

    pub(crate) fn bad_number(n: String, position: Position) -> Self {
        Self {
            kind: LexErrorKind::InvalidNumericLiteral(n),
            position,
        }
    }

    pub(crate) fn eof(position: Position) -> Self {
        Self {
            kind: LexErrorKind::UnexpectedEOF,
            position,
        }
    }

    pub(crate) fn eof_unquoted(position: Position) -> Self {
        Self {
            kind: LexErrorKind::UnclosedQuotedIdentifier,
            position,
        }
    }

    pub(crate) fn eof_string(position: Position) -> Self {
        Self {
            kind: LexErrorKind::UnclosedString,
            position,
        }
    }
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} at {}", self.kind, self.position)
    }
}

impl Error for LexError {}
