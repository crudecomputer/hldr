mod errors;
mod nodes;
mod states;

use errors::*;
use super::lexer::Token;

pub fn parse(input: impl Iterator<Item = Token>) -> Result<nodes::ParseTree, ParseError> {
    let mut context = states::Context::default();
    context.stack.push(states::StackItem::TreeRoot(Box::new(nodes::ParseTree::default())));
    let mut state: Box<dyn states::State> = Box::new(states::Root);

    for token in input {
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
    use crate::v3::lexer::{Keyword as Kwd, Symbol as Sym, Token as Tkn, tokenize};
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
            Tkn::Identifier("my_schema".to_string()),
            Tkn::Symbol(Sym::ParenLeft),
            Tkn::Symbol(Sym::ParenRight),
        ];
        assert_eq!(parse(input.into_iter()), Ok(ParseTree {
            nodes: vec![
                StructuralNode::Schema(Box::new(Schema {
                    alias: None,
                    name: "my_schema".to_string(),
                    nodes: Vec::new(),
                })),
            ],
        }));
    }

    #[test]
    fn test_empty_schema_with_alias() {
        let input = vec![
            Tkn::Keyword(Kwd::Schema),
            Tkn::Identifier("my_other_schema".to_string()),
            Tkn::Keyword(Kwd::As),
            Tkn::Identifier("some_alias".to_string()),
            Tkn::Symbol(Sym::ParenLeft),
            Tkn::Symbol(Sym::ParenRight),
        ];
        assert_eq!(parse(input.into_iter()), Ok(ParseTree {
            nodes: vec![
                StructuralNode::Schema(Box::new(Schema {
                    alias: Some("some_alias".to_string()),
                    name: "my_other_schema".to_string(),
                    nodes: Vec::new(),
                })),
            ],
        }));
    }

    #[test]
    fn test_empty_top_level_table() {
        let input = vec![
            Tkn::Keyword(Kwd::Table),
            Tkn::Identifier("my_table".to_string()),
            Tkn::Symbol(Sym::ParenLeft),
            Tkn::Symbol(Sym::ParenRight),
        ];
        assert_eq!(parse(input.into_iter()), Ok(ParseTree {
            nodes: vec![
                StructuralNode::Table(Box::new(Table {
                    alias: None,
                    name: "my_table".to_string(),
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
            Tkn::Identifier("my_other_table".to_string()),
            Tkn::Keyword(Kwd::As),
            Tkn::Identifier("some_alias".to_string()),
            Tkn::Symbol(Sym::ParenLeft),
            Tkn::Symbol(Sym::ParenRight),
        ];
        assert_eq!(parse(input.into_iter()), Ok(ParseTree {
            nodes: vec![
                StructuralNode::Table(Box::new(Table {
                    alias: Some("some_alias".to_string()),
                    name: "my_other_table".to_string(),
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
            Tkn::Identifier("myschema".to_string()),

            // Open the schema
            Tkn::Symbol(Sym::ParenLeft),

            // Declare the table
            Tkn::Keyword(Kwd::Table),
            Tkn::Identifier("mytable".to_string()),

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
                    name: "myschema".to_string(),
                    nodes: vec![
                        Table {
                            alias: None,
                            name: "mytable".to_string(),
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
            Tkn::Identifier("myschema".to_string()),
            Tkn::Keyword(Kwd::As),
            Tkn::Identifier("s".to_string()),

            // Open the schema
            Tkn::Symbol(Sym::ParenLeft),

            // Declare the table
            Tkn::Keyword(Kwd::Table),
            Tkn::Identifier("mytable".to_string()),
            Tkn::Keyword(Kwd::As),
            Tkn::Identifier("t".to_string()),

            // Open & close the table
            Tkn::Symbol(Sym::ParenLeft),
            Tkn::Symbol(Sym::ParenRight),

            // Close the schema
            Tkn::Symbol(Sym::ParenRight),
        ];
        assert_eq!(parse(input.into_iter()), Ok(ParseTree {
            nodes: vec![
                StructuralNode::Schema(Box::new(Schema {
                    alias: Some("s".to_string()),
                    name: "myschema".to_string(),
                    nodes: vec![
                        Table {
                            alias: Some("t".to_string()),
                            name: "mytable".to_string(),
                            nodes: Vec::new(),
                            schema: None,
                        },
                    ],
                })),
            ],
        }));
    }
}
