use crate::lexer::error::LexError;
use crate::lexer::tokens::{Keyword, Symbol, TokenKind};
use super::prelude::*;
use super::start::Start;

/// State after receiving a valid identifier character.
#[derive(Debug)]
pub(super) struct InIdentifier;

impl State for InIdentifier {
    fn receive(&self, ctx: &mut Context, c: Option<char>) -> ReceiveResult {
        // TODO: Should this be more restrictive about what can terminate an identifier?
        // This does not exclude input like `one@two` or `one'two'` but should those
        // specifically be forbidden? How much should lexer do?
        match c {
            Some(c) if is_identifier_char(c) => {
                ctx.stack.push(c);
                to(InIdentifier)
            }
            _ => {
                let stack = ctx.drain_stack();
                let token = identifier_to_token(stack);
                ctx.add_token(token);
                defer_to(Start, ctx, c)
            }
        }
    }
}

/// State after receiving a valid identifier character.
#[derive(Debug)]
pub(super) struct InQuotedIdentifier;

impl State for InQuotedIdentifier {
    fn receive(&self, ctx: &mut Context, c: Option<char>) -> ReceiveResult {
        match c {
            Some('"') => to(AfterQuotedIdentifier),
            Some(c) => {
                ctx.stack.push(c);
                to(InQuotedIdentifier)
            }
            None => Err(LexError::eof_unquoted(ctx.current_position)),
        }
    }
}

/// State after receiving what might be a closing double-quote unless the next
/// character received is another double-quote, which indicates the previous
/// quote was being escaped and is part of the quoted identifier.
#[derive(Debug)]
pub(super) struct AfterQuotedIdentifier;

impl State for AfterQuotedIdentifier {
    fn receive(&self, ctx: &mut Context, c: Option<char>) -> ReceiveResult {
        ctx.stack.push('"');
        match c {
            Some('"') => to(InQuotedIdentifier),
            _ => {
                let stack = ctx.drain_stack();
                ctx.add_token(TokenKind::QuotedIdentifier(stack));
                defer_to(Start, ctx, c)
            }
        }
    }
}

fn identifier_to_token(s: String) -> TokenKind {
    match s.as_ref() {
        "_" => TokenKind::Symbol(Symbol::Underscore),
        "true" | "t" => TokenKind::Bool(true),
        "false" | "f" => TokenKind::Bool(false),
        "as" => TokenKind::Keyword(Keyword::As),
        "schema" => TokenKind::Keyword(Keyword::Schema),
        "table" => TokenKind::Keyword(Keyword::Table),
        _ => TokenKind::Identifier(s),
    }
}
