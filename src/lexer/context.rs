use crate::Position;
use super::states::{TransitionActions, Action};
use super::tokens::{Token, TokenKind};

#[derive(Debug, PartialEq)]
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

    pub fn respond(&mut self, actions: TransitionActions) {
        use TransitionActions::*;

        match actions {
            Single(action) => {
                self.respond_single(action);
            }
            Double(action1, action2) => {
                self.respond_single(action1);
                // self.reset_start();
                self.respond_single(action2);
            }
        }
    }

    fn respond_single(&mut self, action: Action) {
        use Action::*;

        match action {
            AddToken(kind) => {
                self.add_token(kind);
                self.reset_start();
            }
            ContinueToken => {
                // self.increment_position(false);
            }
            ResetPosition => {
                self.reset_start();
            }
            NoAction => {},
        }
    }

    pub fn increment_position(&mut self, c: char) {
        if ['\r', '\n'].contains(&c) {
            self.current_position.line += 1;
            self.current_position.column = 1;
        } else {
            self.current_position.column += 1;
        }
    }

    fn xincrement_position(&mut self, next_line: bool) {
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

#[cfg(test)]
mod test_context {
    /*
    use pretty_assertions::assert_eq;
    use crate::Position;
    use super::{Action::*, Context};

    #[test]
    fn test_single_action_no_action() {
        let mut actual = Context {
            current_position: Position { line: 1, column: 1 },
            token_start_position: Position { line: 1, column: 1 },
            tokens: vec![],
        };

        actual.respond(vec![NoAction]);

        let expected = Context {
            current_position: Position { line: 1, column: 1 },
            token_start_position: Position { line: 1, column: 1 },
            tokens: vec![],
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_single_action_continue_token() {
        let mut actual = Context {
            current_position: Position { line: 1, column: 1 },
            token_start_position: Position { line: 1, column: 1 },
            tokens: vec![],
        };

        actual.respond(vec![NoAction]);

        let expected = Context {
            current_position: Position { line: 1, column: 2 },
            token_start_position: Position { line: 1, column: 1 },
            tokens: vec![],
        };

        assert_eq!(actual, expected);
    }
     */
}
