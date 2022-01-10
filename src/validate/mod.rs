pub mod error;

use super::parse::*;
pub use error::{ValidateError, ValidateErrorKind};

#[derive(Debug, PartialEq)]
pub struct ValidatedSchemas(Vec<Schema>);

impl ValidatedSchemas {
    pub fn schemas(&self) -> &Vec<Schema> {
        &self.0
    }
}

pub fn validate(schemas: Vec<Schema>) -> Result<ValidatedSchemas, ValidateError> {
    let mut validated: Vec<Schema> = Vec::new();

    for schema in schemas {
        let validated_schema = match validated.iter_mut().find(|v| v.name == schema.name) {
            Some(v) => v,
            None => {
                validated.push(Schema::new(schema.name));
                validated.last_mut().unwrap()
            }
        };

        for table in schema.tables {
            let validated_table = match validated_schema
                .tables
                .iter_mut()
                .find(|t| t.name == table.name)
            {
                Some(t) => t,
                None => {
                    validated_schema.tables.push(Table::new(table.name));
                    validated_schema.tables.last_mut().unwrap()
                }
            };

            for record in table.records {
                if let Some(record_name) = &record.name {
                    let name_present = validated_table
                        .records
                        .iter()
                        .any(|r| r.name == record.name);

                    if name_present {
                        return Err(ValidateError {
                            kind: ValidateErrorKind::DuplicateRecordName(record_name.to_owned()),
                            schema: validated_schema.name.clone(),
                            table: validated_table.name.clone(),
                        });
                    }
                }

                validated_table.records.push(Record::new(record.name));
                let validated_record = validated_table.records.last_mut().unwrap();

                for attribute in record.attributes {
                    let column_present = validated_record
                        .attributes
                        .iter()
                        .any(|a| a.name == attribute.name);

                    if column_present {
                        return Err(ValidateError {
                            kind: ValidateErrorKind::DuplicateColumn {
                                record: validated_record.name.clone(),
                                column: attribute.name.clone(),
                            },
                            schema: validated_schema.name.clone(),
                            table: validated_table.name.clone(),
                        });
                    }

                    validated_record.attributes.push(attribute);
                }
            }
        }
    }

    Ok(ValidatedSchemas(validated))
}

#[cfg(test)]
mod validate_tests {
    use super::*;

    #[test]
    fn empty_is_valid() {
        assert_eq!(validate(Vec::new()), Ok(ValidatedSchemas(Vec::new())));
    }

    #[test]
    fn two_empty_schemas() {
        let input = vec![
            Schema::new("schema1".to_owned()),
            Schema::new("schema2".to_owned()),
        ];
        let output = vec![
            Schema::new("schema1".to_owned()),
            Schema::new("schema2".to_owned()),
        ];

        assert_eq!(validate(input), Ok(ValidatedSchemas(output)));
    }

    #[test]
    fn dedupe_schemas() {
        let input = vec![
            Schema::new("schema1".to_owned()),
            Schema::new("schema1".to_owned()),
        ];
        let output = vec![Schema::new("schema1".to_owned())];

        assert_eq!(validate(input), Ok(ValidatedSchemas(output)));

        let input = vec![
            Schema::new("schema1".to_owned()),
            Schema::new("schema2".to_owned()),
            Schema::new("schema1".to_owned()),
        ];
        let output = vec![
            Schema::new("schema1".to_owned()),
            Schema::new("schema2".to_owned()),
        ];

        assert_eq!(validate(input), Ok(ValidatedSchemas(output)));
    }

    #[test]
    fn dedupe_schemas_empty_tables() {
        let input = vec![
            Schema {
                name: "schema1".to_owned(),
                tables: vec![],
            },
            Schema {
                name: "schema1".to_owned(),
                tables: vec![
                    Table::new("table1".to_owned()),
                    Table::new("table3".to_owned()),
                ],
            },
            Schema {
                name: "schema2".to_owned(),
                tables: vec![Table::new("table1".to_owned())],
            },
            Schema {
                name: "schema1".to_owned(),
                tables: vec![Table::new("table2".to_owned())],
            },
        ];
        let output = vec![
            Schema {
                name: "schema1".to_owned(),
                tables: vec![
                    Table::new("table1".to_owned()),
                    Table::new("table3".to_owned()),
                    Table::new("table2".to_owned()),
                ],
            },
            Schema {
                name: "schema2".to_owned(),
                tables: vec![Table::new("table1".to_owned())],
            },
        ];

        assert_eq!(validate(input), Ok(ValidatedSchemas(output)));
    }

    #[test]
    fn dedupe_tables_with_records() {
        let input = vec![
            Schema {
                name: "schema1".to_owned(),
                tables: vec![],
            },
            Schema {
                name: "schema1".to_owned(),
                tables: vec![
                    Table {
                        name: "table1".to_owned(),
                        records: vec![Record::new(Some("record2".to_owned()))],
                    },
                    Table {
                        name: "table2".to_owned(),
                        records: vec![
                            Record::new(None),
                            Record::new(Some("record1".to_owned())), // Same name as record from public.table1
                            Record::new(None),
                        ],
                    },
                ],
            },
            Schema {
                name: "schema2".to_owned(),
                tables: vec![Table::new("table1".to_owned())],
            },
            Schema {
                name: "schema1".to_owned(),
                tables: vec![Table {
                    name: "table1".to_owned(),
                    records: vec![Record::new(Some("record1".to_owned()))],
                }],
            },
        ];
        let output = vec![
            Schema {
                name: "schema1".to_owned(),
                tables: vec![
                    Table {
                        name: "table1".to_owned(),
                        records: vec![
                            Record::new(Some("record2".to_owned())),
                            Record::new(Some("record1".to_owned())),
                        ],
                    },
                    Table {
                        name: "table2".to_owned(),
                        records: vec![
                            Record::new(None),
                            Record::new(Some("record1".to_owned())),
                            Record::new(None),
                        ],
                    },
                ],
            },
            Schema {
                name: "schema2".to_owned(),
                tables: vec![Table::new("table1".to_owned())],
            },
        ];

        assert_eq!(validate(input), Ok(ValidatedSchemas(output)));
    }

