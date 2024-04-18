use crate::lexer::error::LexError;
use crate::lexer::tokens::{Symbol, TokenKind};

use super::prelude::*;
use super::identifiers::{InIdentifier, InQuotedIdentifier};
use super::numbers::InInteger;
use super::text::InText;
use super::symbols::{AfterPeriod, AfterSingleDash};


/// State corresponding to the start of input or after successfully extracting a token.
pub struct Start;

impl State for Start {
    fn receive(&self, ctx: &mut Context, c: Option<char>) -> ReceiveResult {
        let c = match c {
            Some(c) => c,
            None => return to(Start),
        };

        match c {
            '\r' | '\n' => {
                ctx.add_token(TokenKind::LineSep);
                to(Start)
            }
            '(' => {
                ctx.add_token(TokenKind::Symbol(Symbol::ParenLeft));
                to(Start)
            }
            ')' => {
                ctx.add_token(TokenKind::Symbol(Symbol::ParenRight));
                to(Start)
            }
            '@' => {
                ctx.add_token(TokenKind::Symbol(Symbol::AtSign));
                to(Start)
            }
            ',' => {
                ctx.add_token(TokenKind::Symbol(Symbol::Comma));
                to(Start)
            }
            '.' => {
                ctx.push_stack(c);
                to(AfterPeriod)
            }
            '-' => {
                ctx.push_stack(c);
                to(AfterSingleDash)
            }
            '\'' => {
                ctx.push_stack(c);
                to(InText)
            }
            '"' => {
                ctx.push_stack(c);
                to(InQuotedIdentifier)
            }
            '0'..='9' => defer_to(InInteger, ctx, Some(c)),
            _ if is_identifier_char(c) => defer_to(InIdentifier, ctx, Some(c)),
            _ if is_whitespace(c) => to(Start),
            _ => Err(LexError::bad_char(c, ctx.current_position())),
        }
    }
}
