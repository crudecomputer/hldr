#[derive(Clone, Debug, PartialEq)]
pub enum LexErrorKind {
    // ExpectedComment,
    // ExpectedNumber,
    // UnclosedQuotedIdentifier,
    // UnclosedString,
    UnexpectedEOF,
    UnexpectedCharacter(char),
}

#[derive(Clone, Debug, PartialEq)]
pub struct LexError {
    pub kind: LexErrorKind,
    // pub position: Position,
}

impl LexError {
    pub fn unexpected(c: char /*, position: Position */) -> Self {
        Self { kind: LexErrorKind::UnexpectedCharacter(c)}
    }

    pub fn unexpected_eof(/*, position: Position */) -> Self {
        Self { kind: LexErrorKind::UnexpectedEOF }
    }
}
