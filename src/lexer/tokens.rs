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
        use Keyword::*;

        match self {
            As => write!(f, "as"),
            Schema => write!(f, "schema"),
            Table => write!(f, "table"),
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
        use Symbol::*;

        match self {
            AtSign => write!(f, "@"),
            Comma => write!(f, ","),
            ParenLeft => write!(f, "("),
            ParenRight => write!(f, ")"),
            Period => write!(f, "."),
            Underscore => write!(f, "_"),
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
    SqlFragment(String),
    Symbol(Symbol),
    Text(String),
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use TokenKind::*;

        match self {
            Bool(b) => write!(f, "boolean `{}`", b),
            Identifier(i) => write!(f, "identifier `{}`", i),
            Keyword(k) => write!(f, "keyword `{}`", k),
            LineSep => write!(f, "newline"),
            Number(n) => write!(f, "number `{}`", n),
            QuotedIdentifier(i) => write!(f, "quoted identifier `\"{}\"`", i),
            SqlFragment(s) => write!(f, "SQL fragment `{}`", s),
            Symbol(s) => write!(f, "symbol `{}`", s),
            Text(s) => write!(f, "string '{}'", s),
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
