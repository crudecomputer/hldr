mod errors;
mod states;
mod tokens;

use errors::*;
use tokens::*;

pub fn tokenize(input: impl Iterator<Item = char>) -> Result<Vec<Token>, LexError> {
    let mut context = states::Context::default();
    let mut state: Box<dyn states::State> = Box::new(states::Start);

    for c in input {
        state = state.receive(&mut context, c)?;
    }

    state.receive(&mut context, states::EOF)?;
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
        assert_eq!(tokenize(input.chars()), Ok(vec![
            Token::Whitespace("\t".to_string()),
        ]));
    }

    #[test]
    fn test_input_with_newlines() {
        // "\r\n" should be treated as a single newline per Unicode spec
        let input = "\n\r\r\n\n";
        assert_eq!(tokenize(input.chars()), Ok(vec![
            Token::Newline,
            Token::Newline,
            Token::Newline,
            Token::Newline,
        ]));
    }

    #[test]
    fn test_comment_and_newlines() {
        let input = "\n-- this is -- a comment\r\n";
        assert_eq!(tokenize(input.chars()), Ok(vec![
            Token::Newline,
            Token::Newline,
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
            Token::Whitespace(" ".to_string()),
            Token::Bool(true),
            Token::Whitespace(" ".to_string()),
            Token::Bool(false),
            Token::Whitespace(" ".to_string()),
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
        for input in ["1.1.", ".1.1", "12_.34"] {
            assert_eq!(tokenize(input.chars()), Err(LexError::unexpected('.')));
        }
        for input in ["123_", "12__34", "12._34", "12.34_"] {
            assert_eq!(tokenize(input.chars()), Err(LexError::unexpected('_')));
        }
    }
}