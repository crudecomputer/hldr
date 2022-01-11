use std::{error::Error, fmt};

use postgres::error::Error as PostgresError;

#[derive(Debug)]
pub enum ClientErrorKind {
    Config,
    Connection,
}

#[derive(Debug)]
pub struct ClientError {
    error: PostgresError,
    kind: ClientErrorKind,
}

impl ClientError {
    pub fn config_error(error: PostgresError) -> Self {
        Self { kind: ClientErrorKind::Config, error }
    }

    pub fn connection_error(error: PostgresError) -> Self {
        Self { kind: ClientErrorKind::Connection, error }
    }
}

impl Error for ClientError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.error)
    }
}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use ClientErrorKind::*;

        match self.kind {
            Config => write!(f, "Config error: {}", self.error),
            Connection => write!(f, "Connection error: {}", self.error),
        }
    }
}

#[derive(Debug)]
pub struct LoadError(PostgresError);

impl LoadError {
    pub fn new(e: PostgresError) -> Self {
        Self(e)
    }
}

impl Error for LoadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.0)
    }
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}
