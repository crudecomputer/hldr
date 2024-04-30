use crate::lexer::error::{LexError, LexErrorKind};
use crate::lexer::tokens::{Keyword, Symbol, Token, TokenKind};
use crate::lexer::prelude::*;
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
                let position = stack.start_position;
                let kind = identifier_to_token_kind(stack.consume());
                ctx.add_token(Token { kind, position });
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
                position: ctx.current_position,
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
                let position = stack.start_position;
                let kind = TokenKind::QuotedIdentifier(stack.consume());
                ctx.add_token(Token { kind, position });
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

#[cfg(test)]
mod identifiers_tests {
    use std::any::TypeId;
    use crate::Position;
    use super::*;

    mod in_identifier_tests {
        use pretty_assertions::assert_eq;
        use super::*;

        #[test]
        fn test_receive_identifier_char() {
            let mut ctx = Context::default();
            let stack = Stack::default();
            let state = Box::new(InIdentifier(stack)).receive(&mut ctx, Some('c')).unwrap();

            assert!((*state).type_id() == TypeId::of::<InIdentifier>());
            assert_eq!(Context::default(), ctx);
        }

        #[test]
        fn test_receive_whitespace() {
            let mut ctx = Context::default();
            let mut stack = Stack::new(Position { line: 2, column: 3}, 'x');
            stack.push('y');
            stack.push('z');

            let state = Box::new(InIdentifier(stack)).receive(&mut ctx, Some(' ')).unwrap();

            assert!((*state).type_id() == TypeId::of::<Start>());
            assert_eq!(ctx.into_tokens(), vec![
                Token {
                    kind: TokenKind::Identifier("xyz".to_owned()),
                    position: Position { line: 2, column: 3 },
                }
            ]);
        }

        #[test]
        fn test_receive_terminators() {
            for (c, second_token_kind) in [
                ('\r', TokenKind::LineSep),
                ('\n', TokenKind::LineSep),
                (',', TokenKind::Symbol(Symbol::Comma)),
                ('(', TokenKind::Symbol(Symbol::ParenLeft)),
                (')', TokenKind::Symbol(Symbol::ParenRight)),
            ] {
                let mut ctx = Context::default();
                ctx.current_position = Position { line: 1, column: 3};

                let mut stack = Stack::new(Position { line: 1, column: 1}, 'a');
                stack.push('b');
                stack.push('c');

                let state = Box::new(InIdentifier(stack)).receive(&mut ctx, Some(c)).unwrap();

                assert!((*state).type_id() == TypeId::of::<Start>());
                assert_eq!(ctx.into_tokens(), vec![
                    Token {
                        kind: TokenKind::Identifier("abc".to_owned()),
                        position: Position { line: 1, column: 1 },
                    },
                    Token {
                        kind: second_token_kind,
                        position: Position { line: 1, column: 3},
                    }
                ]);
            }
        }

        #[test]
        fn test_receive_none() {
            let mut ctx = Context::default();
            let mut stack = Stack::new(Position { line: 2, column: 3}, 'x');
            stack.push('y');
            stack.push('z');

            let state = Box::new(InIdentifier(stack)).receive(&mut ctx, Some(' ')).unwrap();

            assert!((*state).type_id() == TypeId::of::<Start>());
            assert_eq!(ctx.into_tokens(), vec![
                Token {
                    kind: TokenKind::Identifier("xyz".to_owned()),
                    position: Position { line: 2, column: 3 },
                }
            ]);
        }
    }

    mod in_quoted_identifier_tests {
        use pretty_assertions::assert_eq;
        use super::*;

        #[test]
        fn test_receive_double_quote() {
            let mut ctx = Context::default();
            let stack = Stack::default();
            let state = Box::new(InQuotedIdentifier(stack)).receive(&mut ctx, Some('"')).unwrap();

            assert!((*state).type_id() == TypeId::of::<AfterQuotedIdentifier>());
            assert_eq!(Context::default(), ctx);
        }

        #[test]
        fn test_receive_identifier_char() {
            let mut ctx = Context::default();
            let stack = Stack::default();
            let state = Box::new(InQuotedIdentifier(stack)).receive(&mut ctx, Some('c')).unwrap();

            assert!((*state).type_id() == TypeId::of::<InQuotedIdentifier>());
            assert_eq!(Context::default(), ctx);
        }

        #[test]
        fn test_receive_whitespace() {
            let mut ctx = Context::default();
            let mut stack = Stack::new(Position { line: 2, column: 3}, 'x');
            stack.push('y');
            stack.push('z');

            let state = Box::new(InQuotedIdentifier(stack)).receive(&mut ctx, Some(' ')).unwrap();

            assert!((*state).type_id() == TypeId::of::<InQuotedIdentifier>());
            assert_eq!(Context::default(), ctx);
        }

