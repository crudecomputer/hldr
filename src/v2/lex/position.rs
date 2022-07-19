use std::fmt;

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
