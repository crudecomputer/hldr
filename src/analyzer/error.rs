use std::error::Error;
use std::fmt;
use crate::Position;

#[derive(Clone, Debug, PartialEq)]
pub enum AnalyzeErrorKind {
    AmbiguousRecord { record: String },
    ColumnNotFound { column: String },
    DuplicateColumn { column: String },
    DuplicateRecord { record: String },
    RecordNotFound { record: String },
}

impl fmt::Display for AnalyzeErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use AnalyzeErrorKind::*;

        match self {
            AmbiguousRecord { record } => {
                write!(f, "ambiguous record name `{}`", record)
            }
            ColumnNotFound { column } => {
                write!(f, "referenced column `{}` not found", column)
            }
            DuplicateColumn { column } => {
                write!(f, "duplicate column name `{}`", column)
            }
            DuplicateRecord { record } => {
                write!(f, "duplicate record name `{}`", record)
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
    pub position: Position,
}

impl fmt::Display for AnalyzeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} ({})", self.kind, self.position)
    }
}

impl Error for AnalyzeError {}
