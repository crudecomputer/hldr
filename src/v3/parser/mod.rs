mod errors;
pub mod nodes;
mod states;

use errors::*;
use super::lexer::Token;

pub fn parse(input: impl Iterator<Item = Token>) -> Result<nodes::ParseTree, ParseError> {
    let mut context = states::Context::default();
    context.stack.push(states::StackItem::TreeRoot(Box::new(nodes::ParseTree::default())));
    let mut state: Box<dyn states::State> = Box::new(states::Root);

    for token in input {
        println!("state {:?} receiving token {:?}", state, token);
        state = state.receive(&mut context, token)?;
    }

    // TODO: Is there a better finalizer token or strategy?
    state.receive(&mut context, Token::LineSep)?;

    match context.stack.pop() {
        Some(states::StackItem::TreeRoot(tree)) => Ok(*tree),
        elt => panic!("Unexpected element on top of final stack: {:?}", elt),
    }
}

#[cfg(test)]
mod tests {
    use crate::v3::lexer::{Keyword as Kwd, Symbol as Sym, Token as Tkn};
    use super::nodes::*;
    use super::*;

    #[test]
    fn test_empty_input() {
        let input: Vec<Token> = Vec::new();
        assert_eq!(parse(input.into_iter()), Ok(ParseTree::default()));
    }

    #[test]
    fn test_empty_schema() {
        let input = vec![
            Tkn::Keyword(Kwd::Schema),
            Tkn::Identifier("my_schema".to_owned()),
            Tkn::Symbol(Sym::ParenLeft),
            Tkn::Symbol(Sym::ParenRight),
        ];
        assert_eq!(parse(input.into_iter()), Ok(ParseTree {
            nodes: vec![
                StructuralNode::Schema(Box::new(Schema {
                    alias: None,
                    name: "my_schema".to_owned(),
                    nodes: Vec::new(),
                })),
            ],
        }));
    }

    #[test]
    fn test_empty_schema_with_alias() {
        let input = vec![
            Tkn::Keyword(Kwd::Schema),
            Tkn::Identifier("my_other_schema".to_owned()),
            Tkn::Keyword(Kwd::As),
            Tkn::Identifier("some_alias".to_owned()),
            Tkn::Symbol(Sym::ParenLeft),
            Tkn::Symbol(Sym::ParenRight),
        ];
        assert_eq!(parse(input.into_iter()), Ok(ParseTree {
            nodes: vec![
                StructuralNode::Schema(Box::new(Schema {
                    alias: Some("some_alias".to_owned()),
                    name: "my_other_schema".to_owned(),
                    nodes: Vec::new(),
                })),
            ],
        }));
    }

    #[test]
    fn test_empty_top_level_table() {
        let input = vec![
            Tkn::Keyword(Kwd::Table),
            Tkn::Identifier("my_table".to_owned()),
            Tkn::Symbol(Sym::ParenLeft),
            Tkn::Symbol(Sym::ParenRight),
        ];
        assert_eq!(parse(input.into_iter()), Ok(ParseTree {
            nodes: vec![
                StructuralNode::Table(Box::new(Table {
                    alias: None,
                    name: "my_table".to_owned(),
                    nodes: Vec::new(),
                    schema: None,
                })),
            ],
        }));
    }

    #[test]
    fn test_empty_top_level_table_with_alias() {
        let input = vec![
            Tkn::Keyword(Kwd::Table),
            Tkn::Identifier("my_other_table".to_owned()),
            Tkn::Keyword(Kwd::As),
            Tkn::Identifier("some_alias".to_owned()),
            Tkn::Symbol(Sym::ParenLeft),
            Tkn::Symbol(Sym::ParenRight),
        ];
        assert_eq!(parse(input.into_iter()), Ok(ParseTree {
            nodes: vec![
                StructuralNode::Table(Box::new(Table {
                    alias: Some("some_alias".to_owned()),
                    name: "my_other_table".to_owned(),
                    nodes: Vec::new(),
                    schema: None,
                })),
            ],
        }));
    }

    #[test]
    fn test_empty_qualified_table() {
        let input = vec![
            // Declare the schema
            Tkn::Keyword(Kwd::Schema),
            Tkn::Identifier("myschema".to_owned()),

            // Open the schema
            Tkn::Symbol(Sym::ParenLeft),

            // Declare the table
            Tkn::Keyword(Kwd::Table),
            Tkn::Identifier("mytable".to_owned()),

            // Open & close the table
            Tkn::Symbol(Sym::ParenLeft),
            Tkn::Symbol(Sym::ParenRight),

            // Close the schema
            Tkn::Symbol(Sym::ParenRight),
        ];
        assert_eq!(parse(input.into_iter()), Ok(ParseTree {
            nodes: vec![
                StructuralNode::Schema(Box::new(Schema {
                    alias: None,
                    name: "myschema".to_owned(),
                    nodes: vec![
                        Table {
                            alias: None,
                            name: "mytable".to_owned(),
                            nodes: Vec::new(),
                            schema: None,
                        },
                    ],
                })),
            ],
        }));
    }

