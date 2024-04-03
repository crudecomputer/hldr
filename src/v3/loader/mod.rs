use std::{collections::HashMap, mem, str::FromStr, time::Duration};
use postgres::{config::Config, Client, NoTls, SimpleQueryMessage, SimpleQueryRow, Transaction};

//use super::{parse::Value, validate::{Item, ValidatedSchemas, ValidatedAttributes}};
use super::parser::nodes::{
    StructuralNode,
    Schema,
    Table,
    Reference,
    Attribute,
    Value,
};
use super::analyzer::ValidatedParseTree;

mod error;
pub use error::{ClientError, LoadError};



// TODO: move this
pub fn new_client(connstr: &str) -> Result<Client, ClientError> {
    let mut config = Config::from_str(connstr).map_err(ClientError::config_error)?;

    config.application_name("hldr");

    if config.get_connect_timeout().is_none() {
        config.connect_timeout(Duration::new(30, 0));
    }

    config.connect(NoTls).map_err(ClientError::connection_error)
}


type LoadResult<T> = Result<T, LoadError>;
type RefMap = HashMap<String, SimpleQueryRow>;


struct Loader<'a, 'b>
where 'b: 'a {
    refmap: RefMap,
    transaction: &'a mut Transaction<'b>,
}

impl<'a, 'b> Loader<'a, 'b> {
    fn new(transaction: &'a mut Transaction<'b>) -> Self {
        Self { refmap: HashMap::new(), transaction }
    }

