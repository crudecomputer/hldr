use crate::lexer::error::LexErrorKind;
use crate::lexer::tokens::{Keyword, Symbol, TokenKind};
use super::prelude::*;
use super::start::Start;

/// State after receiving a valid identifier character.
#[derive(Debug)]
pub(super) struct InIdentifier(pub Stack);

impl State for InIdentifier {
    fn receive(self: Box<Self>, c: Option<char>) -> ReceiveResult {
        use Action::{AddToken, ContinueToken};

        let mut stack = self.0;

        match c {
            Some(c) if is_identifier_char(c) => {
                stack.push(c);
                to(InIdentifier(stack), ContinueToken)
            }
            _ => {
                let kind = identifier_to_token_kind(stack.consume());
                defer_to(Start, c, AddToken(kind))
            }
        }
    }
}

/// State after receiving a valid identifier character.
#[derive(Debug)]
pub(super) struct InQuotedIdentifier(pub Stack);

impl State for InQuotedIdentifier {
    fn receive(self: Box<Self>, c: Option<char>) -> ReceiveResult {
        use Action::{ContinueToken, NoAction};
        use LexErrorKind::UnclosedQuotedIdentifier;
        use TransitionErrorPosition::CurrentPosition;

        let mut stack = self.0;

        match c {
            Some('"') => to(AfterQuotedIdentifier(stack), NoAction),
            Some(c) => {
                stack.push(c);
                to(InQuotedIdentifier(stack), ContinueToken)
            }
            None => Err(TransitionError {
                kind: UnclosedQuotedIdentifier,
                position: CurrentPosition,
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
    fn receive(self: Box<Self>, c: Option<char>) -> ReceiveResult {
        use Action::{AddToken, ContinueToken};

        let mut stack = self.0;
        stack.push('"');

        match c {
            Some('"') => {
                stack.push('"');
                to(InQuotedIdentifier(stack), ContinueToken)
            },
            // FIXME: Disallow char with code zero per:
            // https://www.postgresql.org/docs/current/sql-syntax-lexical.html#SQL-SYNTAX-IDENTIFIERS
            _ => {
                let kind = TokenKind::QuotedIdentifier(stack.consume());
                defer_to(Start, c, AddToken(kind))
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
