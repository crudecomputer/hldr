mod error;
mod tokenizer;

pub use error::LexError;
pub use tokenizer::Token;

pub fn lex(text: &str) -> Result<Vec<Token>, LexError> {
    Ok(tokenizer::Tokenizer::new().tokenize(text)?.tokens)
}

#[cfg(test)]
mod tests {
    use super::{
        error::{LexErrorKind as K, Position as P},
        lex, LexError as E, Token as T,
    };

    fn indent(sp: &str) -> T {
        T::Indent(sp.to_owned())
    }

    #[test]
    fn empty() {
        assert_eq!(lex(""), Ok(vec![]));
    }

    #[test]
    fn whitespace() {
        let file = "  \n\n \t \n\t  \n\n   \n\t";

        assert_eq!(
            lex(file),
            Ok(vec![
                indent("  "),
                T::Newline,
                T::Newline,
                indent(" \t "),
                T::Newline,
                indent("\t  "),
                T::Newline,
                T::Newline,
                indent("   "),
                T::Newline,
                indent("\t"),
                T::Newline,
            ])
        );
    }

    #[test]
    fn comments_ignored() {
        let file = "-- a comment\n  -- another comment\n";

        assert_eq!(lex(file), Ok(vec![T::Newline, indent("  "), T::Newline,]));
    }

    #[test]
    fn comment_incomplete() {
        let file = "-- a comment
  - bad comment";

        assert_eq!(
            lex(file),
            Err(E {
                position: P { line: 2, column: 4 },
                kind: K::UnexpectedCharacter(' '),
            })
        );
    }

    #[test]
    fn comment_unfinished() {
        let file = "-- a comment
  -";

        assert_eq!(
            lex(file),
            Err(E {
                position: P { line: 2, column: 3 },
                kind: K::ExpectedComment,
            })
        );
    }

    #[test]
    fn simple_numbers() {
        let file = "123 0.12341 --a comment
    .1234
1.1235";

        assert_eq!(
            lex(file),
            Ok(vec![
                T::Number("123".to_owned()),
                T::Number("0.12341".to_owned()),
                T::Newline,
                indent("    "),
                T::Number(".1234".to_owned()),
                T::Newline,
                T::Number("1.1235".to_owned()),
                T::Newline,
            ])
        );
    }

    #[test]
    fn double_dots() {
        assert_eq!(
            lex(".."),
            Err(E {
                position: P { line: 1, column: 2 },
                kind: K::UnexpectedCharacter('.'),
            })
        );
    }

    #[test]
    fn double_decimals1() {
        assert_eq!(
            lex(".123."),
            Err(E {
                position: P { line: 1, column: 5 },
                kind: K::UnexpectedCharacter('.'),
            })
        );
    }

    #[test]
    fn double_decimals2() {
        assert_eq!(
            lex("1.123."),
            Err(E {
                position: P { line: 1, column: 6 },
                kind: K::UnexpectedCharacter('.'),
            })
        );
    }

    #[test]
    fn simple_identifiers() {
        let file = "identifier1 ident_ifier2 --a comment
    _ident3";

        assert_eq!(
            lex(file),
            Ok(vec![
                T::Identifier("identifier1".to_owned()),
                T::Identifier("ident_ifier2".to_owned()),
                T::Newline,
                indent("    "),
                T::Identifier("_ident3".to_owned()),
                T::Newline,
            ])
        );
    }

    #[test]
    fn identifier_cant_start_with_number() {
        assert_eq!(
            lex("1asdf"),
            Err(E {
                position: P { line: 1, column: 2 },
                kind: K::UnexpectedCharacter('a'),
            })
        );
    }

    #[test]
    fn bools() {
        let file = "t T true\n\tTrue TRUE f \n  F false False FALSE";

        assert_eq!(
            lex(file),
            Ok(vec![
                T::Boolean(true),
                T::Identifier("T".to_owned()),
                T::Boolean(true),
                T::Newline,
                indent("\t"),
                T::Identifier("True".to_owned()),
                T::Identifier("TRUE".to_owned()),
                T::Boolean(false),
                T::Newline,
                indent("  "),
                T::Identifier("F".to_owned()),
                T::Boolean(false),
                T::Identifier("False".to_owned()),
                T::Identifier("FALSE".to_owned()),
                T::Newline,
            ])
        );
    }

