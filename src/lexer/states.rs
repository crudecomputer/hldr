use crate::Position;
use super::error::LexError;
use super::tokens::{Keyword, Symbol, Token, TokenKind};

type ReceiveResult = Result<Box<dyn State>, LexError>;

fn to<S: State + 'static>(state: S) -> ReceiveResult {
    Ok(Box::new(state))
}

fn defer_to<S: State + 'static>(state: S, ctx: &mut Context, c: Option<char>) -> ReceiveResult {
    state.receive(ctx, c)
}

/// The context accessible for any given state
#[derive(Debug)]
pub struct Context {
    current_position: Position,
    token_start_position: Position,
    stack: Vec<char>,
    tokens: Vec<Token>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            current_position: Position { line: 1, column: 1 },
            token_start_position: Position { line: 1, column: 1 },
            stack: Vec::new(),
            tokens: Vec::new(),
        }
    }

    /// Consumes the Context and returns the collected tokens.
    pub fn into_tokens(self) -> Vec<Token> {
        self.tokens
    }

    /// Drains the stack and returns the contents as a String.
    fn drain_stack(&mut self) -> String {
        self.stack.drain(..).collect()
    }

    fn add_token(&mut self, kind: TokenKind) {
        self.tokens.push(Token { kind, position: self.token_start_position });
    }

    pub fn increment_position(&mut self, c: char) {
        if ['\r', '\n'].contains(&c) {
            self.current_position.line += 1;
            self.current_position.column = 1;
        } else {
            self.current_position.column += 1;
        }

        if self.stack.is_empty() {
            self.reset_start();
        }
    }

    pub fn reset_start(&mut self) {
        self.token_start_position = self.current_position;
    }
}

/// A state in the lexer's state machine.
pub trait State {
    /// Receives a character (or `None` when EOF) and returns the next state.
    fn receive(&self, ctx: &mut Context, c: Option<char>) -> ReceiveResult;

    /// Returns whether or not the given character can successfully terminate the current state,
    /// defaulting to only allowing whitespace, newlines, or EOF to terminate.
    fn can_terminate(&self, c: Option<char>) -> bool {
        c.is_none() || matches!(c, Some(c) if is_whitespace(c) || is_newline(c))
    }
}

/// State corresponding to the start of input or after successfully extracting a token.
pub struct Start;

impl State for Start {
    fn receive(&self, ctx: &mut Context, c: Option<char>) -> ReceiveResult {
        let c = match c {
            Some(c) => c,
            None => return to(Start),
        };

        match c {
            '\r' | '\n' => {
                ctx.add_token(TokenKind::LineSep);
                to(Start)
            },
            '(' => {
                ctx.add_token(TokenKind::Symbol(Symbol::ParenLeft));
                to(Start)
            }
            ')' => {
                ctx.add_token(TokenKind::Symbol(Symbol::ParenRight));
                to(Start)
            }
            '@' => {
                ctx.add_token(TokenKind::Symbol(Symbol::AtSign));
                to(Start)
            }
            ',' => {
                ctx.add_token(TokenKind::Symbol(Symbol::Comma));
                to(Start)
            }
            '.' => {
                to(AfterPeriod)
            }
            '-' => {
                to(AfterSingleDash)
            }
            '\'' => {
                to(InText)
            }
            '"' => {
                to(InQuotedIdentifier)
            }
            '0'..='9' => {
                defer_to(InInteger, ctx, Some(c))
            }
            _ if is_identifier_char(c) => {
                defer_to(InIdentifier, ctx, Some(c))
            }
            _ if is_whitespace(c) => {
                to(Start)
            }
            _ => Err(LexError::bad_char(c, ctx.current_position)),
        }
    }
}

/// State after receiving a period without preceding digits.
struct AfterPeriod;

impl State for AfterPeriod {
    fn receive(&self, ctx: &mut Context, c: Option<char>) -> ReceiveResult {
        match c {
            Some('0'..='9') => {
                ctx.stack.push('.');
                defer_to(InFloat, ctx, c)
            }
            None | Some(_) if self.can_terminate(c) => {
                ctx.add_token(TokenKind::Symbol(Symbol::Period));
                defer_to(Start, ctx, c)
            }
            Some(c) => Err(LexError::bad_char(c, ctx.current_position)),
            _ => unreachable!(),
        }
    }

    fn can_terminate(&self, _c: Option<char>) -> bool {
        // TODO: This was STILL making expectations.
        // Outside of float tokens (which this state does not generate)
        // periods are only used in references, meaning they should only
        // be followed by a plain or quoted identifier, but how much
        // should be forbidden during tokenization? And should references
        // be allowed to have whitespace between the period and the identifier?
        //
        // is_identifier_char(c) || c == '"'
        true
    }
}

/// State after receiving a single dash
struct AfterSingleDash;

impl State for AfterSingleDash {
    fn receive(&self, ctx: &mut Context, c: Option<char>) -> ReceiveResult {
        match c {
            Some('-') => to(InComment),
            Some('0'..='9' | '.') => {
                ctx.stack.push('-');
                defer_to(InInteger, ctx, c)
            }
            Some(c) => Err(LexError::bad_char(c, ctx.current_position)),
            None => Err(LexError::eof(ctx.current_position)),
        }
    }
}

/// State after receiving what might be a closing double-quote unless the next
/// character received is another double-quote, which indicates the previous
/// quote was being escaped and is part of the quoted identifier.
struct AfterQuotedIdentifier;

