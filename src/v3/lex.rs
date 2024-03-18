use std::iter::Peekable;

const NULL: char = '\0';
const EOF: char = NULL;

#[derive(Debug, PartialEq)]
pub enum Keyword {
    As,
}

#[derive(Debug, PartialEq)]
enum Token {
    // Bool(bool),
    // Keyword(Keyword),
    Newline,
    Number(String),
    Whitespace(String),
}

#[derive(Clone, Debug, PartialEq)]
pub enum LexErrorKind {
    // ExpectedComment,
    // ExpectedNumber,
    // UnclosedQuotedIdentifier,
    // UnclosedString,
    UnexpectedCharacter(char),
}

#[derive(Clone, Debug, PartialEq)]
pub struct LexError {
    pub kind: LexErrorKind,
    // pub position: Position,
}

impl LexError {
    fn unexpected(c: char /*, position: Position */) -> Self {
        Self { kind: LexErrorKind::UnexpectedCharacter(c)}
    }
}

/// The context accessible for any given state
#[derive(Default)]
struct Context {
    stack: Vec<char>,
    tokens: Vec<Token>,
}

impl Context {
    fn drain_stack(&mut self) -> String {
        self.stack.drain(..).collect()
    }
}

/// A state in the lexer's state machine.
type ReceiveResult = Result<Box<dyn State>, LexError>;

trait State {
    fn receive(&self, ctx: &mut Context, c: char) -> ReceiveResult;

    fn can_terminate(&self, c: char) -> bool {
        true
    }
}

/// State after receiving double-dashes
struct Comment;

impl State for Comment {
    fn receive(&self, ctx: &mut Context, c: char) -> ReceiveResult {
        match c {
            '\n' => {
                ctx.tokens.push(Token::Newline);
                Ok(Box::new(Start))
            }
            '\r' => {
                Ok(Box::new(CarriageReturn))
            }
            _ => Ok(Box::new(Comment)),
        }
    }
}

/// State after receiving a number
struct InInteger;

impl State for InInteger {
    fn receive(&self, ctx: &mut Context, c: char) -> ReceiveResult {
        // TODO: Better error kind indicating invalid numeric literal
        match c {
            '0'..='9' => {
                ctx.stack.push(c);
                Ok(Box::new(InInteger))
            }
            // Underscores cannot be consecutive and decimal points cannot follow underscores
            '_' | '.' if ctx.stack.last() == Some(&'_') => {
                Err(LexError::unexpected(c))
            }
            '_' => {
                ctx.stack.push(c);
                Ok(Box::new(InInteger))
            }
            '.' => {
                ctx.stack.push(c);
                Ok(Box::new(InFloat))
            }
            _ if self.can_terminate(c) => match ctx.stack.last() {
                Some(&'_') => Err(LexError::unexpected('_')),
                _ => {
                    let stack = ctx.drain_stack();
                    ctx.tokens.push(Token::Number(stack));
                    Start.receive(ctx, c)
                }
            }
            _ => Err(LexError::unexpected(c)),
        }
    }

    fn can_terminate(&self, c: char) -> bool {
        true // TODO: Only whitespace, newlines, and null byte?
    }
}

/// State after receiving a decimal point
struct InFloat;

impl State for InFloat {
    fn receive(&self, ctx: &mut Context, c: char) -> ReceiveResult {
        match c {
            '0'..='9' => {
                ctx.stack.push(c);
                Ok(Box::new(InFloat))
            }
            // Entering into InFloat means there is already a decimal point in the stack
            '.' => {
                Err(LexError::unexpected(c))
            }
            // Underscores can neither be consecutive nor follow a decimal point
            '_' if [Some(&'.'), Some(&'_')].contains(&ctx.stack.last()) => {
                Err(LexError::unexpected(c))
            }
            '_' => {
                ctx.stack.push(c);
                Ok(Box::new(InFloat))
            }
            _ if self.can_terminate(c) => match ctx.stack.last() {
                Some(&'_') => Err(LexError::unexpected('_')),
                _ => {
                    let stack = ctx.drain_stack();
                    ctx.tokens.push(Token::Number(stack));
                    Start.receive(ctx, c)
                }
            }
            _ => Err(LexError::unexpected(c)),
        }
    }

