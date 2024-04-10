use std::error::Error;
use std::fmt;
use crate::Position;
use crate::lexer::tokens::Token;

#[derive(Clone, Debug, PartialEq)]
pub enum ParseErrorKind {
    UnexpectedEOF,
    // Should parser just store token directly alongside kind?
    // Would that work for eof at all? An EOF token should work..
    ExpectedAliasName(Token),
    ExpectedAliasOrScope(Token),
    ExpectedCloseAttribute(Token),
    ExpectedIdentifier(Token),
    ExpectedScope(Token),
    ExpectedSchemaName(Token),
    ExpectedTableName(Token),
    ExpectedValue(Token),
    UnexpectedInSchema(Token),
    UnexpectedInTable(Token),
    UnexpectedInRecord(Token),
    UnexpectedToken(Token),
    // But this one breaks the Token pattern
    RecordNameQuoted(String, Position),
}

impl fmt::Display for ParseErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use ParseErrorKind::*;

        match self {
            ExpectedAliasName(t) => {
                write!(f, "expected identifier for alias name, found {}", t.kind)
            }
            ExpectedAliasOrScope(t) => {
                write!(f, "expected alias or opening parenthesis, found {}", t.kind)
            }
            ExpectedCloseAttribute(t) => {
                write!(f, "expected comma, newline, or closing parenthesis, found {}", t.kind)
            }
            ExpectedIdentifier(t) => {
                write!(f, "expected identifier, found {}", t.kind)
            }
            ExpectedSchemaName(t) => {
                write!(f, "expected identifier for schema name, found {}", t.kind)
            }
            ExpectedTableName(t) => {
                write!(f, "expected identifier for table name, found {}", t.kind)
            }
            ExpectedScope(t) => {
                write!(f, "expected opening parenthesis, found {}", t.kind)
            }
            ExpectedValue(t) => {
                write!(f, "expected value, found {}", t.kind)
            }
            RecordNameQuoted(s, _) => {
                write!(f, "expected unquoted record name in reference, found `\"{}\"`", s)
            }
            UnexpectedEOF => {
                write!(f, "unexpected end of file")
            }
            UnexpectedInSchema(t) => {
                write!(f, "expected table declaration or closing parenthesis, found {}", t.kind)
            }
            UnexpectedInTable(t) => {
                write!(f, "expected record declaration or closing parenthesis, found {}", t.kind)
            }
            UnexpectedInRecord(t) => {
                write!(f, "expected column declaration or closing parenthesis, found {}", t.kind)
            }
            UnexpectedToken(t) => {
                write!(f, "unexpected {}", t.kind)
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

    pub(crate) fn exp_close_attr(t: Token) -> Self {
        Self { kind: ParseErrorKind::ExpectedCloseAttribute(t) }
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
        Self { kind: ParseErrorKind::UnexpectedInRecord(t) }
    }

    pub(crate) fn in_schema(t: Token) -> Self {
        Self { kind: ParseErrorKind::UnexpectedInSchema(t) }
    }

    pub(crate) fn in_table(t: Token) -> Self {
        Self { kind: ParseErrorKind::UnexpectedInTable(t) }
    }

    pub(crate) fn rec_quot(s: String, p: Position) -> Self {
        Self { kind: ParseErrorKind::RecordNameQuoted(s, p) }
    }

    pub(crate) fn token(t: Token) -> Self {
        Self { kind: ParseErrorKind::UnexpectedToken(t) }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use ParseErrorKind::*;

        // TODO: This is an indication that the parser error could use some
        // general tidying up, given that the `RecordNameQuoted` kind is handled
        // the same way as the others but needs to be destructured differently,
        // and only the `UnexpectedEOF` kind is reported without a line number
        match self.kind {
            ExpectedAliasName(ref t)
            | ExpectedAliasOrScope(ref t)
            | ExpectedCloseAttribute(ref t)
            | ExpectedIdentifier(ref t)
            | ExpectedScope(ref t)
            | ExpectedSchemaName(ref t)
            | ExpectedTableName(ref t)
            | ExpectedValue(ref t)
            | UnexpectedInSchema(ref t)
            | UnexpectedInTable(ref t)
            | UnexpectedInRecord(ref t)
            | UnexpectedToken(ref t) => {
                // TODO: Token positions' columns are not always accurate, so they
                // need to be tightened up before reporting in parser errors. Or maybe
                // the column is less relevant for parser errors than it is for lexer?
                write!(f, "{} on line {}", self.kind, t.position.line)
            }
            RecordNameQuoted(_, p) => {
                write!(f, "{} on line {}", self.kind, p.line)
            }
            _ => {
                write!(f, "{}", self.kind)
            }
        }
    }
}

impl Error for ParseError {}