impl State for AfterQuotedIdentifier {
    fn receive(&self, ctx: &mut Context, c: Option<char>) -> ReceiveResult {
        match c {
            Some(c @ '"') => {
                ctx.stack.push(c);
                to(InQuotedIdentifier)
            }
            _ => {
                let stack = ctx.drain_stack();
                ctx.add_token(TokenKind::QuotedIdentifier(stack));
                defer_to(Start, ctx, c)
            }
        }
    }
}

/// State after receiving what might be a closing quote unless the next
/// character received is another single quote, which indicates the previous
/// quote was being escaped and is part of the text string.
struct AfterText;

impl State for AfterText {
    fn receive(&self, ctx: &mut Context, c: Option<char>) -> ReceiveResult {
        match c {
            Some(c @ '\'') => {
                ctx.stack.push(c);
                ctx.stack.push(c);
                to(InText)
            }
            _ => {
                let stack = ctx.drain_stack();
                ctx.add_token(TokenKind::Text(stack));
                defer_to(Start, ctx, c)
            }
        }
    }
}

/// State after receiving double-dashes.
struct InComment;

impl State for InComment {
    fn receive(&self, ctx: &mut Context, c: Option<char>) -> ReceiveResult {
        match c {
            Some(c) if is_newline(c) => {
                ctx.add_token(TokenKind::LineSep);
                to(Start)
            }
            _ => to(InComment),
        }
    }
}

/// State after receiving a decimal point or a digit after having previously received a decimal point.
struct InFloat;

impl State for InFloat {
    fn receive(&self, ctx: &mut Context, c: Option<char>) -> ReceiveResult {
        match c {
            Some(c @ '0'..='9') => {
                ctx.stack.push(c);
                to(InFloat)
            }
            // Entering into InFloat means there is already a decimal point in the stack
            Some('.') => {
                let stack = ctx.drain_stack();
                Err(LexError::bad_number(stack, ctx.token_start_position))
            }
            // Underscores can neither be consecutive nor follow a decimal point
            Some('_') if [Some(&'.'), Some(&'_')].contains(&ctx.stack.last()) => {
                let stack = ctx.drain_stack();
                Err(LexError::bad_number(stack, ctx.current_position))
            }
            Some(c @ '_') => {
                ctx.stack.push(c);
                to(InFloat)
            }
            None | Some(_) if self.can_terminate(c) => {
                let stack = ctx.drain_stack();
                match ctx.stack.last() {
                    Some(&'_') => {
                        Err(LexError::bad_number(stack, ctx.token_start_position))
                    }
                    _ => {
                        ctx.add_token(TokenKind::Number(stack));
                        defer_to(Start, ctx, c)
                    }
                }
            }
            Some(c) => Err(LexError::bad_char(c, ctx.current_position)),
            _ => unreachable!(),
        }
    }
}

/// State after receiving a valid identifier character.
struct InIdentifier;

impl State for InIdentifier {
    fn receive(&self, ctx: &mut Context, c: Option<char>) -> ReceiveResult {
        // TODO: Should this be more restrictive about what can terminate an identifier?
        // This does not exclude input like `one@two` or `one'two'` but should those
        // specifically be forbidden? How much should lexer do?
        match c {
            Some(c) if is_identifier_char(c) => {
                ctx.stack.push(c);
                to(InIdentifier)
            }
            _ => {
                let stack = ctx.drain_stack();
                let token = identifier_to_token(stack);
                ctx.add_token(token);
                defer_to(Start, ctx, c)
            }
        }
    }
}

/// State after receiving a digit without having previously received a decimal point.
struct InInteger;

impl State for InInteger {
    fn receive(&self, ctx: &mut Context, c: Option<char>) -> ReceiveResult {
        match c {
            Some(c @ '0'..='9') => {
                ctx.stack.push(c);
                to(InInteger)
            }
            // Underscores cannot be consecutive and decimal points cannot follow underscores
            Some('_' | '.') if ctx.stack.last() == Some(&'_') => {
                let stack = ctx.drain_stack();
                Err(LexError::bad_number(stack, ctx.token_start_position))
            }
            Some(c @ '_') => {
                ctx.stack.push(c);
                to(InInteger)
            }
            Some(c @ '.') => {
                ctx.stack.push(c);
                to(InFloat)
            }
            None | Some(_) if self.can_terminate(c) => {
                let stack = ctx.drain_stack();
                match ctx.stack.last() {
                    Some(&'_') => {
                        Err(LexError::bad_number(stack, ctx.token_start_position))
                    }
                    _ => {
                        ctx.add_token(TokenKind::Number(stack));
                        defer_to(Start, ctx, c)
                    }
                }
            }
            Some(c) => Err(LexError::bad_char(c, ctx.current_position)),
            _ => unreachable!()
        }
    }
}

/// State after receiving a valid identifier character.
struct InQuotedIdentifier;

impl State for InQuotedIdentifier {
    fn receive(&self, ctx: &mut Context, c: Option<char>) -> ReceiveResult {
        match c {
            Some('"') => to(AfterQuotedIdentifier),
            Some(c) => {
                ctx.stack.push(c);
                to(InQuotedIdentifier)
            }
            None => Err(LexError::eof_unquoted(ctx.current_position)),
        }
    }
}

/// State after receiving a single quote and inside a string literal.
struct InText;

impl State for InText {
    fn receive(&self, ctx: &mut Context, c: Option<char>) -> ReceiveResult {
        match c {
            Some('\'') => to(AfterText),
            Some(c) => {
                ctx.stack.push(c);
                to(InText)
            }
            None => Err(LexError::eof_string(ctx.current_position)),
        }
    }
}

fn identifier_to_token(s: String) -> TokenKind {
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

fn is_newline(c: char) -> bool {
    ['\r', '\n'].contains(&c)
}
