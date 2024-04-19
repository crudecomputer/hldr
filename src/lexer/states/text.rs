use crate::lexer::error::LexError;
use crate::lexer::tokens::TokenKind;
use super::prelude::*;
use super::start::Start;

/// State after receiving a single quote and inside a string literal.
#[derive(Debug)]
pub struct InText;

impl State for InText {
    fn receive(&self, ctx: &mut Context, c: Option<char>) -> ReceiveResult {
        match c {
            Some('\'') => to(AfterText),
            Some(c) => {
                ctx.stack.push(c);
                to(InText)
            }
            None => Err(LexError::eof_string(ctx.current_position)),
        }
    }
}

/// State after receiving what might be a closing quote unless the next
/// character received is another single quote, which indicates the previous
/// quote was being escaped and is part of the text string.
#[derive(Debug)]
pub struct AfterText;

impl State for AfterText {
    fn receive(&self, ctx: &mut Context, c: Option<char>) -> ReceiveResult {
        ctx.stack.push('\'');
        match c {
            Some(c @ '\'') => {
                ctx.stack.push(c);
                to(InText)
            }
            _ => {
                let stack = ctx.drain_stack();
                ctx.add_token(TokenKind::Text(stack));
                defer_to(Start, ctx, c)
            }
        }
    }
}
