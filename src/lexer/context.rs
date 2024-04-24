use crate::Position;
use super::tokens::{Token, TokenKind};

#[derive(Debug, PartialEq)]
pub struct Context {
    current_position: Position,
    pub in_token: bool,
    token_start_position: Position,
    tokens: Vec<Token>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            current_position: Position { line: 1, column: 1 },
            in_token: false,
            token_start_position: Position { line: 1, column: 1 },
            tokens: Vec::new(),
        }
    }

    /// Consumes the Context and returns the collected tokens.
    pub fn into_tokens(self) -> Vec<Token> {
        self.tokens
    }

    pub fn current_position(&self) -> Position {
        self.current_position
    }

    pub fn token_start_position(&self) -> Position {
        self.token_start_position
    }

    pub fn increment_position(&mut self, c: char) {
        if ['\r', '\n'].contains(&c) {
            self.current_position.line += 1;
            self.current_position.column = 1;
        } else {
            self.current_position.column += 1;
        }

        if !self.in_token {
            self.reset_start();
        }
    }

    pub fn add_token(&mut self, kind: TokenKind) {
        self.tokens.push(Token {
            kind,
            position: self.token_start_position,
        });
        self.in_token = false;
    }

    pub fn reset_start(&mut self) {
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
