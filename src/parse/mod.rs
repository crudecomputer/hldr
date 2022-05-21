pub mod error;
mod parser;

use super::lex::Token;
pub use error::{ParseError, ParseErrorKind};
use parser::Parser;


#[derive(Clone, Debug, PartialEq)]
pub struct ReferenceValue {
    pub schema: String,
    pub table: String,
    pub record: String,
    pub column: String,
}

#[derive(Debug, PartialEq)]
pub enum Value {
    Boolean(bool),
    Number(String),
    Text(String),
    Reference(Box<ReferenceValue>)
}

#[derive(Debug, PartialEq)]
pub struct Attribute {
    pub name: String,
    pub value: Value,
}

#[derive(Debug, PartialEq)]
pub struct Record {
    pub name: Option<String>,
    pub attributes: Vec<Attribute>,
}

impl Record {
    pub fn new(name: Option<String>) -> Self {
        Self {
            name,
            attributes: Vec::new(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Table {
    pub name: String,
    pub records: Vec<Record>,
}

impl Table {
    pub fn new(name: String) -> Self {
        Self {
            name,
            records: Vec::new(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Schema {
    pub name: String,
    pub tables: Vec<Table>,
}

impl Schema {
    pub fn new(name: String) -> Self {
        Self {
            name,
            tables: Vec::new(),
        }
    }
}

pub fn parse(tokens: Vec<Token>) -> Result<Vec<Schema>, ParseError> {
    Ok(Parser::new().parse(tokens)?.schemas)
}

#[cfg(test)]
mod tests {
    use super::{Token as T, *};
    use super::super::lex::lex;

    #[test]
    fn empty() {
        assert_eq!(parse(vec![]), Ok(vec![]));
    }

    #[test]
    fn schema() {
        let tokens = vec![T::Newline, T::Identifier("public".to_owned())];

        assert_eq!(
            parse(tokens),
            Ok(vec![Schema {
                name: "public".to_owned(),
                tables: Vec::new(),
            },])
        );
    }

    #[test]
    fn schemas() {
        let tokens = vec![
            T::Identifier("schema1".to_owned()),
            T::Newline,
            T::Newline,
            T::Identifier("schema2".to_owned()),
            T::Newline,
        ];

        assert_eq!(
            parse(tokens),
            Ok(vec![
                Schema {
                    name: "schema1".to_owned(),
                    tables: Vec::new(),
                },
                Schema {
                    name: "schema2".to_owned(),
                    tables: Vec::new(),
                },
            ])
        );

        let tokens = vec![
            T::Newline,
            T::Newline,
            T::Identifier("schema1".to_owned()),
            T::Newline,
            T::Identifier("schema2".to_owned()),
        ];

        assert_eq!(
            parse(tokens),
            Ok(vec![
                Schema {
                    name: "schema1".to_owned(),
                    tables: Vec::new(),
                },
                Schema {
                    name: "schema2".to_owned(),
                    tables: Vec::new(),
                },
            ])
        );
    }

    #[test]
    fn schemas_without_newlines() {
        let tokens = vec![
            T::Identifier("schema1".to_owned()),
            T::Identifier("schema2".to_owned()),
        ];

        assert_eq!(
            parse(tokens),
            Err(ParseError::unexpected_token(
                1,
                T::Identifier("schema2".to_owned())
            ))
        );
    }

    #[test]
    fn table() {
        let tokens = vec![
            T::Identifier("public".to_owned()),
            T::Newline,
            T::Indent("  ".to_owned()),
            T::Identifier("my_table".to_owned()),
        ];

        assert_eq!(
            parse(tokens),
            Ok(vec![Schema {
                name: "public".to_owned(),
                tables: vec![Table {
                    name: "my_table".to_owned(),
                    records: Vec::new(),
                }],
            },])
        );
    }

    #[test]
    fn tables() {
        let tokens = vec![
            T::Identifier("public".to_owned()),
            T::Newline,
            T::Indent("    ".to_owned()),
            T::Identifier("table1".to_owned()),
            T::Newline,
            T::Indent("    ".to_owned()),
            T::Identifier("table2".to_owned()),
        ];

        assert_eq!(
            parse(tokens),
            Ok(vec![Schema {
                name: "public".to_owned(),
                tables: vec![
                    Table {
                        name: "table1".to_owned(),
                        records: Vec::new(),
                    },
                    Table {
                        name: "table2".to_owned(),
                        records: Vec::new(),
                    },
                ],
            },])
        );
    }

    #[test]
    fn tables_quoted_identifiers() {
        let tokens = vec![
            T::QuotedIdentifier("public schema".to_owned()),
            T::Newline,
            T::Indent("    ".to_owned()),
            T::QuotedIdentifier("a table".to_owned()),
            T::Newline,
            T::Indent("    ".to_owned()),
            T::QuotedIdentifier("another table".to_owned()),
        ];

        assert_eq!(
            parse(tokens),
            Ok(vec![Schema {
                name: "public schema".to_owned(),
                tables: vec![
                    Table {
                        name: "a table".to_owned(),
                        records: Vec::new(),
                    },
                    Table {
                        name: "another table".to_owned(),
                        records: Vec::new(),
                    },
                ],
            },])
        );
    }

    #[test]
    fn tables_without_newlines() {
        let tokens = vec![
            T::Identifier("schema1".to_owned()),
            T::Newline,
            T::Indent("    ".to_owned()),
            T::Identifier("table1".to_owned()),
            T::Identifier("table2".to_owned()),
        ];

        assert_eq!(
            parse(tokens),
            Err(ParseError::unexpected_token(
                2,
                T::Identifier("table2".to_owned())
            ))
        );
    }

    #[test]
    fn named_record() {
        let tokens = lex(
"public
  person
    kevin"
        ).unwrap();

        assert_eq!(
            parse(tokens),
            Ok(vec![Schema {
                name: "public".to_owned(),
                tables: vec![Table {
                    name: "person".to_owned(),
                    records: vec![Record {
                        name: Some("kevin".to_owned()),
                        attributes: Vec::new(),
                    }]
                }],
            },])
        );
    }

    #[test]
    fn named_records() {
        let tokens = lex(
"public
  person
    stacey
    kevin
  pet
    eiyre
    cupid"
        ).unwrap();

        assert_eq!(
            parse(tokens),
            Ok(vec![Schema {
                name: "public".to_owned(),
                tables: vec![
                    Table {
                        name: "person".to_owned(),
                        records: vec![
                            Record {
                                name: Some("stacey".to_owned()),
                                attributes: Vec::new(),
                            },
                            Record {
                                name: Some("kevin".to_owned()),
                                attributes: Vec::new(),
                            },
                        ]
                    },
                    Table {
                        name: "pet".to_owned(),
                        records: vec![
                            Record {
                                name: Some("eiyre".to_owned()),
                                attributes: Vec::new(),
                            },
                            Record {
                                name: Some("cupid".to_owned()),
                                attributes: Vec::new(),
                            },
                        ]
                    },
                ],
            },])
        );
    }

    #[test]
    fn anonymous_record() {
        let tokens = vec![
            T::Identifier("public".to_owned()),
            T::Newline,
            T::Indent("  ".to_owned()),
            T::Identifier("person".to_owned()),
            T::Newline,
            T::Indent("    ".to_owned()),
            T::Underscore,
        ];

        assert_eq!(
            parse(tokens),
            Ok(vec![Schema {
                name: "public".to_owned(),
                tables: vec![Table {
                    name: "person".to_owned(),
                    records: vec![Record {
                        name: None,
                        attributes: Vec::new(),
                    }]
                }],
            },])
        );
    }

    #[test]
    fn anonymous_records() {
        let tokens = vec![
            T::Identifier("public".to_owned()),
            T::Newline,
            T::Indent("  ".to_owned()),
            T::Identifier("person".to_owned()),
            T::Newline,
            T::Indent("    ".to_owned()),
            T::Underscore,
            T::Newline,
            T::Indent("    ".to_owned()),
            T::Underscore,
            T::Newline,
            T::Indent("  ".to_owned()),
            T::Identifier("pet".to_owned()),
            T::Newline,
            T::Indent("    ".to_owned()),
            T::Underscore,
            T::Newline,
            T::Indent("    ".to_owned()),
            T::Underscore,
        ];

        assert_eq!(
            parse(tokens),
            Ok(vec![Schema {
                name: "public".to_owned(),
                tables: vec![
                    Table {
                        name: "person".to_owned(),
                        records: vec![
                            Record {
                                name: None,
                                attributes: Vec::new(),
                            },
                            Record {
                                name: None,
                                attributes: Vec::new(),
                            },
                        ]
                    },
                    Table {
                        name: "pet".to_owned(),
                        records: vec![
                            Record {
                                name: None,
                                attributes: Vec::new(),
                            },
                            Record {
                                name: None,
                                attributes: Vec::new(),
                            },
                        ]
                    },
                ],
            },])
        );
    }

    #[test]
    fn mixed_records() {
        let tokens = vec![
            T::Identifier("public".to_owned()),
            T::Newline,
            T::Indent("  ".to_owned()),
            T::Identifier("person".to_owned()),
            T::Newline,
            T::Indent("    ".to_owned()),
            T::Identifier("kevin".to_owned()),
            T::Newline,
            T::Indent("    ".to_owned()),
            T::Underscore,
            T::Newline,
            T::Indent("  ".to_owned()),
            T::Identifier("pet".to_owned()),
            T::Newline,
            T::Indent("    ".to_owned()),
            T::Underscore,
            T::Newline,
            T::Indent("    ".to_owned()),
            T::Identifier("eiyre".to_owned()),
        ];

        assert_eq!(
            parse(tokens),
            Ok(vec![Schema {
                name: "public".to_owned(),
                tables: vec![
                    Table {
                        name: "person".to_owned(),
                        records: vec![
                            Record {
                                name: Some("kevin".to_owned()),
                                attributes: Vec::new(),
                            },
                            Record {
                                name: None,
                                attributes: Vec::new(),
                            },
                        ]
                    },
                    Table {
                        name: "pet".to_owned(),
                        records: vec![
                            Record {
                                name: None,
                                attributes: Vec::new(),
                            },
                            Record {
                                name: Some("eiyre".to_owned()),
                                attributes: Vec::new(),
                            },
                        ]
                    },
                ],
            },])
        );
    }

    #[test]
    fn invalid_records() {
        let tokens = vec![
            T::Identifier("schema1".to_owned()),
            T::Newline,
            T::Indent("  ".to_owned()),
            T::Identifier("table1".to_owned()),
            T::Newline,
            T::Indent("    ".to_owned()),
            T::Identifier("record1".to_owned()),
            T::Identifier("another_identifier".to_owned()),
        ];

        assert_eq!(
            parse(tokens),
            Err(ParseError::unexpected_token(
                3,
                T::Identifier("another_identifier".to_owned())
            ))
        );
    }

    #[test]
    fn boolean_attribute() {
        let tokens = vec![
            T::Identifier("public".to_owned()),
            T::Newline,
            T::Indent("\t".to_owned()),
            T::Identifier("person".to_owned()),
            T::Newline,
            T::Indent("\t\t".to_owned()),
            T::Identifier("kevin".to_owned()),
            T::Newline,
            T::Indent("\t\t\t".to_owned()),
            T::Identifier("likes_coffee".to_owned()),
            T::Boolean(true),
        ];

        assert_eq!(
            parse(tokens),
            Ok(vec![Schema {
                name: "public".to_owned(),
                tables: vec![Table {
                    name: "person".to_owned(),
                    records: vec![Record {
                        name: Some("kevin".to_owned()),
                        attributes: vec![Attribute {
                            name: "likes_coffee".to_owned(),
                            value: Value::Boolean(true),
                        }]
                    }]
                }]
            },])
        );
    }

    #[test]
    fn number_attribute() {
        let tokens = vec![
            T::Identifier("public".to_owned()),
            T::Newline,
            T::Indent("\t".to_owned()),
            T::Identifier("person".to_owned()),
            T::Newline,
            T::Indent("\t\t".to_owned()),
            T::Identifier("kevin".to_owned()),
            T::Newline,
            T::Indent("\t\t\t".to_owned()),
            T::Identifier("pets".to_owned()),
            T::Number("2".to_owned()),
        ];

        assert_eq!(
            parse(tokens),
            Ok(vec![Schema {
                name: "public".to_owned(),
                tables: vec![Table {
                    name: "person".to_owned(),
                    records: vec![Record {
                        name: Some("kevin".to_owned()),
                        attributes: vec![Attribute {
                            name: "pets".to_owned(),
                            value: Value::Number("2".to_owned()),
                        }]
                    }]
                }]
            }])
        );
        let tokens = vec![
            T::Identifier("public".to_owned()),
            T::Newline,
            T::Indent("\t".to_owned()),
            T::Identifier("person".to_owned()),
            T::Newline,
            T::Indent("\t\t".to_owned()),
            T::Identifier("kevin".to_owned()),
            T::Newline,
            T::Indent("\t\t\t".to_owned()),
            T::Identifier("pets".to_owned()),
            T::Number("2.5".to_owned()),
        ];

        assert_eq!(
            parse(tokens),
            Ok(vec![Schema {
                name: "public".to_owned(),
                tables: vec![Table {
                    name: "person".to_owned(),
                    records: vec![Record {
                        name: Some("kevin".to_owned()),
                        attributes: vec![Attribute {
                            name: "pets".to_owned(),
                            value: Value::Number("2.5".to_owned()),
                        }]
                    }]
                }]
            }])
        );
    }

    #[test]
    fn text_attribute() {
        let tokens = vec![
            T::Identifier("public".to_owned()),
            T::Newline,
            T::Indent("\t".to_owned()),
            T::Identifier("person".to_owned()),
            T::Newline,
            T::Indent("\t\t".to_owned()),
            T::Identifier("kevin".to_owned()),
            T::Newline,
            T::Indent("\t\t\t".to_owned()),
            T::Identifier("name".to_owned()),
            T::Text("Kevin".to_owned()),
        ];

        assert_eq!(
            parse(tokens),
            Ok(vec![Schema {
                name: "public".to_owned(),
                tables: vec![Table {
                    name: "person".to_owned(),
                    records: vec![Record {
                        name: Some("kevin".to_owned()),
                        attributes: vec![Attribute {
                            name: "name".to_owned(),
                            value: Value::Text("Kevin".to_owned()),
                        }]
                    }]
                }]
            },])
        );
    }

    #[test]
    fn fully_qualified_reference_attribute() {
        let tokens = lex(
"public
  person
    kevin
      column1 schema.table@record.column2"
        ).unwrap();

        assert_eq!(
            parse(tokens),
            Ok(vec![Schema {
                name: "public".to_owned(),
                tables: vec![Table {
                    name: "person".to_owned(),
                    records: vec![Record {
                        name: Some("kevin".to_owned()),
                        attributes: vec![Attribute {
                            name: "column1".to_owned(),
                            value: Value::Reference(Box::new(ReferenceValue {
                                schema: "schema".to_owned(),
                                table: "table".to_owned(),
                                record: "record".to_owned(),
                                column: "column2".to_owned(),
                            })),
                        }]
                    }]
                }]
            },])
        );

        let tokens = vec![
            T::QuotedIdentifier("public".to_owned()),
            T::Newline,
            T::Indent("\t".to_owned()),
            T::QuotedIdentifier("person".to_owned()),
            T::Newline,
            T::Indent("\t\t".to_owned()),
            T::Identifier("kevin".to_owned()),
            T::Newline,
            T::Indent("\t\t\t".to_owned()),
            T::QuotedIdentifier("column1".to_owned()),
            T::QuotedIdentifier("schema".to_owned()),
            T::Period,
            T::QuotedIdentifier("table".to_owned()),
            T::AtSign,
            T::QuotedIdentifier("record".to_owned()),
            T::Period,
            T::QuotedIdentifier("column2".to_owned()),
            T::Newline,
        ];

        assert_eq!(
            parse(tokens),
            Ok(vec![Schema {
                name: "public".to_owned(),
                tables: vec![Table {
                    name: "person".to_owned(),
                    records: vec![Record {
                        name: Some("kevin".to_owned()),
                        attributes: vec![Attribute {
                            name: "column1".to_owned(),
                            value: Value::Reference(Box::new(ReferenceValue {
                                schema: "schema".to_owned(),
                                table: "table".to_owned(),
                                record: "record".to_owned(),
                                column: "column2".to_owned(),
                            })),
                        }]
                    }]
                }]
            },])
        );
    }

    #[test]
    fn table_qualified_reference_attribute() {
        let tokens = lex(
"public
  person
    kevin
      column1 table@record.column2"
        ).unwrap();

        assert_eq!(
            parse(tokens),
            Ok(vec![Schema {
                name: "public".to_owned(),
                tables: vec![Table {
                    name: "person".to_owned(),
                    records: vec![Record {
                        name: Some("kevin".to_owned()),
                        attributes: vec![Attribute {
                            name: "column1".to_owned(),
                            value: Value::Reference(Box::new(ReferenceValue {
                                schema: "public".to_owned(),
                                table: "table".to_owned(),
                                record: "record".to_owned(),
                                column: "column2".to_owned(),
                            })),
                        }]
                    }]
                }]
            },])
        );

        let tokens = vec![
            T::QuotedIdentifier("public".to_owned()),
            T::Newline,
            T::Indent("\t".to_owned()),
            T::QuotedIdentifier("person".to_owned()),
            T::Newline,
            T::Indent("\t\t".to_owned()),
            T::Identifier("kevin".to_owned()),
            T::Newline,
            T::Indent("\t\t\t".to_owned()),
            T::QuotedIdentifier("column1".to_owned()),
            T::QuotedIdentifier("table".to_owned()),
            T::AtSign,
            T::QuotedIdentifier("record".to_owned()),
            T::Period,
            T::QuotedIdentifier("column2".to_owned()),
            T::Newline,
        ];

        assert_eq!(
            parse(tokens),
            Ok(vec![Schema {
                name: "public".to_owned(),
                tables: vec![Table {
                    name: "person".to_owned(),
                    records: vec![Record {
                        name: Some("kevin".to_owned()),
                        attributes: vec![Attribute {
                            name: "column1".to_owned(),
                            value: Value::Reference(Box::new(ReferenceValue {
                                schema: "public".to_owned(),
                                table: "table".to_owned(),
                                record: "record".to_owned(),
                                column: "column2".to_owned(),
                            })),
                        }]
                    }]
                }]
            },])
        );
    }

    #[test]
    fn lotsa_attributes() {
        let tokens = vec![
            T::Identifier("public".to_owned()),
            T::Newline,
            T::Indent("\t".to_owned()),
            T::Identifier("person".to_owned()),
            T::Newline,
            T::Indent("\t\t".to_owned()),
            T::Identifier("kevin".to_owned()),
            T::Newline,
            T::Indent("\t\t\t".to_owned()),
            T::Identifier("name".to_owned()),
            T::Text("Kevin".to_owned()),
            T::Newline,
            T::Indent("\t\t\t".to_owned()),
            T::Identifier("age".to_owned()),
            T::Number("39".to_owned()),
            T::Newline,
            T::Indent("\t\t".to_owned()),
            T::Underscore,
            T::Newline,
            T::Indent("\t\t\t".to_owned()),
            T::Identifier("name".to_owned()),
            T::Text("Nobody".to_owned()),
            T::Newline,
            T::Indent("\t\t\t".to_owned()),
            T::QuotedIdentifier("IsIdentifiable?".to_owned()),
            T::Boolean(false),
            T::Newline,
            T::Identifier("private_schema".to_owned()),
            T::Newline,
            T::Indent("\t".to_owned()),
            T::Identifier("pet".to_owned()),
            T::Newline,
            T::Indent("\t\t".to_owned()),
            T::Identifier("cupid".to_owned()),
            T::Newline,
            T::Indent("\t\t\t".to_owned()),
            T::Identifier("loves_belly_scratches".to_owned()),
            T::Boolean(true),
        ];

        assert_eq!(
            parse(tokens),
            Ok(vec![
                Schema {
                    name: "public".to_owned(),
                    tables: vec![Table {
                        name: "person".to_owned(),
                        records: vec![
                            Record {
                                name: Some("kevin".to_owned()),
                                attributes: vec![
                                    Attribute {
                                        name: "name".to_owned(),
                                        value: Value::Text("Kevin".to_owned()),
                                    },
                                    Attribute {
                                        name: "age".to_owned(),
                                        value: Value::Number("39".to_owned()),
                                    },
                                ],
                            },
                            Record {
                                name: None,
                                attributes: vec![
                                    Attribute {
                                        name: "name".to_owned(),
                                        value: Value::Text("Nobody".to_owned()),
                                    },
                                    Attribute {
                                        name: "IsIdentifiable?".to_owned(),
                                        value: Value::Boolean(false),
                                    },
                                ],
                            },
                        ],
                    },],
                },
                Schema {
                    name: "private_schema".to_owned(),
                    tables: vec![Table {
                        name: "pet".to_owned(),
                        records: vec![Record {
                            name: Some("cupid".to_owned()),
                            attributes: vec![Attribute {
                                name: "loves_belly_scratches".to_owned(),
                                value: Value::Boolean(true),
                            },],
                        },],
                    },],
                }
            ])
        );
    }

    #[test]
    fn expecting_attribute_wrong_token() {
        let tokens = vec![
            T::Identifier("public".to_owned()),
            T::Newline,
            T::Indent("\t".to_owned()),
            T::Identifier("person".to_owned()),
            T::Newline,
            T::Indent("\t\t".to_owned()),
            T::Identifier("kevin".to_owned()),
            T::Newline,
            T::Indent("\t\t\t".to_owned()),
            T::Boolean(false),
            T::Newline,
        ];

        assert_eq!(
            parse(tokens),
            Err(ParseError::unexpected_token(4, T::Boolean(false)))
        );
    }

    #[test]
    fn attribute_without_value() {
        let tokens = vec![
            T::Identifier("public".to_owned()),
            T::Newline,
            T::Indent("\t".to_owned()),
            T::Identifier("person".to_owned()),
            T::Newline,
            T::Indent("\t\t".to_owned()),
            T::Identifier("kevin".to_owned()),
            T::Newline,
            T::Indent("\t\t\t".to_owned()),
            T::Identifier("name".to_owned()),
            T::Newline,
        ];

        assert_eq!(parse(tokens), Err(ParseError::missing_column_value(4)));
    }

    #[test]
    fn attribute_without_value_newline() {
        let tokens = vec![
            T::Identifier("public".to_owned()),
            T::Newline,
            T::Indent("\t".to_owned()),
            T::Identifier("person".to_owned()),
            T::Newline,
            T::Indent("\t\t".to_owned()),
            T::Identifier("kevin".to_owned()),
            T::Newline,
            T::Indent("\t\t\t".to_owned()),
            T::Identifier("name".to_owned()),
            T::Newline,
        ];

        assert_eq!(parse(tokens), Err(ParseError::missing_column_value(4)));
    }

    #[test]
    fn attribute_with_extra_identifier() {
        let tokens = vec![
            T::Identifier("public".to_owned()),
            T::Newline,
            T::Indent("\t".to_owned()),
            T::Identifier("person".to_owned()),
            T::Newline,
            T::Indent("\t\t".to_owned()),
            T::Identifier("kevin".to_owned()),
            T::Newline,
            T::Indent("\t\t\t".to_owned()),
            T::Identifier("name".to_owned()),
            T::Text("Kevin".to_owned()),
            T::Identifier("name".to_owned()),
        ];

        assert_eq!(
            parse(tokens),
            Err(ParseError::unexpected_token(
                4,
                T::Identifier("name".to_owned())
            ))
        );
    }

    #[test]
    fn attribute_with_incomplete_fully_qualified_reference() {
        let tokens = lex(
"public
  person
    kevin
      column1 schema1"
        ).unwrap();

        assert_eq!(
            parse(tokens),
            Err(ParseError::unexpected_token(
                4,
                T::Identifier("schema1".to_owned())
            ))
        );

        let tokens = lex(
"public
  person
    kevin
      column1 schema1."
        ).unwrap();

        assert_eq!(
            parse(tokens),
            Err(ParseError::incomplete_reference(4, "schema1.".to_owned()))
        );

        let tokens = lex(
"public
  person
    kevin
      column1 schema1.table1"
        ).unwrap();

        assert_eq!(
            parse(tokens),
            Err(ParseError::incomplete_reference(4, "schema1.table1".to_owned()))
        );

        let tokens = lex(
"public
  person
    kevin
      column1 schema1.table1@"
        ).unwrap();

        assert_eq!(
            parse(tokens),
            Err(ParseError::incomplete_reference(4, "schema1.table1@".to_owned()))
        );

        let tokens = lex(
"public
  person
    kevin
      column1 schema1.table1@record1"
        ).unwrap();

        assert_eq!(
            parse(tokens),
            Err(ParseError::incomplete_reference(4, "schema1.table1@record1".to_owned()))
        );

        let tokens = lex(
"public
  person
    kevin
      column1 schema1.table1@record1."
        ).unwrap();

        assert_eq!(
            parse(tokens),
            Err(ParseError::incomplete_reference(4, "schema1.table1@record1.".to_owned()))
        );
    }

    #[test]
    fn first_line_indented() {
        let tokens = vec![
            T::Indent("\t".to_owned()),
            T::Identifier("identifier".to_owned()),
            T::Newline,
        ];

        assert_eq!(parse(tokens), Err(ParseError::missing_schema(1)));

        let tokens = vec![
            T::Indent("\t\t\t".to_owned()),
            T::Identifier("identifier".to_owned()),
            T::Newline,
        ];

        assert_eq!(parse(tokens), Err(ParseError::missing_schema(1)));
    }

    #[test]
    fn inconsistent_indent() {
        let tokens = vec![
            T::Identifier("schema".to_owned()),
            T::Newline,
            T::Indent("\t\t".to_owned()),
            T::Identifier("identifier".to_owned()),
            T::Newline,
            T::Indent("\t".to_owned()),
            T::Identifier("identifier".to_owned()),
            T::Newline,
        ];

        assert_eq!(
            parse(tokens),
            Err(ParseError::inconsistent_indent(
                3,
                "\t\t".to_owned(),
                "\t".to_owned()
            ))
        );
    }

    #[test]
    fn missing_table() {
        // For a record to be detected in a schema *without* a table present,
        // there needs to have already been an indentation unit set by a table
        // in a previous schema. Otherwise, the indentation prior to the record's
        // identifier will be set as the indentation unit and, being at indentation
        // level 1, the identifer will be parsed as a table name.
        let tokens = vec![
            T::Identifier("schema1".to_owned()),
            T::Newline,
            T::Indent("\t".to_owned()),
            T::Identifier("table1".to_owned()),
            T::Newline,
            T::Identifier("schema2".to_owned()),
            T::Newline,
            T::Indent("\t\t".to_owned()),
            T::Underscore,
        ];

        assert_eq!(parse(tokens), Err(ParseError::missing_table(4)));
    }

    #[test]
    fn missing_record() {
        let tokens = vec![
            T::Identifier("schema".to_owned()),
            T::Newline,
            T::Indent("\t".to_owned()),
            T::Identifier("table".to_owned()),
            T::Newline,
            T::Indent("\t\t\t".to_owned()),
            T::Identifier("name".to_owned()),
            T::Text("anonymous".to_owned()),
        ];

        assert_eq!(parse(tokens), Err(ParseError::missing_record(3)));
    }
}
