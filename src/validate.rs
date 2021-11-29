use super::parse::*;

#[derive(Debug, PartialEq)]
pub struct ValidatedSchemas(Vec<Schema>);

impl ValidatedSchemas {
    pub fn schemas(&self) -> &Vec<Schema> {
        &self.0
    }
}

pub fn validate(schemas: Vec<Schema>) -> ValidatedSchemas {
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
            let validated_table = match validated_schema.tables.iter_mut().find(|t| t.name == table.name) {
                Some(t) => t,
                None => {
                    validated_schema.tables.push(Table::new(table.name));
                    validated_schema.tables.last_mut().unwrap()
                }
            };

            for record in table.records {
                if record.name.is_some() {
                    assert!(
                        validated_table.records.iter().find(|r| r.name == record.name).is_none(),
                        "Duplicate record '{}' in '{}.{}'",
                        record.name.unwrap(),
                        validated_schema.name,
                        validated_table.name,
                    );
                }

                validated_table.records.push(Record::new(record.name));
                let validated_record = validated_table.records.last_mut().unwrap();

                for attribute in record.attributes {
                    assert!(
                        validated_record.attributes.iter().find(|a| a.name == attribute.name).is_none(),
                        "Duplicate attribute '{}' for record '{}' in '{}.{}'",
                        attribute.name,
                        validated_record.name.clone().unwrap_or("_".to_owned()),
                        validated_schema.name,
                        validated_table.name,
                    );

                    validated_record.attributes.push(attribute);
                }
            }
        }
    }

    ValidatedSchemas(validated)
}

#[cfg(test)]
mod validate_tests {
    use super::*;

    #[test]
    fn empty_is_valid() {
        assert_eq!(validate(Vec::new()), ValidatedSchemas(Vec::new()));
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

        assert_eq!(validate(input), ValidatedSchemas(output));
    }

