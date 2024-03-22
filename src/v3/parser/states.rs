use std::mem;

use crate::v3::lexer::{
    Keyword as Kwd,
    Symbol as Sym,
    Token as Tkn,
};
use super::errors::ParseError;
use super::nodes;

type ParseResult = Result<Box<dyn State>, ParseError>;

pub trait State {
    fn receive(&mut self, ctx: &mut Context, t: Tkn) -> ParseResult;
}

fn to<S: State + 'static>(state: S) -> ParseResult {
    Ok(Box::new(state))
}

#[derive(Debug)]
pub enum StackItem {
    TreeRoot(Box<nodes::ParseTree>),
    Schema(Box<nodes::Schema>),
    Table(Box<nodes::Table>),
    Record(Box<nodes::Record>),
}

enum PushedTableTo {
    Schema,
    TreeRoot,
}

#[derive(Default)]
pub struct Context {
    pub stack: Vec<StackItem>,
}

impl Context {
    fn push_schema(&mut self, schema_name: String, alias: Option<String>) {
        let schema = nodes::Schema::new(schema_name, alias);
        self.stack.push(StackItem::Schema(Box::new(schema)));
    }

    fn push_table(&mut self, table_name: String, alias: Option<String>) {
        let table = nodes::Table::new(table_name, alias);
        self.stack.push(StackItem::Table(Box::new(table)));
    }

    fn push_record(&mut self, record_name: Option<String>) {
        let record = nodes::Record::new(record_name);
        self.stack.push(StackItem::Record(Box::new(record)));
    }

    // These utility methods all panic if certain expectations are not met, primarily
    // because that indicates faulty logic in the parser rather than unexpected
    // tokens in the token stream.
    fn pop_schema_or_panic(&mut self) -> nodes::Schema {
        match self.stack.pop() {
            Some(StackItem::Schema(schema)) => *schema,
            elt => panic!("expected schema on stack; received {:?}", elt),
        }
    }

    fn pop_table_or_panic(&mut self) -> nodes::Table {
        match self.stack.pop() {
            Some(StackItem::Table(table)) => *table,
            elt => panic!("expected table on stack; received {:?}", elt),
        }
    }

    fn pop_record_or_panic(&mut self) -> nodes::Record {
        match self.stack.pop() {
            Some(StackItem::Record(record)) => *record,
            elt => panic!("expected record on stack; received {:?}", elt),
        }
    }

    fn push_schema_to_root_or_panic(&mut self, schema: nodes::Schema) {
        match self.stack.last_mut() {
            Some(StackItem::TreeRoot(tree)) => {
                tree.nodes.push(nodes::StructuralNode::Schema(Box::new(schema)));
            }
            elt => panic!("expected tree root on stack; received {:?}", elt),
        }
    }

    fn push_table_to_parent_or_panic(&mut self, table: nodes::Table) -> PushedTableTo{
        match self.stack.last_mut() {
            Some(StackItem::TreeRoot(tree)) => {
                let node = nodes::StructuralNode::Table(Box::new(table));
                tree.nodes.push(node);
                PushedTableTo::TreeRoot
            }
            Some(StackItem::Schema(schema)) => {
                schema.nodes.push(table);
                PushedTableTo::Schema
            }
            elt => panic!("expected tree root or schema on stack; received {:?}", elt),
        }
    }

    fn push_record_to_table_or_panic(&mut self, record: nodes::Record) {
        match self.stack.last_mut() {
            Some(StackItem::Table(table)) => {
                table.nodes.push(record);
            }
            elt => panic!("expected table on stack; received {:?}", elt),
        }
    }
}

/// Root state that can expect top-level entities.
pub struct Root;

