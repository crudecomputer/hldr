use std::fmt;

use super::{error::LexError, Position};

const NULL: char = '\0';

/// Set of all keyword tokens
#[derive(Clone, Debug, PartialEq)]
pub enum Keyword {
    As,
}

impl fmt::Display for Keyword {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Keyword::*;

        Ok(match self{
            As => write!(f, "as")?,
        })
    }
}

/// Set of allowed number tokens
#[derive(Clone, Debug, PartialEq)]
pub enum Number {
    Int(String),
    Float(String),
}

impl fmt::Display for Number {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Number::*;

        Ok(match self{
            Int(n) => write!(f, "{}", n)?,
            Float(n) => write!(f, "{}", n)?,
        })
    }
}

/// Set of possible stand-alone symbol tokens
#[derive(Clone, Debug, PartialEq)]
pub enum Symbol {
    AtSign,
    Period,
    Underscore,
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Symbol::*;

        Ok(match self{
            AtSign => write!(f, "@")?,
            Period => write!(f, ".")?,
            Underscore => write!(f, "_")?,
        })
    }
}

/// Set of possible whitespace tokens
#[derive(Clone, Debug, PartialEq)]
pub enum Whitespace {
    Inline(String),
    Newline,
}

impl fmt::Display for Whitespace {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Whitespace::*;

        Ok(match self{
            Inline(i) => write!(f, "whitespace `{}`", i)?,
            Newline => write!(f, "newline")?,
        })
    }
}

/// Set of all possible tokens
#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Boolean(bool),
    Comment(String),
    Identifier(String),
    Keyword(Keyword),
    Number(Number),
    QuotedIdentifier(String),
    Symbol(Symbol),
    Text(String),
    Whitespace(Whitespace),
}

/// A token with its position
#[derive(Clone, Debug, PartialEq)]
pub struct TokenPosition {
    token: Token,
    start_position: Position,
    end_position: Position,
}

#[derive(Clone, Debug, PartialEq)]
pub enum State {
    Comment,
    Dash,
    Float,
    Identifier,
    Integer,
    QuotedIdentifier,
    Period,
    Start,
    Text,
    Underscore,
    Whitespace,
}

#[derive(Debug)]
pub(super) struct Tokenizer {
    state: State,
    stack: Vec<char>,
    start_position: Position,
    end_position: Position,
    pub tokens: Vec<TokenPosition>,
}

impl Tokenizer {
    pub fn new() -> Self {
        Self {
            start_position: Position { line: 1, column: 1 },
            end_position: Position { line: 1, column: 1 },
            state: State::Start,
            stack: Vec::new(),
            tokens: Vec::new(),
        }
    }

    fn add_token(&mut self, token: Token) {
        self.tokens.push(TokenPosition {
            token,
            start_position: self.start_position,
            end_position: self.end_position,
        })
    }