    #[test]
    fn test_empty_qualified_table_with_aliases() {
        let input = vec![
            // Declare the schema
            Tkn::Keyword(Kwd::Schema),
            Tkn::Identifier("myschema".to_owned()),
            Tkn::Keyword(Kwd::As),
            Tkn::Identifier("s".to_owned()),

            // Open the schema
            Tkn::Symbol(Sym::ParenLeft),

            // Declare the table
            Tkn::Keyword(Kwd::Table),
            Tkn::Identifier("mytable".to_owned()),
            Tkn::Keyword(Kwd::As),
            Tkn::Identifier("t".to_owned()),

            // Open & close the table
            Tkn::Symbol(Sym::ParenLeft),
            Tkn::Symbol(Sym::ParenRight),

            // Close the schema
            Tkn::Symbol(Sym::ParenRight),
        ];
        assert_eq!(parse(input.into_iter()), Ok(ParseTree {
            nodes: vec![
                StructuralNode::Schema(Box::new(Schema {
                    alias: Some("s".to_owned()),
                    name: "myschema".to_owned(),
                    nodes: vec![
                        Table {
                            alias: Some("t".to_owned()),
                            name: "mytable".to_owned(),
                            nodes: Vec::new(),
                            schema: None, // FIXME: Should this field exist? If so, populate it.
                        },
                    ],
                })),
            ],
        }));
    }

    #[test]
    fn test_empty_records() {
        let input = vec![
            Tkn::Keyword(Kwd::Table),
            Tkn::Identifier("mytable".to_owned()),
            Tkn::Symbol(Sym::ParenLeft),

            // Declare and close a named record
            Tkn::Identifier("myrecord".to_owned()),
            Tkn::Symbol(Sym::ParenLeft),
            Tkn::Symbol(Sym::ParenRight),

            // Declare and close an explicit anonymous record
            Tkn::Symbol(Sym::Underscore),
            Tkn::Symbol(Sym::ParenLeft),
            Tkn::Symbol(Sym::ParenRight),

            // Declare and close an implicit anonymous record
            Tkn::Symbol(Sym::ParenLeft),
            Tkn::Symbol(Sym::ParenRight),

            // Close the table
            Tkn::Symbol(Sym::ParenRight),
        ];
        assert_eq!(parse(input.into_iter()), Ok(ParseTree {
            nodes: vec![
                StructuralNode::Table(Box::new(Table {
                    alias: None,
                    name: "mytable".to_owned(),
                    nodes: vec![
                        Record {
                            name: Some("myrecord".to_owned()),
                            nodes: Vec::new(),
                        },
                        Record::default(),
                        Record::default(),
                    ],
                    schema: None,
                })),
            ],
        }));
    }

