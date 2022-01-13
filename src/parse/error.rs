use std::{error::Error, fmt};

use super::Token;

#[derive(Debug, PartialEq)]
pub enum ParseErrorKind {
    InconsistentIndent { unit: String, received: String },
    MissingColumnValue,
    MissingRecord,
    MissingSchema,
    MissingTable,
    UnexpectedIndentLevel(usize),
    UnexpectedToken(Token),

    // These should only happen from a bug in the lexer or
    // if indent tokens are manually created
    EmptyIndent,
    InvalidIndent(String),
}

#[derive(Debug, PartialEq)]
pub struct ParseError {
    pub line: usize,
    pub kind: ParseErrorKind,
}

impl ParseError {
    pub fn empty_indent(line: usize) -> Self {
        Self {
            kind: ParseErrorKind::EmptyIndent,
            line,
        }
    }

    pub fn inconsistent_indent(line: usize, unit: String, received: String) -> Self {
        Self {
            kind: ParseErrorKind::InconsistentIndent { unit, received },
            line,
        }
    }

    pub fn invalid_indent(line: usize, indent: String) -> Self {
        Self {
            kind: ParseErrorKind::InvalidIndent(indent),
            line,
        }
    }

    pub fn missing_column_value(line: usize) -> Self {
        Self {
            kind: ParseErrorKind::MissingColumnValue,
            line,
        }
    }

    pub fn missing_record(line: usize) -> Self {
        Self {
            kind: ParseErrorKind::MissingRecord,
            line,
        }
    }

    pub fn missing_schema(line: usize) -> Self {
        Self {
            kind: ParseErrorKind::MissingSchema,
            line,
        }
    }

    pub fn missing_table(line: usize) -> Self {
        Self {
            kind: ParseErrorKind::MissingTable,
            line,
        }
    }

    pub fn unexpected_indent_level(line: usize, level: usize) -> Self {
        Self {
            kind: ParseErrorKind::UnexpectedIndentLevel(level),
            line,
        }
    }

    pub fn unexpected_token(line: usize, t: Token) -> Self {
        Self {
            kind: ParseErrorKind::UnexpectedToken(t),
            line,
        }
    }
}

impl Error for ParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use ParseErrorKind::*;

        match &self.kind {
            EmptyIndent => write!(f, "Empty indent on line {}", self.line,),
            InconsistentIndent { unit, received } => write!(
                f,
                "Expected indentation unit of '{}', received '{}' on line {}",
                unit, received, self.line,
            ),
            InvalidIndent(indent) => {
                write!(f, "Invalid indentation '{}' on line {}", indent, self.line,)
            }
            MissingColumnValue => write!(f, "Missing column value on line {}", self.line,),
            MissingRecord => write!(f, "No record present for column on line {}", self.line,),
            MissingSchema => write!(f, "No schema present for object on line {}", self.line,),
            MissingTable => write!(f, "No table present for object on line {}", self.line,),
            UnexpectedIndentLevel(level) => write!(
                f,
                "Unexpected indentation level {} on line {}",
                level, self.line,
            ),
            UnexpectedToken(t) => {
                write!(f, "Unexpected ")?;
                match t {
                    Token::AtSign => write!(f, "'@'")?,
                    Token::Boolean(b) => write!(f, "`{}`", b)?,
                    Token::Identifier(i) => write!(f, "identifier `{}`", i)?,
                    Token::Indent(_) => write!(f, "indent")?,
                    Token::Newline => write!(f, "newline")?,
                    Token::Number(_) => write!(f, "number")?,
                    Token::Period => write!(f, "'.'")?,
                    Token::QuotedIdentifier(i) => write!(f, "quoted identifier `\"{}\"`", i)?,
                    Token::Text(_) => write!(f, "string")?,
                    Token::Underscore => write!(f, "underscore")?,
                }
                write!(f, " on line {}", self.line)
            }
        }
    }
}