    fn load_table(&mut self, schema: Option<&Schema>, table: &Table) -> LoadResult<()> {
        // TODO: A lot of this is copy-pasta from analyzer
        //
        // *something something* visitor pattern
        let qualified_table_name = match schema {
            Some(schema) => format!(r#""{}"."{}""#, schema.name, table.name),
            None => format!(r#""{}""#, table.name),
        };

        let table_scope = match schema {
            Some(schema) => format!(
                "{}.{}",
                schema.alias.as_ref().unwrap_or(&schema.name),
                table.alias.as_ref().unwrap_or(&table.name),
            ),
            None => table.alias.as_ref().unwrap_or(&table.name).to_owned(),
        };

        for record in &table.nodes {
            let row = self.insert(&qualified_table_name, &table_scope, &record.nodes)?;

            if let Some(name) = &record.name {
                let key = format!("{}.{}", table_scope, name);

                if self.refmap.insert(key, row).is_some() {
                    panic!("duplicate record in table {}: {}", table.name, name);
                }
            }
        }

        Ok(())
    }

    fn insert(
        &mut self,
        qualified_table_name: &str,
        table_scope: &str,
        attributes: &[Attribute],
    ) -> Result<SimpleQueryRow, LoadError> {
        let statement = InsertStatement::build()
            .attributes(attributes)
            .current_scope(table_scope)
            .qualified_table_name(qualified_table_name)
            .refmap(&self.refmap)
            .finish()?;

        let resp = self.transaction
            .simple_query(statement.as_ref())
            .map_err(LoadError::new)?
            .remove(0);

        match resp {
            SimpleQueryMessage::Row(row) => Ok(row),
            _ => unreachable!(),
        }
    }
}

struct InsertStatementBuilder<'a, 'c, 'q, 'r> {
    attributes: &'a [Attribute],
    attribute_indexes: HashMap<&'a str, usize>,
    current_scope: &'c str,
    qualified_table_name: &'q str,
    refmap: Option<&'r RefMap>,
}

impl<'a, 'c, 'q, 'r> InsertStatementBuilder<'a, 'c, 'q, 'r> {
    fn attributes(mut self, attributes: &'a [Attribute]) -> Self {
        self.attributes = attributes;
        self.attribute_indexes = HashMap::new();
        self
    }

    fn current_scope(mut self, current_scope: &'c str) -> Self {
        self.current_scope = current_scope;
        self
    }

    fn qualified_table_name(mut self, qualified_table_name: &'q str) -> Self {
        self.qualified_table_name = qualified_table_name;
        self
    }

    fn refmap(mut self, refmap: &'r RefMap) -> Self {
        self.refmap = Some(refmap);
        self
    }

    fn finish(mut self) -> Result<InsertStatement, LoadError> {
        // TODO: Use bind params and clean this up in general
        let mut columns = String::new();
        let mut values = String::new();

        for (i, attribute) in self.attributes.iter().enumerate() {
            columns.push('"');
            columns.push_str(&attribute.name);
            columns.push('"');

            self.write_value(attribute, &mut values)?;

            // Only add this after to prevent cyclic references
            self.attribute_indexes.insert(&attribute.name, i);

            if i < self.attributes.len() - 1 {
                columns.push_str(", ");
                values.push_str(", ");
            }
        }

        let statement = format!(
            r#"
            INSERT INTO {} ({}) VALUES ({})
            RETURNING *
        "#,
            self.qualified_table_name, columns, values,
        );
        println!("{}", statement);

        Ok(InsertStatement(statement))
    }

    fn write_value(&self, attribute: &Attribute, out: &mut String) -> Result<(), LoadError> {
        match &attribute.value {
            Value::Bool(b) => out.push_str(&b.to_string()),
            Value::Number(n) => out.push_str(&n),
            Value::Reference(r) if r.record.is_none() => {
                // Column-reference could refer to a literal value, another
                // column reference, or a reference to a different record
                println!("{:?}", self.attribute_indexes);
                println!("{:?}", r);

                let index = self.attribute_indexes
                    .get(&r.column.as_ref())
                    .expect("missing column");

                let attribute = &self.attributes[*index];

                self.write_value(attribute, out)?;
            }
            Value::Reference(r) => {
                let val = self.follow_ref(r)?;
                out.push_str(&val);
            }
            Value::Text(t) => {
                out.push('\'');
                out.push_str(&t);
                out.push('\'');
            }
        }

        Ok(())
    }

    fn follow_ref(&self, refval: &Reference) -> Result<String, LoadError> {
        let key = match (refval.schema.as_ref(), refval.table.as_ref(), refval.record.as_ref()) {
            (Some(schema), Some(table), Some(record)) => {
                format!("{}.{}.{}", schema, table, record)
            }
            (None, Some(table), Some(record)) => {
                format!("{}.{}", table, record)
            }
            (None, None, Some(record)) => {
                format!("{}.{}", self.current_scope, record)
            }
            // Column-references are handled differently, as there is no record in
            // the map to look up
            _ => panic!("invalid reference"),
        };

        let col: &str = refval.column.as_ref();
        let row = self.refmap.expect("no refmap set").get(&key).unwrap();
        let val = row.try_get(col);

        Ok(val
            .expect(&format!("no column '{}' in record {}", col, key))
            .map_or_else(|| "null".to_owned(), |v| format!("'{}'", v)))
    }
}

struct InsertStatement(String);

impl InsertStatement {
    fn build() -> InsertStatementBuilder<'static, 'static, 'static, 'static> {
        InsertStatementBuilder {
            attributes: &[],
            attribute_indexes: HashMap::new(),
            current_scope: "",
            qualified_table_name: "",
            refmap: None,
        }
    }

    fn as_ref(&self) -> &str {
        &self.0
    }
}

pub fn load(transaction: &mut Transaction, tree: ValidatedParseTree) -> LoadResult<()> {
    let mut loader = Loader::new(transaction);

    for node in tree.into_inner().nodes {
        match node {
            StructuralNode::Schema(mut schema) => {
                // TODO: This is a smell
                let nodes = mem::take(&mut schema.nodes);

                for table in nodes {
                    loader.load_table(Some(&schema), &table)?;
                }
            }
            StructuralNode::Table(table) => {
                loader.load_table(None, &table)?;
            }
        }
    }

    Ok(())
}
