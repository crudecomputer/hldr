use crate::lexer::tokens::TokenKind;
use super::prelude::*;
use super::start::Start;

/// State after receiving double-dashes.
#[derive(Debug, PartialEq)]
pub struct InComment;

impl State for InComment {
    fn receive(&self, ctx: &mut Context, c: Option<char>) -> ReceiveResult {
        match c {
            Some(c) if is_newline(c) => {
                ctx.add_token(TokenKind::LineSep);
                to(Start)
            }
            _ => to(InComment),
        }
    }
}
