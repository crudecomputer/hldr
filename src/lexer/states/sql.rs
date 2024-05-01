use crate::lexer::error::{LexError, LexErrorKind};
use crate::lexer::tokens::{Token, TokenKind};
use crate::lexer::prelude::*;
use super::start::Start;

/// State after receiving a backtick.
#[derive(Debug)]
pub(super) struct InSqlSelect(pub Stack);

impl State for InSqlSelect {
    fn receive(self: Box<Self>, ctx: &mut Context, c: Option<char>) -> ReceiveResult {
        use LexErrorKind::UnclosedString;

        let mut stack = self.0;

        match c {
            Some('`') => {
                to(AfterSqlSelect(stack))
            }
            Some(c) => {
                stack.push(c);
                to(InSqlSelect(stack))
            }
            None => Err(LexError {
                kind: UnclosedString,
                position: ctx.current_position,
            }),
        }
    }
}

/// State after receiving what might be a closing backtick unless the next
/// character received is another backtick, which indicates the previous
/// backtick was being escaped and is part of the SQL select statement.
#[derive(Debug)]
pub(super) struct AfterSqlSelect(pub Stack);

impl State for AfterSqlSelect {
    fn receive(self: Box<Self>, ctx: &mut Context, c: Option<char>) -> ReceiveResult {
        let mut stack = self.0;

        match c {
            Some('`') => {
                // Unlike when storing text strings or quoted identifiers, this does not
                // need to preserve the double-backticks as part of the SQL fragment literal,
                // since text strings and quoted identifiers have to remain properly escaped
                // when passing to the database
                stack.push('`');
                to(InSqlSelect(stack))
            }
            _ => {
                let position = stack.start_position;
                let kind = TokenKind::SqlFragment(stack.consume());
                ctx.add_token(Token { kind, position });
                defer_to(Start, ctx, c)
            }
        }
    }
}
