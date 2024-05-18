use std::error::Error;
use std::fmt;
// use crate::Position;

#[derive(Clone, Debug, PartialEq)]
pub enum AnalyzeErrorKind {
    AmbiguousRecord { record: String },
    ColumnNotFound { column: String },
    DuplicateColumn { scope: String, column: String },
    DuplicateRecord { scope: String, record: String },
    RecordNotFound { record: String },
}

impl fmt::Display for AnalyzeErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use AnalyzeErrorKind::*;

        match self {
            AmbiguousRecord { record } => {
                write!(f, "ambiguous record `{}`", record)
            }
            ColumnNotFound { column } => {
                write!(f, "referenced column `{}` not found", column)
            }
            DuplicateColumn { scope, column } => {
                // TODO: Need position
                write!(f, "duplicate column `{}` in scope `{}`", column, scope)
            }
            DuplicateRecord { scope, record } => {
                write!(f, "duplicate record `{}` in scope `{}`", record, scope)
            }
            RecordNotFound { record } => {
                write!(f, "record `{}` not found", record)
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct AnalyzeError {
    pub kind: AnalyzeErrorKind,
    // pub position: Position,
}

impl fmt::Display for AnalyzeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.kind)
    }
}

impl Error for AnalyzeError {}