    #[test]
    fn duplicate_record_names() {
        assert_eq!(
            validate(vec![Schema {
                name: "schema1".to_owned(),
                tables: vec![Table {
                    name: "table1".to_owned(),
                    records: vec![
                        Record::new(Some("record1".to_owned())),
                        Record::new(Some("record1".to_owned())),
                    ],
                }],
            }]),
            Err(ValidateError {
                kind: ValidateErrorKind::DuplicateRecordName("record1".to_owned()),
                schema: "schema1".to_owned(),
                table: "table1".to_owned(),
            })
        );
    }

    #[test]
    fn attributes() {
        let input = vec![
            Schema {
                name: "schema1".to_owned(),
                tables: vec![Table {
                    name: "table1".to_owned(),
                    records: vec![Record {
                        name: None,
                        attributes: vec![
                            Attribute {
                                name: "attr1".to_owned(),
                                value: Value::Text("Attr1".to_owned()),
                            },
                            Attribute {
                                name: "attr2".to_owned(),
                                value: Value::Number("123".to_owned()),
                            },
                        ],
                    }],
                }],
            },
            Schema {
                name: "schema1".to_owned(),
                tables: vec![Table {
                    name: "table1".to_owned(),
                    records: vec![Record {
                        name: Some("my_record".to_owned()),
                        attributes: vec![
                            Attribute {
                                name: "attr1".to_owned(),
                                value: Value::Text("Attr1".to_owned()),
                            },
                            Attribute {
                                name: "attr3".to_owned(),
                                value: Value::Boolean(true),
                            },
                        ],
                    }],
                }],
            },
        ];
        let output = vec![Schema {
            name: "schema1".to_owned(),
            tables: vec![Table {
                name: "table1".to_owned(),
                records: vec![
                    Record {
                        name: None,
                        attributes: vec![
                            Attribute {
                                name: "attr1".to_owned(),
                                value: Value::Text("Attr1".to_owned()),
                            },
                            Attribute {
                                name: "attr2".to_owned(),
                                value: Value::Number("123".to_owned()),
                            },
                        ],
                    },
                    Record {
                        name: Some("my_record".to_owned()),
                        attributes: vec![
                            Attribute {
                                name: "attr1".to_owned(),
                                value: Value::Text("Attr1".to_owned()),
                            },
                            Attribute {
                                name: "attr3".to_owned(),
                                value: Value::Boolean(true),
                            },
                        ],
                    },
                ],
            }],
        }];

        assert_eq!(validate(input), Ok(ValidatedSchemas(output)));
    }

    #[test]
    fn duplicate_attribute_names_anonymous() {
        assert_eq!(
            validate(vec![Schema {
                name: "schema1".to_owned(),
                tables: vec![Table {
                    name: "table1".to_owned(),
                    records: vec![Record {
                        name: None,
                        attributes: vec![
                            Attribute {
                                name: "attr1".to_owned(),
                                value: Value::Text("Attr1-a".to_owned()),
                            },
                            Attribute {
                                name: "attr2".to_owned(),
                                value: Value::Text("Attr2".to_owned()),
                            },
                            Attribute {
                                name: "attr1".to_owned(),
                                value: Value::Text("Attr1-b".to_owned()),
                            },
                        ],
                    }],
                }],
            }]),
            Err(ValidateError {
                kind: ValidateErrorKind::DuplicateColumn {
                    record: None,
                    column: "attr1".to_owned(),
                },
                schema: "schema1".to_owned(),
                table: "table1".to_owned(),
            })
        );
    }

    #[test]
    fn duplicate_attribute_names_named_record() {
        assert_eq!(
            validate(vec![Schema {
                name: "schema1".to_owned(),
                tables: vec![Table {
                    name: "table1".to_owned(),
                    records: vec![Record {
                        name: Some("my_record".to_owned()),
                        attributes: vec![
                            Attribute {
                                name: "attr1".to_owned(),
                                value: Value::Text("Attr1-a".to_owned()),
                            },
                            Attribute {
                                name: "attr2".to_owned(),
                                value: Value::Text("Attr2".to_owned()),
                            },
                            Attribute {
                                name: "attr1".to_owned(),
                                value: Value::Text("Attr1-b".to_owned()),
                            },
                        ],
                    }],
                }],
            }]),
            Err(ValidateError {
                kind: ValidateErrorKind::DuplicateColumn {
                    record: Some("my_record".to_owned()),
                    column: "attr1".to_owned(),
                },
                schema: "schema1".to_owned(),
                table: "table1".to_owned(),
            })
        );
    }
}