    fn receive(&mut self, c: char) -> Result<State, LexError> {
        Ok(match self.state {
            State::Start => match c {
                NULL => {
                    State::Start
                }
                '@' => {
                    self.add_token(Token::Symbol(Symbol::AtSign));
                    self.reset_position();
                    State::Start
                }
                '-' => {
                    self.end_position.column += 1;
                    State::Dash
                }
                '_' => {
                    State::Underscore
                }
                '\'' => {
                    State::Text
                }
                '"' => {
                    State::QuotedIdentifier
                }
                '.' => {
                    State::Period
                }
                '0'..='9' => {
                    self.stack.push(c);
                    State::Integer
                }
                c if is_newline(c) => {
                    self.add_token(Token::Whitespace(Whitespace::Newline));

                    self.start_position.line += 1;
                    self.start_position.column = 1;
                    self.end_position = self.start_position;

                    State::Start
                }
                c if is_inline_whitespace(c) => {
                    self.stack.push(c);
                    State::Whitespace
                }
                c if is_valid_identifier(c) => {
                    self.stack.push(c);
                    State::Identifier
                }
                _ => return Err(self.unexpected(c))
            }
            State::Comment => match c {
                NULL => {
                    let stack = self.drain_stack();
                    let token = Token::Comment(stack);

                    self.add_token(token);
                    self.reset_with(c)?
                }
                _ if is_newline(c) => {
                    let stack = self.drain_stack();
                    let token = Token::Comment(stack);

                    self.add_token(token);
                    self.reset_with(c)?
                }
                _ => {
                    self.stack.push(c);
                    self.end_position.column += 1;
                    State::Comment
                }
            }
            State::Dash => match c {
                '-' => State::Comment,
                _ => return Err(self.unexpected(c))
            }
            State::Float => match c {
                '0'..='9' => {
                    self.stack.push(c);
                    self.end_position.column += 1;
                    State::Float
                }
                '.' => return Err(self.unexpected(c)),
                _ => {
                    let stack = self.drain_stack();
                    let token = Token::Number(Number::Float(stack));

                    self.add_token(token);
                    self.reset_with(c)?
                }
            }
            State::Identifier => match c {
                c if is_valid_identifier(c) => {
                    self.stack.push(c);
                    self.end_position.column += 1;
                    State::Identifier
                }
                _ => {
                    let stack = self.drain_stack();
                    let token = identifier_to_token(stack);

                    self.add_token(token);
                    self.reset_with(c)?
                }
            }
            State::Integer => match c {
                '0'..='9' => {
                    self.stack.push(c);
                    self.end_position.column += 1;
                    State::Integer
                }
                '.' => {
                    self.stack.push(c);
                    self.end_position.column += 1;
                    State::Float
                }
                _ => {
                    let stack = self.drain_stack();
                    let token = Token::Number(Number::Int(stack));

                    self.add_token(token);
                    self.reset_with(c)?
                }
            }
            State::QuotedIdentifier => match c {
                '"' => {
                    let stack = self.drain_stack();
                    let token = Token::QuotedIdentifier(stack);

                    self.add_token_and_advance(token)
                }
                _ => {
                    self.stack.push(c);
                    self.end_position.column += 1;
                    State::QuotedIdentifier
                }
            }
            State::Period => match c {
                '0'..='9' => {
                    self.stack.push('.');
                    self.stack.push(c);
                    self.end_position.column += 1;
                    State::Float
                }
                _ => {
                    self.add_token(Token::Symbol(Symbol::Period));
                    self.reset_with(c)?
                }
            }
            State::Text => match c {
                '\'' => {
                    let stack = self.drain_stack();
                    let token = Token::Text(stack);

                    self.add_token_and_advance(token)
                }
                _ => {
                    self.stack.push(c);
                    self.end_position.column += 1;
                    State::Text
                }
            }
            State::Underscore => match c {
                c if is_valid_identifier(c) => {
                    self.stack.push('_');
                    self.stack.push(c);
                    self.end_position.column += 1;
                    State::Identifier
                }
                _ => {
                    self.add_token(Token::Symbol(Symbol::Underscore));
                    self.reset_with(c)?
                }
            }
            State::Whitespace => match c {
                ' ' | '\t' => {
                    self.stack.push(c);
                    self.end_position.column += 1;
                    State::Whitespace
                }
                _ => {
                    let stack = self.drain_stack();
                    let token = Token::Whitespace(Whitespace::Inline(stack));

                    self.add_token(token);
                    self.reset_with(c)?
                }
            }
        })
    }

    fn unexpected(&self, c: char) -> LexError {
        let mut position = self.end_position;

        position.column += 1;
        LexError::unexpected_character(position, c)
    }

    /// Useful for adding a token when a character is encountered that needs
    /// to be skipped, such as closing single- or double- quotes
    fn add_token_and_advance(&mut self, token: Token) -> State {
        self.end_position.column += 1;
        self.add_token(token);
        self.reset_position();
        State::Start
    }

    fn reset_position(&mut self) {
        self.end_position.column += 1;
        self.start_position = self.end_position;
    }

    fn reset_with(&mut self, c: char) -> Result<State, LexError> {
        self.reset_position();
        self.state = State::Start;

        self.receive(c)
    }

    fn drain_stack(&mut self) -> String {
        self.stack.drain(..).collect()
    }

    pub fn tokenize(mut self, input: &str) -> Result<Self, LexError> {
        for c in input.chars() {
            self.state = self.receive(c)?;
        }

        // An 'escape hatch' to make sure the last state/stack are processed
        // if not ending back at the 'start' state
        self.state = self.receive(NULL)?;

        Ok(self)
    }
}

fn is_newline(c: char) -> bool {
    c == '\n'
}

fn is_inline_whitespace(c: char) -> bool {
    c == ' ' || c == '\t'
}

fn is_valid_identifier(c: char) -> bool {
    c == '_' || (
        // "alphabetic" isn't enough because that precludes other unicode chars
        // that are valid in postgres identifiers
        !c.is_control() && !c.is_whitespace() && !c.is_ascii_punctuation()
    )
}

