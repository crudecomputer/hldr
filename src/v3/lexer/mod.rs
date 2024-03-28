mod errors;
mod states;
mod tokens;

pub use errors::*;
pub use tokens::{Keyword, Symbol, Token};

pub fn tokenize(input: impl Iterator<Item = char>) -> Result<Vec<Token>, LexError> {
    let mut context = states::Context::default();
    let mut state: Box<dyn states::State> = Box::new(states::Start);

    for c in input {
        state = state.receive(&mut context, c)?;
    }

    if let Err(_) = state.receive(&mut context, states::EOF) {
        return Err(LexError::unexpected_eof());
    }

    println!("{:?}", context);

    Ok(context.into_tokens())
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::states::*;

    #[test]
    fn test_empty_input() {
        assert_eq!(tokenize("".chars()), Ok(Vec::new()));
    }

    #[test]
    fn test_null_input() {
        let input = format!("{}\t{}", NULL, NULL);
        assert_eq!(tokenize(input.chars()), Ok(Vec::new()));
    }

    #[test]
    fn test_input_with_newlines() {
        let input = "\n\r\r\n\n";
        assert_eq!(tokenize(input.chars()), Ok(vec![
            Token::LineSep,
        ]));
    }

    #[test]
    fn test_comment_and_newlines() {
        let input = "\n-- this is -- a comment\r\n";
        assert_eq!(tokenize(input.chars()), Ok(vec![
            Token::LineSep,
            Token::LineSep,
        ]));
    }

    #[test]
    fn test_keywords() {
        let input = "as";
        assert_eq!(tokenize(input.chars()), Ok(vec![
            Token::Keyword(Keyword::As),
        ]));
    }

    #[test]
    fn test_bools() {
        let input = "true t false f";
        assert_eq!(tokenize(input.chars()), Ok(vec![
            Token::Bool(true),
            Token::Bool(true),
            Token::Bool(false),
            Token::Bool(false),
        ]));
    }

    #[test]
    fn test_identifiers() {
        for ident in [
            "something", "anything",
            "more_things", "__and_more__",
            "even_this_💝_",
            // Postgres interprets these as column names rather than numbers with "trailing junk"
            "_123", "_1__23",
        ] {
            assert_eq!(tokenize(ident.chars()), Ok(vec![
                Token::Identifier(ident.to_string()),
            ]));
        }
    }

    #[test]
    fn test_quoted_identifiers() {
        let input = "\"this is an identifier\" \"and so
        is this\"";
        assert_eq!(tokenize(input.chars()), Ok(vec![
            Token::QuotedIdentifier("this is an identifier".to_string()),
            Token::QuotedIdentifier("and so\n        is this".to_string()),
        ]));
    }

    #[test]
    fn test_numbers() {
        for num in [
            "0", "0.", ".0",
            "123", "-456", "12.34", "-45.67",
            "1.", ".2", "-3.", "-.4",
            "1_2", "1_2_3", "12_34", "1_2.3_4", "1_2.3_4_5",
        ] {
            assert_eq!(tokenize(num.chars()), Ok(vec![
                Token::Number(num.to_string()),
            ]));
        }
    }

    #[test]
    fn test_malformed_numbers() {
        for input in ["1.1. ", ".1.1 ", "12_.34"] {
            assert_eq!(tokenize(input.chars()), Err(LexError::unexpected('.')));
        }
        for input in ["123_ ", "12__34", "12._34", "12.34_ "] {
            assert_eq!(tokenize(input.chars()), Err(LexError::unexpected('_')));
        }
    }

    #[test]
    fn test_text() {
        let input = "'this is text'  'and this is too, isn''t that cool?' 'and
        this!'";
        assert_eq!(tokenize(input.chars()), Ok(vec![
            Token::Text("this is text".to_string()),
            Token::Text("and this is too, isn't that cool?".to_string()),
            Token::Text("and\n        this!".to_string()),
        ]));
    }

    #[test]
    fn test_underscores() {
        let input = "_ _ _one two_";
        assert_eq!(tokenize(input.chars()), Ok(vec![
            Token::Symbol(Symbol::Underscore),
            Token::Symbol(Symbol::Underscore),
            Token::Identifier("_one".to_string()),
            Token::Identifier("two_".to_string()),
        ]));
    }

    #[test]
    fn test_other_symbols_followed_by_identifiers() {
        let input = r#" .one ."two" @three @"four" "#;
        assert_eq!(tokenize(input.chars()), Ok(vec![
            Token::Symbol(Symbol::Period),
            Token::Identifier("one".to_string()),

            Token::Symbol(Symbol::Period),
            Token::QuotedIdentifier("two".to_string()),

            Token::Symbol(Symbol::AtSign),
            Token::Identifier("three".to_string()),

            Token::Symbol(Symbol::AtSign),
            Token::QuotedIdentifier("four".to_string()),
        ]));
    }

    #[test]
    fn test_symbols_followed_by_whitespace() {
        for input in ["one. ", "\"two\". ", /*"three@ ", "\"four\"@ "*/] {
            assert_eq!(tokenize(input.chars()), Err(LexError::unexpected(' ')));
        }
    }
}
