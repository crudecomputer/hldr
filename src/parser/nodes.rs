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
        Self { identity, nodes: Vec::new() }
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
        Self { identity, nodes: Vec::new() }
    }
}

#[derive(Debug, Default, PartialEq)]
pub struct Record {
    pub name: Option<String>,
    pub nodes: Vec<Attribute>,
}

impl Record {
    pub fn new(name: Option<String>) -> Self {
        Self { name, nodes: Vec::new() }
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
    Number(Box<String>),
    Reference(Box<Reference>),
    Text(Box<String>),
}

// TODO: This should be handled by an enum, because this structure doesn't forbid
// having a schema BUT not having a table, etc. and invalid situations should not
// be representable.
#[derive(Debug, PartialEq)]
pub struct Reference {
    pub schema: Option<String>,
    pub table: Option<String>,
    pub record: Option<String>,
    pub column: String,
}
