use crate::lexer::tokens::TokenKind;
use super::prelude::*;
use super::start::Start;

/// State after receiving double-dashes.
#[derive(Debug, PartialEq)]
pub struct InComment;

impl State for InComment {
    fn receive(self: Box<Self>, ctx: &mut Context, c: Option<char>) -> ReceiveResult {
        match c {
            Some(c) if is_newline(c) => {
                let kind = TokenKind::LineSep;
                ctx.add_token(kind);
                to(Start)
            }
            _ => to(InComment),
        }
    }
}

#[cfg(test)]
mod tests {
    /*
    use std::any::TypeId;
    use crate::Position;
    use crate::lexer::tokens::{Token, TokenKind};
    use super::*;

    #[test]
    fn test_newlines() {
        for (line, column) in [(1, 1), (2, 3), (5, 8)] {
            for c in ['\r', '\n'] {
                let mut ctx = Context::new();
                ctx.token_start_position = Position { line, column };
                let state = InComment.receive(&mut ctx, Some(c)).unwrap();

                assert!((*state).type_id() == TypeId::of::<Start>());

                assert!(ctx.stack.is_empty());
                assert_eq!(ctx.tokens, vec![
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
            let mut ctx = Context::new();
            let state = InComment.receive(&mut ctx, Some(c)).unwrap();

            assert!((*state).type_id() == TypeId::of::<InComment>());

            assert!(ctx.stack.is_empty());
            assert!(ctx.tokens.is_empty());
        }
    }

    #[test]
    fn test_none() {
        let mut ctx = Context::new();
        let state = InComment.receive(&mut ctx, None).unwrap();

        assert!((*state).type_id() == TypeId::of::<InComment>());

        assert!(ctx.stack.is_empty());
        assert!(ctx.tokens.is_empty());
    }
     */
}
