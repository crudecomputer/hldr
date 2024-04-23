use crate::lexer::error::LexErrorKind;
use crate::lexer::tokens::{Symbol, TokenKind};
use super::prelude::*;
use super::comments::InComment;
use super::numbers::{InFloat, InInteger};
use super::start::Start;

/// State after receiving a period without preceding digits.
#[derive(Debug)]
pub(super) struct AfterPeriod(pub Stack);

impl State for AfterPeriod {
    fn receive(self: Box<Self>, c: Option<char>) -> ReceiveResult {
        use Action::{AddToken, ContinueToken};
        let mut stack = self.0;

        match c {
            Some(c @ '0'..='9') => {
                stack.push(c);
                to(InFloat(stack), ContinueToken)
            }
            _ => {
                let kind = TokenKind::Symbol(Symbol::Period);
                defer_to(Start, c, AddToken(kind))
            }
        }
    }
}

/// State after receiving a single dash
#[derive(Debug)]
pub(super) struct AfterSingleDash(pub Stack);

impl State for AfterSingleDash {
    fn receive(self: Box<Self>, c: Option<char>) -> ReceiveResult {
        use Action::{ContinueToken, ResetPosition};
        use LexErrorKind::{UnexpectedCharacter, UnexpectedEOF};
        use TransitionErrorPosition::CurrentPosition;

        let mut stack = self.0;

        match c {
            Some('-') => {
                to(InComment, ResetPosition)
            }
            Some(c @ '0'..='9') => {
                stack.push(c);
                to(InInteger(stack), ContinueToken)
            }
            Some(c @ '.') => {
                stack.push(c);
                to(InFloat(stack), ContinueToken)
            }
            Some(c) => Err(TransitionError {
                kind: UnexpectedCharacter(c),
                position: CurrentPosition,
            }),
            None => Err(TransitionError {
                kind: UnexpectedEOF,
                position: CurrentPosition,
            }),
        }
    }
}
