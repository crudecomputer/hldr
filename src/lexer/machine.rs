use super::Token;

#[derive(Clone, Debug, PartialEq)]
pub enum State {
    ExpectingComment,
    Comment,
    Indent,
    //Identifier,
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
                            self.stack.clear();
                            State::Comment
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
                                    self.stack.push(c);
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
                            self.stack.push(c);
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
                        _ => unexpected(),
                    }

                    State::Whitespace => match c {
                        ' ' | '\t' => State::Whitespace,
                        '-' => {
                            self.stack.push(c);
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
                        _ => unexpected(),
                    }
                }

            }

            match self.state {
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
