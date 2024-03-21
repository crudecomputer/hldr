#[derive(Debug, PartialEq)]
pub enum Keyword {
    As,
    Schema,
    Table,
}

#[derive(Debug, PartialEq)]
pub enum Symbol {
    AtSign,
    Comma,
    ParenLeft,
    ParenRight,
    Period,
    Underscore,
}

#[derive(Debug, PartialEq)]
pub enum Token {
    Bool(bool),
    Identifier(String),
    Keyword(Keyword),
    LineSep,
    Number(String),
    QuotedIdentifier(String),
    Symbol(Symbol),
    Text(String),
}