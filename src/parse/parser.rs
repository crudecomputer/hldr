use super::{error::ParseError, Attribute, Record, Schema, Table, Token, Value, ReferenceValue};

#[derive(Debug, PartialEq)]
pub enum State {
    LineStart,
    CreatedSchema,
    CreatedTable,
    CreatedRecord,
    CreatedAttribute,
    ExpectingTable,
    ExpectingRecord,
    ExpectingColumn,
    ExpectingValue(String),
    IdentifierExpectingReferenceValue {
        column: String,
        identifier: String,
    },
    SchemaQualifiedReferenceValueExpectingTable {
        column: String,
        schema: String,
    },
    SchemaQualifiedReferenceValueExpectingAtSign {
        column: String,
        schema: String,
        table: String,
    },
    SchemaQualifiedReferenceValueExpectingRecord {
        column: String,
        schema: String,
        table: String,
    },
    SchemaQualifiedReferenceValueExpectingRecordPeriod {
        column: String,
        schema: String,
        table: String,
        record: String,
    },
    SchemaQualifiedReferenceValueExpectingColumn {
        column: String,
        schema: String,
        table: String,
        record: String,
    },
}

#[derive(Debug)]
pub(super) struct Parser {
    indent_unit: Option<String>,
    state: State,
    pub schemas: Vec<Schema>,
}

impl Parser {
    pub fn new() -> Self {
        Self {
            indent_unit: None,
            schemas: Vec::new(),
            state: State::LineStart,
        }
    }

