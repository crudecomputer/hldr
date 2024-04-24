use crate::lexer::error::{LexError, LexErrorKind};
use crate::lexer::tokens::TokenKind;
use super::prelude::*;
use super::start::Start;

/// State after receiving a single quote and inside a string literal.
#[derive(Debug)]
pub(super) struct InText(pub Stack);

impl State for InText {
    fn receive(self: Box<Self>, ctx: &mut Context, c: Option<char>) -> ReceiveResult {
        use LexErrorKind::UnclosedString;

        let mut stack = self.0;

        match c {
            Some('\'') => {
                to(AfterText(stack))
            }
            Some(c) => {
                stack.push(c);
                to(InText(stack))
            }
            None => Err(LexError {
                kind: UnclosedString,
                position: ctx.current_position(),
            }),
        }
    }
}

/// State after receiving what might be a closing quote unless the next
/// character received is another single quote, which indicates the previous
/// quote was being escaped and is part of the text string.
#[derive(Debug)]
pub(super) struct AfterText(pub Stack);

impl State for AfterText {
    fn receive(self: Box<Self>, ctx: &mut Context, c: Option<char>) -> ReceiveResult {
        let mut stack = self.0;
        stack.push('\'');

        match c {
            Some('\'') => {
                stack.push('\'');
                to(InText(stack))
            }
            _ => {
                let kind = TokenKind::Text(stack.consume());
                ctx.add_token(kind);
                defer_to(Start, ctx, c)
            }
        }
    }
}
