use std::collections::HashMap;

use crate::parse::{Attribute, Value};

pub trait Item {
    fn name(&self) -> &str;
}

/// Collection type to allow storing ordered items while also
/// allowing lookup by key name
#[derive(Debug, PartialEq)]
pub struct OrderedHashMap<T: Item> {
    items: Vec<T>,
    map: HashMap<String, usize>,
}

impl<T: Item> OrderedHashMap<T> {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            map: HashMap::new(),
        }
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.map.contains_key(key)
    }

    pub fn add(&mut self, item: T) {
        if self.contains_key(&item.name()) {
            panic!("already contains {}", item.name());
        }

        let item_name = item.name().to_owned();

        self.items.push(item);
        self.map.insert(item_name, self.items.len() - 1);
    }

    // TODO: Should this be IntoIterator instead?
    pub fn items(&self) -> &[T] {
        &self.items
    }
}

impl<T: Item + From<String>> OrderedHashMap<T> {
    pub fn get_or_create_mut(&mut self, key: impl Into<String>) -> &mut T {
        let key: String = key.into();

        let index = match self.map.get(&key) {
            Some(i) => i,
            None => {
                self.items.push(T::from(key.clone()));
                self.map.insert(key.to_owned(), self.items.len() - 1);
                self.map.get(&key).unwrap()
            }
        };

        &mut self.items[*index]
    }
}

/// An attribute that has been validated
#[derive(Debug, PartialEq)]
pub struct ValidatedAttribute(Attribute);

impl ValidatedAttribute {
    pub fn new(attr: Attribute) -> Self {
        Self(attr)
    }

    pub fn value(&self) -> &Value {
        &self.0.value
    }
}

impl Item for ValidatedAttribute {
    fn name(&self) -> &str {
        &self.0.name
    }
}

pub type ValidatedAttributes = OrderedHashMap<ValidatedAttribute>;

/// A named record whose attributes have all been validated
#[derive(Debug, PartialEq)]
pub struct ValidatedNamedRecord {
    name: String,
    attributes: ValidatedAttributes,
}

impl ValidatedNamedRecord {
    pub fn attributes(&self) -> &ValidatedAttributes {
        &self.attributes
    }

    pub(super) fn attributes_mut(&mut self) -> &mut ValidatedAttributes {
        &mut self.attributes
    }
}

impl From<String> for ValidatedNamedRecord {
    fn from(s: String) -> Self {
        Self {
            name: s.into(),
            attributes: OrderedHashMap::new(),
        }
    }
}

impl Item for ValidatedNamedRecord {
    fn name(&self) -> &str {
        &self.name
    }
}

pub type ValidatedNamedRecords = OrderedHashMap<ValidatedNamedRecord>;

/// An anonymous record whose attributes have all been validated
#[derive(Debug, PartialEq)]
pub struct ValidatedAnonymousRecord {
    attributes: ValidatedAttributes,
}

impl ValidatedAnonymousRecord {
    pub fn new() -> Self {
        Self {
            attributes: OrderedHashMap::new()
        }
    }

    pub fn attributes(&self) -> &ValidatedAttributes {
        &self.attributes
    }

    pub(super) fn attributes_mut(&mut self) -> &mut OrderedHashMap<ValidatedAttribute> {
        &mut self.attributes
    }
}

/// A collection of anonymous records that have all been validated
#[derive(Debug, PartialEq)]
pub struct ValidatedAnonymousRecords(Vec<ValidatedAnonymousRecord>);

impl ValidatedAnonymousRecords {
    pub(super) fn create(&mut self) -> &mut ValidatedAnonymousRecord {
        self.0.push(ValidatedAnonymousRecord::new());
        self.0.last_mut().unwrap()
    }

    pub fn items(&self) -> &[ValidatedAnonymousRecord] {
        &self.0
    }
}

/// A table whose named and anonymous records have all been validated
#[derive(Debug, PartialEq)]
pub struct ValidatedTable {
    name: String,
    named_records: ValidatedNamedRecords,
    anonymous_records: ValidatedAnonymousRecords,
}

impl ValidatedTable {
    pub fn anonymous_records(&self) -> &ValidatedAnonymousRecords {
        &self.anonymous_records
    }

    pub(super) fn anonymous_records_mut(&mut self) -> &mut ValidatedAnonymousRecords {
        &mut self.anonymous_records
    }

    pub fn named_records(&self) -> &ValidatedNamedRecords {
        &self.named_records
    }

    pub(super) fn named_records_mut(&mut self) -> &mut ValidatedNamedRecords {
        &mut self.named_records
    }
}

impl From<String> for ValidatedTable {
    fn from(s: String) -> Self {
        Self {
            name: s.into(),
            named_records: OrderedHashMap::new(),
            anonymous_records: ValidatedAnonymousRecords(Vec::new()),
        }
    }
}

impl Item for ValidatedTable {
    fn name(&self) -> &str {
        &self.name
    }
}

pub type ValidatedTables = OrderedHashMap<ValidatedTable>;

/// A schema whose tables and records have all been validated
#[derive(Debug, PartialEq)]
pub struct ValidatedSchema {
    name: String,
    tables: ValidatedTables,
}

impl ValidatedSchema {
    pub fn tables(&self) -> &ValidatedTables {
        &self.tables
    }

    pub(super) fn tables_mut(&mut self) -> &mut ValidatedTables {
        &mut self.tables
    }
}

impl From<String> for ValidatedSchema {
    fn from(s: String) -> Self {
        Self {
            name: s.into(),
            tables: OrderedHashMap::new(),
        }
    }
}

impl Item for ValidatedSchema {
    fn name(&self) -> &str {
        &self.name
    }
}

/// A collection of schemas that have all been validated
#[derive(Debug, PartialEq)]
pub struct ValidatedSchemas(OrderedHashMap<ValidatedSchema>);

impl ValidatedSchemas {
    pub fn new() -> Self {
        Self(OrderedHashMap::new())
    }

    pub fn schemas(&self) -> &OrderedHashMap<ValidatedSchema> {
        &self.0
    }

    pub(super) fn schemas_mut(&mut self) -> &mut OrderedHashMap<ValidatedSchema> {
        &mut self.0
    }
}
