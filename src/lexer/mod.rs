
mod machine;

use machine::Machine;

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Boolean(bool),
    Colon,
    Identifier(String),
    Indent(String),
    Number(String),
    Text(String),
    Underscore,
}

pub fn lex(text: &str) -> Vec<Token> {
    let mut machine = Machine::new();
    machine.lex(text);
    machine.tokens
}

#[cfg(test)]
mod tests {
    use super::{
        Token as T,
        lex,
    };
    
    fn indent(sp: &str) -> T {
        T::Indent(sp.to_owned())
    }

    #[test]
    fn empty() {
        assert_eq!(lex(""), vec![]);
    }

    #[test]
    fn whitespace() {
        let file = "  \n\n \t \n\t  \n\n   \n\t";

        assert_eq!(lex(file), vec![
            T::Indent("  ".to_owned()),
            T::Indent(" \t ".to_owned()),
            T::Indent("\t  ".to_owned()),
            T::Indent("   ".to_owned()),
            T::Indent("\t".to_owned()),
        ]);
    }

    #[test]
    fn comments_ignored() {
        let file = "-- a comment\n  -- another comment\n";

        assert_eq!(lex(file), vec![
            T::Indent("  ".to_owned()),
        ]);
    }

    #[test]
    #[should_panic(expected = "Unexpected character ' ' (line 2, column 4)")]
    fn comment_incomplete() {
        let file =
"-- a comment
  - bad comment";
        lex(file);
    }

    #[test]
    #[should_panic(expected = "Expected comment (line 2, column 3)")]
    fn comment_unfinished() {
        let file =
"-- a comment
  -";
        lex(file);
    }

    #[test]
    fn simple_numbers() {
        let file =
"123 0.12341 --a comment
    .1234
1.1235";

        assert_eq!(lex(file), vec![
            T::Number("123".to_owned()),
            T::Number("0.12341".to_owned()),
            T::Indent("    ".to_owned()),
            T::Number(".1234".to_owned()),
            T::Number("1.1235".to_owned()),
        ]);
    }

    #[test]
    #[should_panic(expected = "Unexpected character '.' (line 1, column 2)")]
    fn double_dots() { lex(".."); }

    #[test]
    #[should_panic(expected = "Unexpected character '.' (line 1, column 5)")]
    fn double_decimals1() { lex(".123."); }


    #[test]
    #[should_panic(expected = "Unexpected character '.' (line 1, column 6)")]
    fn double_decimals2() { lex("1.123."); }

    //#[test]
    fn simple_identifiers() {
        let file =
"identifier1 ident2 --a comment
    ident3";

        assert_eq!(lex(file), vec![
            T::Identifier("identifier1".to_owned()),
            T::Identifier("ident2".to_owned()),
            T::Indent("    ".to_owned()),
            T::Identifier("ident3".to_owned()),
        ]);
    }

    //#[test]
    fn good_file() {
        let file =
"public
  -- This is a newline comment
  pet
    cupid:
      name 'Cupid' -- This is a trailing comment
      species 'cat'
      lap_cat true

    _:
      name 'Eiyre'
      lap_cat false

  person
    kevin:
      name 'Kevin'
      age 38

schema1
  message
    _:
      text 'Hello, world!'
";

        assert_eq!(lex(file), vec![
            T::Identifier("public".to_owned()),

            indent("  "),
            T::Identifier("pet".to_owned()),

            indent("    "),
            T::Identifier("cupid".to_owned()),
            T::Colon,

            indent("      "),
            T::Identifier("name".to_owned()),
            T::Text("Cupid".to_owned()),
            indent("      "),
            T::Identifier("species".to_owned()),
            T::Text("cat".to_owned()),
            indent("      "),
            T::Identifier("lap_cat".to_owned()),
            T::Boolean(true),

            indent("    "),
            T::Underscore,
            T::Colon,

            indent("      "),
            T::Identifier("name".to_owned()),
            T::Text("Eiyre".to_owned()),
            indent("      "),
            T::Identifier("lap_cat".to_owned()),
            T::Boolean(false),

            indent("  "),
            T::Identifier("person".to_owned()),

            indent("    "),
            T::Identifier("kevin".to_owned()),
            T::Colon,

            indent("      "),
            T::Identifier("name".to_owned()),
            T::Text("Kevin".to_owned()),
            indent("      "),
            T::Identifier("age".to_owned()),
            T::Number("38".to_owned()),

            T::Identifier("schema1".to_owned()),

            indent("  "),
            T::Identifier("message".to_owned()),

            indent("    "),
            T::Underscore,
            T::Colon,

            indent("      "),
            T::Identifier("text".to_owned()),
            T::Text("Hello, world!".to_owned()),
        ]);
    }
}