    #[test]
    fn dedupe_schemas() {
        let input = vec![
            Schema::new("schema1".to_owned()),
            Schema::new("schema1".to_owned()),
        ];
        let output = vec![
            Schema::new("schema1".to_owned()),
        ];

        assert_eq!(validate(input), ValidatedSchemas(output));

        let input = vec![
            Schema::new("schema1".to_owned()),
            Schema::new("schema2".to_owned()),
            Schema::new("schema1".to_owned()),
        ];
        let output = vec![
            Schema::new("schema1".to_owned()),
            Schema::new("schema2".to_owned()),
        ];

        assert_eq!(validate(input), ValidatedSchemas(output));
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
                tables: vec![
                    Table::new("table1".to_owned()),
                ],
            },
            Schema {
                name: "schema1".to_owned(),
                tables: vec![
                    Table::new("table2".to_owned()),
                ],
            },
        ];
        let output = vec![
            Schema {
                name: "schema1".to_owned(),
                tables: vec![
                    Table::new("table1".to_owned()),
                    Table::new("table3".to_owned()),
                    Table::new("table2".to_owned()),
                ]
            },
            Schema {
                name: "schema2".to_owned(),
                tables: vec![
                    Table::new("table1".to_owned()),
                ]
            },
        ];

        assert_eq!(validate(input), ValidatedSchemas(output));
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
                        records: vec![
                            Record::new(Some("record2".to_owned())),
                        ],
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
                tables: vec![
                    Table::new("table1".to_owned()),
                ],
            },
            Schema {
                name: "schema1".to_owned(),
                tables: vec![
                    Table {
                        name: "table1".to_owned(),
                        records: vec![
                            Record::new(Some("record1".to_owned())),
                        ],
                    },
                ],
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
                tables: vec![
                    Table::new("table1".to_owned()),
                ]
            },
        ];

        assert_eq!(validate(input), ValidatedSchemas(output));
    }

    #[test]
    #[should_panic(expected = "Duplicate record 'record1' in 'schema1.table1'")]
    fn duplicate_record_names() {
        validate(vec![
            Schema {
                name: "schema1".to_owned(),
                tables: vec![
                    Table {
                        name: "table1".to_owned(),
                        records: vec![
                            Record::new(Some("record1".to_owned())),
                            Record::new(Some("record1".to_owned())),
                        ],
                    },
                ],
            }
        ]);
    }

    #[test]
    fn attributes() {
        let input = vec![
            Schema {
                name: "schema1".to_owned(),
                tables: vec![
                    Table {
                        name: "table1".to_owned(),
                        records: vec![
                            Record {
                                name: None,
                                attributes: vec![
                                    Attribute { name: "attr1".to_owned(), value: Value::Text("Attr1".to_owned()) },
                                    Attribute { name: "attr2".to_owned(), value: Value::Number("123".to_owned()) },
                                ],
                            },
                        ],
                    },
                ],
            },
            Schema {
                name: "schema1".to_owned(),
                tables: vec![
                    Table {
                        name: "table1".to_owned(),
                        records: vec![
                            Record {
                                name: Some("my_record".to_owned()),
                                attributes: vec![
                                    Attribute { name: "attr1".to_owned(), value: Value::Text("Attr1".to_owned()) },
                                    Attribute { name: "attr3".to_owned(), value: Value::Boolean(true) },
                                ],
                            },
                        ],
                    },
                ],
            },
        ];
        let output = vec![
            Schema {
                name: "schema1".to_owned(),
                tables: vec![
                    Table {
                        name: "table1".to_owned(),
                        records: vec![
                            Record {
                                name: None,
                                attributes: vec![
                                    Attribute { name: "attr1".to_owned(), value: Value::Text("Attr1".to_owned()) },
                                    Attribute { name: "attr2".to_owned(), value: Value::Number("123".to_owned()) },
                                ],
                            },
                            Record {
                                name: Some("my_record".to_owned()),
                                attributes: vec![
                                    Attribute { name: "attr1".to_owned(), value: Value::Text("Attr1".to_owned()) },
                                    Attribute { name: "attr3".to_owned(), value: Value::Boolean(true) },
                                ],
                            },
                        ],
                    },
                ],
            }
        ];

        assert_eq!(validate(input), ValidatedSchemas(output));
    }

    #[test]
    #[should_panic(expected = "Duplicate attribute 'attr1' for record '_' in 'schema1.table1'")]
    fn duplicate_attribute_names_anonymous() {
        validate(vec![
            Schema {
                name: "schema1".to_owned(),
                tables: vec![
                    Table {
                        name: "table1".to_owned(),
                        records: vec![
                            Record {
                                name: None,
                                attributes: vec![
                                    Attribute { name: "attr1".to_owned(), value: Value::Text("Attr1-a".to_owned()) },
                                    Attribute { name: "attr2".to_owned(), value: Value::Text("Attr2".to_owned()) },
                                    Attribute { name: "attr1".to_owned(), value: Value::Text("Attr1-b".to_owned()) },
                                ],
                            },
                        ],
                    },
                ],
            }
        ]);
    }

    #[test]
    #[should_panic(expected = "Duplicate attribute 'attr1' for record 'my_record' in 'schema1.table1'")]
    fn duplicate_attribute_names_named_record() {
        validate(vec![
            Schema {
                name: "schema1".to_owned(),
                tables: vec![
                    Table {
                        name: "table1".to_owned(),
                        records: vec![
                            Record {
                                name: Some("my_record".to_owned()),
                                attributes: vec![
                                    Attribute { name: "attr1".to_owned(), value: Value::Text("Attr1-a".to_owned()) },
                                    Attribute { name: "attr2".to_owned(), value: Value::Text("Attr2".to_owned()) },
                                    Attribute { name: "attr1".to_owned(), value: Value::Text("Attr1-b".to_owned()) },
                                ],
                            },
                        ],
                    },
                ],
            }
        ]);
    }
}
