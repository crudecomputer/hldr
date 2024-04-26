use crate::lexer::tokens::{Token, TokenKind};
use crate::lexer::prelude::*;
use super::start::Start;

/// State after receiving double-dashes.
#[derive(Debug, PartialEq)]
pub struct InComment;

impl State for InComment {
    fn receive(self: Box<Self>, ctx: &mut Context, c: Option<char>) -> ReceiveResult {
        match c {
            Some(c) if is_newline(c) => {
                let kind = TokenKind::LineSep;
                ctx.add_token(Token { kind, position: ctx.current_position });
                to(Start)
            }
            _ => to(InComment),
        }
    }
}

#[cfg(test)]
mod in_comment_tests {
    use std::any::TypeId;
    use pretty_assertions::assert_eq;

    use crate::Position;
    use crate::lexer::tokens::{Token, TokenKind};
    use super::*;

    #[test]
    fn test_newlines() {
        for (line, column) in [(1, 1), (2, 3), (5, 8)] {
            for c in ['\r', '\n'] {
                let mut ctx = Context::default();
                ctx.current_position = Position { line, column };
                let state = Box::new(InComment).receive(&mut ctx, Some(c)).unwrap();

                assert!((*state).type_id() == TypeId::of::<Start>());
                assert_eq!(ctx.into_tokens(), vec![
                    Token {
                        kind: TokenKind::LineSep,
                        position: Position { line, column },
                    },
                ]);
            }
        }
    }

    #[test]
    fn test_others() {
        for c in ['a', '1', ' ', '\t', '\0'] {
            let mut ctx = Context::default();
            let state = Box::new(InComment).receive(&mut ctx, Some(c)).unwrap();

            assert!((*state).type_id() == TypeId::of::<InComment>());
            assert_eq!(Context::default(), ctx);
        }
    }

    #[test]
    fn test_none() {
        let mut ctx = Context::default();
        let state = Box::new(InComment).receive(&mut ctx, None).unwrap();

        assert!((*state).type_id() == TypeId::of::<InComment>());
        assert_eq!(Context::default(), ctx);

    }
}