        #[test]
        fn test_receive_terminators() {
            for c in ['\r', '\n', ',', '(', ')'] {
                let mut ctx = Context::default();
                let mut stack = Stack::new(Position { line: 1, column: 1}, 'a');
                stack.push('b');
                stack.push('c');

                let state = Box::new(InQuotedIdentifier(stack)).receive(&mut ctx, Some(c)).unwrap();

                assert!((*state).type_id() == TypeId::of::<InQuotedIdentifier>());
                assert_eq!(Context::default(), ctx);
            }
        }

        #[test]
        fn test_receive_none() {
            let mut ctx = Context::default();
            ctx.current_position = Position { line: 7, column: 11 };

            let stack = Stack::default();
            let err = Box::new(InQuotedIdentifier(stack)).receive(&mut ctx, None).err().unwrap();

            // assert!((*state).type_id() == TypeId::of::<InQuotedIdentifier>());
            assert!(ctx.into_tokens().is_empty());
            assert_eq!(
                LexError {
                    kind: LexErrorKind::UnclosedQuotedIdentifier,
                    position: Position { line: 7, column: 11 },
                },
                err,
            )
        }
    }

    mod after_quoted_identifier_tests {
        use pretty_assertions::assert_eq;
        use super::*;

        #[test]
        fn test_receive_double_quote() {
            let mut ctx = Context::default();
            let stack = Stack::default();
            let state = Box::new(AfterQuotedIdentifier(stack)).receive(&mut ctx, Some('"')).unwrap();

            assert!((*state).type_id() == TypeId::of::<InQuotedIdentifier>());
            assert_eq!(Context::default(), ctx);
        }

        #[test]
        fn test_receive_whitespace() {
            let mut ctx = Context::default();
            let mut stack = Stack::new(Position { line: 2, column: 3}, 'x');
            stack.push('y');
            stack.push('z');

            let state = Box::new(AfterQuotedIdentifier(stack)).receive(&mut ctx, Some(' ')).unwrap();

            assert!((*state).type_id() == TypeId::of::<Start>());
            assert_eq!(ctx.into_tokens(), vec![
                Token {
                    // FIXME: Remove doublequotes from quoted identifiers
                    kind: TokenKind::QuotedIdentifier("xyz\"".to_owned()),
                    position: Position { line: 2, column: 3 },
                }
            ]);
        }

        #[test]
        fn test_receive_terminators() {
            for (c, second_token_kind) in [
                ('\r', TokenKind::LineSep),
                ('\n', TokenKind::LineSep),
                (',', TokenKind::Symbol(Symbol::Comma)),
                ('(', TokenKind::Symbol(Symbol::ParenLeft)),
                (')', TokenKind::Symbol(Symbol::ParenRight)),
            ] {
                let mut ctx = Context::default();
                ctx.current_position = Position { line: 1, column: 3};

                let mut stack = Stack::new(Position { line: 1, column: 1}, 'a');
                stack.push('b');
                stack.push('c');

                let state = Box::new(InIdentifier(stack)).receive(&mut ctx, Some(c)).unwrap();

                assert!((*state).type_id() == TypeId::of::<Start>());
                assert_eq!(ctx.into_tokens(), vec![
                    Token {
                        kind: TokenKind::Identifier("abc".to_owned()),
                        position: Position { line: 1, column: 1 },
                    },
                    Token {
                        kind: second_token_kind,
                        position: Position { line: 1, column: 3},
                    }
                ]);
            }
        }


    }

    mod identifier_to_token_tests {
        use super::{Keyword, Symbol, TokenKind, identifier_to_token_kind};

        #[test]
        fn test_underscore() {
            assert_eq!(
                identifier_to_token_kind("_".to_owned()),
                TokenKind::Symbol(Symbol::Underscore),
            );
        }

        #[test]
        fn test_keyword_as() {
            assert_eq!(
                identifier_to_token_kind("as".to_owned()),
                TokenKind::Keyword(Keyword::As),
            );
        }

        #[test]
        fn test_keyword_schema() {
            assert_eq!(
                identifier_to_token_kind("schema".to_owned()),
                TokenKind::Keyword(Keyword::Schema),
            );
        }

        #[test]
        fn test_keyword_table() {
            assert_eq!(
                identifier_to_token_kind("table".to_owned()),
                TokenKind::Keyword(Keyword::Table),
            );
        }

        #[test]
        fn test_bool_true() {
            for ident in ["t", "true"] {
                assert_eq!(
                    identifier_to_token_kind(ident.to_owned()),
                    TokenKind::Bool(true),
                );
            }
        }

        #[test]
        fn test_bool_false() {
            for ident in ["f", "false"] {
                assert_eq!(
                    identifier_to_token_kind(ident.to_owned()),
                    TokenKind::Bool(false),
                );
            }
        }

        #[test]
        fn test_anything_else() {
            for ident in ["__", "True", "FALSE", "_something", "12345", "!@#$"] {
                assert_eq!(
                    identifier_to_token_kind(ident.to_owned()),
                    TokenKind::Identifier(ident.to_owned()),
                );
            }
        }
    }
}
