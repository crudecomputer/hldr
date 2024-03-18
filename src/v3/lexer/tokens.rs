#[derive(Debug, PartialEq)]
pub enum Keyword {
    As,
}

#[derive(Debug, PartialEq)]
pub enum Token {
    Bool(bool),
    Keyword(Keyword),
    Identifier(String),
    Newline,
    Number(String),
    Underscore,
    Whitespace(String),
}