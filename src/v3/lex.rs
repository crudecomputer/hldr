const NULL: char = '\0';

enum Token {
    Newline,
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

/// A state in the lexer's state machine.
type ReceiveResult = Result<Box<dyn State>, LexError>;

trait State {
    fn receive(self, c: char) -> ReceiveResult;
}

/// State after receiving double-dashes
struct InComment;

impl State for InComment {
    fn receive(self, c: char) -> ReceiveResult {
        match c {
            // TODO: Treat \r\n as single newline
            // See http://www.unicode.org/reports/tr14/tr14-32.html#BreakingRules
            _ if ['\r', '\n'].contains(&c) => {
                let token = Token::Newline;
                Ok(Box::new(Start))
            }
            _ => Ok(Box::new(InComment)),
        }
    }
}

/// State after receiving a single dash
struct SingleDash;

impl State for SingleDash {
    fn receive(self, c: char) -> ReceiveResult {
        match c {
            '-' => Ok(Box::new(InComment)),
            _ => Err(LexError::unexpected(c)),
        }
    }
}

/// State corresponding to the start of input or after finishing a token.
struct Start;

impl State for Start {
    fn receive(self, c: char) -> ReceiveResult {
        match c {
            NULL => {
                Ok(Box::new(self))
            }
            '-' => {
                // self.end_position.column += 1;
                Ok(Box::new(SingleDash))
            }
            _ => Err(LexError::unexpected(c)),
        }
    }
}

struct Machine {
    tokens: Vec<Token>,
}

impl Machine {
    fn new() -> Self {
        Self { tokens: Vec::new() }
    }

    fn tokenize(&mut self, )
}