    fn can_terminate(&self, c: char) -> bool {
        true // TODO: Only whitespace or newlines?
    }
}

/// State after receiving whitespace
struct Whitespace;

impl State for Whitespace {
    fn receive(&self, ctx: &mut Context, c: char) -> ReceiveResult {
        match c {
            c if is_whitespace(c) => {
                ctx.stack.push(c);
                Ok(Box::new(Whitespace))
            }
            _ => {
                let stack = ctx.drain_stack();
                ctx.tokens.push(Token::Whitespace(stack));
                Start.receive(ctx, c)
            }
        }
    }
}

/// State after receiving a single dash
struct SingleDash;

impl State for SingleDash {
    fn receive(&self, ctx: &mut Context, c: char) -> ReceiveResult {
        match c {
            '-' => Ok(Box::new(Comment)),
            '0'..='9' | '.' => {
                ctx.stack.push('-');
                InInteger.receive(ctx, c)
            }
            _ => Err(LexError::unexpected(c)),
        }
    }
}

/// State after receiving a carriage return.
struct CarriageReturn;

impl State for CarriageReturn {
    fn receive(&self, ctx: &mut Context, c: char) -> ReceiveResult {
        ctx.tokens.push(Token::Newline);
        
        match c {
            '\n' => Ok(Box::new(Start)),
            _ => Start.receive(ctx, c),
        }
    }
}

/// State corresponding to the start of input or after finishing a token.
struct Start;

impl State for Start {
    fn receive(&self, ctx: &mut Context, c: char) -> ReceiveResult {
        match c {
            NULL => {
                Ok(Box::new(Start))
            }
            '\n' => {
                ctx.tokens.push(Token::Newline);
                Ok(Box::new(Start))
            }
            '\r' => {
                Ok(Box::new(CarriageReturn))
            }
            '-' => {
                Ok(Box::new(SingleDash))
            }
            '0'..='9' | '.' => {
                // Allow InInteger to add decimal point to the stack and transition to InFloat
                InInteger.receive(ctx, c)
            }
            _ if is_whitespace(c) => {
                Whitespace.receive(ctx, c)
            }
            _ => Err(LexError::unexpected(c)),
        }
    }
}

fn is_whitespace(c: char) -> bool {
    c == ' ' || c == '\t'
}

fn tokenize(input: impl Iterator<Item = char>) -> Result<Vec<Token>, LexError> {
    let mut context = Context::default();
    let mut state: Box<dyn State> = Box::new(Start);

    for c in input {
        state = state.receive(&mut context, c)?;
    }

    state.receive(&mut context, EOF)?;
    Ok(context.tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

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

    // #[test]
    // fn test_keywords() {
    //     let input = "as";
    //     assert_eq!(tokenize(input.chars()), Ok(vec![
    //         Token::Keyword(Keyword::As),
    //     ]));
    // }

    // #[test]
    // fn test_bools() {
    //     let input = "true t false f";
    //     assert_eq!(tokenize(input.chars()), Ok(vec![
    //         Token::Bool(true),
    //         Token::Newline,
    //         Token::Bool(true),
    //         Token::Bool(false),
    //         Token::Bool(false),
    //     ]));
    // }

    #[test]
    fn test_numbers() {
        for num in [
            "0", "0.", ".0",
            "123", "-456", "12.34", "-45.67",
            "1.", ".2", "-3.", "-.4",
            "1_2", "1_2_3", "1_2.3_4", "1_2.3_4_5",
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
        for input in ["_123", "456_", "12__34", "12._34", "12.34_"] {
            assert_eq!(tokenize(input.chars()), Err(LexError::unexpected('_')));
        }
    }
}