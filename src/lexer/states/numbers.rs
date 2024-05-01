use crate::lexer::error::{LexError, LexErrorKind};
use crate::lexer::tokens::{Token, TokenKind};
use crate::lexer::prelude::*;
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
                position: ctx.current_position,
            }),
            // Underscores can neither be consecutive nor follow a decimal point
            Some('_') if matches!(stack.top(), Some('.' | '_')) => {
                Err(LexError {
                    kind: UnexpectedCharacter('_'),
                    position: ctx.current_position,
                })
            }
            Some(c @ '0'..='9' | c @ '_') => {
                stack.push(c);
                to(InFloat(stack))
            }
            None | Some(_) if can_terminate(c) => match stack.top() {
                Some('_') => Err(LexError {
                    position: stack.start_position,
                    kind: InvalidNumericLiteral(stack.consume()),
                }),
                _ => {
                    let position = stack.start_position;
                    let kind = TokenKind::Number(stack.consume());
                    ctx.add_token(Token { kind, position });
                    defer_to(Start, ctx, c)
                }
            },
            Some(c) => Err(LexError {
                kind: UnexpectedCharacter(c),
                position: ctx.current_position,
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
                    position: ctx.current_position,
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
                    position: stack.start_position,
                    kind: InvalidNumericLiteral(stack.consume()),
                }),
                _ => {
                    let position = stack.start_position;
                    let kind = TokenKind::Number(stack.consume());
                    ctx.add_token(Token { kind, position });
                    defer_to(Start, ctx, c)
                }
            },
            Some(c) => Err(LexError {
                kind: UnexpectedCharacter(c),
                position: ctx.current_position,
            }),
            _ => unreachable!(),
        }
    }
}

fn can_terminate(c: Option<char>) -> bool {
    c.is_none()
        || matches!(c, Some(')'))
        || matches!(c, Some(c) if is_whitespace(c) || is_newline(c))
}

#[cfg(test)]
mod numbers_tests {
    use std::any::TypeId;
    use crate::Position;
    use super::*;

    mod in_integer_tests {
        use super::*;

        #[test]
        fn test_digit_after_digit() {
            let mut ctx = Context::default();
            let stack = Stack::new(Position::default(), '6');

            let state = Box::new(InInteger(stack)).receive(&mut ctx, Some('7')).unwrap();

            assert!((*state).type_id() == TypeId::of::<InInteger>());
            assert_eq!(Context::default(), ctx);
        }

        #[test]
        fn test_underscore_after_digit() {
            let mut ctx = Context::default();
            let stack = Stack::new(Position::default(), '9');

            let state = Box::new(InInteger(stack)).receive(&mut ctx, Some('_')).unwrap();

            assert!((*state).type_id() == TypeId::of::<InInteger>());
            assert_eq!(Context::default(), ctx);
        }

        #[test]
        fn test_period_after_digit() {
            let mut ctx = Context::default();
            let stack = Stack::new(Position::default(), '9');

            let state = Box::new(InInteger(stack)).receive(&mut ctx, Some('.')).unwrap();

            assert!((*state).type_id() == TypeId::of::<InFloat>());
            assert_eq!(Context::default(), ctx);
        }

        #[test]
        fn test_underscore_after_underscore() {
            let mut ctx = Context::new(Position { line: 9, column: 10 }, None);
            let stack = Stack::new(Position::default(), '_');

            let err = Box::new(InInteger(stack)).receive(&mut ctx, Some('_')).err().unwrap();

            assert_eq!(Context::new(Position { line: 9, column: 10 }, None), ctx);
            assert_eq!(
                LexError {
                    kind: LexErrorKind::UnexpectedCharacter('_'),
                    position: Position { line: 9, column: 10 },
                },
                err,
            );
        }

        #[test]
        fn test_period_after_underscore() {
            let mut ctx = Context::new(Position { line: 9, column: 10 }, None);
            let stack = Stack::new(Position::default(), '_');

            let err = Box::new(InInteger(stack)).receive(&mut ctx, Some('.')).err().unwrap();

            assert_eq!(Context::new(Position { line: 9, column: 10 }, None), ctx);
            assert_eq!(
                LexError {
                    kind: LexErrorKind::UnexpectedCharacter('.'),
                    position: Position { line: 9, column: 10 },
                },
                err,
            );
        }
    }

    mod in_float_tests {

    }
}
