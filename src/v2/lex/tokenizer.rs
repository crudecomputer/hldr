use std::fmt;

use super::{error::LexError, Position};

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
    Underscore,
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Symbol::*;

        Ok(match self{
            AtSign => write!(f, "@")?,
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
    Start,
    Float,
    Identifier,
    Integer,
    QuotedIdentifier,
    Period,
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
                '\0' => {
                    State::Start
                }
                '@' => {
                    self.add_token(Token::Symbol(Symbol::AtSign));
                    State::Start
                }
                '_' => {
                    self.stack.push(c);
                    State::Underscore
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
                    let token = Token::QuotedIdentifier(stack.to_owned());

                    // Unlike other places, we want to discard this character but we
                    // still need to advance the positions properly and include the
                    // character's position as the end point of the token
                    self.end_position.column += 1;
                    self.add_token(token);

                    self.reset_position();
                    State::Start
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
                _ => return Err(self.unexpected(c))
            }
            State::Underscore => match c {
                c if is_valid_identifier(c) => {
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
        self.receive('\0')?;

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
    use super::{Token as T, Whitespace as WS};
    use super::*;

    fn tokenize(input: &str) -> Result<Vec<TokenPosition>, LexError> {
        Ok(Tokenizer::new().tokenize(input)?.tokens)
    }

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
                tp((1, 1), (1,  4), T::Boolean(true)),
                tp((1, 5), (1,  8), T::Whitespace(WS::Inline(" \t  ".to_owned()))),
                tp((1, 9), (1, 13), T::Boolean(false)),
            ])
        );
    }

    #[test]
    fn identifiers() {
        let input = "other \t  _things \n Here_that_are_Identifiers";

        assert_eq!(
            tokenize(input),
            Ok(vec![
                tp((1,  1), (1,  5), T::Identifier("other".to_owned())),
                tp((1,  6), (1,  9), T::Whitespace(WS::Inline(" \t  ".to_owned()))),
                tp((1, 10), (1, 16), T::Identifier("_things".to_owned())),
                tp((1, 17), (1, 17), T::Whitespace(WS::Inline(" ".to_owned()))),
                tp((1, 18), (1, 18), T::Whitespace(WS::Newline)),

                tp((2,  1), (2,  1), T::Whitespace(WS::Inline(" ".to_owned()))),
                tp((2,  2), (2, 26), T::Identifier("Here_that_are_Identifiers".to_owned())),
            ])
        );
    }

    #[test]
    fn keywords() {
        let input = "one_thing as another_thing";

        assert_eq!(
            tokenize(input),
            Ok(vec![
                tp((1,  1), (1,  9), T::Identifier("one_thing".to_owned())),
                tp((1, 10), (1, 10), T::Whitespace(WS::Inline(" ".to_owned()))),
                tp((1, 11), (1, 12), T::Keyword(Keyword::As)),
                tp((1, 13), (1, 13), T::Whitespace(WS::Inline(" ".to_owned()))),
                tp((1, 14), (1, 26), T::Identifier("another_thing".to_owned())),
            ])
        )
    }

    #[test]
    fn numbers() {
        let input = "12 12. 12.34 .34";

        assert_eq!(
            tokenize(input),
            Ok(vec![
                tp((1,  1), (1,  2), T::Number(Number::Int("12".to_owned()))),
                tp((1,  3), (1,  3), T::Whitespace(WS::Inline(" ".to_owned()))),
                tp((1,  4), (1,  6), T::Number(Number::Float("12.".to_owned()))),
                tp((1,  7), (1,  7), T::Whitespace(WS::Inline(" ".to_owned()))),
                tp((1,  8), (1, 12), T::Number(Number::Float("12.34".to_owned()))),
                tp((1, 13), (1, 13), T::Whitespace(WS::Inline(" ".to_owned()))),
                tp((1, 14), (1, 16), T::Number(Number::Float(".34".to_owned()))),
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
                tp((1,  1), (1,  4), T::Identifier("some".to_owned())),
                tp((1,  5), (1,  5), T::Whitespace(WS::Inline(" ".to_owned()))),
                tp((1,  6), (1, 24), T::QuotedIdentifier("quoted identifier".to_owned())),
                tp((1, 25), (1, 25), T::Whitespace(WS::Inline(" ".to_owned()))),
                tp((1, 26), (1, 26), T::Whitespace(WS::Newline)),
                tp((2,  1), (2,  1), T::Whitespace(WS::Inline(" ".to_owned()))),
                tp((2,  2), (2, 15), T::QuotedIdentifier("and here too".to_owned())),
            ])
        );
    }

    #[test]
    fn text() {
        assert!(false);
    }

    #[test]
    fn symbols() {
        // This doesn't test periods, single quotes, or double quotes because they
        // are special cases that are part of other tokens
        let input = "@_";

        assert_eq!(
            tokenize(input),
            Ok(vec![
                tp((1, 1), (1, 1), T::Symbol(Symbol::AtSign)),
                tp((1, 1), (1, 1), T::Symbol(Symbol::Underscore)),
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
      birthdate  '1900-01-01'
      likes_pizza true

  pet
    p1
      name     'pet 1'
      person_id p@p1.id
      species  'cat'

    _
      name     'pet 2'
      person_id p@p2.id
      species   @p1.species

  things
    _
      cost  123.456
      units 5

  \"quoted identifier table name\"";

        Tokenizer::new().tokenize(input);
    }
}
