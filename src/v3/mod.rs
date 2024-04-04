use std::fmt;

pub mod analyzer;
pub mod lexer;
pub mod loader;
pub mod parser;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(line {}, column {})", self.line, self.column)
    }
}
