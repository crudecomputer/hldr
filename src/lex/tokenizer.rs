use std::fmt;

use super::error::{LexError, Position};

#[derive(Clone, Debug, PartialEq)]
pub enum Keyword {
    As,
}

impl fmt::Display for Keyword {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Keyword::*;

        Ok(match self{
            As => write!(f, "as")?,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    AtSign,
    Boolean(bool),
    Identifier(String),
    Indent(String),
    Keyword(Keyword),
    Newline,
    Number(String),
    Period,
    QuotedIdentifier(String),
    Text(String),
    Underscore,
}

#[derive(Clone, Debug, PartialEq)]
pub enum State {
    Comment,
    Decimal,
    DecimalExpectingNumber,
    ExpectingComment,
    // TODO: "expecting an identifier" seems out of scope for a lexer
    ExpectingIdentifier,
    Indent,
    Identifier,
    Integer,
    LineStart,
    QuotedIdentifierClosed,
    QuotedIdentifierOpen,
    TextClosed,
    TextOpen,
    Whitespace,
}

#[derive(Debug)]
pub(super) struct Tokenizer {
    state: State,
    stack: Vec<char>,
    pub tokens: Vec<Token>,
}

impl Tokenizer {
    pub fn new() -> Self {
        Self {
            state: State::LineStart,
            stack: Vec::new(),
            tokens: Vec::new(),
        }
    }

    pub fn tokenize(mut self, input: &str) -> Result<Self, LexError> {
        let mut position = Position { line: 0, column: 0 };

        for line in input.lines() {
            self.state = State::LineStart;
            position.line += 1;
            position.column = 0;

            for c in line.chars() {
                position.column += 1;

                let unexpected = || Err(LexError::unexpected_character(position, c));

                self.state = match self.state {
                    State::Comment => State::Comment,

                    State::Decimal => match c {
                        '0'..='9' => {
                            self.stack.push(c);
                            State::Decimal
                        }
                        ' ' | '\t' => {
                            let num: String = self.stack.drain(..).collect();
                            self.tokens.push(Token::Number(num));
                            State::Whitespace
                        }
                        _ => return unexpected(),
                    },

                    State::DecimalExpectingNumber => match c {
                        '0'..='9' => {
                            self.stack.push('.');
                            self.stack.push(c);
                            State::Decimal
                        }
                        _ => return unexpected(),
                    },

                    State::ExpectingComment => match c {
                        '-' => State::Comment,
                        _ => return unexpected(),
                    },

                    State::ExpectingIdentifier => match c {
                        '"' => State::QuotedIdentifierOpen,
                        c if valid_identifier_char(c) => {
                            self.stack.push(c);
                            State::Identifier
                        },
                        _ => return unexpected(),
                    },

                    State::Identifier => match c {
                        c if valid_identifier_char(c) => {
                            self.stack.push(c);
                            State::Identifier
                        }
                        ' ' | '\t' | '.' | '@' => {
                            let ident: String = self.stack.drain(..).collect();
                            self.tokens.push(identifier_to_token(ident));

                            match c {
                                ' ' | '\t' => State::Whitespace,
                                '@' => {
                                    self.tokens.push(Token::AtSign);
                                    State::ExpectingIdentifier
                                },
                                '.' => {
                                    self.tokens.push(Token::Period);
                                    State::ExpectingIdentifier
                                },
                                _ => unreachable!()
                            }
                        }
                        _ => return unexpected(),
                    },

                    State::Indent => match c {
                        ' ' | '\t' => {
                            self.stack.push(c);
                            State::Indent
                        }
                        _ => {
                            let indent: String = self.stack.drain(..).collect();
                            self.tokens.push(Token::Indent(indent));

                            // TODO: Should this be accepting numbers or text values?
                            //
                            // Syntantically only identifiers are allowed after an indent,
                            // but is that the job of the parser to exclude them?
                            match c {
                                '"' => State::QuotedIdentifierOpen,
                                '\'' => State::TextOpen,
                                '-' => State::ExpectingComment,
                                '.' => State::DecimalExpectingNumber,
                                '@' => {
                                    self.tokens.push(Token::AtSign);
                                    State::Whitespace
                                },
                                '0'..='9' => {
                                    self.stack.push(c);
                                    State::Integer
                                }
                                c if valid_identifier_char(c) => {
                                    self.stack.push(c);
                                    State::Identifier
                                }
                                _ => return unexpected(),
                            }
                        }
                    },

                    State::Integer => match c {
                        '0'..='9' => {
                            self.stack.push(c);
                            State::Integer
                        }
                        '.' => {
                            self.stack.push(c);
                            State::Decimal
                        }
                        ' ' | '\t' => {
                            let num: String = self.stack.drain(..).collect();
                            self.tokens.push(Token::Number(num));
                            State::Whitespace
                        }
                        _ => return unexpected(),
                    },

                    State::LineStart => match c {
                        '"' => State::QuotedIdentifierOpen,
                        '\'' => State::TextOpen,
                        '-' => State::ExpectingComment,
                        '.' => State::DecimalExpectingNumber,
                        ' ' | '\t' => {
                            self.stack.push(c);
                            State::Indent
                        }
                        '@' => {
                            self.tokens.push(Token::AtSign);
                            State::Whitespace
                        },
                        '0'..='9' => {
                            self.stack.push(c);
                            State::Integer
                        }
                        c if valid_identifier_char(c) => {
                            self.stack.push(c);
                            State::Identifier
                        }
                        _ => return unexpected(),
                    },

                    State::QuotedIdentifierOpen => match c {
                        '"' => State::QuotedIdentifierClosed,
                        _ => {
                            self.stack.push(c);
                            State::QuotedIdentifierOpen
                        }
                    },

                    State::QuotedIdentifierClosed => match c {
                        // This accounts for escaping double quotes in a quoted identifier,
                        // eg. "as""df" or "as""""df"
                        '"' => {
                            self.stack.push('"');
                            State::QuotedIdentifierOpen
                        }
                        ' ' | '\t' | '@' | '.' => {
                            let text: String = self.stack.drain(..).collect();
                            self.tokens.push(Token::QuotedIdentifier(text));

                            match c {
                                ' ' | '\t' => State::Whitespace,
                                '@' => {
                                    self.tokens.push(Token::AtSign);
                                    // FIXME lexing shouldn't "expect"
                                    State::ExpectingIdentifier
                                },
                                '.' => {
                                    self.tokens.push(Token::Period);
                                    State::ExpectingIdentifier
                                }
                                _ => unreachable!()
                            }
                        }
                        _ => return unexpected(),
                    },

                    State::TextOpen => match c {
                        '\'' => State::TextClosed,
                        _ => {
                            self.stack.push(c);
                            State::TextOpen
                        }
                    },

                    State::TextClosed => match c {
                        '\'' => {
                            self.stack.push('\'');
                            State::TextOpen
                        }
                        ' ' | '\t' => {
                            let text: String = self.stack.drain(..).collect();
                            self.tokens.push(Token::Text(text));
                            State::Whitespace
                        }
                        _ => return unexpected(),
                    },

                    // TODO: This has become something of a generic 'starter' state
                    State::Whitespace => match c {
                        ' ' | '\t' => State::Whitespace,
                        '"' => State::QuotedIdentifierOpen,
                        '\'' => State::TextOpen,
                        '-' => State::ExpectingComment,
                        '@' => {
                            self.tokens.push(Token::AtSign);
                            State::Whitespace
                        }
                        '0'..='9' => {
                            self.stack.push(c);
                            State::Integer
                        }
                        '.' => {
                            self.stack.push(c);
                            State::Decimal
                        }
                        c if valid_identifier_char(c) => {
                            self.stack.push(c);
                            State::Identifier
                        }
                        _ => return unexpected(),
                    },
                }
            }

            match self.state {
                State::DecimalExpectingNumber => {
                    return Err(LexError::expected_number(position));
                }
                State::ExpectingComment => {
                    return Err(LexError::expected_comment(position));
                }
                State::Identifier => {
                    let ident: String = self.stack.drain(..).collect();
                    self.tokens.push(identifier_to_token(ident));
                }
                State::Indent => {
                    let indent: String = self.stack.drain(..).collect();
                    self.tokens.push(Token::Indent(indent));
                }
                State::Integer | State::Decimal => {
                    let num: String = self.stack.drain(..).collect();
                    self.tokens.push(Token::Number(num));
                }
                State::QuotedIdentifierClosed => {
                    let text: String = self.stack.drain(..).collect();
                    self.tokens.push(Token::QuotedIdentifier(text));
                }
                State::QuotedIdentifierOpen => {
                    return Err(LexError::unclosed_quoted_identifier(position));
                }
                State::TextClosed => {
                    let text: String = self.stack.drain(..).collect();
                    self.tokens.push(Token::Text(text));
                }
                State::TextOpen => {
                    return Err(LexError::unclosed_string(position));
                }
                _ => {}
            }

            self.tokens.push(Token::Newline);
        }

        Ok(self)
    }
}

fn identifier_to_token(s: String) -> Token {
    match s.as_ref() {
        "_" => Token::Underscore,
        "true" | "t" => Token::Boolean(true),
        "false" | "f" => Token::Boolean(false),
        "as" => Token::Keyword(Keyword::As),
        _ => Token::Identifier(s),
    }
}

fn valid_identifier_char(c: char) -> bool {
    c == '_'
        || (
            // "alphabetic" isn't enough because that precludes other unicode chars
            // that are valid in postgres identifiers
            !c.is_control() && !c.is_whitespace() && !c.is_ascii_punctuation()
        )
}

#[cfg(test)]
mod tests {
    #[test]
    fn identifier_tokens() {
        use super::{identifier_to_token, Token as T};

        let assert = |s: &str, token: T| {
            assert_eq!(
                identifier_to_token(s.to_owned()),
                token,
                "{} - {:?}",
                s,
                token,
            )
        };

        assert("_", T::Underscore);

        for x in ["true", "t"] {
            assert(x, T::Boolean(true));
        }

        for x in ["false", "f"] {
            assert(x, T::Boolean(false));
        }

        for x in [
            "T", "True", "TRUE", "F", "False", "FALSE", "anything", "else",
        ] {
            assert(x, T::Identifier(x.to_owned()));
        }
    }

    #[test]
    fn valid_identifier_chars() {
        use super::valid_identifier_char as valid;

        for c in 'a'..'z' {
            assert!(valid(c), "{}", c);
        }
        for c in 'A'..'Z' {
            assert!(valid(c), "{}", c);
        }
        for c in '0'..'9' {
            assert!(valid(c), "{}", c);
        }

        assert!(valid('_'));
        assert!(valid('💝'));
    }

    #[test]
    fn invalid_identifier_chars() {
        use super::valid_identifier_char as valid;

        for c in [
            '`', '~', '!', '@', '#', '$', '%', '^', '&', '*', '(', ')', '-', '=', '+', '[', '{',
            ']', '}', '\\', '|', ';', ':', '\'', '"', ',', '<', '.', '>', '/', '?',
        ] {
            assert!(!valid(c), "{}", c);
        }
    }
}
