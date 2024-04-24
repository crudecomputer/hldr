use crate::lexer::error::{LexError, LexErrorKind};
use crate::lexer::tokens::{Symbol, TokenKind};

use super::prelude::*;
use super::identifiers::{InIdentifier, InQuotedIdentifier};
use super::numbers::InInteger;
use super::text::InText;
use super::symbols::{AfterPeriod, AfterSingleDash};


/// State corresponding to the start of input or after successfully extracting a token.
#[derive(Debug)]
pub struct Start;

impl State for Start {
    #[rustfmt::skip]
    fn receive(self: Box<Self>, ctx: &mut Context, c: Option<char>) -> ReceiveResult {
        use LexErrorKind::UnexpectedCharacter;

        let c = match c {
            Some(c) => c,
            None => return to(Start),
        };

        match c {
            '\r' | '\n' => {
                let kind = TokenKind::LineSep;
                ctx.add_token(kind);
                to(Start)
            }
            '(' => {
                let kind = TokenKind::Symbol(Symbol::ParenLeft);
                ctx.add_token(kind);
                to(Start)
            }
            ')' => {
                let kind = TokenKind::Symbol(Symbol::ParenRight);
                ctx.add_token(kind);
                to(Start)
            }
            '@' => {
                let kind = TokenKind::Symbol(Symbol::AtSign);
                ctx.add_token(kind);
                to(Start)
            }
            ',' => {
                let kind = TokenKind::Symbol(Symbol::Comma);
                ctx.add_token(kind);
                to(Start)
            }
            '.' => {
                let stack = Stack::from(c);
                ctx.in_token = true;
                to(AfterPeriod(stack))
            }
            '-' => {
                let stack = Stack::from(c);
                ctx.in_token = true;
                to(AfterSingleDash(stack))
            }
            '\'' => {
                let stack = Stack::from(c);
                ctx.in_token = true;
                to(InText(stack))
            }
            '"' => {
                let stack = Stack::from(c);
                ctx.in_token = true;
                to(InQuotedIdentifier(stack))
            }
            '0'..='9' => {
                let stack = Stack::from(c);
                ctx.in_token = true;
                to(InInteger(stack))
            }
            c if is_identifier_char(c) => {
                let stack = Stack::from(c);
                ctx.in_token = true;
                to(InIdentifier(stack))
            }
            _ if is_whitespace(c) => {
                to(Start)
            }
            _ => Err(LexError {
                kind: UnexpectedCharacter(c),
                position: ctx.current_position(),
            })
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
     */
}
