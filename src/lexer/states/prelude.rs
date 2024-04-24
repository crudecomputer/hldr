use std::any;
use std::fmt;
pub use crate::lexer::context::Context;
use crate::lexer::error::LexError;

pub type ReceiveResult = Result<Box<dyn State>, LexError>;

#[derive(Debug, Default, PartialEq)]
pub(super) struct Stack(String);

impl Stack {
    pub fn consume(self) -> String {
        self.0
    }

    pub fn push(&mut self, c: char) {
        self.0.push(c);
    }

    pub fn top(&self) -> Option<char> {
        self.0.chars().rev().next()
    }
}

impl From<char> for Stack {
    fn from(c: char) -> Self {
        Self(String::from(c))
    }
}


/// A state in the lexer's state machine.
pub trait State : any::Any + fmt::Debug {
    /// Receives a character (or `None` when EOF) and returns the next state.
    fn receive(self: Box<Self>, ctx: &mut Context, c: Option<char>) -> ReceiveResult;

    /// Returns whether or not the given character can successfully terminate the current state,
    /// defaulting to only allowing whitespace, newlines, or EOF to terminate.
    fn can_terminate(&self, c: Option<char>) -> bool {
        c.is_none() || matches!(c, Some(c) if is_whitespace(c) || is_newline(c))
    }
}

/// Utility for boxing the state and returning a single-action transition
pub(super) fn to<S: State + 'static>(state: S) -> ReceiveResult {
    Ok(Box::new(state))
}

/// Utility for passing the character to a different state to determine the appropriate
/// next state and chaining the supplied action to one generated by the state being
/// deferred to.
pub(super) fn defer_to<S: State + 'static>(state: S, ctx: &mut Context, c: Option<char>) -> ReceiveResult {
    Box::new(state).receive(ctx, c)
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
