pub mod error;
pub mod nodes;
mod states;

use super::lexer::tokens::Token;

use error::{ParseError, ParseErrorKind};

pub fn parse(input: impl Iterator<Item = Token>) -> Result<nodes::ParseTree, ParseError> {
    let mut context = states::Context::default();
    context
        .stack
        .push(states::StackItem::TreeRoot(Box::default()));
    let mut state: Box<dyn states::State> = Box::new(states::Root);

    for token in input {
        state = state.receive(&mut context, Some(token))?;
    }

    state.receive(&mut context, None)?;

    match context.stack.pop() {
        Some(states::StackItem::TreeRoot(tree)) => Ok(*tree),
        _ => Err(ParseError {
            kind: ParseErrorKind::UnexpectedEOF,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::parse;
    use crate::lexer::tokenize;
    use crate::lexer::tokens::Token;
    use crate::parser::nodes::*;

    fn tokens(input: &str) -> impl Iterator<Item = Token> {
        tokenize(input.chars()).unwrap().into_iter()
    }

    #[test]
    fn test_empty_input() {
        let input: Vec<Token> = Vec::new();
        assert_eq!(parse(input.into_iter()), Ok(ParseTree::default()));
    }

    #[test]
    fn test_empty_schema() {
        /*
        let position = Position::default();
        let input = vec![
            Tkn { position, kind: TKnd::Keyword(Kwd::Schema) },
            Tkn { position, kind: TKnd::Identifier("my_schema".to_owned()) },
            Tkn { position, kind: TKnd::Symbol(Sym::ParenLeft) },
            Tkn { position, kind: TKnd::Symbol(Sym::ParenRight) },
        ];
         */

        let input = tokens("schema my_schema ()");

        assert_eq!(
            parse(input),
            Ok(ParseTree {
                nodes: vec![StructuralNode::Schema(Box::new(Schema {
                    identity: StructuralIdentity {
                        alias: None,
                        name: "my_schema".to_owned(),
                    },
                    nodes: Vec::new(),
                })),],
            }),
        );
    }

    #[test]
    fn test_empty_schema_with_alias() {
        /*
        let position = Position::default();
        let input = vec![
            Tkn { position, kind: TKnd::Keyword(Kwd::Schema) },
            Tkn { position, kind: TKnd::Identifier("my_other_schema".to_owned()) },
            Tkn { position, kind: TKnd::Keyword(Kwd::As) },
            Tkn { position, kind: TKnd::Identifier("some_alias".to_owned()) },
            Tkn { position, kind: TKnd::Symbol(Sym::ParenLeft) },
            Tkn { position, kind: TKnd::Symbol(Sym::ParenRight) },
        ];
        */

        let input = tokens("schema my_other_schema as some_alias ()");

        assert_eq!(
            parse(input),
            Ok(ParseTree {
                nodes: vec![StructuralNode::Schema(Box::new(Schema {
                    identity: StructuralIdentity {
                        alias: Some("some_alias".to_owned()),
                        name: "my_other_schema".to_owned(),
                    },
                    nodes: Vec::new(),
                })),],
            }),
        );
    }

    #[test]
    fn test_empty_top_level_table() {
        /*
        let position = Position::default();
        let input = vec![
            Tkn {
                kind: TKnd::Keyword(Kwd::Table),
                position,
            },
            Tkn {
                kind: TKnd::Identifier("my_table".to_owned()),
                position,
            },
            Tkn {
                kind: TKnd::Symbol(Sym::ParenLeft),
                position,
            },
            Tkn {
                kind: TKnd::Symbol(Sym::ParenRight),
                position,
            },
        ];
        */

        let input = tokens("table my_table ()");

        assert_eq!(
            parse(input),
            Ok(ParseTree {
                nodes: vec![StructuralNode::Table(Box::new(Table {
                    identity: StructuralIdentity {
                        alias: None,
                        name: "my_table".to_owned(),
                    },
                    nodes: Vec::new(),
                })),],
            }),
        );
    }

    #[test]
    fn test_empty_top_level_table_with_alias() {
        /*
        let position = Position::default();
        let input = vec![
            Tkn {
                kind: TKnd::Keyword(Kwd::Table),
                position,
            },
            Tkn {
                kind: TKnd::Identifier("my_other_table".to_owned()),
                position,
            },
            Tkn {
                kind: TKnd::Keyword(Kwd::As),
                position,
            },
            Tkn {
                kind: TKnd::Identifier("some_alias".to_owned()),
                position,
            },
            Tkn {
                kind: TKnd::Symbol(Sym::ParenLeft),
                position,
            },
            Tkn {
                kind: TKnd::Symbol(Sym::ParenRight),
                position,
            },
        ];
        */

        let input = tokens("table my_other_table as another_alias ()");

        assert_eq!(
            parse(input.into_iter()),
            Ok(ParseTree {
                nodes: vec![StructuralNode::Table(Box::new(Table {
                    identity: StructuralIdentity {
                        alias: Some("another_alias".to_owned()),
                        name: "my_other_table".to_owned(),
                    },
                    nodes: Vec::new(),
                })),],
            }),
        );
    }

    #[test]
    fn test_empty_qualified_table() {
        let input = tokens(
            "
            schema myschema (
                table mytable (
                )
            )
        ",
        );

        assert_eq!(
            parse(input),
            Ok(ParseTree {
                nodes: vec![StructuralNode::Schema(Box::new(Schema {
                    identity: StructuralIdentity {
                        alias: None,
                        name: "myschema".to_owned(),
                    },
                    nodes: vec![Table {
                        identity: StructuralIdentity {
                            alias: None,
                            name: "mytable".to_owned(),
                        },
                        nodes: Vec::new(),
                    },],
                })),],
            }),
        );
    }

    #[test]
    fn test_empty_qualified_table_with_aliases() {
        let input = tokens(
            "
            schema myschema as s1 (
                table mytable as t1 (
                )
            )
        ",
        );

        assert_eq!(
            parse(input),
            Ok(ParseTree {
                nodes: vec![StructuralNode::Schema(Box::new(Schema {
                    identity: StructuralIdentity {
                        alias: Some("s1".to_owned()),
                        name: "myschema".to_owned(),
                    },
                    nodes: vec![Table {
                        identity: StructuralIdentity {
                            alias: Some("t1".to_owned()),
                            name: "mytable".to_owned(),
                        },
                        nodes: Vec::new(),
                    },],
                })),],
            }),
        );
    }

    #[test]
    fn test_empty_records() {
        let input = tokens(
            "
            schema s1 (
                table t1 (
                    record1 ()
                    _ ()
                    ()
                )
            )
            table t2 (
                ()
                _ ()
                record2 ()
            )
        ",
        );

        assert_eq!(
            parse(input),
            Ok(ParseTree {
                nodes: vec![
                    StructuralNode::Schema(Box::new(Schema {
                        identity: StructuralIdentity {
                            alias: None,
                            name: "s1".to_owned(),
                        },
                        nodes: vec![Table {
                            identity: StructuralIdentity {
                                alias: None,
                                name: "t1".to_owned(),
                            },
                            nodes: vec![
                                Record {
                                    name: Some("record1".to_owned()),
                                    nodes: Vec::new(),
                                },
                                Record::default(),
                                Record::default(),
                            ],
                        },],
                    })),
                    StructuralNode::Table(Box::new(Table {
                        identity: StructuralIdentity {
                            alias: None,
                            name: "t2".to_owned(),
                        },
                        nodes: vec![
                            Record::default(),
                            Record::default(),
                            Record {
                                name: Some("record2".to_owned()),
                                nodes: Vec::new(),
                            },
                        ],
                    })),
                ],
            })
        );
    }

    #[test]
    fn test_records_with_values() {
        let input = tokens(
            r#"
            schema s1 (
                table t1 (
                    record1 (
                        -- literal values
                        col1 123
                        col2 true
                        col3 'hello!'

                        -- column reference
                        col4 @col3
                    )
                    (
                        -- record-qualified reference
                        col @record1.col1
                    )
                )
            )
            table t2 (
                (
                    -- schema reference
                    colx @s1.t1.record1.col2
                )
                (
                    -- with quoted identifiers
                    coly @"s1"."t1".record1."col2"
                )
                record2 (col 1234)
                _ ()
            )
            table t3 (
                -- top-level table reference
                (col @t2.record2.col)
            )
        "#,
        );

        let t1 = Table {
            identity: StructuralIdentity {
                alias: None,
                name: "t1".to_owned(),
            },
            nodes: vec![
                Record {
                    name: Some("record1".to_owned()),
                    nodes: vec![
                        Attribute {
                            name: "col1".to_owned(),
                            value: Value::Number(Box::new("123".to_owned())),
                        },
                        Attribute {
                            name: "col2".to_owned(),
                            value: Value::Bool(true),
                        },
                        Attribute {
                            name: "col3".to_owned(),
                            value: Value::Text(Box::new("'hello!'".to_owned())),
                        },
                        Attribute {
                            name: "col4".to_owned(),
                            value: Value::Reference(Box::new(Reference {
                                schema: None,
                                table: None,
                                record: None,
                                column: "col3".to_owned(),
                            })),
                        },
                    ],
                },
                Record {
                    name: None,
                    nodes: vec![Attribute {
                        name: "col".to_owned(),
                        value: Value::Reference(Box::new(Reference {
                            schema: None,
                            table: None,
                            record: Some("record1".to_owned()),
                            column: "col1".to_owned(),
                        })),
                    }],
                },
            ],
        };
        let t2 = Table {
            identity: StructuralIdentity {
                alias: None,
                name: "t2".to_owned(),
            },
            nodes: vec![
                Record {
                    name: None,
                    nodes: vec![Attribute {
                        name: "colx".to_owned(),
                        value: Value::Reference(Box::new(Reference {
                            schema: Some("s1".to_owned()),
                            table: Some("t1".to_owned()),
                            record: Some("record1".to_owned()),
                            column: "col2".to_owned(),
                        })),
                    }],
                },
                Record {
                    name: None,
                    nodes: vec![Attribute {
                        name: "coly".to_owned(),
                        value: Value::Reference(Box::new(Reference {
                            // TODO: Should these actually be explicitly quoted?
                            schema: Some("\"s1\"".to_owned()),
                            table: Some("\"t1\"".to_owned()),
                            record: Some("record1".to_owned()),
                            column: "\"col2\"".to_owned(),
                        })),
                    }],
                },
                Record {
                    name: Some("record2".to_owned()),
                    nodes: vec![Attribute {
                        name: "col".to_owned(),
                        value: Value::Number(Box::new("1234".to_owned())),
                    }],
                },
                Record::default(),
            ],
        };
        let t3 = Table {
            identity: StructuralIdentity {
                alias: None,
                name: "t3".to_owned(),
            },
            nodes: vec![Record {
                name: None,
                nodes: vec![Attribute {
                    name: "col".to_owned(),
                    value: Value::Reference(Box::new(Reference {
                        schema: None,
                        table: Some("t2".to_owned()),
                        record: Some("record2".to_owned()),
                        column: "col".to_owned(),
                    })),
                }],
            }],
        };

        let expected = Ok(ParseTree {
            nodes: vec![
                StructuralNode::Schema(Box::new(Schema {
                    identity: StructuralIdentity {
                        alias: None,
                        name: "s1".to_owned(),
                    },
                    nodes: vec![t1],
                })),
                StructuralNode::Table(Box::new(t2)),
                StructuralNode::Table(Box::new(t3)),
            ],
        });
        let result = parse(input);

        assert_eq!(result, expected);
    }
}
