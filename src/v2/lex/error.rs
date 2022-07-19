use std::{error::Error, fmt};
use super::Position;

#[derive(Clone, Debug, PartialEq)]
pub enum LexErrorKind {
    ExpectedComment,
    ExpectedNumber,
    UnclosedQuotedIdentifier,
    UnclosedString,
    UnexpectedCharacter(char),
}

#[derive(Clone, Debug, PartialEq)]
pub struct LexError {
    pub position: Position,
    pub kind: LexErrorKind,
}

impl LexError {
    pub fn expected_comment(position: Position) -> Self {
        Self {
            kind: LexErrorKind::ExpectedComment,
            position,
        }
    }

    pub fn expected_number(position: Position) -> Self {
        Self {
            kind: LexErrorKind::ExpectedNumber,
            position,
        }
    }
    pub fn unclosed_quoted_identifier(position: Position) -> Self {
        Self {
            kind: LexErrorKind::UnclosedQuotedIdentifier,
            position,
        }
    }

    pub fn unclosed_string(position: Position) -> Self {
        Self {
            kind: LexErrorKind::UnclosedString,
            position,
        }
    }

    pub fn unexpected_character(position: Position, c: char) -> Self {
        Self {
            kind: LexErrorKind::UnexpectedCharacter(c),
            position,
        }
    }
}

impl Error for LexError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use LexErrorKind::*;

        match self.kind {
            ExpectedComment => write!(f, "Expected comment {}", self.position),
            ExpectedNumber => write!(f, "Expected number {}", self.position),
            UnclosedQuotedIdentifier => write!(f, "Unclosed quoted identifier {}", self.position),
            UnclosedString => write!(f, "Unclosed string {}", self.position),
            UnexpectedCharacter(c) => write!(f, "Unexpected character `{}` {}", c, self.position),
        }
    }
}
