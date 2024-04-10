use std::fmt;
use crate::Position;

#[derive(Clone, Debug, PartialEq)]
pub enum Keyword {
    As,
    Schema,
    Table,
}

impl fmt::Display for Keyword {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Keyword::As => write!(f, "as"),
            Keyword::Schema => write!(f, "schema"),
            Keyword::Table => write!(f, "table"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Symbol {
    AtSign,
    Comma,
    ParenLeft,
    ParenRight,
    Period,
    Underscore,
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Symbol::AtSign => write!(f, "@"),
            Symbol::Comma => write!(f, ","),
            Symbol::ParenLeft => write!(f, "("),
            Symbol::ParenRight => write!(f, ")"),
            Symbol::Period => write!(f, "."),
            Symbol::Underscore => write!(f, "_"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum TokenKind {
    Bool(bool),
    Identifier(String),
    Keyword(Keyword),
    LineSep,
    Number(String),
    QuotedIdentifier(String),
    Symbol(Symbol),
    Text(String),
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TokenKind::Bool(b) => write!(f, "boolean `{}`", b),
            TokenKind::Identifier(i) => write!(f, "identifier `{}`", i),
            TokenKind::Keyword(k) => write!(f, "keyword `{}`", k),
            TokenKind::LineSep => write!(f, "newline"),
            TokenKind::Number(n) => write!(f, "number `{}`", n),
            TokenKind::QuotedIdentifier(i) => write!(f, "quoted identifier `\"{}\"`", i),
            TokenKind::Symbol(s) => write!(f, "symbol `{}`", s),
            TokenKind::Text(s) => write!(f, "string '{}'", s),
        }

    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub position: Position,
}
