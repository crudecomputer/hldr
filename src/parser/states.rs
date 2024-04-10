use std::mem;
use crate::lexer::tokens::{
    Keyword,
    Symbol,
    Token,
    TokenKind,
};
use super::error::ParseError;
use super::nodes;

type ParseResult = Result<Box<dyn State>, ParseError>;

pub trait State: std::fmt::Debug {
    fn receive(&mut self, ctx: &mut Context, t: Option<Token>) -> ParseResult;
}

fn to<S: State + 'static>(state: S) -> ParseResult {
    Ok(Box::new(state))
}

fn defer_to<S: State + 'static>(state: &mut S, ctx: &mut Context, t: Option<Token>) -> ParseResult {
    state.receive(ctx, t)
}

#[derive(Debug)]
pub enum StackItem {
    TreeRoot(Box<nodes::ParseTree>),
    Schema(Box<nodes::Schema>),
    Table(Box<nodes::Table>),
    Record(Box<nodes::Record>),
    Attribute(Box<nodes::Attribute>),
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

    fn push_attribute(&mut self, name: String, value: nodes::Value) {
        let attribute = nodes::Attribute::new(name, value);
        self.stack.push(StackItem::Attribute(Box::new(attribute)));
    }

    // These utility methods all panic if certain expectations are not met,
    // primarily because that indicates faulty logic in the parser rather than
    // unexpected tokens in the token stream. In other words, unless I am woefully
    // mistaken, there should not be any combination of tokens that can result in
    // panics. Instead, bad tokens should always result in parse errors.
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

