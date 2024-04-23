use crate::Position;
use super::states::Action;
use super::tokens::{Token, TokenKind};

#[derive(Debug)]
pub(super) struct Context {
    pub current_position: Position,
    pub token_start_position: Position,
    pub tokens: Vec<Token>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            current_position: Position { line: 1, column: 1 },
            token_start_position: Position { line: 1, column: 1 },
            tokens: Vec::new(),
        }
    }

    /// Consumes the Context and returns the collected tokens.
    pub fn into_tokens(self) -> Vec<Token> {
        self.tokens
    }

    pub fn respond(&mut self, actions: Vec<Action>) {
        use Action::*;

        for action in actions {
            match action {
                AddToken(kind) => {
                    let wrap_line = matches!(kind, TokenKind::LineSep);

                    self.add_token(kind);
                    self.increment_position(wrap_line);
                    self.reset_start();
                }
                ContinueToken => {
                    self.increment_position(false);
                }
                ResetPosition => {
                    self.increment_position(false);
                    self.reset_start();
                }
                NoAction => {},
            }
        }
    }

    fn increment_position(&mut self, next_line: bool) {
        if next_line {
            self.current_position.line += 1;
            self.current_position.column = 1;
        } else {
            self.current_position.column += 1;
        }
    }

    //pub fn increment_position(&mut self, c: char) {
        //if ['\r', '\n'].contains(&c) {
            //self.current_position.line += 1;
            //self.current_position.column = 1;
        //} else {
            //self.current_position.column += 1;
        //}

        //if self.stack.is_empty() {
            //self.reset_start();
        //}
    //}

    fn add_token(&mut self, kind: TokenKind) {
        self.tokens.push(Token {
            kind,
            position: self.token_start_position,
        });
    }

    pub(super) fn reset_start(&mut self) {
        self.token_start_position = self.current_position;
    }
}
