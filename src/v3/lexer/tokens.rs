#[derive(Debug, PartialEq)]
pub enum Keyword {
    As,
}

#[derive(Debug, PartialEq)]
pub enum Symbol {
    Hash,
    Period,
    Underscore,
}

#[derive(Debug, PartialEq)]
pub enum Token {
    Bool(bool),
    Identifier(String),
    Keyword(Keyword),
    Newline,
    Number(String),
    Symbol(Symbol),
    Text(String),
    Whitespace(String),
}