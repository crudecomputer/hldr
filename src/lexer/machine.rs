use unicode_segmentation::UnicodeSegmentation;

use super::Token;

#[derive(Clone, Debug, PartialEq)]
pub enum State {
    Start,
    ExpectingComment,
    InComment,
    InIndent,
}

#[derive(Debug)]
pub(super) struct Machine {
    state: State,
    stack: Vec<String>, // Vec of graphemes for now
    pub tokens: Vec<Token>,
}

impl Machine {
    pub fn new() -> Self {
        Self {
            state: State::Start,
            stack: Vec::new(),
            tokens: Vec::new(),
        }
    }

    pub fn lex(&mut self, input: &str) {
        for (line_num, line) in input.lines().enumerate() {
            self.state = State::Start;

            let mut last_char_num = 0;

            for (char_num, grapheme) in line.graphemes(true).enumerate() {
                last_char_num = char_num;
                
                let unexpected = || panic!(
                    "Unexpected character '{}' (line {}, column {})",
                    grapheme,
                    line_num + 1,
                    char_num + 1,
                );

                self.state = match self.state {
                    State::Start => match grapheme {
                        "-" => {
                            self.stack.push(grapheme.to_owned());
                            State::ExpectingComment
                        }
                        " " | "\t" => {
                            self.stack.push(grapheme.to_owned());
                            State::InIndent
                        }
                        _ => unexpected(),
                    }

                    State::InIndent => match grapheme {
                        " " | "\t" => {
                            self.stack.push(grapheme.to_owned());
                            State::InIndent
                        }
                        _ => {
                            let indent: String = self.stack.drain(..).collect();
                            self.tokens.push(Token::Indent(indent));

                            match grapheme {
                                "-" => {
                                    self.stack.push(grapheme.to_owned());
                                    State::ExpectingComment
                                }
                                _ => unexpected(),
                            }
                        }
                    }

                    State::ExpectingComment => match grapheme {
                        "-" => {
                            self.stack.clear();
                            State::InComment
                        }
                        _ => unexpected(),
                    }

                    State::InComment => State::InComment
                }

            }

            match self.state {
                State::InIndent => {
                    let indent: String = self.stack.drain(..).collect();
                    self.tokens.push(Token::Indent(indent));
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

#[cfg(test)]
mod tests {
}
