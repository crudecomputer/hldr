use crate::lexer::error::{LexError, LexErrorKind};
use crate::lexer::tokens::{Symbol, Token, TokenKind};
use crate::lexer::prelude::*;
use super::comments::InComment;
use super::numbers::{InFloat, InInteger};
use super::start::Start;

/// State after receiving a period without preceding digits.
#[derive(Debug)]
pub(super) struct AfterPeriod(pub Stack);

impl State for AfterPeriod {
    fn receive(self: Box<Self>, ctx: &mut Context, c: Option<char>) -> ReceiveResult {
        let mut stack = self.0;

        match c {
            Some(c @ '0'..='9') => {
                stack.push(c);
                to(InFloat(stack))
            }
            _ => {
                let kind = TokenKind::Symbol(Symbol::Period);
                ctx.add_token(Token { kind, position: stack.start_position });
                defer_to(Start, ctx, c)
            }
        }
    }
}

/// State after receiving a single dash
#[derive(Debug)]
pub(super) struct AfterSingleDash(pub Stack);

impl State for AfterSingleDash {
    fn receive(self: Box<Self>, ctx: &mut Context, c: Option<char>) -> ReceiveResult {
        use LexErrorKind::{UnexpectedCharacter, UnexpectedEOF};

        let mut stack = self.0;

        match c {
            Some('-') => {
                to(InComment)
            }
            Some(c @ '0'..='9') => {
                stack.push(c);
                to(InInteger(stack))
            }
            Some(c @ '.') => {
                stack.push(c);
                to(InFloat(stack))
            }
            Some(c) => Err(LexError {
                kind: UnexpectedCharacter(c),
                position: ctx.current_position,
            }),
            None => Err(LexError {
                kind: UnexpectedEOF,
                position: ctx.current_position,
            }),
        }
    }
}
