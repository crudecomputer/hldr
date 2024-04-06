use std::error::Error;
use std::fmt;
use crate::lexer::tokens::Token;

// TODO: These need positions
#[derive(Clone, Debug, PartialEq)]
pub enum ParseErrorKind {
    // Should parser just store token directly alongside kind?
    // Would that work for eof at all? An EOF token should work..
    ExpectedAliasName(Token),
    ExpectedAliasOrScope(Token),
    ExpectedCloseOrNewline(Token),
    ExpectedIdentifier(Token),
    ExpectedScope(Token),
    ExpectedSchemaName(Token),
    ExpectedTableName(Token),
    ExpectedValue(Token),
    // TODO: But this one breaks the Token pattern, and has no position
    RecordNameQuoted(String),
    UnexpectedEOF,
    UnexpectedInSchema(Token),
    UnexpectedInTable(Token),
    UnexpectedInRecord(Token),
    UnexpectedToken(Token),
}

impl fmt::Display for ParseErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use ParseErrorKind::*;

        match self {
            ExpectedAliasName(t) => {
                write!(f, "expected identifier for alias name, found `{}`", t.kind)
            }
            ExpectedAliasOrScope(t) => {
                write!(f, "expected alias or opening parenthesis, found `{}`", t.kind)
            }
            ExpectedCloseOrNewline(t) => {
                write!(f, "expected newline or closing parenthesis, found `{}`", t.kind)
            }
            ExpectedIdentifier(t) => {
                write!(f, "expected identifier, found `{}`", t.kind)
            }
            ExpectedSchemaName(t) => {
                write!(f, "expected identifier for schema name, found `{}`", t.kind)
            }
            ExpectedTableName(t) => {
                write!(f, "expected identifier for table name, found `{}`", t.kind)
            }
            ExpectedScope(t) => {
                write!(f, "expected opening parenthesis, found `{}`", t.kind)
            }
            ExpectedValue(t) => {
                write!(f, "expected value, found `{}`", t.kind)
            }
            RecordNameQuoted(s) => {
                write!(f, "expected unquoted record name in reference, found `{}`", s)
            }
            UnexpectedEOF => {
                write!(f, "unexpected end of file")
            }
            UnexpectedInSchema(t) => {
                write!(f, "expected table declaration or closing parenthesis, found `{}`", t.kind)
            }
            UnexpectedInTable(t) => {
                write!(f, "expected record declaration or closing parenthesis, found `{}`", t.kind)
            }
            UnexpectedInRecord(t) => {
                write!(f, "expected column declaration or closing parenthesis, found `{}`", t.kind)
            }
            UnexpectedToken(t) => {
                write!(f, "unexpected token `{}`", t.kind)
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct ParseError {
    pub kind: ParseErrorKind,
}

impl ParseError {
    pub(crate) fn alias_or_scope(t: Token) -> Self {
        Self { kind: ParseErrorKind::ExpectedAliasOrScope(t) }
    }

    pub(crate) fn eof() -> Self {
        Self { kind: ParseErrorKind::UnexpectedEOF }
    }

    pub(crate) fn exp_alias(t: Token) -> Self {
        Self { kind: ParseErrorKind::ExpectedAliasName(t) }
    }

    pub(crate) fn exp_close(t: Token) -> Self {
        Self { kind: ParseErrorKind::ExpectedCloseOrNewline(t) }
    }

    pub(crate) fn exp_ident(t: Token) -> Self {
        Self { kind: ParseErrorKind::ExpectedIdentifier(t) }
    }

    pub(crate) fn exp_scope(t: Token) -> Self {
        Self { kind: ParseErrorKind::ExpectedScope(t) }
    }

    pub(crate) fn exp_schema(t: Token) -> Self {
        Self { kind: ParseErrorKind::ExpectedSchemaName(t) }
    }

    pub(crate) fn exp_table(t: Token) -> Self {
        Self { kind: ParseErrorKind::ExpectedTableName(t) }
    }

    pub(crate) fn exp_value(t: Token) -> Self {
        Self { kind: ParseErrorKind::ExpectedValue(t) }
    }

    pub(crate) fn in_record(t: Token) -> Self {
        Self { kind: ParseErrorKind::UnexpectedInTable(t) }
    }

    pub(crate) fn in_schema(t: Token) -> Self {
        Self { kind: ParseErrorKind::UnexpectedInSchema(t) }
    }

    pub(crate) fn in_table(t: Token) -> Self {
        Self { kind: ParseErrorKind::UnexpectedInTable(t) }
    }

    pub(crate) fn rec_quot(s: String) -> Self {
        Self { kind: ParseErrorKind::RecordNameQuoted(s) }
    }

    pub(crate) fn token(t: Token) -> Self {
        Self { kind: ParseErrorKind::UnexpectedToken(t) }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.kind)
    }
}

impl Error for ParseError {}