fn identifier_to_token(s: String) -> Token {
    match s.as_ref() {
        "_" => Token::Symbol(Symbol::Underscore),
        "true" | "t" => Token::Boolean(true),
        "false" | "f" => Token::Boolean(false),
        "as" => Token::Keyword(Keyword::As),
        _ => Token::Identifier(s),
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::super::error::LexErrorKind;
    use super::*;

    mod helpers {
        use super::super::*;

        pub fn at_sign() -> Token {
            Token::Symbol(Symbol::AtSign)
        }

        pub fn boolean(b: bool) -> Token {
            Token::Boolean(b)
        }

        pub fn comment(c: &str) -> Token {
            Token::Comment(c.to_owned())
        }

        pub fn float(n: &str) -> Token {
            Token::Number(Number::Float(n.to_owned()))
        }

        pub fn identifier(i: &str) -> Token {
            Token::Identifier(i.to_owned())
        }

        pub fn int(n: &str) -> Token {
            Token::Number(Number::Int(n.to_owned()))
        }

        pub fn newline() -> Token {
            Token::Whitespace(Whitespace::Newline)
        }

        pub fn period() -> Token {
            Token::Symbol(Symbol::Period)
        }

        pub fn quoted(i: &str) -> Token {
            Token::QuotedIdentifier(i.to_owned())
        }

        pub fn space() -> Token {
            spaces(1)
        }

        pub fn spaces(n: usize) -> Token {
            Token::Whitespace(Whitespace::Inline(" ".repeat(n)))
        }

        pub fn text(t: &str) -> Token {
            Token::Text(t.to_owned())
        }

        pub fn underscore() -> Token {
            Token::Symbol(Symbol::Underscore)
        }

        pub fn whitespace(ws: &str) -> Token {
            Token::Whitespace(Whitespace::Inline(ws.to_owned()))
        }

        pub fn tokenize(input: &str) -> Result<Vec<TokenPosition>, LexError> {
            Ok(Tokenizer::new().tokenize(input)?.tokens)
        }

        #[test]
        fn helpers_tests() {
            assert_eq!(at_sign(), Token::Symbol(Symbol::AtSign));

            assert_eq!(boolean(true), Token::Boolean(true));
            assert_eq!(boolean(false), Token::Boolean(false));

            assert_eq!(comment("some comment"), Token::Comment("some comment".to_owned()));
            assert_eq!(comment("another comment"), Token::Comment("another comment".to_owned()));

            assert_eq!(float("0.12"), Token::Number(Number::Float("0.12".to_owned())));
            assert_eq!(float("1.23"), Token::Number(Number::Float("1.23".to_owned())));

            assert_eq!(identifier("wat"), Token::Identifier("wat".to_owned()));
            assert_eq!(identifier("hey"), Token::Identifier("hey".to_owned()));

            assert_eq!(int("12"), Token::Number(Number::Int("12".to_owned())));
            assert_eq!(int("123"), Token::Number(Number::Int("123".to_owned())));

            assert_eq!(newline(), Token::Whitespace(Whitespace::Newline));

            assert_eq!(period(), Token::Symbol(Symbol::Period));

            assert_eq!(quoted("wat"), Token::QuotedIdentifier("wat".to_owned()));
            assert_eq!(quoted("hey"), Token::QuotedIdentifier("hey".to_owned()));

            assert_eq!(space(), Token::Whitespace(Whitespace::Inline(" ".to_owned())));
            assert_eq!(spaces(3), Token::Whitespace(Whitespace::Inline("   ".to_owned())));
            assert_eq!(spaces(7), Token::Whitespace(Whitespace::Inline("       ".to_owned())));

            assert_eq!(text("wat"), Token::Text("wat".to_owned()));
            assert_eq!(text("hey"), Token::Text("hey".to_owned()));

            assert_eq!(underscore(), Token::Symbol(Symbol::Underscore));

            assert_eq!(whitespace("   "), Token::Whitespace(Whitespace::Inline("   ".to_owned())));
            assert_eq!(whitespace(" \t \t "), Token::Whitespace(Whitespace::Inline(" \t \t ".to_owned())));
        }
    }

    use helpers::*;

    fn tp(start: (usize, usize), end: (usize, usize), token: Token) -> TokenPosition {
        TokenPosition {
            start_position: Position {
                line: start.0,
                column: start.1,
            },
            end_position: Position {
                line: end.0,
                column: end.1,
            },
            token,
        }
    }

    #[test]
    fn bools() {
        let input = "true \t  false";

        assert_eq!(
            tokenize(input),
            Ok(vec![
                tp((1, 1), (1,  4), boolean(true)),
                tp((1, 5), (1,  8), whitespace(" \t  ")),
                tp((1, 9), (1, 13), boolean(false)),
            ])
        );
    }

    #[test]
    fn identifiers() {
        let input = "other \t  _things \n Here_that_are_Identifiers";

        assert_eq!(
            tokenize(input),
            Ok(vec![
                tp((1,  1), (1,  5), identifier("other")),
                tp((1,  6), (1,  9), whitespace(" \t  ")),
                tp((1, 10), (1, 16), identifier("_things")),
                tp((1, 17), (1, 17), space()),
                tp((1, 18), (1, 18), newline()),

                tp((2,  1), (2,  1), space()),
                tp((2,  2), (2, 26), identifier("Here_that_are_Identifiers")),
            ])
        );
    }

    #[test]
    fn keywords() {
        let input = "one_thing as another_thing";

        assert_eq!(
            tokenize(input),
            Ok(vec![
                tp((1,  1), (1,  9), identifier("one_thing")),
                tp((1, 10), (1, 10), space()),
                tp((1, 11), (1, 12), Token::Keyword(Keyword::As)),
                tp((1, 13), (1, 13), space()),
                tp((1, 14), (1, 26), identifier("another_thing")),
            ])
        )
    }

    #[test]
    fn numbers() {
        let input = "12 12. 12.34 .34";

        assert_eq!(
            tokenize(input),
            Ok(vec![
                tp((1,  1), (1,  2), int("12")),
                tp((1,  3), (1,  3), space()),
                tp((1,  4), (1,  6), float("12.")),
                tp((1,  7), (1,  7), space()),
                tp((1,  8), (1, 12), float("12.34")),
                tp((1, 13), (1, 13), space()),
                tp((1, 14), (1, 16), float(".34")),
            ])
        );
    }

    #[test]
    fn numbers_double_decimal_fails() {
        let input = "12.34.56";

        assert_eq!(
            tokenize(input),
            Err(LexError {
                kind: LexErrorKind::UnexpectedCharacter('.'),
                position: Position {
                    line: 1,
                    column: 6,
                },
            })
        )
    }

    #[test]
    fn quoted_identifiers() {
        let input = "some \"quoted identifier\" \n \"and here too\"";

        assert_eq!(
            tokenize(input),
            Ok(vec![
                tp((1,  1), (1,  4), identifier("some")),
                tp((1,  5), (1,  5), space()),
                tp((1,  6), (1, 24), quoted("quoted identifier")),
                tp((1, 25), (1, 25), space()),
                tp((1, 26), (1, 26), newline()),
                tp((2,  1), (2,  1), space()),
                tp((2,  2), (2, 15), quoted("and here too")),
            ])
        );
    }

    #[test]
    fn texts() {
        let input = "identifier 'some text' \"and a quoted identifier\"";

        assert_eq!(
            tokenize(input),
            Ok(vec![
                tp((1,  1), (1, 10), identifier("identifier")),
                tp((1, 11), (1, 11), space()),
                tp((1, 12), (1, 22), text("some text")),
                tp((1, 23), (1, 23), space()),
                tp((1, 24), (1, 48), quoted("and a quoted identifier")),
            ])
        );
    }

    #[test]
    fn symbols() {
        // This doesn't test single quotes or double quotes because they
        // are special cases that are part of other tokens
        let input = "@._";

        assert_eq!(
            tokenize(input),
            Ok(vec![
                tp((1, 1), (1, 1), at_sign()),
                tp((1, 2), (1, 2), period()),
                tp((1, 3), (1, 3), underscore()),
            ])
        );

        let input = "_  ";

        assert_eq!(
            tokenize(input),
            Ok(vec![
                tp((1, 1), (1, 1), underscore()),
                tp((1, 2), (1, 3), spaces(2)),
            ])
        );
    }

    #[test]
    fn full_syntax() {
        let input =
"schema1
  person as p
    _
      name 'anon'

    p1
      name       'person 1'
      likes_pizza true

  pet
    _
      age       13
      weight    12.25
      person_id p@p1.id

    _
      person_id @p1.id

    _
      person_id schema1.person@p1.id

  \"quoted identifier\" -- and a comment

-- and another comment";

        let blank = |line| tp((line, 1), (line, 1), newline());

        assert_eq!(
            tokenize(input),
            Ok(vec![
                tp(( 1,  1), ( 1,  7), identifier("schema1")),
                tp(( 1,  8), ( 1,  8), newline()),

                tp(( 2,  1), ( 2,  2), spaces(2)),
                tp(( 2,  3), ( 2,  8), identifier("person")),
                tp(( 2,  9), ( 2,  9), space()),
                tp(( 2, 10), ( 2, 11), Token::Keyword(Keyword::As)),
                tp(( 2, 12), ( 2, 12), space()),
                tp(( 2, 13), ( 2, 13), identifier("p")),
                tp(( 2, 14), ( 2, 14), newline()),

                tp(( 3,  1), ( 3,  4), spaces(4)),
                tp(( 3,  5), ( 3,  5), underscore()),
                tp(( 3,  6), ( 3,  6), newline()),

                tp(( 4,  1), ( 4,  6), spaces(6)),
                tp(( 4,  7), ( 4, 10), identifier("name")),
                tp(( 4, 11), ( 4, 11), space()),
                tp(( 4, 12), ( 4, 17), text("anon")), // length includes single quotes
                tp(( 4, 18), ( 4, 18), newline()),

                blank(5),

                tp(( 6,  1), ( 6,  4), spaces(4)),
                tp(( 6,  5), ( 6,  6), identifier("p1")),
                tp(( 6,  7), ( 6,  7), newline()),

                tp(( 7,  1), ( 7,  6), spaces(6)),
                tp(( 7,  7), ( 7, 10), identifier("name")),
                tp(( 7, 11), ( 7, 17), spaces(7)),
                tp(( 7, 18), ( 7, 27), text("person 1")),
                tp(( 7, 28), ( 7, 28), newline()),

                tp(( 8,  1), ( 8,  6), spaces(6)),
                tp(( 8,  7), ( 8, 17), identifier("likes_pizza")),
                tp(( 8, 18), ( 8, 18), space()),
                tp(( 8, 19), ( 8, 22), boolean(true)),
                tp(( 8, 23), ( 8, 23), newline()),

                blank(9),

                tp((10,  1), (10,  2), spaces(2)),
                tp((10,  3), (10,  5), identifier("pet")),
                tp((10,  6), (10,  6), newline()),

                tp((11,  1), (11,  4), spaces(4)),
                tp((11,  5), (11,  5), underscore()),
                tp((11,  6), (11,  6), newline()),

                tp((12,  1), (12,  6), spaces(6)),
                tp((12,  7), (12,  9), identifier("age")),
                tp((12, 10), (12, 16), spaces(7)),
                tp((12, 17), (12, 18), int("13")),
                tp((12, 19), (12, 19), newline()),

                tp((13,  1), (13,  6), spaces(6)),
                tp((13,  7), (13, 12), identifier("weight")),
                tp((13, 13), (13, 16), spaces(4)),
                tp((13, 17), (13, 21), float("12.25")),
                tp((13, 22), (13, 22), newline()),

                tp((14,  1), (14,  6), spaces(6)),
                tp((14,  7), (14, 15), identifier("person_id")),
                tp((14, 16), (14, 16), space()),
                tp((14, 17), (14, 17), identifier("p")),
                tp((14, 18), (14, 18), at_sign()),
                tp((14, 19), (14, 20), identifier("p1")),
                tp((14, 21), (14, 21), period()),
                tp((14, 22), (14, 23), identifier("id")),
                tp((14, 24), (14, 24), newline()),

                blank(15),

                tp((16,  1), (16,  4), spaces(4)),
                tp((16,  5), (16,  5), underscore()),
                tp((16,  6), (16,  6), newline()),

                tp((17,  1), (17,  6), spaces(6)),
                tp((17,  7), (17, 15), identifier("person_id")),
                tp((17, 16), (17, 16), space()),
                tp((17, 17), (17, 17), at_sign()),
                tp((17, 18), (17, 19), identifier("p1")),
                tp((17, 20), (17, 20), period()),
                tp((17, 21), (17, 22), identifier("id")),
                tp((17, 23), (17, 23), newline()),

                blank(18),

                tp((19,  1), (19,  4), spaces(4)),
                tp((19,  5), (19,  5), underscore()),
                tp((19,  6), (19,  6), newline()),

                tp((20,  1), (20,  6), spaces(6)),
                tp((20,  7), (20, 15), identifier("person_id")),
                tp((20, 16), (20, 16), space()),
                tp((20, 17), (20, 23), identifier("schema1")),
                tp((20, 24), (20, 24), period()),
                tp((20, 25), (20, 30), identifier("person")),
                tp((20, 31), (20, 31), at_sign()),
                tp((20, 32), (20, 33), identifier("p1")),
                tp((20, 34), (20, 34), period()),
                tp((20, 35), (20, 36), identifier("id")),
                tp((20, 37), (20, 37), newline()),

                blank(21),

                tp((22,  1), (22,  2), spaces(2)),
                tp((22,  3), (22, 21), quoted("quoted identifier")),
                tp((22, 22), (22, 22), space()),
                tp((22, 23), (22, 38), comment(" and a comment")),
                tp((22, 39), (22, 39), newline()),

                blank(23),

                tp((24,  1), (24, 22), comment(" and another comment")),
            ])
        );
    }
}
