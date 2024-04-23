use crate::lexer::error::LexErrorKind;
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
    fn receive(self: Box<Self>, c: Option<char>) -> ReceiveResult {
        use Action::{
            AddToken,
            ContinueToken,
            NoAction,
            ResetPosition,
        };
        use LexErrorKind::UnexpectedCharacter;
        use TransitionErrorPosition::CurrentPosition;

        let c = match c {
            Some(c) => c,
            None => return to(Start, NoAction),
        };

        match c {
            '\r' | '\n' => {
                let kind = TokenKind::LineSep;
                to(Start, AddToken(kind))
            }
            '(' => {
                let kind = TokenKind::Symbol(Symbol::ParenLeft);
                to(Start, AddToken(kind))
            }
            ')' => {
                let kind = TokenKind::Symbol(Symbol::ParenRight);
                to(Start, AddToken(kind))
            }
            '@' => {
                let kind = TokenKind::Symbol(Symbol::AtSign);
                to(Start, AddToken(kind))
            }
            ',' => {
                let kind = TokenKind::Symbol(Symbol::Comma);
                to(Start, AddToken(kind))
            }
            '.' => {
                let stack = Stack::from(c);
                to(AfterPeriod(stack), ContinueToken)
            }
            '-' => {
                let stack = Stack::from(c);
                to(AfterSingleDash(stack), ContinueToken)
            }
            '\'' => {
                let stack = Stack::from(c);
                to(InText(stack), ContinueToken)
            }
            '"' => {
                let stack = Stack::from(c);
                to(InQuotedIdentifier(stack), ContinueToken)
            }
            '0'..='9' => {
                let stack = Stack::from(c);
                to(InInteger(stack), ContinueToken)
            }
            c if is_identifier_char(c) => {
                let stack = Stack::from(c);
                to(InIdentifier(stack), ContinueToken)
            }
            _ if is_whitespace(c) => {
                to(Start, ResetPosition)
            }
            _ => Err(TransitionError {
                kind: UnexpectedCharacter(c),
                position: CurrentPosition,
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
