#[derive(Clone, Copy, Debug)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

#[derive(Debug)]
pub enum LexErrorKind {
    ExpectedComment,
    UnclosedQuotedIdentifier,
    UnclosedString,
    UnexpectedCharacter(char),
}

#[derive(Debug)]
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
