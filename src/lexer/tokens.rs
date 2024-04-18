use crate::Position;
use std::fmt;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_keyword() {
        use Keyword::*;

        assert_eq!(format!("{}", As), "as");
        assert_eq!(format!("{}", Schema), "schema");
        assert_eq!(format!("{}", Table), "table");
    }

    #[test]
    fn test_display_symbol() {
        use Symbol::*;

        assert_eq!(format!("{}", AtSign), "@");
        assert_eq!(format!("{}", Comma), ",");
        assert_eq!(format!("{}", ParenLeft), "(");
        assert_eq!(format!("{}", ParenRight), ")");
        assert_eq!(format!("{}", Period), ".");
        assert_eq!(format!("{}", Underscore), "_");
    }

    #[test]
    fn test_display_token_kind() {
        use super::Keyword::As;
        use super::Symbol::Comma;
        use TokenKind::*;

        assert_eq!(format!("{}", Bool(true)), "boolean `true`");
        assert_eq!(format!("{}", Identifier("foo".to_string())), "identifier `foo`");
        assert_eq!(format!("{}", Keyword(As)), "keyword `as`");
        assert_eq!(format!("{}", LineSep), "newline");
        assert_eq!(format!("{}", Number("42".to_string())), "number `42`");
        assert_eq!(format!("{}", QuotedIdentifier("foo".to_string())), "quoted identifier `\"foo\"`");
        assert_eq!(format!("{}", Symbol(Comma)), "symbol `,`");
        assert_eq!(format!("{}", Text("foo".to_string())), "string 'foo'");
    }
}
