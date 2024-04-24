use crate::lexer::error::{LexError, LexErrorKind};
use crate::lexer::tokens::TokenKind;
use super::prelude::*;
use super::start::Start;

/// State after receiving a decimal point or a digit after having previously received a decimal point.
#[derive(Debug)]
pub(super) struct InFloat(pub Stack);

impl State for InFloat {
    fn receive(self: Box<Self>, ctx: &mut Context, c: Option<char>) -> ReceiveResult {
        use LexErrorKind::{InvalidNumericLiteral, UnexpectedCharacter};

        let mut stack = self.0;

        match c {
            // Entering into InFloat means there is already a decimal point in the stack
            Some('.') => Err(LexError {
                kind: UnexpectedCharacter('.'),
                position: ctx.current_position(),
            }),
            // Underscores can neither be consecutive nor follow a decimal point
            Some('_') if matches!(stack.top(), Some('.' | '_')) => {
                Err(LexError {
                    kind: UnexpectedCharacter('_'),
                    position: ctx.current_position(),
                })
            }
            Some(c @ '0'..='9' | c @ '_') => {
                stack.push(c);
                to(InFloat(stack))
            }
            None | Some(_) if can_terminate(c) => match stack.top() {
                Some('_') => Err(LexError {
                    kind: InvalidNumericLiteral(stack.consume()),
                    position: ctx.token_start_position(),
                }),
                _ => {
                    let kind = TokenKind::Number(stack.consume());
                    ctx.add_token(kind);
                    defer_to(Start, ctx, c)
                }
            },
            Some(c) => Err(LexError {
                kind: UnexpectedCharacter(c),
                position: ctx.current_position(),
            }),
            _ => unreachable!(),
        }
    }
}


/// State after receiving a digit without having previously received a decimal point.
#[derive(Debug)]
pub(super) struct InInteger(pub Stack);

impl State for InInteger {
    fn receive(self: Box<Self>, ctx: &mut Context, c: Option<char>) -> ReceiveResult {
        use LexErrorKind::{InvalidNumericLiteral, UnexpectedCharacter};

        let mut stack = self.0;

        match c {
            // Underscores cannot be consecutive and decimal points cannot follow underscores
            Some(c @ '_' | c @ '.') if matches!(stack.top(), Some('_')) => {
                Err(LexError {
                    kind: UnexpectedCharacter(c),
                    position: ctx.current_position(),
                })
            }
            Some(c @ '0'..='9' | c @ '_') => {
                stack.push(c);
                to(InInteger(stack))
            }
            Some(c @ '.') => {
                stack.push(c);
                to(InFloat(stack))
            }
            None | Some(_) if can_terminate(c) => match stack.top() {
                Some('_') => Err(LexError {
                    kind: InvalidNumericLiteral(stack.consume()),
                    position: ctx.token_start_position(),
                }),
                _ => {
                    let kind = TokenKind::Number(stack.consume());
                    ctx.add_token(kind);
                    defer_to(Start, ctx, c)
                }
            },
            Some(c) => Err(LexError {
                kind: UnexpectedCharacter(c),
                position: ctx.current_position(),
            }),
            _ => unreachable!(),
        }
    }
}

// TODO: This is another indication that the `can_terminate` logic needs overhauling
fn can_terminate(c: Option<char>) -> bool {
    c.is_none()
        || matches!(c, Some(')'))
        || matches!(c, Some(c) if is_whitespace(c) || is_newline(c))
}