    fn pop_attribute_or_panic(&mut self) -> nodes::Attribute {
        match self.stack.pop() {
            Some(StackItem::Attribute(attribute)) => *attribute,
            elt => panic!("expected attribute on stack; received {:?}", elt),
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

    fn push_attribute_to_record_or_panic(&mut self, attribute: nodes::Attribute) {
        match self.stack.last_mut() {
            Some(StackItem::Record(record)) => {
                record.nodes.push(attribute);
            }
            elt => panic!("expected record on stack; received {:?}", elt),
        }
    }
}

/// Root state that can expect top-level entities.
#[derive(Debug)]
pub struct Root;

impl State for Root {
    fn receive(&mut self, _ctx: &mut Context, t: Option<Token>) -> ParseResult {
        // TODO: There are a few general patterns that have emerged in the states:
        //   - How to handle `None`
        //   - The error type to return as default case for unexpected token
        //
        // These are good indicators that the State trait & usage could be refined
        let t = match t {
            Some(t) => t,
            None => return to(Root),
        };
        match t.kind {
            TokenKind::LineSep => {
                to(Root)
            }
            TokenKind::Keyword(Keyword::Schema) => {
                to(schema_states::DeclaringSchema)
            }
            TokenKind::Keyword(Keyword::Table) => {
                to(table_states::DeclaringTable)
            }
            _ => Err(ParseError::token(t)),
        }
    }
}

mod schema_states {
    use super::*;

    /// State after receiving the `schema` keyword for declaration.
    #[derive(Debug)]
    pub struct DeclaringSchema;

    impl State for DeclaringSchema {
        fn receive(&mut self, _ctx: &mut Context, t: Option<Token>) -> ParseResult {
            let t = match t {
                Some(t) => t,
                None => return Err(ParseError::eof()),
            };
            match t.kind {
                TokenKind::Identifier(ident) | TokenKind::QuotedIdentifier(ident) => {
                    to(ReceivedSchemaName(ident))
                }
                _ => Err(ParseError::exp_schema(t)),
            }
        }
    }

    /// State after receiving the schema name during declaration.
    #[derive(Debug)]
    struct ReceivedSchemaName(String);

    impl State for ReceivedSchemaName {
        fn receive(&mut self, ctx: &mut Context, t: Option<Token>) -> ParseResult {
            let schema_name = mem::take(&mut self.0);
            let t = match t {
                Some(t) => t,
                None => return Err(ParseError::eof()),
            };
            match t.kind {
                TokenKind::Keyword(Keyword::As) => {
                    to(DeclaringSchemaAlias(schema_name))
                }
                TokenKind::Symbol(Symbol::ParenLeft) => {
                    ctx.push_schema(schema_name, None);
                    to(InSchemaScope)
                }
                _ => Err(ParseError::alias_or_scope(t)),
            }
        }
    }

    /// State after receiving the `as` keyword during schema declaration.
    #[derive(Debug)]
    struct DeclaringSchemaAlias(String);

    impl State for DeclaringSchemaAlias {
        fn receive(&mut self, _ctx: &mut Context, t: Option<Token>) -> ParseResult {
            let schema_name = mem::take(&mut self.0);
            let t = match t {
                Some(t) => t,
                None => return Err(ParseError::eof()),
            };
            match t.kind {
                // Unlike the true database name, aliases do not support quoted identifiers
                TokenKind::Identifier(ident) => {
                    to(ReceivedSchemaAlias(schema_name, ident))
                }
                _ => Err(ParseError::exp_alias(t)),
            }
        }
    }

    #[derive(Debug)]
    struct ReceivedSchemaAlias(String, String);

    impl State for ReceivedSchemaAlias {
        fn receive(&mut self, ctx: &mut Context, t: Option<Token>) -> ParseResult {
            let t = match t {
                Some(t) => t,
                None => return Err(ParseError::eof()),
            };
            match t.kind {
                TokenKind::Symbol(Symbol::ParenLeft) => {
                    let schema_name = mem::take(&mut self.0);
                    let alias = mem::take(&mut self.1);
                    ctx.push_schema(schema_name, Some(alias));
                    to(InSchemaScope)
                }
                _ => Err(ParseError::exp_scope(t)),
            }
        }
    }

    #[derive(Debug)]
    pub struct InSchemaScope;

    impl State for InSchemaScope {
        fn receive(&mut self, ctx: &mut Context, t: Option<Token>) -> ParseResult {
            let t = match t {
                Some(t) => t,
                None => return Err(ParseError::eof()),
            };
            match t.kind {
                TokenKind::Symbol(Symbol::ParenRight) => {
                    let schema = ctx.pop_schema_or_panic();
                    ctx.push_schema_to_root_or_panic(schema);
                    to(Root)
                }
                TokenKind::Keyword(Keyword::Table) => {
                    to(table_states::DeclaringTable)
                }
                TokenKind::LineSep => {
                    to(InSchemaScope)
                }
                _ => Err(ParseError::in_schema(t)),
            }
        }
    }
}

mod table_states {
    use super::*;

    /// State after receiving the `table` keyword for declaration.
    #[derive(Debug)]
    pub struct DeclaringTable;

    impl State for DeclaringTable {
        fn receive(&mut self, _ctx: &mut Context, t: Option<Token>) -> ParseResult {
            let t = match t {
                Some(t) => t,
                None => return Err(ParseError::eof()),
            };
            match t.kind {
                TokenKind::Identifier(ident) | TokenKind::QuotedIdentifier(ident) => {
                    to(ReceivedTableName(ident))
                }
                _ => Err(ParseError::exp_table(t)),
            }
        }
    }

    /// State after receiving the table name during declaration.
    #[derive(Debug)]
    struct ReceivedTableName(String);

    impl State for ReceivedTableName {
        fn receive(&mut self, ctx: &mut Context, t: Option<Token>) -> ParseResult {
            let table_name = mem::take(&mut self.0);
            let t = match t {
                Some(t) => t,
                None => return Err(ParseError::eof()),
            };
            match t.kind {
                TokenKind::Keyword(Keyword::As) => {
                    to(DeclaringTableAlias(table_name))
                }
                TokenKind::Symbol(Symbol::ParenLeft) => {
                    ctx.push_table(table_name, None);
                    to(InTableScope)
                }
                _ => Err(ParseError::alias_or_scope(t)),
            }
        }
    }

    #[derive(Debug)]
    struct DeclaringTableAlias(String);

    impl State for DeclaringTableAlias {
        fn receive(&mut self, _ctx: &mut Context, t: Option<Token>) -> ParseResult {
            let table_name = mem::take(&mut self.0);
            let t = match t {
                Some(t) => t,
                None => return Err(ParseError::eof()),
            };
            match t.kind {
                TokenKind::Identifier(ident) => {
                    to(ReceivedTableAlias(table_name, ident))
                }
                _ => Err(ParseError::exp_alias(t)),
            }
        }
    }

    #[derive(Debug)]
    struct ReceivedTableAlias(String, String);

    impl State for ReceivedTableAlias {
        fn receive(&mut self, ctx: &mut Context, t: Option<Token>) -> ParseResult {
            let t = match t {
                Some(t) => t,
                None => return Err(ParseError::eof()),
            };
            match t.kind {
                TokenKind::Symbol(Symbol::ParenLeft) => {
                    let table_name = mem::take(&mut self.0);
                    let alias = mem::take(&mut self.1);
                    ctx.push_table(table_name, Some(alias));
                    to(InTableScope)
                }
                _ => Err(ParseError::exp_scope(t)),
            }
        }
    }

    #[derive(Debug)]
    pub struct InTableScope;

    impl State for InTableScope {
        fn receive(&mut self, ctx: &mut Context, t: Option<Token>) -> ParseResult {
            let t = match t {
                Some(t) => t,
                None => return Err(ParseError::eof()),
            };
            match t.kind {
                TokenKind::Symbol(Symbol::ParenRight) => {
                    let table = ctx.pop_table_or_panic();

                    match ctx.push_table_to_parent_or_panic(table) {
                        PushedTableTo::TreeRoot => to(Root),
                        PushedTableTo::Schema => to(schema_states::InSchemaScope),
                    }
                }
                TokenKind::Identifier(ident) => {
                    to(record_states::ReceivedRecordName(ident))
                }
                TokenKind::Symbol(Symbol::Underscore) => {
                    to(record_states::ReceivedExplicitAnonymousRecord)
                }
                TokenKind::Symbol(Symbol::ParenLeft) => {
                    ctx.push_record(None);
                    to(record_states::InRecordScope)
                }
                TokenKind::LineSep => {
                    to(InTableScope)
                }
                _ => Err(ParseError::in_table(t)),
            }
        }
    }
}

mod record_states {
    use super::*;

    /// State after receiving a record name in the table scope.
    #[derive(Debug)]
    pub struct ReceivedRecordName(pub String);

    impl State for ReceivedRecordName {
        fn receive(&mut self, ctx: &mut Context, t: Option<Token>) -> ParseResult {
            let record_name = mem::take(&mut self.0);
            let t = match t {
                Some(t) => t,
                None => return Err(ParseError::eof()),
            };
            match t.kind {
                TokenKind::Symbol(Symbol::ParenLeft) => {
                    ctx.push_record(Some(record_name));
                    to(InRecordScope)
                }
                _ => Err(ParseError::exp_scope(t)),
            }
        }
    }

    /// State after receiving an `_` in the table scope.
    #[derive(Debug)]
    pub struct ReceivedExplicitAnonymousRecord;

    impl State for ReceivedExplicitAnonymousRecord {
        fn receive(&mut self, ctx: &mut Context, t: Option<Token>) -> ParseResult {
            let t = match t {
                Some(t) => t,
                None => return Err(ParseError::eof()),
            };
            match t.kind {
                TokenKind::Symbol(Symbol::ParenLeft) => {
                    ctx.push_record(None);
                    to(InRecordScope)
                }
                _ => Err(ParseError::exp_scope(t)),
            }
        }
    }

    #[derive(Debug)]
    pub struct InRecordScope;

    impl State for InRecordScope {
        fn receive(&mut self, ctx: &mut Context, t: Option<Token>) -> ParseResult {
            let t = match t {
                Some(t) => t,
                None => return Err(ParseError::eof()),
            };
            match t.kind {
                TokenKind::Symbol(Symbol::ParenRight) => {
                    let record = ctx.pop_record_or_panic();
                    ctx.push_record_to_table_or_panic(record);
                    to(table_states::InTableScope)
                }
                TokenKind::Identifier(ident) | TokenKind::QuotedIdentifier(ident) => {
                    to(attribute_states::ReceivedAttributeName(ident))
                }
                TokenKind::LineSep => {
                    to(InRecordScope)
                }
                _ => Err(ParseError::in_record(t)),
            }
        }
    }
}

mod attribute_states {
    use self::record_states::InRecordScope;

    use super::*;

    #[derive(Debug)]
    struct Identifier {
        quoted: bool,
        value: String,
    }

    #[derive(Debug)]
    pub struct ReceivedAttributeName(pub String);

    impl State for ReceivedAttributeName {
        fn receive(&mut self, ctx: &mut Context, t: Option<Token>) -> ParseResult {
            let attribute_name = mem::take(&mut self.0);
            let t = match t {
                Some(t) => t,
                None => return Err(ParseError::eof()),
            };
            match t.kind {
                TokenKind::Bool(b) => {
                    let value = nodes::Value::Bool(b);
                    ctx.push_attribute(attribute_name, value);
                    to(ReceivedAttributeValue)
                }
                TokenKind::Number(n) => {
                    let value = nodes::Value::Number(Box::new(n));
                    ctx.push_attribute(attribute_name, value);
                    to(ReceivedAttributeValue)
                }
                TokenKind::Symbol(Symbol::AtSign) => {
                    to(ReceivedReferenceStart(attribute_name))
                }
                TokenKind::Text(t) => {
                    let value = nodes::Value::Text(Box::new(t));
                    ctx.push_attribute(attribute_name, value);
                    to(ReceivedAttributeValue)
                }
                _ => Err(ParseError::exp_value(t)),
            }
        }
    }

    #[derive(Debug)]
    pub struct ReceivedReferenceStart(pub String);

    impl State for ReceivedReferenceStart {
        fn receive(&mut self, _ctx: &mut Context, t: Option<Token>) -> ParseResult {
            let attribute_name = mem::take(&mut self.0);
            let t = match t {
                Some(t) => t,
                None => return Err(ParseError::eof()),
            };
            let quoted = if let &TokenKind::QuotedIdentifier(_) = &t.kind { true } else { false };
            match t.kind {
                TokenKind::Identifier(ident) | TokenKind::QuotedIdentifier(ident) => {
                    let identifiers = vec![Identifier { quoted, value: ident }];
                    to(ReceivedReferenceIdentifier(attribute_name, identifiers))
                }
                _ => Err(ParseError::exp_ident(t)),
            }
        }
    }

    #[derive(Debug)]
    pub struct ReceivedReferenceIdentifier(String, Vec<Identifier>);

    impl State for ReceivedReferenceIdentifier {
        fn receive(&mut self, ctx: &mut Context, t: Option<Token>) -> ParseResult {
            let attribute_name = mem::take(&mut self.0);
            let mut identifiers = mem::take(&mut self.1);
            let t = match t {
                Some(t) => t,
                None => return Err(ParseError::eof()),
            };
            match t.kind {
                TokenKind::Symbol(Symbol::Period) if identifiers.len() < 4 => {
                    to(ReceivedReferenceSeparator(attribute_name, identifiers))
                }
                TokenKind::LineSep
                | TokenKind::Symbol(Symbol::Comma)
                | TokenKind::Symbol(Symbol::ParenRight) if identifiers.len() < 5 => {
                    let (column, record, table, schema) = (
                        // In this state there should always be at least one identifier
                        identifiers.pop().expect("expected element"),
                        identifiers.pop(),
                        identifiers.pop(),
                        identifiers.pop(),
                    );
                    if let Some(Identifier{ quoted: true, value }) = &record {
                        return Err(ParseError::rec_quot(value.to_owned(), t.position));
                    }
                    // The reference value node has no concept of whether or not the original
                    // token was quoted or not
                    let reference = nodes::Reference {
                        schema: schema.map(|s| s.value),
                        table: table.map(|t| t.value),
                        record: record.map(|r| r.value),
                        column: column.value,
                    };
                    let attribute = nodes::Attribute {
                        name: attribute_name,
                        value: nodes::Value::Reference(Box::new(reference)),
                    };
                    ctx.push_attribute_to_record_or_panic(attribute);

                    // TODO: This pattern is getting a bit gross. There needs to be a cleaner way of ending,
                    // since all values need to handle this line sep/comma/paren pattern.
                    match t.kind {
                        TokenKind::Symbol(Symbol::ParenRight) => defer_to(&mut InRecordScope, ctx, Some(t)),
                        _ => to(record_states::InRecordScope),
                    }
                }
                _ => Err(ParseError::token(t)),
            }
        }
    }

    #[derive(Debug)]
    pub struct ReceivedReferenceSeparator(String, Vec<Identifier>);

    impl State for ReceivedReferenceSeparator {
        fn receive(&mut self, _ctx: &mut Context, t: Option<Token>) -> ParseResult {
            // TODO: This is probably code smell at this point. Since the context
            // already makes so many assumptions about what is on its stack and
            // panics if items are wrong, should these all just be pushing to the
            // `ctx.stack_items` and popping off each step?
            let attribute_name = mem::take(&mut self.0);
            let mut identifiers = mem::take(&mut self.1);
            let t = match t {
                Some(t) => t,
                None => return Err(ParseError::eof()),
            };
            let quoted = if let TokenKind::QuotedIdentifier(_) = &t.kind { true } else { false };

            // Quoted identifiers are allowed in schema, table, and columns
            // names but not record names, eg. the following patterns are valid:
            //
            //   @ (un)quoted . (un)quoted . UNQUOTED   . (un)quoted
            //   @ (un)quoted . UNQUOTED   . (un)quoted
            //   @ UNQUOTED   . (un)quoted
            //   @ quo(un)ted
            //
            // This implies that simple or positional length checks are insufficient
            // to determine whether or not a quoted identifier is valid, as it depends
            // on the final form of the reference, so this state defers all checks to
            // the next state.
            match t.kind {
                TokenKind::Identifier(ident) | TokenKind::QuotedIdentifier(ident) => {
                    identifiers.push(Identifier { quoted, value: ident });
                    to(ReceivedReferenceIdentifier(attribute_name, identifiers))
                }
                _ => Err(ParseError::exp_ident(t)),
            }
        }
    }

    #[derive(Debug)]
    pub struct ReceivedAttributeValue;

    impl State for ReceivedAttributeValue {
        fn receive(&mut self, ctx: &mut Context, t: Option<Token>) -> ParseResult {
            let t = match t {
                Some(t) => t,
                None => return Err(ParseError::eof()),
            };
            match t.kind {
                TokenKind::Symbol(Symbol::Comma)
                | TokenKind::LineSep
                | TokenKind::Symbol(Symbol::ParenRight) => {
                    let attribute = ctx.pop_attribute_or_panic();
                    ctx.push_attribute_to_record_or_panic(attribute);

                    match t.kind {
                        TokenKind::Symbol(Symbol::ParenRight) => defer_to(&mut InRecordScope, ctx, Some(t)),
                        _ => to(record_states::InRecordScope),
                    }
                }
                _ => Err(ParseError::exp_close_attr(t)),
            }
        }
    }
}