    #[test]
    fn quoted_identifiers() {
        let file = r#""some identifier" ident_ifier2 -- a "quoted comment"
    "-- another""@""""identifier"
"#;

        assert_eq!(
            lex(file),
            Ok(vec![
                T::QuotedIdentifier("some identifier".to_owned()),
                T::Identifier("ident_ifier2".to_owned()),
                T::Newline,
                indent("    "),
                T::QuotedIdentifier(r#"-- another"@""identifier"#.to_owned()),
                T::Newline,
            ])
        );
    }

    #[test]
    fn unclosed_quoted_identifier() {
        assert_eq!(
            lex("\"asdf"),
            Err(E {
                position: P { line: 1, column: 5 },
                kind: K::UnclosedQuotedIdentifier,
            })
        );
    }

    #[test]
    fn test_strings() {
        let file = "'some string'
'another''s string' too 'and again'";

        assert_eq!(
            lex(file),
            Ok(vec![
                T::Text("some string".to_owned()),
                T::Newline,
                T::Text("another's string".to_owned()),
                T::Identifier("too".to_owned()),
                T::Text("and again".to_owned()),
                T::Newline,
            ])
        );
    }

    #[test]
    fn unclosed_string() {
        assert_eq!(
            lex("'asdf"),
            Err(E {
                position: P { line: 1, column: 5 },
                kind: K::UnclosedString,
            })
        );
    }

    #[test]
    fn underscore_after_indent() {
        assert_eq!(
            lex("\t_"),
            Ok(vec![indent("\t"), T::Underscore, T::Newline,])
        );
    }

    #[test]
    fn underscore_with_comment() {
        assert_eq!(
            lex("\t_ -- some comment"),
            Ok(vec![indent("\t"), T::Underscore, T::Newline,])
        );
    }

    #[test]
    fn good_file() {
        let file = r#"public
  -- This is a newline comment
  pet
    cupid
      name 'Cupid' -- This is a trailing comment
      species 'cat'
      lap_cat true

    _
      name 'Eiyre'
      lap_cat false

  person
    kevin
      name 'Kevin'
      age 39
      favorite_book 'Cat''s Cradle'

"quoted @ schema"
  message
    _
      text 'Hello, world!'
"#;

        assert_eq!(
            lex(file),
            Ok(vec![
                T::Identifier("public".to_owned()),
                T::Newline,
                indent("  "),
                T::Newline,
                indent("  "),
                T::Identifier("pet".to_owned()),
                T::Newline,
                indent("    "),
                T::Identifier("cupid".to_owned()),
                T::Newline,
                indent("      "),
                T::Identifier("name".to_owned()),
                T::Text("Cupid".to_owned()),
                T::Newline,
                indent("      "),
                T::Identifier("species".to_owned()),
                T::Text("cat".to_owned()),
                T::Newline,
                indent("      "),
                T::Identifier("lap_cat".to_owned()),
                T::Boolean(true),
                T::Newline,
                T::Newline,
                indent("    "),
                T::Underscore,
                T::Newline,
                indent("      "),
                T::Identifier("name".to_owned()),
                T::Text("Eiyre".to_owned()),
                T::Newline,
                indent("      "),
                T::Identifier("lap_cat".to_owned()),
                T::Boolean(false),
                T::Newline,
                T::Newline,
                indent("  "),
                T::Identifier("person".to_owned()),
                T::Newline,
                indent("    "),
                T::Identifier("kevin".to_owned()),
                T::Newline,
                indent("      "),
                T::Identifier("name".to_owned()),
                T::Text("Kevin".to_owned()),
                T::Newline,
                indent("      "),
                T::Identifier("age".to_owned()),
                T::Number("39".to_owned()),
                T::Newline,
                indent("      "),
                T::Identifier("favorite_book".to_owned()),
                T::Text("Cat's Cradle".to_owned()),
                T::Newline,
                T::Newline,
                T::QuotedIdentifier("quoted @ schema".to_owned()),
                T::Newline,
                indent("  "),
                T::Identifier("message".to_owned()),
                T::Newline,
                indent("    "),
                T::Underscore,
                T::Newline,
                indent("      "),
                T::Identifier("text".to_owned()),
                T::Text("Hello, world!".to_owned()),
                T::Newline,
            ])
        );
    }
}
