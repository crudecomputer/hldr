use crate::lexer::error::{LexError, LexErrorKind};
use crate::lexer::tokens::{Keyword, Symbol, TokenKind};
use super::prelude::*;
use super::start::Start;

/// State after receiving a valid identifier character.
#[derive(Debug)]
pub(super) struct InIdentifier(pub Stack);

impl State for InIdentifier {
    fn receive(self: Box<Self>, ctx: &mut Context, c: Option<char>) -> ReceiveResult {
        let mut stack = self.0;

        match c {
            Some(c) if is_identifier_char(c) => {
                stack.push(c);
                to(InIdentifier(stack))
            }
            _ => {
                let kind = identifier_to_token_kind(stack.consume());
                ctx.add_token(kind);
                defer_to(Start, ctx, c)
            }
        }
    }
}

/// State after receiving a valid identifier character.
#[derive(Debug)]
pub(super) struct InQuotedIdentifier(pub Stack);

impl State for InQuotedIdentifier {
    fn receive(self: Box<Self>, ctx: &mut Context, c: Option<char>) -> ReceiveResult {
        use LexErrorKind::UnclosedQuotedIdentifier;

        let mut stack = self.0;

        match c {
            Some('"') => to(AfterQuotedIdentifier(stack)),
            Some(c) => {
                stack.push(c);
                to(InQuotedIdentifier(stack))
            }
            None => Err(LexError {
                kind: UnclosedQuotedIdentifier,
                position: ctx.current_position(),
            }),
        }
    }
}

/// State after receiving what might be a closing double-quote unless the next
/// character received is another double-quote, which indicates the previous
/// quote was being escaped and is part of the quoted identifier.
#[derive(Debug)]
pub(super) struct AfterQuotedIdentifier(pub Stack);

impl State for AfterQuotedIdentifier {
    fn receive(self: Box<Self>, ctx: &mut Context, c: Option<char>) -> ReceiveResult {
        let mut stack = self.0;
        stack.push('"');

        match c {
            Some('"') => {
                stack.push('"');
                to(InQuotedIdentifier(stack))
            },
            // FIXME: Disallow char with code zero per:
            // https://www.postgresql.org/docs/current/sql-syntax-lexical.html#SQL-SYNTAX-IDENTIFIERS
            _ => {
                let kind = TokenKind::QuotedIdentifier(stack.consume());
                ctx.add_token(kind);
                defer_to(Start, ctx, c)
            }
        }
    }
}

fn identifier_to_token_kind(s: String) -> TokenKind {
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
