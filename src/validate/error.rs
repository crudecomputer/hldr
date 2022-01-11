use std::{error::Error, fmt};

#[derive(Debug, PartialEq)]
pub enum ValidateErrorKind {
    DuplicateRecordName(String),
    DuplicateColumn {
        record: Option<String>,
        column: String,
    },
}

#[derive(Debug, PartialEq)]
pub struct ValidateError {
    pub kind: ValidateErrorKind,
    pub schema: String,
    pub table: String,
}

impl Error for ValidateError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

impl fmt::Display for ValidateError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use ValidateErrorKind::*;

        match &self.kind {
            DuplicateRecordName(name) => write!(
                f,
                "Duplicate record '{}' in '{}.{}'",
                name, self.schema, self.table,
            ),
            DuplicateColumn { record, column } => write!(
                f,
                "Duplicate column '{}' for record '{}' in '{}.{}'",
                column,
                record
                    .as_ref()
                    .map(|name| format!("record '{}'", name))
                    .unwrap_or_else(|| "anonymous record".to_owned()),
                self.schema,
                self.table,
            ),
        }
    }
}
