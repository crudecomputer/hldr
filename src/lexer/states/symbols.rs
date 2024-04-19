use crate::lexer::error::LexError;
use crate::lexer::tokens::{Symbol, TokenKind};

use super::prelude::*;
use super::comments::InComment;
use super::numbers::{InFloat, InInteger};
use super::start::Start;

/// State after receiving a period without preceding digits.
#[derive(Debug)]
pub struct AfterPeriod;

impl State for AfterPeriod {
    fn receive(&self, ctx: &mut Context, c: Option<char>) -> ReceiveResult {
        match c {
            Some('0'..='9') => defer_to(InFloat, ctx, c),
            None | Some(_) if self.can_terminate(c) => {
                ctx.add_token(TokenKind::Symbol(Symbol::Period));
                ctx.clear_stack();
                ctx.reset_start();
                defer_to(Start, ctx, c)
            }
            Some(c) => Err(LexError::bad_char(c, ctx.current_position)),
            _ => unreachable!(),
        }
    }

    fn can_terminate(&self, _c: Option<char>) -> bool {
        // TODO: This was STILL making expectations.
        // Outside of float tokens (which this state does not generate)
        // periods are only used in references, meaning they should only
        // be followed by a plain or quoted identifier, but how much
        // should be forbidden during tokenization? And should references
        // be allowed to have whitespace between the period and the identifier?
        //
        // is_identifier_char(c) || c == '"'
        true
    }
}

/// State after receiving a single dash
#[derive(Debug)]
pub struct AfterSingleDash;

impl State for AfterSingleDash {
    fn receive(&self, ctx: &mut Context, c: Option<char>) -> ReceiveResult {
        match c {
            Some('-') => {
                // Clears the existing single dash from the stack so that the context
                // will be able to reset its start positioning logic for subsequent tokens
                ctx.clear_stack();
                to(InComment)
            }
            Some('0'..='9' | '.') => defer_to(InInteger, ctx, c),
            Some(c) => Err(LexError::bad_char(c, ctx.current_position)),
            None => Err(LexError::eof(ctx.current_position)),
        }
    }
}
