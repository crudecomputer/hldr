pub mod error;
mod prelude;
mod states;
pub mod tokens;

use error::LexError;
use prelude::{Context, State};
use states::Start;
use tokens::Token;

pub fn tokenize(input: impl Iterator<Item = char>) -> Result<Vec<Token>, LexError> {
    let mut ctx = Context::default();
    let mut state: Box<dyn State> = Box::new(Start);

    for c in input {
        state = state.receive(&mut ctx, Some(c))?;
        ctx.advance_position(c);
    }

    state.receive(&mut ctx, None)?;

    let tokens = ctx.into_tokens();
    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use super::tokenize;
    use crate::lexer::error::{LexError, LexErrorKind};
    use crate::lexer::tokens::{Keyword, Symbol, Token, TokenKind};
    use crate::Position;

    fn tokens(input: &str) -> Vec<Token> {
        tokenize(input.chars()).unwrap()
    }

    fn bad_char(c: char, position: Position) -> LexError {
        LexError {
            kind: LexErrorKind::UnexpectedCharacter(c),
            position,
        }
    }

    fn bad_number(n: String, position: Position) -> LexError {
        LexError {
            kind: LexErrorKind::InvalidNumericLiteral(n),
            position,
        }
    }

    // fn eof(position: Position) -> LexError {
    //     LexError {
    //         kind: LexErrorKind::UnexpectedEOF,
    //         position,
    //     }
    // }

    // fn eof_unquoted(position: Position) -> LexError {
    //     LexError {
    //         kind: LexErrorKind::UnclosedQuotedIdentifier,
    //         position,
    //     }
    // }

    // fn eof_string(position: Position) -> LexError {
    //     LexError {
    //         kind: LexErrorKind::UnclosedString,
    //         position,
    //     }
    // }


    #[test]
    fn test_empty_input() {
        assert_eq!(tokenize("".chars()), Ok(Vec::new()));
    }

    #[test]
    fn test_input_with_newlines() {
        let input = "\n\r\n\n";
        assert_eq!(
            tokens(input),
            vec![
                Token {
                    kind: TokenKind::LineSep,
                    position: Position { line: 1, column: 1 }
                },
                Token {
                    kind: TokenKind::LineSep,
                    position: Position { line: 2, column: 1 }
                },
                Token {
                    kind: TokenKind::LineSep,
                    position: Position { line: 3, column: 1 }
                },
                Token {
                    kind: TokenKind::LineSep,
                    position: Position { line: 4, column: 1 }
                },
            ]
        );
    }

    #[test]
    fn test_comment_and_newlines() {
        let input = "\n-- this is -- a comment\n";
        assert_eq!(
            tokens(input),
            vec![
                Token {
                    kind: TokenKind::LineSep,
                    position: Position { line: 1, column: 1 }
                },
                Token {
                    kind: TokenKind::LineSep,
                    position: Position {
                        line: 2,
                        column: 24
                    }
                },
            ]
        );
    }

    #[test]
    fn test_keywords() {
        let input = "as schema table";
        assert_eq!(
            tokens(input),
            vec![
                Token {
                    kind: TokenKind::Keyword(Keyword::As),
                    position: Position { line: 1, column: 1 },
                },
                Token {
                    kind: TokenKind::Keyword(Keyword::Schema),
                    position: Position { line: 1, column: 4 },
                },
                Token {
                    kind: TokenKind::Keyword(Keyword::Table),
                    position: Position { line: 1, column: 11 },
                },
            ]
        );
    }

    #[test]
    fn test_bools() {
        let input = "true t false f";
        assert_eq!(
            tokens(input),
            vec![
                Token {
                    kind: TokenKind::Bool(true),
                    position: Position { line: 1, column: 1 },
                },
                Token {
                    kind: TokenKind::Bool(true),
                    position: Position { line: 1, column: 6 },
                },
                Token {
                    kind: TokenKind::Bool(false),
                    position: Position { line: 1, column: 8 },
                },
                Token {
                    kind: TokenKind::Bool(false),
                    position: Position {
                        line: 1,
                        column: 14
                    },
                },
            ]
        );
    }

    #[test]
    fn test_identifiers() {
        for ident in [
            "something",
            "anything",
            "more_things",
            "__and_more__",
            "even_this_💝_",
            // Postgres interprets these as column names rather than numbers with "trailing junk",
            // so these should be interpreted as identifiers
            "_123",
            "_1__23",
        ] {
            assert_eq!(
                tokens(ident),
                vec![Token {
                    kind: TokenKind::Identifier(ident.to_owned()),
                    position: Position { line: 1, column: 1 },
                }]
            );
        }
    }

    #[test]
    fn test_quoted_identifiers() {
        let input = r#""this is an identifier" "and so
        is this" "and this""#;
        assert_eq!(
            tokens(input),
            vec![
                Token {
                    kind: TokenKind::QuotedIdentifier("\"this is an identifier\"".to_string()),
                    position: Position { line: 1, column: 1 },
                },
                Token {
                    kind: TokenKind::QuotedIdentifier("\"and so\n        is this\"".to_string()),
                    position: Position {
                        line: 1,
                        column: 25
                    },
                },
                Token {
                    kind: TokenKind::QuotedIdentifier("\"and this\"".to_string()),
                    position: Position { line: 2, column: 18 },
                },
            ]
        );
    }

    #[test]
    fn test_numbers() {
        for num in [
            "0",
            "0.",
            ".0",
            "123",
            "-456",
            "12.34",
            "-45.67",
            "1.",
            ".2",
            "-3.",
            "-.4",
            "1_2",
            "1_2_3",
            "12_34",
            "1_2.3_4",
            "1_2.3_4_5",
        ] {
            assert_eq!(
                tokens(num),
                vec![Token {
                    kind: TokenKind::Number(num.to_owned()),
                    position: Position { line: 1, column: 1 },
                }],
                "{}",
                num,
            );
        }
    }

    #[test]
    fn test_malformed_numbers() {
        for (input, column) in [("1.1. ", 4), (".1.1 ", 3), ("12_.34", 4)] {
            assert_eq!(
                tokenize(input.chars()),
                Err(bad_char('.', Position { line: 1, column })),
                "{}",
                input,
            );
        }
        for (input, column) in [("12__34", 4), ("12._34", 4)] {
            assert_eq!(
                tokenize(input.chars()),
                Err(bad_char('_', Position { line: 1, column })),
                "{}",
                input,
            );
        }
        for input in ["123_ ", "12.34_ "] {
            assert_eq!(
                tokenize(input.chars()),
                Err(bad_number(
                    input.trim_end().to_string(),
                    Position { line: 1, column: 1 }
                )),
                "{}",
                input,
            );
        }
    }

    #[test]
    fn test_text() {
        let input = "'this is text'  'and this is too, isn''t that cool?' 'and
        this!'";
        assert_eq!(
            tokens(input),
            vec![
                Token {
                    kind: TokenKind::Text("'this is text'".to_string()),
                    position: Position { line: 1, column: 1 },
                },
                Token {
                    kind: TokenKind::Text("'and this is too, isn''t that cool?'".to_string()),
                    position: Position {
                        line: 1,
                        column: 17
                    },
                },
                Token {
                    kind: TokenKind::Text("'and\n        this!'".to_string()),
                    position: Position {
                        line: 1,
                        column: 54
                    },
                },
            ]
        );
    }

    #[test]
    fn test_underscores() {
        let input = "_ _ _one two_";
        assert_eq!(
            tokens(input),
            vec![
                Token {
                    kind: TokenKind::Symbol(Symbol::Underscore),
                    position: Position { line: 1, column: 1 },
                },
                Token {
                    kind: TokenKind::Symbol(Symbol::Underscore),
                    position: Position { line: 1, column: 3 },
                },
                Token {
                    kind: TokenKind::Identifier("_one".to_string()),
                    position: Position { line: 1, column: 5 },
                },
                Token {
                    kind: TokenKind::Identifier("two_".to_string()),
                    position: Position {
                        line: 1,
                        column: 10
                    },
                },
            ]
        );
    }

    #[test]
    fn test_other_symbols_followed_by_identifiers() {
        let input = r#" .one ."two" @three @"four" "#;
        assert_eq!(
            tokens(input),
            vec![
                Token {
                    kind: TokenKind::Symbol(Symbol::Period),
                    position: Position { line: 1, column: 2 },
                },
                Token {
                    kind: TokenKind::Identifier("one".to_string()),
                    position: Position { line: 1, column: 3 },
                },
                Token {
                    kind: TokenKind::Symbol(Symbol::Period),
                    position: Position { line: 1, column: 7 },
                },
                Token {
                    kind: TokenKind::QuotedIdentifier("\"two\"".to_string()),
                    position: Position { line: 1, column: 8 },
                },
                Token {
                    kind: TokenKind::Symbol(Symbol::AtSign),
                    position: Position {
                        line: 1,
                        column: 14
                    },
                },
                Token {
                    kind: TokenKind::Identifier("three".to_string()),
                    position: Position {
                        line: 1,
                        column: 15
                    },
                },
                Token {
                    kind: TokenKind::Symbol(Symbol::AtSign),
                    position: Position {
                        line: 1,
                        column: 21
                    },
                },
                Token {
                    kind: TokenKind::QuotedIdentifier("\"four\"".to_string()),
                    position: Position {
                        line: 1,
                        column: 22
                    },
                },
            ]
        );
    }
}