    pub fn parse(mut self, tokens: Vec<Token>) -> Result<Self, ParseError> {
        use State::*;

        let mut line = 1;

        for token in tokens {
            self.state = match self.state {
                LineStart => match token {
                    Token::Newline => {
                        line += 1;
                        LineStart
                    }
                    Token::Indent(indent) => {
                        if indent.is_empty() {
                            return Err(ParseError::empty_indent(line));
                        }

                        if !indent.trim().is_empty() {
                            return Err(ParseError::invalid_indent(line, indent));
                        }

                        let unit = match &self.indent_unit {
                            Some(u) => u,
                            None => {
                                self.indent_unit = Some(indent.clone());
                                self.indent_unit.as_ref().unwrap()
                            }
                        };

                        match indent_level(unit, &indent) {
                            Some(level) => match level {
                                1 => ExpectingTable,
                                2 => ExpectingRecord,
                                3 => ExpectingColumn,
                                n => return Err(ParseError::unexpected_indent_level(line, n)),
                            },
                            None => {
                                return Err(ParseError::inconsistent_indent(
                                    line,
                                    unit.clone(),
                                    indent,
                                ))
                            }
                        }
                    }
                    Token::Identifier(ident) | Token::QuotedIdentifier(ident) => {
                        self.schemas.push(Schema::new(ident));
                        CreatedSchema
                    }
                    _ => return Err(ParseError::unexpected_token(line, token)),
                },
                CreatedSchema | CreatedTable | CreatedRecord | CreatedAttribute => match token {
                    Token::Newline => {
                        line += 1;
                        LineStart
                    }
                    _ => return Err(ParseError::unexpected_token(line, token)),
                },
                ExpectingTable => match token {
                    Token::Newline => {
                        line += 1;
                        LineStart
                    }
                    Token::Identifier(ident) | Token::QuotedIdentifier(ident) => {
                        self.schemas
                            .last_mut()
                            .ok_or_else(|| ParseError::missing_schema(line))?
                            .tables
                            .push(Table::new(ident));

                        CreatedTable
                    }
                    _ => return Err(ParseError::unexpected_token(line, token)),
                },

                ExpectingRecord => match token {
                    Token::Newline => {
                        line += 1;
                        LineStart
                    }
                    Token::Identifier(_) | Token::Underscore => {
                        let name = match token {
                            Token::Identifier(ident) => Some(ident),
                            Token::Underscore => None,
                            _ => unreachable!(),
                        };

                        self.schemas
                            .last_mut()
                            // It shouldn't actually be possible to return this error here,
                            // since `ExpectingRecord` will only be reached if the indentation
                            // unit has already been set, meaning a line containing a table has
                            // to have already been successfully parsed, meaning a schema should
                            // have to be present.
                            .ok_or_else(|| ParseError::missing_schema(line))?
                            .tables
                            .last_mut()
                            .ok_or_else(|| ParseError::missing_table(line))?
                            .records
                            .push(Record::new(name));

                        CreatedRecord
                    }
                    _ => return Err(ParseError::unexpected_token(line, token)),
                },

                ExpectingColumn => match token {
                    Token::Newline => {
                        line += 1;
                        LineStart
                    }
                    Token::Identifier(ident) | Token::QuotedIdentifier(ident) => {
                        ExpectingValue(ident)
                    }
                    _ => return Err(ParseError::unexpected_token(line, token)),
                },

                ExpectingValue(column) => match token {
                    Token::Boolean(_) | Token::Number(_) | Token::Text(_) => {
                        let value = match token {
                            Token::Boolean(b) => Value::Boolean(b),
                            Token::Number(b) => Value::Number(b),
                            Token::Text(b) => Value::Text(b),
                            _ => unreachable!(),
                        };

                        self.schemas
                            .last_mut()
                            .ok_or_else(|| ParseError::missing_schema(line))? // Should never return error
                            .tables
                            .last_mut()
                            .ok_or_else(|| ParseError::missing_table(line))? // Should never return error
                            .records
                            .last_mut()
                            .ok_or_else(|| ParseError::missing_record(line))?
                            .attributes
                            .push(Attribute {
                                name: column,
                                value,
                            });

                        CreatedAttribute
                    }
                    Token::Identifier(i) | Token::QuotedIdentifier(i) => {
                        IdentifierExpectingReferenceValue {
                            column,
                            identifier: i,
                        }
                    }
                    _ => return Err(ParseError::missing_column_value(line)),
                },

                IdentifierExpectingReferenceValue { column, identifier } => match token {
                    Token::AtSign => {
                        let schema = self.schemas
                            .last_mut()
                            .ok_or_else(|| ParseError::missing_schema(line))? // Should never return error
                            .name.clone();

                        SchemaQualifiedReferenceValueExpectingRecord {
                            column,
                            schema,
                            table: identifier,

                        }
                    },
                    Token::Period => SchemaQualifiedReferenceValueExpectingTable {
                        column,
                        schema: identifier,
                    },
                    // Saying this identifier itself was unexpected (not obviously being intended
                    // as a reference) seems clearer than saying the newline is the problem
                    Token::Newline => return Err(ParseError::unexpected_token(line, Token::Identifier(identifier))),
                    _ => return Err(ParseError::unexpected_token(line, token)),
                },

                SchemaQualifiedReferenceValueExpectingTable { column, schema } => match token {
                    Token::Identifier(i) | Token::QuotedIdentifier(i) => {
                        SchemaQualifiedReferenceValueExpectingAtSign {
                            column,
                            schema,
                            table: i,
                        }
                    },
                    Token::Newline => return Err(ParseError::incomplete_reference(
                        line,
                        format!("{}.", schema),
                    )),
                    _ => return Err(ParseError::unexpected_token(line, token)),
                },

                SchemaQualifiedReferenceValueExpectingAtSign { column, schema, table } => match token {
                    Token::AtSign => {
                        SchemaQualifiedReferenceValueExpectingRecord {
                            column,
                            schema,
                            table,
                        }
                    },
                    Token::Newline => return Err(ParseError::incomplete_reference(
                        line,
                        format!("{}.{}", schema, table),
                    )),
                    _ => return Err(ParseError::unexpected_token(line, token)),
                },

                SchemaQualifiedReferenceValueExpectingRecord { column, schema, table } => match token {
                    Token::Identifier(i) | Token::QuotedIdentifier(i) => {
                        SchemaQualifiedReferenceValueExpectingRecordPeriod {
                            column,
                            schema,
                            table,
                            record: i,
                        }
                    },
                    Token::Newline => return Err(ParseError::incomplete_reference(
                        line,
                        format!("{}.{}@", schema, table),
                    )),
                    _ => return Err(ParseError::unexpected_token(line, token)),
                },

                SchemaQualifiedReferenceValueExpectingRecordPeriod { column, schema, table, record } => match token {
                    Token::Period => {
                        SchemaQualifiedReferenceValueExpectingColumn {
                            column,
                            schema,
                            table,
                            record,
                        }
                    },
                    Token::Newline => return Err(ParseError::incomplete_reference(
                        line,
                        format!("{}.{}@{}", schema, table, record),
                    )),
                    _ => return Err(ParseError::unexpected_token(line, token)),
                },

                SchemaQualifiedReferenceValueExpectingColumn { column, schema, table, record } => match token {
                    Token::Identifier(i) | Token::QuotedIdentifier(i) => {
                        let value = Value::Reference(Box::new(ReferenceValue {
                            schema,
                            table,
                            record,
                            column: i,
                        }));

                        self.schemas
                            .last_mut()
                            .ok_or_else(|| ParseError::missing_schema(line))? // Should never return error
                            .tables
                            .last_mut()
                            .ok_or_else(|| ParseError::missing_table(line))? // Should never return error
                            .records
                            .last_mut()
                            .ok_or_else(|| ParseError::missing_record(line))?
                            .attributes
                            .push(Attribute {
                                name: column,
                                value,
                            });

                        CreatedAttribute
                    },
                    Token::Newline => return Err(ParseError::incomplete_reference(
                        line,
                        format!("{}.{}@{}.", schema, table, record),
                    )),
                    _ => return Err(ParseError::unexpected_token(line, token)),
                },
            };

        }

        /*
        match &self.state {
            ExpectingValue(_) => {
                return Err(ParseError::missing_column_value(line));
            },
            IdentifierExpectingReferenceValue { identifier, .. } => {
                return Err(ParseError::unexpected_token(line, Token::Identifier(identifier.clone())));
            },
            _ => (),
        }
        */

        Ok(self)
    }
}

