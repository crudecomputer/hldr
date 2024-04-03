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


struct Loader<'a, 'b>
where 'b: 'a {
    refmap: HashMap<String, SimpleQueryRow>,
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
        // TODO: Use bind params and clean this up in general
        let mut columns = String::new();
        let mut values = String::new();

        // let mut values = Vec::new();
        // let mut attribute_values = HashMap::new();

        for (i, attribute) in attributes.iter().enumerate() {
            columns.push('"');
            columns.push_str(&attribute.name);
            columns.push('"');

            match &attribute.value {
                Value::Bool(b) => values.push_str(&b.to_string()),
                Value::Number(n) => values.push_str(&n),
                Value::Reference(r) if r.record.is_none() => {
                    unimplemented!("column references not yet implemented");
                }
                Value::Reference(r) => {
                    let val = self.follow_ref(table_scope, r)?;
                    values.push_str(&val);
                }
                Value::Text(t) => {
                    values.push('\'');
                    values.push_str(&t);
                    values.push('\'');
                }
            }

            if i < attributes.len() - 1 {
                columns.push_str(", ");
                values.push_str(", ");
            }
        }

        /*
        for (i, attribute) in attributes.iter().enumerate() {
            column_map.insert(&attribute.name, i);

            columns.push('"');
            columns.push_str(&attribute.name);
            columns.push('"');

            match &attribute.value {
                Value::Boolean(b) => values.push_str(&b.to_string()),
                Value::Number(n) => values.push_str(&n),
                Value::Reference(r) if r.record.is_none() => {
                    // If the column is referencing
                    unimplemented!("column references not yet implemented");
                }
                Value::Reference(r) => {
                    let val = self.follow_ref(table_scope, r)?;
                    values.push_str(&val);
                }
                Value::Text(t) => {
                    values.push('\'');
                    values.push_str(&t);
                    values.push('\'');
                }
            }

            if i < attributes.len() - 2 {
                columns.push_str(", ");
                values.push_str(", ");
            }
        }
        */

        let statement = format!(
            r#"
            INSERT INTO {} ({}) VALUES ({})
            RETURNING *
        "#,
            qualified_table_name, columns, values,
        );
        println!("{}", statement);

        let resp = self.transaction
            .simple_query(&statement)
            .map_err(LoadError::new)?
            .remove(0);

        match resp {
            SimpleQueryMessage::Row(row) => Ok(row),
            _ => unreachable!(),
        }
    }

    fn follow_ref(&self, current_scope: &str, refval: &Reference) -> Result<String, LoadError> {
        let key = match (refval.schema.as_ref(), refval.table.as_ref(), refval.record.as_ref()) {
            (Some(schema), Some(table), Some(record)) => {
                format!("{}.{}.{}", schema, table, record)
            }
            (None, Some(table), Some(record)) => {
                format!("{}.{}", table, record)
            }
            (None, None, Some(record)) => {
                format!("{}.{}", current_scope, record)
            }
            // Column-references are handled differently, as there is no record in
            // the map to look up
            _ => panic!("invalid reference"),
        };

        let col: &str = refval.column.as_ref();
        let row = self.refmap.get(&key).unwrap();
        let val = row.try_get(col);

        Ok(val
            .expect(&format!("no column '{}' in record {}", col, key))
            .map_or_else(|| "null".to_owned(), |v| format!("'{}'", v)))
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
