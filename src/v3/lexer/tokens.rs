use crate::v3::Position;

#[derive(Clone, Debug, PartialEq)]
pub enum Keyword {
    As,
    Schema,
    Table,
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

#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub position: Position,
}
