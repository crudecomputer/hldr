use super::errors::*;
use super::tokens::*;

pub const NULL: char = '\0';
pub const EOF: char = NULL;

type ReceiveResult = Result<Box<dyn State>, LexError>;

/// The context accessible for any given state
#[derive(Default)]
pub struct Context {
    stack: Vec<char>,
    tokens: Vec<Token>,
}

impl Context {
    /// Consumes the Context and returns the collected tokens.
    pub fn into_tokens(self) -> Vec<Token> {
        self.tokens
    }

    /// Drains the stack and returns the contents as a String.
    fn drain_stack(&mut self) -> String {
        self.stack.drain(..).collect()
    }
}

/// A state in the lexer's state machine.
pub trait State {
    /// Receives a character and returns the next state.
    fn receive(&self, ctx: &mut Context, c: char) -> ReceiveResult;

    /// Returns whether or not the given character can successfully terminate the current state.
    fn can_terminate(&self, _c: char) -> bool {
        true
    }
}

/// State corresponding to the start of input or after successfully extracting a token.
pub struct Start;

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
                Ok(Box::new(AfterCarriageReturn))
            }
            '-' => {
                Ok(Box::new(AfterSingleDash))
            }
            '0'..='9' | '.' => {
                // Allow InInteger to add decimal point to the stack and transition to InFloat
                InInteger.receive(ctx, c)
            }
            _ if is_identifier_char(c) => {
                InIdentifier.receive(ctx, c)
            }
            _ if is_whitespace(c) => {
                InWhitespace.receive(ctx, c)
            }
            _ => Err(LexError::unexpected(c)),
        }
    }
}

/// State after receiving a carriage return.
struct AfterCarriageReturn;

impl State for AfterCarriageReturn {
    fn receive(&self, ctx: &mut Context, c: char) -> ReceiveResult {
        ctx.tokens.push(Token::Newline);
        
        match c {
            '\n' => Ok(Box::new(Start)),
            _ => Start.receive(ctx, c),
        }
    }
}

/// State after receiving a single dash
struct AfterSingleDash;

impl State for AfterSingleDash {
    fn receive(&self, ctx: &mut Context, c: char) -> ReceiveResult {
        match c {
            '-' => Ok(Box::new(InComment)),
            '0'..='9' | '.' => {
                ctx.stack.push('-');
                InInteger.receive(ctx, c)
            }
            _ => Err(LexError::unexpected(c)),
        }
    }
}

/// State after receiving double-dashes.
struct InComment;

impl State for InComment {
    fn receive(&self, ctx: &mut Context, c: char) -> ReceiveResult {
        match c {
            '\n' => {
                ctx.tokens.push(Token::Newline);
                Ok(Box::new(Start))
            }
            '\r' => {
                Ok(Box::new(AfterCarriageReturn))
            }
            _ => Ok(Box::new(InComment)),
        }
    }
}

/// State after receiving a decimal point or a digit after having previously received a decimal point.
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

    fn can_terminate(&self, _c: char) -> bool {
        true // TODO: Only whitespace, newlines, and null byte?
    }
}

/// State after receiving a valid identifier character.
struct InIdentifier;

impl State for InIdentifier {
    fn receive(&self, ctx: &mut Context, c: char) -> ReceiveResult {
        match c {
            _ if is_identifier_char(c) => {
                ctx.stack.push(c);
                Ok(Box::new(InIdentifier))
            }
            _ if self.can_terminate(c) => {
                let stack = ctx.drain_stack();
                let token = identifier_to_token(stack);
                ctx.tokens.push(token);
                Start.receive(ctx, c)
            }
            _ => Err(LexError::unexpected(c)),
        }
    }

    fn can_terminate(&self, _c: char) -> bool {
        true // TODO: Only whitespace, newlines, and null byte?
    }
}

/// State after receiving a digit without having previously received a decimal point.
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

    fn can_terminate(&self, _c: char) -> bool {
        true // TODO: Only whitespace, newlines, and null byte?
    }
}

/// State after receiving whitespace
struct InWhitespace;

impl State for InWhitespace {
    fn receive(&self, ctx: &mut Context, c: char) -> ReceiveResult {
        match c {
            c if is_whitespace(c) => {
                ctx.stack.push(c);
                Ok(Box::new(InWhitespace))
            }
            _ => {
                let stack = ctx.drain_stack();
                ctx.tokens.push(Token::Whitespace(stack));
                Start.receive(ctx, c)
            }
        }
    }
}

fn identifier_to_token(s: String) -> Token {
    match s.as_ref() {
        "_" => Token::Underscore,
        "true" | "t" => Token::Bool(true),
        "false" | "f" => Token::Bool(false),
        "as" => Token::Keyword(Keyword::As),
        _ => Token::Identifier(s),
    }
}

fn is_identifier_char(c: char) -> bool {
    (
        c == '_' || c.is_alphabetic()
    ) || (
        // `char.is_alphabetic` isn't enough because that precludes other unicode chars
        // that are valid in postgres identifiers, eg:
        //     create table love (💝 text);
        //     > CREATE TABLE 
        //
        // There is, however, a very strong chance the below conditions are not fully accurate.
        !c.is_control() && !c.is_whitespace() && !c.is_ascii_punctuation()
    )
}

fn is_whitespace(c: char) -> bool {
    c == ' ' || c == '\t'
}