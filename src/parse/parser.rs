/*

detect indentation level



*/
use super::{
    Attribute,
    Record,
    Schema,
    Table,
    Token,
    Value,
};

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

    pub fn parse(mut self, tokens: Vec<Token>) -> Self {
        use State::*;

        for token in tokens {
            self.state = match self.state {

                LineStart => match token {
                    Token::Newline => {
                        LineStart
                    }
                    Token::Indent(indent) => {
                        if indent.len() == 0 {
                            panic!("Empty indent received");
                        }

                        if indent.trim().len() > 0 {
                            panic!("Non-whitespace indent received");
                        }

                        if let None = &self.indent_unit {
                            self.indent_unit = Some(indent.clone());
                        }

                        match indent_level(self.indent_unit.as_ref().unwrap(), &indent) {
                            Some(level) => match level {
                                1 => ExpectingTable,
                                2 => ExpectingRecord,
                                3 => ExpectingColumn,
                                _ => panic!("Unexpected indentation level")
                            }
                            None => panic!("Inconsistent indent")
                        }
                    }
                    Token::Identifier(ident) | Token::QuotedIdentifier(ident) => {
                        self.schemas.push(Schema::new(ident));
                        CreatedSchema
                    }
                    _ => panic!("Unexpected token {:?}", token)
                }

                CreatedSchema | CreatedTable | CreatedRecord | CreatedAttribute => match token {
                    Token::Newline => LineStart,
                    _ => panic!("Unexpected token {:?}", token)
                }

                ExpectingTable => match token {
                    Token::Newline => {
                        LineStart
                    }
                    Token::Identifier(ident) | Token::QuotedIdentifier(ident) => {
                        self
                            .schemas.last_mut().expect("No schema to add table to")
                            .tables.push(Table::new(ident));

                        CreatedTable
                    }

                    _ => panic!("Unexpected token {:?}", token)
                }

                ExpectingRecord => match token {
                    Token::Newline => {
                        LineStart
                    }
                    Token::Identifier(_) | Token::Underscore => {
                        let name = match token {
                            Token::Identifier(ident) => Some(ident),
                            Token::Underscore => None,
                            _ => unreachable!()
                        };

                        self
                            .schemas.last_mut().expect("No schema to find table in")
                            .tables.last_mut().expect("No table to add record to")
                            .records.push(Record::new(name));

                        CreatedRecord
                    }
                    _ => panic!("Unexpected token {:?}", token)
                }

                ExpectingColumn => match token {
                    Token::Newline => {
                        LineStart
                    }
                    Token::Identifier(ident) | Token::QuotedIdentifier(ident) => {
                        ExpectingValue(ident)
                    }
                    _ => panic!("Unexpected token {:?}", token)
                }

                ExpectingValue(column) => match token {
                    Token::Boolean(_) | Token::Number(_) | Token::Text(_) => {
                        let value = match token {
                            Token::Boolean(b) => Value::Boolean(b),
                            Token::Number(b) => Value::Number(b),
                            Token::Text(b) => Value::Text(b),
                            _ => unreachable!()
                        };

                        self
                            .schemas.last_mut().expect("No schema to find table in")
                            .tables.last_mut().expect("No table to find record in")
                            .records.last_mut().expect("No record to add attribute to")
                            .attributes.push(Attribute { name: column, value });

                        CreatedAttribute
                    }
                    _ => panic!("Expected value for attribute")
                }

                _ => unreachable!()
            }
        }

        match self.state {
            ExpectingValue(_) => panic!("Expected value for attribute"),
            _ => {}
        }

        self
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
        if p.len() > 0 {
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

            assert_valid(&spaces(4), &spaces(0),  0);
            assert_valid(&spaces(4), &spaces(4),  1);
            assert_valid(&spaces(4), &spaces(8),  2);
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