impl State for Root {
    fn receive(&mut self, ctx: &mut Context, t: Tkn) -> ParseResult {
        match t {
            // TODO: An explicit "EOF" token would likely be better
            Tkn::LineSep => {
                to(Root)
            }
            Tkn::Keyword(Kwd::Schema) => {
                to(schema_states::DeclaringSchema)
            }
            Tkn::Keyword(Kwd::Table) => {
                to(table_states::DeclaringTable)
            }
            _ => Err(ParseError),
        }
    }
}

mod schema_states {
    use super::*;

    /// State after receiving the `schema` keyword for declaration.
    pub struct DeclaringSchema;

    impl State for DeclaringSchema {
        fn receive(&mut self, ctx: &mut Context, t: Tkn) -> ParseResult {
            match t {
                Tkn::Identifier(ident) | Tkn::QuotedIdentifier(ident) => {
                    to(ReceivedSchemaName(ident))
                }
                _ => Err(ParseError),
            }
        }
    }

    /// State after receiving the schema name during declaration.
    struct ReceivedSchemaName(String);

    impl State for ReceivedSchemaName {
        fn receive(&mut self, ctx: &mut Context, t: Tkn) -> ParseResult {
            let schema_name = mem::take(&mut self.0);

            match t {
                Tkn::Keyword(Kwd::As) => {
                    to(DeclaringSchemaAlias(schema_name))
                }
                Tkn::Symbol(Sym::ParenLeft) => {
                    ctx.push_schema(schema_name, None);
                    to(InSchemaScope)
                }
                _ => Err(ParseError),
            }
        }
    }

    /// State after receiving the `as` keyword during schema declaration.
    struct DeclaringSchemaAlias(String);

    impl State for DeclaringSchemaAlias {
        fn receive(&mut self, ctx: &mut Context, t: Tkn) -> ParseResult {
            let schema_name = mem::take(&mut self.0);

            match t {
                // Unlike the true database name, aliases do not support quoted identifiers
                Tkn::Identifier(ident) => {
                    to(ReceivedSchemaAlias(schema_name, ident))
                }
                _ => Err(ParseError),
            }
        }
    }

    struct ReceivedSchemaAlias(String, String);

    impl State for ReceivedSchemaAlias {
        fn receive(&mut self, ctx: &mut Context, t: Tkn) -> ParseResult {
            match t {
                Tkn::Symbol(Sym::ParenLeft) => {
                    let schema_name = mem::take(&mut self.0);
                    let alias = mem::take(&mut self.1);
                    ctx.push_schema(schema_name, Some(alias));
                    to(InSchemaScope)
                }
                _ => Err(ParseError),
            }
        }
    }

    /// State
    pub struct InSchemaScope;

    impl State for InSchemaScope {
        fn receive(&mut self, ctx: &mut Context, t: Tkn) -> ParseResult {
            match t {
                Tkn::Symbol(Sym::ParenRight) => {
                    let schema = ctx.pop_schema_or_panic();
                    ctx.push_schema_to_root_or_panic(schema);
                    to(Root)
                }
                Tkn::Keyword(Kwd::Table) => {
                    to(table_states::DeclaringTable)
                }
                _ => Err(ParseError),
            }
        }
    }
}

mod table_states {
    use super::*;

    /// State after receiving the `table` keyword for declaration.
    pub struct DeclaringTable;

    impl State for DeclaringTable {
        fn receive(&mut self, ctx: &mut Context, t: Tkn) -> ParseResult {
            match t {
                Tkn::Identifier(ident) | Tkn::QuotedIdentifier(ident) => {
                    to(ReceivedTableName(ident))
                }
                _ => Err(ParseError),
            }
        }
    }

    /// State after receiving the table name during declaration.
    struct ReceivedTableName(String);

    impl State for ReceivedTableName {
        fn receive(&mut self, ctx: &mut Context, t: Tkn) -> ParseResult {
            let table_name = mem::take(&mut self.0);

            match t {
                Tkn::Keyword(Kwd::As) => {
                    to(DeclaringTableAlias(table_name))
                }
                Tkn::Symbol(Sym::ParenLeft) => {
                    ctx.push_table(table_name, None);
                    to(InTableScope)
                }
                _ => Err(ParseError),
            }
        }
    }

