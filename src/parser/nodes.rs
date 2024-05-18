use crate::Position;

#[derive(Debug, Default, PartialEq)]
pub struct ParseTree {
    pub nodes: Vec<StructuralNode>,
}

#[derive(Debug, PartialEq)]
pub enum StructuralNode {
    Schema(Box<SchemaNode>),
    Table(Box<TableNode>),
}

#[derive(Debug, PartialEq)]
pub struct StructuralNodeIdentity {
    pub alias: Option<String>,
    pub name: String,
}

impl StructuralNodeIdentity {
    pub fn new(name: String, alias: Option<String>) -> Self {
        Self { alias, name }
    }
}

#[derive(Debug, PartialEq)]
pub struct SchemaNode {
    pub identity: StructuralNodeIdentity,
    pub nodes: Vec<TableNode>,
}

impl SchemaNode {
    pub fn new(name: String, alias: Option<String>) -> Self {
        let identity = StructuralNodeIdentity::new(name, alias);
        Self {
            identity,
            nodes: Vec::new(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct TableNode {
    pub identity: StructuralNodeIdentity,
    pub nodes: Vec<RecordNode>,
}

impl TableNode {
    pub fn new(name: String, alias: Option<String>) -> Self {
        let identity = StructuralNodeIdentity::new(name, alias);
        Self {
            identity,
            nodes: Vec::new(),
        }
    }
}

#[derive(Debug, Default, PartialEq)]
pub struct RecordNode {
    pub name: Option<String>,
    pub nodes: Vec<AttributeNode>,
    pub position: Position,
}

impl RecordNode {
    pub fn new(name: Option<String>, position: Position) -> Self {
        Self {
            name,
            nodes: Vec::new(),
            position,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct AttributeNode {
    pub name: String,
    pub value: Value,
    pub position: Position,
}

impl AttributeNode {
    pub fn new(name: String, value: Value, position: Position) -> Self {
        Self { name, value, position }
    }
}

#[derive(Debug, PartialEq)]
pub enum Value {
    Bool(bool),
    Number(String),
    Reference(Reference),
    SqlFragment(String),
    Text(String),
}

/// The set of possible reference types, with varying levels
/// of qualification.
#[derive(Debug, PartialEq)]
pub enum Reference {
    ColumnLevel(ColumnLevelReference),
    RecordLevel(RecordLevelReference),
    TableLevel(TableLevelReference),
    SchemaLevel(SchemaLevelReference),
}

/// The set of possible column reference values, either explicit
/// with a name or implicit without one, in which case the column
/// being referenced is inferred from the attribute.
#[derive(Debug, PartialEq)]
pub enum ReferencedColumn {
    Explicit(String),
    Implicit,
}

/// References to a column in the same record, eg:
///
///     @column
#[derive(Debug, PartialEq)]
pub struct ColumnLevelReference {
    pub column: String,
}

/// References that are record-qualified with either explicit or implicit
/// column reference, eg:
///
///     @record.column  -- explicit column
///     @record.        -- implicit column
#[derive(Debug, PartialEq)]
pub struct RecordLevelReference {
    pub record: String,
    pub column: ReferencedColumn,
}

/// References that are table-qualified with either explicit or implicit
/// column reference, eg:
///
///     @table.record.column  -- explicit column
///     @table.record.        -- implicit column
#[derive(Debug, PartialEq)]
pub struct TableLevelReference {
    pub table: String,
    pub record: String,
    pub column: ReferencedColumn,
}

/// References that are schema-qualified with either explicit or implicit
/// column reference, eg:
///
///     @schema.table.record.column -- explicit column
///     @schema.table.record.       -- implicit column
#[derive(Debug, PartialEq)]
pub struct SchemaLevelReference {
    pub schema: String,
    pub table: String,
    pub record: String,
    pub column: ReferencedColumn,
}
