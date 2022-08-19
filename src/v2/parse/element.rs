#[derive(Clone, Debug, PartialEq)]
pub struct ReferenceValue {
    pub schema: String,
    pub table: String,
    pub record: String,
    pub column: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Boolean(bool),
    Number(String),
    Text(String),
    Reference(Box<ReferenceValue>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Attribute {
    pub name: String,
    pub value: Value,
}

#[derive(Debug, Default, PartialEq)]
pub struct Record {
    pub schema: String,
    pub table: String,
    pub name: Option<String>,
    pub attributes: Vec<Attribute>,
}

impl Record {
    pub fn anonymous(schema: &str, table: &str) -> Self {
        Self {
            schema: schema.to_owned(),
            table: schema.to_owned(),
            name: None,
            attributes: Vec::new(),
        }
    }

    pub fn named(schema: &str, table: &str, name: &str) -> Self {
        Self {
            schema: schema.to_owned(),
            table: schema.to_owned(),
            name: Some(name.to_owned()),
            attributes: Vec::new(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Element {
    Record(Record),
}

/*

#[derive(Debug, Default, PartialEq)]
pub struct Table {
    pub alias: Option<String>,
    pub name: String,
    pub records: Vec<Record>,
}

impl Table {
    pub fn new(name: String) -> Self {
        Self {
            alias: None,
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
    pub fn new(name: String) -> Self {
        Self {
            name,
            tables: Vec::new(),
        }
    }
}
*/