    struct DeclaringTableAlias(String);

    impl State for DeclaringTableAlias {
        fn receive(&mut self, ctx: &mut Context, t: Tkn) -> ParseResult {
            let table_name = mem::take(&mut self.0);

            match t {
                Tkn::Identifier(ident) => {
                    to(ReceivedTableAlias(table_name, ident))
                }
                _ => Err(ParseError),
            }
        }
    }

    struct ReceivedTableAlias(String, String);

    impl State for ReceivedTableAlias {
        fn receive(&mut self, ctx: &mut Context, t: Tkn) -> ParseResult {
            match t {
                Tkn::Symbol(Sym::ParenLeft) => {
                    let table_name = mem::take(&mut self.0);
                    let alias = mem::take(&mut self.1);
                    ctx.push_table(table_name, Some(alias));
                    to(InTableScope)
                }
                _ => Err(ParseError),
            }
        }
    }

    pub struct InTableScope;

    impl State for InTableScope {
        fn receive(&mut self, ctx: &mut Context, t: Tkn) -> ParseResult {
            match t {
                Tkn::Symbol(Sym::ParenRight) => {
                    let table = ctx.pop_table_or_panic();

                    match ctx.push_table_to_parent_or_panic(table) {
                        PushedTableTo::TreeRoot => to(Root),
                        PushedTableTo::Schema => to(schema_states::InSchemaScope),
                    }
                }
                Tkn::Identifier(ident) => {
                    to(record_states::ReceivedRecordName(ident))
                }
                Tkn::Symbol(Sym::Underscore) => {
                    to(record_states::ReceivedExplicitAnonymousRecord)
                }
                Tkn::Symbol(Sym::ParenLeft) => {
                    ctx.push_record(None);
                    to(record_states::InRecordScope)
                }
                _ => Err(ParseError),
            }
        }
    }
}

mod record_states {
   use super::*;

   /// State after receiving a record name in the table scope.
   pub struct ReceivedRecordName(pub String);

   impl State for ReceivedRecordName {
        fn receive(&mut self, ctx: &mut Context, t: Tkn) -> ParseResult {
            let record_name = mem::take(&mut self.0);

            match t {
                Tkn::Symbol(Sym::ParenLeft) => {
                    ctx.push_record(Some(record_name));
                    println!("{:?}", ctx.stack);
                    to(InRecordScope)
                }
                _ => Err(ParseError),
            }
        }
    }

    /// State after receiving an `_` in the table scope.
    pub struct ReceivedExplicitAnonymousRecord;

    impl State for ReceivedExplicitAnonymousRecord {
        fn receive(&mut self, ctx: &mut Context, t: Tkn) -> ParseResult {
            match t {
                Tkn::Symbol(Sym::ParenLeft) => {
                    ctx.push_record(None);
                    to(InRecordScope)
                }
                _ => Err(ParseError),
            }
        }
    }

    pub struct InRecordScope;

    impl State for InRecordScope {
        fn receive(&mut self, ctx: &mut Context, t: Tkn) -> ParseResult {
            match t {
                Tkn::Symbol(Sym::ParenRight) => {
                    let record = ctx.pop_record_or_panic();
                    ctx.push_record_to_table_or_panic(record);
                    to(table_states::InTableScope)
                }
                /* Update below for attributes list
                Tkn::Identifier(ident) => {
                    to(record_states::ReceivedRecordName(ident))
                }
                Tkn::Symbol(Sym::Underscore) => {
                    to(record_states::ReceivedExplicitAnonymousRecord)
                }
                Tkn::Symbol(Sym::ParentLeft) => {
                    ctx.push_record(None);
                    to(InRecordScope)
                }
                */
                _ => Err(ParseError),
            }
        }
    }
}
