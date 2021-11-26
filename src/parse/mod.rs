

/*
schema
  table
    name1
      col1 val
      col2 val

    name2
      col1 val

    _
      col1 val
      col2 val
      col3 val


*/
mod parser;

use super::lex::Token;
use parser::Parser;

#[derive(Debug, PartialEq)]
pub enum Value {
    Boolean(bool),
    Number(String),
    Text(String),
}

#[derive(Debug, PartialEq)]
pub struct Attribute {
    name: String,
    value: Value,
}

#[derive(Debug, PartialEq)]
pub struct Record {
    name: Option<String>,
    attributes: Vec<Attribute>,
}

impl Record {
    fn new(name: Option<String>) -> Self {
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
    fn new(name: String) -> Self {
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
    fn new(name: String) -> Self {
        Self {
            name,
            tables: Vec::new(),
        }
    }
}

pub fn parse(tokens: Vec<Token>) -> Vec<Schema> {
    Parser::new().parse(tokens).schemas
}

#[cfg(test)]
mod tests {
    use super::{*, Token as T};

    #[test]
    fn empty() {
        assert_eq!(parse(vec![]), vec![]);
    }

    #[test]
    fn schema() {
        let tokens = vec![
            T::Newline,
            T::Identifier("public".to_owned()),
        ];

        assert_eq!(parse(tokens), vec![

            Schema {
                name: "public".to_owned(),
                tables: Vec::new(),
            },
        ]);
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

        assert_eq!(parse(tokens), vec![
            Schema {
                name: "schema1".to_owned(),
                tables: Vec::new(),
            },
            Schema {
                name: "schema2".to_owned(),
                tables: Vec::new(),
            },
        ]);

        let tokens = vec![
            T::Newline,
            T::Newline,
            T::Identifier("schema1".to_owned()),
            T::Newline,
            T::Identifier("schema2".to_owned()),
        ];

        assert_eq!(parse(tokens), vec![
            Schema {
                name: "schema1".to_owned(),
                tables: Vec::new(),
            },
            Schema {
                name: "schema2".to_owned(),
                tables: Vec::new(),
            },
        ]);
    }

    #[test]
    #[should_panic(expected = "Unexpected token")]
    fn schemas_without_newlines() {
        let tokens = vec![
            T::Identifier("schema1".to_owned()),
            T::Identifier("schema2".to_owned()),
        ];

        parse(tokens);
    }

    #[test]
    fn table() {
        let tokens = vec![
            T::Identifier("public".to_owned()),
            T::Newline,
            T::Indent("  ".to_owned()),
            T::Identifier("my_table".to_owned()),
        ];

        assert_eq!(parse(tokens), vec![
            Schema {
                name: "public".to_owned(),
                tables: vec![
                    Table {
                        name: "my_table".to_owned(),
                        records: Vec::new(),
                    }
                ],
            },
        ]);
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

        assert_eq!(parse(tokens), vec![
            Schema {
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
            },
        ]);
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

        assert_eq!(parse(tokens), vec![
            Schema {
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
            },
        ]);
    }

    #[test]
    #[should_panic(expected = "Unexpected token")]
    fn tables_without_newlines() {
        let tokens = vec![
            T::Identifier("schema1".to_owned()),
            T::Newline,
            T::Indent("    ".to_owned()),
            T::Identifier("table1".to_owned()),
            T::Indent("    ".to_owned()),
            T::Identifier("table2".to_owned()),
        ];

        parse(tokens);

        let tokens = vec![
            T::Identifier("schema1".to_owned()),
            T::Newline,
            T::Indent("    ".to_owned()),
            T::Identifier("table1".to_owned()),
            T::Identifier("table2".to_owned()),
        ];

        parse(tokens);

        let tokens = vec![
            T::Identifier("schema1".to_owned()),
            T::Indent("    ".to_owned()),
            T::Identifier("table1".to_owned()),
            T::Identifier("table2".to_owned()),
        ];

        parse(tokens);
    }

    #[test]
    fn named_record() {
        let tokens = vec![
            T::Identifier("public".to_owned()),
            T::Newline,
            T::Indent("  ".to_owned()),
            T::Identifier("person".to_owned()),
            T::Newline,
            T::Indent("    ".to_owned()),
            T::Identifier("kevin".to_owned()),
        ];

        assert_eq!(parse(tokens), vec![
            Schema {
                name: "public".to_owned(),
                tables: vec![
                    Table {
                        name: "person".to_owned(),
                        records: vec![
                            Record {
                                name: Some("kevin".to_owned()),
                                attributes: Vec::new(),
                            }
                        ]
                    }
                ],
            },
        ]);
    }

    #[test]
    fn named_records() {
        let tokens = vec![
            T::Identifier("public".to_owned()),
            T::Newline,
            T::Indent("  ".to_owned()),
            T::Identifier("person".to_owned()),
            T::Newline,
            T::Indent("    ".to_owned()),
            T::Identifier("stacey".to_owned()),
            T::Newline,
            T::Indent("    ".to_owned()),
            T::Identifier("kevin".to_owned()),
            T::Newline,
            T::Indent("  ".to_owned()),
            T::Identifier("pet".to_owned()),
            T::Newline,
            T::Indent("    ".to_owned()),
            T::Identifier("eiyre".to_owned()),
            T::Newline,
            T::Indent("    ".to_owned()),
            T::Identifier("cupid".to_owned()),
        ];

        assert_eq!(parse(tokens), vec![
            Schema {
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
            },
        ]);
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

        assert_eq!(parse(tokens), vec![
            Schema {
                name: "public".to_owned(),
                tables: vec![
                    Table {
                        name: "person".to_owned(),
                        records: vec![
                            Record {
                                name: None,
                                attributes: Vec::new(),
                            }
                        ]
                    }
                ],
            },
        ]);
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

        assert_eq!(parse(tokens), vec![
            Schema {
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
            },
        ]);
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

        assert_eq!(parse(tokens), vec![
            Schema {
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
            },
        ]);
    }

    //#[test]
    fn single_record() {
        let tokens = vec![
            T::Identifier("public".to_owned()),
            T::Newline,

            T::Indent("  ".to_owned()),
            T::Identifier("person".to_owned()),
            T::Newline,

            T::Indent("    ".to_owned()),
            T::Identifier("kevin".to_owned()),
            T::Newline,

            T::Indent("      ".to_owned()),
            T::Identifier("name".to_owned()),
            T::Text("Kevin".to_owned()),
            T::Newline,
        ];

        assert_eq!(parse(tokens), vec![
            Schema {
                name: "public".to_owned(),
                tables: vec![
                    Table {
                        name: "person".to_owned(),
                        records: vec![
                            Record {
                                name: Some("kevin".to_owned()),
                                attributes: vec![
                                    Attribute {
                                        name: "name".to_owned(),
                                        value: Value::Text("Kevin".to_owned()),
                                    }
                                ]
                            }
                        ]
                    }
                ]
            }
        ]);
    }
}