    #[test]
    #[rustfmt::skip]
    fn test_record_with_values() {
        let input = vec![
            Tkn::Keyword(Kwd::Table),
            Tkn::Identifier("mytable".to_owned()),
            Tkn::Symbol(Sym::ParenLeft),

                // Named record
                Tkn::Identifier("myrecord".to_owned()),
                Tkn::Symbol(Sym::ParenLeft),

                    Tkn::Identifier("column_one".to_owned()),
                    Tkn::Number("123".to_owned()),
                    Tkn::LineSep,

                    Tkn::Identifier("column_two".to_owned()),
                    Tkn::Text("Hello, world!".to_owned()),
                    Tkn::LineSep,

                Tkn::Symbol(Sym::ParenRight),

                // Open anonymous record
                Tkn::Symbol(Sym::ParenLeft),

                    Tkn::Identifier("column_one".to_owned()),
                    Tkn::Bool(true),
                    Tkn::Symbol(Sym::Comma),

                    // Column reference
                    Tkn::Identifier("column_two".to_owned()),
                    Tkn::Symbol(Sym::AtSign),
                    Tkn::Identifier("col1".to_owned()),

                    Tkn::Symbol(Sym::Comma),

                    // Record-qualified reference
                    Tkn::Identifier("column_two".to_owned()),
                    Tkn::Symbol(Sym::AtSign),
                    Tkn::Identifier("myrecord".to_owned()),
                    Tkn::Symbol(Sym::Period),
                    Tkn::Identifier("col1".to_owned()),

                    Tkn::LineSep,

                    // Table-qualified reference
                    Tkn::Identifier("column_two".to_owned()),
                    Tkn::Symbol(Sym::AtSign),
                    Tkn::Identifier("mytable".to_owned()),
                    Tkn::Symbol(Sym::Period),
                    Tkn::Identifier("myrecord".to_owned()),
                    Tkn::Symbol(Sym::Period),
                    Tkn::Identifier("col2".to_owned()),

                    Tkn::Symbol(Sym::Comma),

                    // Schema-qualified reference
                    Tkn::Identifier("column_two".to_owned()),
                    Tkn::Symbol(Sym::AtSign),
                    Tkn::Identifier("myschema".to_owned()),
                    Tkn::Symbol(Sym::Period),
                    Tkn::Identifier("mytable".to_owned()),
                    Tkn::Symbol(Sym::Period),
                    Tkn::Identifier("myrecord".to_owned()),
                    Tkn::Symbol(Sym::Period),
                    Tkn::Identifier("col3".to_owned()),

                    Tkn::LineSep,

                    // Schema-qualified reference with quoted identifiers
                    Tkn::Identifier("column_two".to_owned()),
                    Tkn::Symbol(Sym::AtSign),
                    Tkn::QuotedIdentifier("myschema".to_owned()),
                    Tkn::Symbol(Sym::Period),
                    Tkn::QuotedIdentifier("mytable".to_owned()),
                    Tkn::Symbol(Sym::Period),
                    Tkn::Identifier("myrecord".to_owned()),
                    Tkn::Symbol(Sym::Period),
                    Tkn::Identifier("col4".to_owned()),

                    Tkn::Symbol(Sym::Comma),

                Tkn::Symbol(Sym::ParenRight),

            // Close the table
            Tkn::Symbol(Sym::ParenRight),
        ];
        assert_eq!(parse(input.into_iter()), Ok(ParseTree {
            nodes: vec![
                StructuralNode::Table(Box::new(Table {
                    alias: None,
                    name: "mytable".to_owned(),
                    nodes: vec![
                        Record {
                            name: Some("myrecord".to_owned()),
                            nodes: vec![
                                Attribute {
                                    name: "column_one".to_owned(),
                                    value: Value::Number(Box::new("123".to_owned())),
                                },
                                Attribute {
                                    name: "column_two".to_owned(),
                                    value: Value::Text(Box::new("Hello, world!".to_owned())),
                                },
                            ],
                        },
                        Record {
                            name: None,
                            nodes: vec![
                                Attribute {
                                    name: "column_one".to_owned(),
                                    value: Value::Bool(true),
                                },
                                Attribute {
                                    name: "column_two".to_owned(),
                                    value: Value::Reference(
                                        Box::new(
                                            Reference {
                                                schema: None,
                                                table: None,
                                                record: None,
                                                column: "col1".to_owned(),
                                            },
                                        ),
                                    ),
                                },
                                Attribute {
                                    name: "column_two".to_owned(),
                                    value: Value::Reference(
                                        Box::new(
                                            Reference {
                                                schema: None,
                                                table: None,
                                                record: Some("myrecord".to_owned()),
                                                column: "col1".to_owned(),
                                            },
                                        ),
                                    ),
                                },
                                Attribute {
                                    name: "column_two".to_owned(),
                                    value: Value::Reference(
                                        Box::new(
                                            Reference {
                                                schema: None,
                                                table: Some("mytable".to_owned()),
                                                record: Some("myrecord".to_owned()),
                                                column: "col2".to_owned(),
                                            },
                                        ),
                                    ),
                                },
                                Attribute {
                                    name: "column_two".to_owned(),
                                    value: Value::Reference(
                                        Box::new(
                                            Reference {
                                                schema: Some("myschema".to_owned()),
                                                table: Some("mytable".to_owned()),
                                                record: Some("myrecord".to_owned()),
                                                column: "col3".to_owned(),
                                            },
                                        ),
                                    ),
                                },
                                Attribute {
                                    name: "column_two".to_owned(),
                                    value: Value::Reference(
                                        Box::new(
                                            Reference {
                                                schema: Some("myschema".to_owned()),
                                                table: Some("mytable".to_owned()),
                                                record: Some("myrecord".to_owned()),
                                                column: "col4".to_owned(),
                                            },
                                        ),
                                    ),
                                },
                            ],
                        },
                    ],
                    schema: None,
                })),
            ],
        }));
    }
}
