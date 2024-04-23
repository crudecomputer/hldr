use std::any;
use std::fmt;
use crate::lexer::error::LexErrorKind;
use crate::lexer::tokens::{Token, TokenKind};
use crate::Position;

pub type ReceiveResult = Result<Transition, TransitionError>;

/*
[Transition]Actions and TransitionErrors are used to abstract over
interactions with the context such that `State::receive` needs to
have no access to the Context or directly manipulate it; instead,
actions and error directives tell the caller how to update the context
or generate proper errors based on context's tracked positions
 */

#[derive(Debug, PartialEq)]
pub enum Action {
    AddToken(TokenKind),
    ContinueToken,
    NoAction,
    ResetPosition, // TODO: Less explicitly about positioning?
}

/// Some transitions can generate multiple actions. For example, after having
/// received a `'.'` chracter and the current character is a newline, then
/// the transition will generate two successive `AddToken` actions, since it
/// wasn't known in the previous state whether the period would be a symbol
/// token or leading into a float token. There should not, however, be a situation
/// where more than three actions are required in a single transition.
#[derive(Debug)]
pub enum TransitionAction {
    Single(Action),
    Dual(Action, Action),
}

#[derive(Debug)]
pub(super) struct Transition {
    pub state: Box<dyn State>,
    pub action: TransitionAction,
}

pub(super) enum TransitionErrorPosition {
    CurrentPosition,
    TokenStartPosition,
}

pub(super) struct TransitionError {
    pub kind: LexErrorKind,
    pub position: TransitionErrorPosition,
}


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


    pub(super) fn respond(&mut self, action: Action) {
        use Action::*;

        match action {
            AddToken(kind) => {
                self.tokens.push(Token {
                    kind,
                    position: self.token_start_position,
                });
                self.increment_position(matches!(kind, TokenKind::LineSep));
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
pub trait State : any::Any + fmt::Debug {
    /// Receives a character (or `None` when EOF) and returns the next state.
    fn receive(self: Box<Self>, c: Option<char>) -> ReceiveResult;

    /// Returns whether or not the given character can successfully terminate the current state,
    /// defaulting to only allowing whitespace, newlines, or EOF to terminate.
    fn can_terminate(&self, c: Option<char>) -> bool {
        c.is_none() || matches!(c, Some(c) if is_whitespace(c) || is_newline(c))
    }
}

/// Utility for boxing the state and returning a single-action transition
pub(super) fn to<S: State + 'static>(state: S, action: Action) -> ReceiveResult {
    let state = Box::new(state);
    Ok(Transition { state, action: TransitionAction::Single(action) })
}

/// Utility for passing the character to a different state to determine the appropriate
/// next state and chaining the supplied action to one generated by the state being
/// deferred to.
pub(super) fn defer_to<S: State + 'static>(deferred_to: S, c: Option<char>, previous_action: Action) -> ReceiveResult {
    let mut transition = Box::new(deferred_to).receive(c)?;
    match transition.action {
        TransitionAction::Single(next_action) => {
            transition.action = TransitionAction::Dual(previous_action, next_action);
            Ok(transition)
        }
        _ => panic!("only single action expected"),
    }
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