fn indent_level(unit: &str, indent: &str) -> Option<usize> {
    // A valid indent, when split by the indent unit, will yield
    // a collection of empty strings, with the collection length
    // being one longer than the "indent level".
    //
    // Eg: "    ".split("  ") => ["", "", ""]
    let parts: Vec<&str> = indent.split(unit).collect();

    for p in &parts {
        if !p.is_empty() {
            return None;
        }
    }

    Some(parts.len() - 1)
}

#[cfg(test)]
mod tests {

    mod indent_level {
        use super::super::indent_level;

        fn spaces(count: usize) -> String {
            " ".repeat(count)
        }

        fn tabs(count: usize) -> String {
            "\t".repeat(count)
        }

        #[test]
        fn valid_indents() {
            let assert_valid = |unit: &str, indent: &str, expected_level: usize| {
                assert_eq!(
                    indent_level(unit, indent),
                    Some(expected_level),
                    "unit: {:?}, indent: {:?}, level: {}",
                    unit,
                    indent,
                    expected_level,
                );
            };

            // Only explicitly care about expected indentation levels...
            //
            //   - schema name: 0
            //   - table name:  1
            //   - record name: 2
            //   - attribute:   3
            //
            // ... and indentation levels defined by...
            //
            //   - single space
            //   - double space
            //   - quadruple space
            //   - tab

            assert_valid(&spaces(1), &spaces(0), 0);
            assert_valid(&spaces(1), &spaces(1), 1);
            assert_valid(&spaces(1), &spaces(2), 2);
            assert_valid(&spaces(1), &spaces(3), 3);

            assert_valid(&spaces(2), &spaces(0), 0);
            assert_valid(&spaces(2), &spaces(2), 1);
            assert_valid(&spaces(2), &spaces(4), 2);
            assert_valid(&spaces(2), &spaces(6), 3);

            assert_valid(&spaces(4), &spaces(0), 0);
            assert_valid(&spaces(4), &spaces(4), 1);
            assert_valid(&spaces(4), &spaces(8), 2);
            assert_valid(&spaces(4), &spaces(12), 3);

            assert_valid(&tabs(1), &tabs(0), 0);
            assert_valid(&tabs(1), &tabs(1), 1);
            assert_valid(&tabs(1), &tabs(2), 2);
            assert_valid(&tabs(1), &tabs(3), 3);
        }

        #[test]
        fn invalid_indents() {
            let assert_invalid = |unit: &str, indent: &str| {
                assert_eq!(
                    indent_level(unit, indent),
                    None,
                    "unit: {:?}, indent: {:?}",
                    unit,
                    indent,
                );
            };
            assert_invalid(&spaces(2), &spaces(1));
            assert_invalid(&spaces(2), &spaces(3));
            assert_invalid(&spaces(2), &spaces(5));

            assert_invalid(&spaces(4), &spaces(1));
            assert_invalid(&spaces(4), &spaces(2));
            assert_invalid(&spaces(4), &spaces(3));
            assert_invalid(&spaces(4), &spaces(5));
            assert_invalid(&spaces(4), &spaces(6));
            assert_invalid(&spaces(4), &spaces(7));
            assert_invalid(&spaces(4), &spaces(9));

            assert_invalid(&spaces(1), &tabs(1));
            assert_invalid(&tabs(1), &spaces(1));
        }
    }
}
