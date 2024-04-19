use crate::lexer::error::LexError;
use crate::lexer::tokens::TokenKind;
use super::prelude::*;
use super::start::Start;

/// State after receiving a decimal point or a digit after having previously received a decimal point.
#[derive(Debug)]
pub struct InFloat;

impl State for InFloat {
    fn receive(&self, ctx: &mut Context, c: Option<char>) -> ReceiveResult {
        match c {
            Some(c @ '0'..='9') => {
                ctx.stack.push(c);
                to(InFloat)
            }
            // Entering into InFloat means there is already a decimal point in the stack
            Some('.') => Err(LexError::bad_char('.', ctx.current_position)),
            // Underscores can neither be consecutive nor follow a decimal point
            Some('_') if [Some(&'.'), Some(&'_')].contains(&ctx.stack.last()) => {
                ctx.clear_stack();
                Err(LexError::bad_char('_', ctx.current_position))
            }
            Some(c @ '_') => {
                ctx.stack.push(c);
                to(InFloat)
            }
            None | Some(_) if self.can_terminate(c) => match ctx.stack.last() {
                Some(&'_') => {
                    let stack = ctx.drain_stack();
                    Err(LexError::bad_number(stack, ctx.token_start_position))
                }
                _ => {
                    let stack = ctx.drain_stack();
                    ctx.add_token(TokenKind::Number(stack));
                    defer_to(Start, ctx, c)
                }
            },
            Some(c) => Err(LexError::bad_char(c, ctx.current_position)),
            _ => unreachable!(),
        }
    }

    fn can_terminate(&self, c: Option<char>) -> bool {
        // TODO: This is another indication that the `can_terminate` logic needs overhauling
        c.is_none()
            || matches!(c, Some(')'))
            || matches!(c, Some(c) if is_whitespace(c) || is_newline(c))
    }
}


/// State after receiving a digit without having previously received a decimal point.
#[derive(Debug)]
pub struct InInteger;

impl State for InInteger {
    fn receive(&self, ctx: &mut Context, c: Option<char>) -> ReceiveResult {
        match c {
            Some(c @ '0'..='9') => {
                ctx.stack.push(c);
                to(InInteger)
            }
            // Underscores cannot be consecutive and decimal points cannot follow underscores
            Some(c @ '_' | c @ '.') if ctx.stack.last() == Some(&'_') => {
                Err(LexError::bad_char(c, ctx.current_position))
            }
            Some(c @ '_') => {
                ctx.stack.push(c);
                to(InInteger)
            }
            Some(c @ '.') => {
                ctx.stack.push(c);
                to(InFloat)
            }
            None | Some(_) if self.can_terminate(c) => match ctx.stack.last() {
                Some(&'_') => {
                    let stack = ctx.drain_stack();
                    Err(LexError::bad_number(stack, ctx.token_start_position))
                }
                _ => {
                    let stack = ctx.drain_stack();
                    ctx.add_token(TokenKind::Number(stack));
                    defer_to(Start, ctx, c)
                }
            },
            Some(c) => Err(LexError::bad_char(c, ctx.current_position)),
            _ => unreachable!(),
        }
    }

    fn can_terminate(&self, c: Option<char>) -> bool {
        // TODO: This is another indication that the `can_terminate` logic needs overhauling
        c.is_none()
            || matches!(c, Some(')'))
            || matches!(c, Some(c) if is_whitespace(c) || is_newline(c))
    }
}
