use std::{error::Error, fmt};

use postgres::error::Error as PostgresError;

use crate::{lex, parse, validate, load};

//use 

#[derive(Debug)]
pub enum HldrErrorKind {
    LexError,
    ParseError,
    ValidateError,
    ClientError,
    LoadError,
    GeneralDatabaseError,
}

#[derive(Debug)]
pub struct HldrError {
    kind: HldrErrorKind,
    error: Box<dyn Error>,
}

impl From<lex::LexError> for HldrError {
    fn from(error: lex::LexError) -> Self {
        HldrError {
            kind: HldrErrorKind::LexError,
            error: Box::new(error),
        }
    }
}

impl From<parse::ParseError> for HldrError {
    fn from(error: parse::ParseError) -> Self {
        HldrError {
            kind: HldrErrorKind::ParseError,
            error: Box::new(error),
        }
    }
}

impl From<validate::ValidateError> for HldrError {
    fn from(error: validate::ValidateError) -> Self {
        HldrError {
            kind: HldrErrorKind::ValidateError,
            error: Box::new(error),
        }
    }
}

impl From<load::ClientError> for HldrError {
    fn from(error: load::ClientError) -> Self {
        HldrError {
            kind: HldrErrorKind::ClientError,
            error: Box::new(error),
        }
    }
}

impl From<load::LoadError> for HldrError {
    fn from(error: load::LoadError) -> Self {
        HldrError {
            kind: HldrErrorKind::LoadError,
            error: Box::new(error),
        }
    }
}

impl From<PostgresError> for HldrError {
    fn from(error: PostgresError) -> Self {
        HldrError {
            kind: HldrErrorKind::GeneralDatabaseError,
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
