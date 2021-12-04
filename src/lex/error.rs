use std::{error::Error, fmt};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(line {}, column {}", self.line, self.column)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum LexErrorKind {
    ExpectedComment,
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
        Self { position, kind: LexErrorKind::ExpectedComment }
    }

    pub fn unclosed_quoted_identifier(position: Position) -> Self {
        Self { position, kind: LexErrorKind::UnclosedQuotedIdentifier }
    }

    pub fn unclosed_string(position: Position) -> Self {
        Self { position, kind: LexErrorKind::UnclosedString }
    }

    pub fn unexpected_character(position: Position, c: char) -> Self {
        Self { position, kind: LexErrorKind::UnexpectedCharacter(c) }
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
            UnclosedQuotedIdentifier => write!(f, "Unclosed quoted identifier {}", self.position),
            UnclosedString => write!(f, "Unclosed string {}", self.position),
            UnexpectedCharacter(c) => write!(f, "Unexpected character '{}' {}", c, self.position),
        }
    }
}
