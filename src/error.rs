use postgres;
use std::error::Error;
use std::fmt;
use std::io;

use crate::{analyzer, lexer, loader, parser};

#[derive(Debug)]
pub enum HldrErrorKind {
    IoError,
    LexError,
    ParseError,
    ValidateError,
    ClientError,
    LoadError,
    GeneralDatabaseError,
}

#[derive(Debug)]
pub struct HldrError {
    pub kind: HldrErrorKind,
    pub error: Box<dyn Error>,
}

impl From<io::Error> for HldrError {
    fn from(error: io::Error) -> Self {
        HldrError {
            kind: HldrErrorKind::IoError,
            error: Box::new(error),
        }
    }
}

impl From<postgres::error::Error> for HldrError {
    fn from(error: postgres::error::Error) -> Self {
        HldrError {
            kind: HldrErrorKind::GeneralDatabaseError,
            error: Box::new(error),
        }
    }
}

impl From<lexer::error::LexError> for HldrError {
    fn from(error: lexer::error::LexError) -> Self {
        HldrError {
            kind: HldrErrorKind::LexError,
            error: Box::new(error),
        }
    }
}

impl From<parser::error::ParseError> for HldrError {
    fn from(error: parser::error::ParseError) -> Self {
        HldrError {
            kind: HldrErrorKind::ParseError,
            error: Box::new(error),
        }
    }
}

impl From<analyzer::error::AnalyzeError> for HldrError {
    fn from(error: analyzer::error::AnalyzeError) -> Self {
        HldrError {
            kind: HldrErrorKind::ValidateError,
            error: Box::new(error),
        }
    }
}

impl From<loader::error::ClientError> for HldrError {
    fn from(error: loader::error::ClientError) -> Self {
        HldrError {
            kind: HldrErrorKind::ClientError,
            error: Box::new(error),
        }
    }
}

impl From<loader::error::LoadError> for HldrError {
    fn from(error: loader::error::LoadError) -> Self {
        HldrError {
            kind: HldrErrorKind::LoadError,
            error: Box::new(error),
        }
    }
}

impl Error for HldrError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.error.source()
    }
}

impl fmt::Display for HldrError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.error.fmt(f)
    }
}
