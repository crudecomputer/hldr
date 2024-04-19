use std::fmt;
use crate::lexer::error::LexError;
use crate::lexer::tokens::{Token, TokenKind};
use crate::Position;

pub type ReceiveResult = Result<Box<dyn State>, LexError>;

/// The context accessible for any given state
#[derive(Debug)]
pub struct Context {
    pub(super) current_position: Position,
    pub(super) token_start_position: Position,
    pub(super) stack: Vec<char>,
    pub(super) tokens: Vec<Token>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            current_position: Position { line: 1, column: 1 },
            token_start_position: Position { line: 1, column: 1 },
            stack: Vec::new(),
            tokens: Vec::new(),
        }
    }

    pub fn increment_position(&mut self, c: char) {
        if ['\r', '\n'].contains(&c) {
            self.current_position.line += 1;
            self.current_position.column = 1;
        } else {
            self.current_position.column += 1;
        }

        if self.stack.is_empty() {
            self.reset_start();
        }
    }

    /// Consumes the Context and returns the collected tokens.
    pub fn into_tokens(self) -> Vec<Token> {
        self.tokens
    }

    /// Drains the stack and returns the contents as a String.
    pub(super) fn drain_stack(&mut self) -> String {
        self.stack.drain(..).collect()
    }

    /// Clears the stack.
    pub(super) fn clear_stack(&mut self) {
        self.stack.clear();
    }

    pub(super) fn add_token(&mut self, kind: TokenKind) {
        self.tokens.push(Token {
            kind,
            position: self.token_start_position,
        });
    }

    pub(super) fn reset_start(&mut self) {
        self.token_start_position = self.current_position;
    }
}

/// A state in the lexer's state machine.
pub trait State : fmt::Debug {
    /// Receives a character (or `None` when EOF) and returns the next state.
    fn receive(&self, ctx: &mut Context, c: Option<char>) -> ReceiveResult;

    /// Returns whether or not the given character can successfully terminate the current state,
    /// defaulting to only allowing whitespace, newlines, or EOF to terminate.
    fn can_terminate(&self, c: Option<char>) -> bool {
        c.is_none() || matches!(c, Some(c) if is_whitespace(c) || is_newline(c))
    }
}

pub(super) fn to<S: State + 'static>(state: S) -> ReceiveResult {
    Ok(Box::new(state))
}

pub(super) fn defer_to<S: State + 'static>(state: S, ctx: &mut Context, c: Option<char>) -> ReceiveResult {
    state.receive(ctx, c)
}

pub(super) fn is_identifier_char(c: char) -> bool {
    c == '_'
        || c.is_alphabetic()
        || (
            // `char.is_alphabetic` isn't enough because that precludes other unicode chars
            // that are valid in postgres identifiers, eg:
            //     create table love (💝 text);
            //     > CREATE TABLE
            //
            // There is, however, a very strong chance the below conditions are not fully accurate.
            !c.is_control() && !c.is_whitespace() && !c.is_ascii_punctuation()
        )
}

pub(super) fn is_whitespace(c: char) -> bool {
    c == ' ' || c == '\t'
}

pub(super) fn is_newline(c: char) -> bool {
    ['\r', '\n'].contains(&c)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_is_whitespace() {
        assert!(super::is_whitespace(' '));
        assert!(super::is_whitespace('\t'));
        assert!(!super::is_whitespace('\r'));
        assert!(!super::is_whitespace('\n'));
    }

    #[test]
    fn test_is_newline() {
        assert!(super::is_newline('\r'));
        assert!(super::is_newline('\n'));
        assert!(!super::is_newline(' '));
        assert!(!super::is_newline('\t'));
    }

    #[test]
    fn test_is_identifier_char() {
        assert!(super::is_identifier_char('a'));
        assert!(super::is_identifier_char('Z'));
        assert!(super::is_identifier_char('7'));
        assert!(super::is_identifier_char('_'));
        assert!(super::is_identifier_char('💝'));

        assert!(!super::is_identifier_char(' '));
        assert!(!super::is_identifier_char('\t'));
        assert!(!super::is_identifier_char('\r'));
        assert!(!super::is_identifier_char('\n'));
        assert!(!super::is_identifier_char('.'));
        assert!(!super::is_identifier_char('-'));
    }
}
