use super::Token;

#[derive(Clone, Debug, PartialEq)]
pub enum State {
    ExpectingComment,
    Comment,
    Indent,
    Identifier,
    Integer,
    Decimal,
    LineStart,
    QuotedIdentifierOpen,
    QuotedIdentifierClosed,
    TextOpen,
    TextClosed,
    Whitespace,
}

#[derive(Debug)]
pub(super) struct Machine {
    state: State,
    stack: Vec<char>,
    pub tokens: Vec<Token>,
}

impl Machine {
    pub fn new() -> Self {
        Self {
            state: State::LineStart,
            stack: Vec::new(),
            tokens: Vec::new(),
        }
    }

    pub fn lex(&mut self, input: &str) {
        for (line_num, line) in input.lines().enumerate() {
            self.state = State::LineStart;

            let mut last_char_num = 0;

            for (char_num, c) in line.chars().enumerate() {
                last_char_num = char_num;
                
                let unexpected = || panic!(
                    "Unexpected character '{}' (line {}, column {})",
                    c,
                    line_num + 1,
                    char_num + 1,
                );

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
                        _ => unexpected(),
                    }

                    State::ExpectingComment => match c {
                        '-' => {
                            State::Comment
                        }
                        _ => unexpected(),
                    }

                    State::Identifier => match c {
                        c if valid_identifier_char(c) => {
                            self.stack.push(c);
                            State::Identifier
                        }
                        ' ' | '\t' => {
                            let ident: String = self.stack.drain(..).collect();
                            self.tokens.push(identifier_to_token(ident));
                            State::Whitespace
                        }
                        _ => unexpected(),
                    }

                    State::Indent => match c {
                        ' ' | '\t' => {
                            self.stack.push(c);
                            State::Indent
                        }
                        _ => {
                            let indent: String = self.stack.drain(..).collect();
                            self.tokens.push(Token::Indent(indent));

                            match c {
                                '"' => {
                                    State::QuotedIdentifierOpen
                                }
                                '\'' => {
                                    State::TextOpen
                                }
                                '-' => {
                                    State::ExpectingComment
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
                                _ => unexpected(),
                            }
                        }
                    }

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
                        _ => unexpected(),
                    }

                    State::LineStart => match c {
                        '"' => {
                            State::QuotedIdentifierOpen
                        }
                        '\'' => {
                            State::TextOpen
                        }
                        '-' => {
                            State::ExpectingComment
                        }
                        ' ' | '\t' => {
                            self.stack.push(c);
                            State::Indent
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
                        _ => unexpected(),
                    }

                    State::QuotedIdentifierOpen => match c {
                        '"' => {
                            State::QuotedIdentifierClosed
                        }
                        _ => {
                            self.stack.push(c);
                            State::QuotedIdentifierOpen
                        }
                    }

                    State::QuotedIdentifierClosed => match c {
                        '"' => {
                            self.stack.push('"');
                            State::QuotedIdentifierOpen
                        }
                        ' ' | '\t' => {
                            let text: String = self.stack.drain(..).collect();
                            self.tokens.push(Token::QuotedIdentifier(text));
                            State::Whitespace
                        }
                        _ => unexpected()
                    }

                    State::TextOpen => match c {
                        '\'' => {
                            State::TextClosed
                        }
                        _ => {
                            self.stack.push(c);
                            State::TextOpen
                        }
                    }

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
                        _ => unexpected()
                    }

                    State::Whitespace => match c {
                        ' ' | '\t' => {
                            State::Whitespace
                        }
                        '"' => {
                            State::QuotedIdentifierOpen
                        }
                        '\'' => {
                            State::TextOpen
                        }
                        '-' => {
                            State::ExpectingComment
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
                        _ => unexpected(),
                    }
                }

            }

            match self.state {
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
                State::ExpectingComment => {
                    panic!(
                        "Expected comment (line {}, column {})",
                        line_num + 1,
                        last_char_num + 1,
                    );
                }
                State::QuotedIdentifierOpen => {
                    panic!(
                        "Quoted identifier not closed (line {}, column {})",
                        line_num + 1,
                        last_char_num + 1,
                    )
                }
                State::QuotedIdentifierClosed => {
                    let text: String = self.stack.drain(..).collect();
                    self.tokens.push(Token::QuotedIdentifier(text));
                }
                State::TextOpen => {
                    panic!(
                        "String not closed (line {}, column {})",
                        line_num + 1,
                        last_char_num + 1,
                    )
                }
                State::TextClosed => {
                    let text: String = self.stack.drain(..).collect();
                    self.tokens.push(Token::Text(text));
                }
                _ => {}
            }
        }
        println!("{:#?}", self);
    }
}

fn identifier_to_token(s: String) -> Token {
    match s.as_ref() {
        "true"  | "t" => Token::Boolean(true),
        "false" | "f" => Token::Boolean(false),
        _ => Token::Identifier(s),
    }
}

fn valid_identifier_char(c: char) -> bool {
    c == '_' || (
        // "alphabetic" isn't enough because that precludes other unicode chars
        // that are valid in postgres identifiers
        !c.is_control() &&
        !c.is_whitespace() &&
        !c.is_ascii_punctuation()
    )
}

#[cfg(test)]
mod tests {

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
            '`', '~', '!', '@', '#', '$', '%', '^', '&', '*',
            '(', ')', '-', '=', '+', '[', '{', ']', '}', '\\',
            '|', ';', ':', '\'', '"', ',', '<', '.', '>', '/',
            '?',
        ] {
            assert!(!valid(c), "{}", c);
        }
    }
}
