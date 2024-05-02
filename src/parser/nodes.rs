#[derive(Debug, Default, PartialEq)]
pub struct ParseTree {
    pub nodes: Vec<StructuralNode>,
}

#[derive(Debug, PartialEq)]
pub enum StructuralNode {
    Schema(Box<Schema>),
    Table(Box<Table>),
}

#[derive(Debug, PartialEq)]
pub struct StructuralIdentity {
    pub alias: Option<String>,
    pub name: String,
}

impl StructuralIdentity {
    pub fn new(name: String, alias: Option<String>) -> Self {
        Self { alias, name }
    }
}

#[derive(Debug, PartialEq)]
pub struct Schema {
    pub identity: StructuralIdentity,
    pub nodes: Vec<Table>,
}

impl Schema {
    pub fn new(name: String, alias: Option<String>) -> Self {
        let identity = StructuralIdentity::new(name, alias);
        Self {
            identity,
            nodes: Vec::new(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Table {
    pub identity: StructuralIdentity,
    pub nodes: Vec<Record>,
}

impl Table {
    pub fn new(name: String, alias: Option<String>) -> Self {
        let identity = StructuralIdentity::new(name, alias);
        Self {
            identity,
            nodes: Vec::new(),
        }
    }
}

#[derive(Debug, Default, PartialEq)]
pub struct Record {
    pub name: Option<String>,
    pub nodes: Vec<Attribute>,
}

impl Record {
    pub fn new(name: Option<String>) -> Self {
        Self {
            name,
            nodes: Vec::new(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Attribute {
    pub name: String,
    pub value: Value,
}

impl Attribute {
    pub fn new(name: String, value: Value) -> Self {
        Self { name, value }
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
