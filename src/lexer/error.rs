use std::error::Error;
use std::fmt;
use crate::Position;

#[derive(Clone, Debug, PartialEq)]
pub enum LexErrorKind {
    // ExpectedComment,
    // ExpectedNumber,
    // UnclosedQuotedIdentifier,
    // UnclosedString,
    UnexpectedEOF,
    UnexpectedCharacter(char),
}

impl fmt::Display for LexErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LexErrorKind::UnexpectedEOF => {
                write!(f, "unexpected end of file")
            }
            LexErrorKind::UnexpectedCharacter(c) => {
                write!(f, "unexpected character '{}'", c)
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
    pub fn unexpected(c: char, position: Position) -> Self {
        Self { kind: LexErrorKind::UnexpectedCharacter(c), position }
    }

    pub fn unexpected_eof(position: Position) -> Self {
        Self { kind: LexErrorKind::UnexpectedEOF, position  }
    }
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} at {}", self.kind, self.position)
    }
}

impl Error for LexError {}
