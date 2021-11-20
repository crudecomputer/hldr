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
                            self.tokens.push(Token::Identifier(ident));
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

                    State::Whitespace => match c {
                        ' ' | '\t' => State::Whitespace,
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
                    self.tokens.push(Token::Identifier(ident));
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
                    panic!("Expected comment (line {}, column {})", line_num + 1, last_char_num + 1);
                }
                _ => {}
            }
        }
        println!("{:#?}", self);
    }
}

fn valid_identifier_char(c: char) -> bool {
    c == '_' || (
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
