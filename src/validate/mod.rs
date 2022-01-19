mod types;
pub mod error;

use super::parse::*;
pub use types::*;
pub use error::{ValidateError, ValidateErrorKind};


pub fn validate(schemas: Vec<Schema>) -> Result<ValidatedSchemas, ValidateError> {
    let mut vschemas = ValidatedSchemas::new();

    for schema in schemas {
        let vschema = vschemas.schemas_mut().get_or_create_mut(&schema.name);

        for table in schema.tables {
            let vtable = vschema.tables_mut().get_or_create_mut(&table.name);

            for record in table.records {
                let vattrs = match &record.name {
                    Some(name) => {
                        if vtable.named_records().contains_key(name) {
                            return Err(ValidateError {
                                kind: ValidateErrorKind::DuplicateRecordName(name.to_owned()),
                                schema: schema.name,
                                table: table.name,
                            });
                        }

                        // TODO: vtable shouldn't use `get_or_create_mut`
                        vtable.named_records_mut().get_or_create_mut(name).attributes_mut()
                    },
                    None => vtable.anonymous_records_mut().create().attributes_mut(),
                };

                for attribute in record.attributes {
                    if vattrs.contains_key(&attribute.name) {
                        return Err(ValidateError {
                            kind: ValidateErrorKind::DuplicateColumn {
                                record: record.name,
                                column: attribute.name,
                            },
                            schema: schema.name,
                            table: table.name,
                        });
                    }

                    //if let Value::Reference(r) = &attribute.value {
                        //let record_present = validated.iter()
                            //.find(|v| v.name == r.schema);
                    //}

                    vattrs.add(ValidatedAttribute::new(attribute));
                }
            }
        }
    }

    Ok(vschemas)
}

#[cfg(test)]
mod validate_tests {
    use super::*;

    #[test]
    fn empty_is_valid() {
        assert_eq!(validate(Vec::new()), Ok(ValidatedSchemas::new()));
    }

    #[test]
    fn two_empty_schemas() {
        let input = vec![
            Schema::new("schema1".to_owned()),
            Schema::new("schema2".to_owned()),
        ];

        let mut expected = ValidatedSchemas::new();
        expected.schemas_mut().get_or_create_mut("schema1");
        expected.schemas_mut().get_or_create_mut("schema2");

        assert_eq!(validate(input), Ok(expected));
    }

    #[test]
    fn dedupe_schemas() {
        let input = vec![
            Schema::new("schema1".to_owned()),
            Schema::new("schema1".to_owned()),
        ];
        let mut expected = ValidatedSchemas::new();
        expected.schemas_mut().get_or_create_mut("schema1");

        assert_eq!(validate(input), Ok(expected));

        let input = vec![
            Schema::new("schema1".to_owned()),
            Schema::new("schema2".to_owned()),
            Schema::new("schema1".to_owned()),
        ];

        let mut expected = ValidatedSchemas::new();
        expected.schemas_mut().get_or_create_mut("schema1");
        expected.schemas_mut().get_or_create_mut("schema2");

        assert_eq!(validate(input), Ok(expected));
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

        let mut expected = ValidatedSchemas::new();

        let schema1 = expected.schemas_mut().get_or_create_mut("schema1");
        schema1.tables_mut().get_or_create_mut("table1");
        schema1.tables_mut().get_or_create_mut("table3");
        schema1.tables_mut().get_or_create_mut("table2");

        let schema2 = expected.schemas_mut().get_or_create_mut("schema2");
        schema2.tables_mut().get_or_create_mut("table1");

        assert_eq!(validate(input), Ok(expected));
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

        let mut expected = ValidatedSchemas::new();

        let schema1 = expected.schemas_mut().get_or_create_mut("schema1");

        let table1 = schema1.tables_mut().get_or_create_mut("table1");
        table1.named_records_mut().get_or_create_mut("record2");
        table1.named_records_mut().get_or_create_mut("record1");

        let table2 = schema1.tables_mut().get_or_create_mut("table2");
        table2.anonymous_records_mut().create();
        table2.named_records_mut().get_or_create_mut("record1");
        table2.anonymous_records_mut().create();

        let schema2 = expected.schemas_mut().get_or_create_mut("schema2");
        schema2.tables_mut().get_or_create_mut("table1");

        assert_eq!(validate(input), Ok(expected));
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

        let mut expected = ValidatedSchemas::new();
        let schema1 = expected.schemas_mut().get_or_create_mut("schema1");
        let table1 = schema1.tables_mut().get_or_create_mut("table1");

        let attrs = table1.anonymous_records_mut().create().attributes_mut();
        attrs.add(ValidatedAttribute::new(Attribute {
            name: "attr1".to_owned(),
            value: Value::Text("Attr1".to_owned()),
        }));
        attrs.add(ValidatedAttribute::new(Attribute {
            name: "attr2".to_owned(),
            value: Value::Number("123".to_owned()),
        }));

        let attrs = table1.named_records_mut().get_or_create_mut("my_record").attributes_mut();
        attrs.add(ValidatedAttribute::new(Attribute {
            name: "attr1".to_owned(),
            value: Value::Text("Attr1".to_owned()),
        }));
        attrs.add(ValidatedAttribute::new(Attribute {
            name: "attr3".to_owned(),
            value: Value::Boolean(true),
        }));

        assert_eq!(validate(input), Ok(expected));
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

    #[test]
    fn reference_value_no_matching_record() {
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
                                value: Value::Reference(Box::new(ReferenceValue {
                                    schema: "schema2".to_owned(),
                                    table: "table2".to_owned(),
                                    record: "record2".to_owned(),
                                    column: "column2".to_owned(),
                                })),
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

    #[test]
    fn reference_value_comes_before_record() {
    }

    #[test]
    fn reference_value_good() {
    }
}
