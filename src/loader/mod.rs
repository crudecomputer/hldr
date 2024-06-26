pub mod error;

use crate::analyzer::ValidatedParseTree;
use crate::parser::nodes::{
    Attribute,
    Reference,
    ReferencedColumn,
    StructuralIdentity,
    StructuralNode,
    Table,
    Value,
};
use error::{ClientError, LoadError};
use postgres::{config::Config, Client, NoTls, SimpleQueryMessage, SimpleQueryRow, Transaction};
use std::{collections::HashMap, str::FromStr, time::Duration};

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
where
    'b: 'a,
{
    refmap: RefMap,
    transaction: &'a mut Transaction<'b>,
}

impl<'a, 'b> Loader<'a, 'b> {
    fn new(transaction: &'a mut Transaction<'b>) -> Self {
        Self {
            refmap: HashMap::new(),
            transaction,
        }
    }

    fn load_table(&mut self, schema: Option<&StructuralIdentity>, table: &Table) -> LoadResult<()> {
        // TODO: A lot of this is copy-pasta from analyzer
        //
        // *something something* visitor pattern
        let qualified_table_name = match schema {
            Some(schema) => format!(r#""{}"."{}""#, schema.name, table.identity.name),
            None => format!(r#""{}""#, table.identity.name),
        };
        let table_scope = {
            let scope = table
                .identity
                .alias
                .as_ref()
                .unwrap_or(&table.identity.name);
            match schema {
                Some(schema) => format!(
                    "{}.{}",
                    schema.alias.as_ref().unwrap_or(&schema.name),
                    scope,
                ),
                None => scope.to_owned(),
            }
        };

        for record in &table.nodes {
            let row = self.insert(&qualified_table_name, &table_scope, &record.nodes)?;

            if let Some(name) = &record.name {
                let key = format!("{}.{}", table_scope, name);

                if self.refmap.insert(key, row).is_some() {
                    panic!("duplicate record in table {}: {}", table_scope, name);
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
        let statement = InsertStatement::build(self.transaction)
            .attributes(attributes)
            .current_scope(table_scope)
            .qualified_table_name(qualified_table_name)
            .refmap(&self.refmap)
            .finish()?;

        let resp = self
            .transaction
            .simple_query(statement.as_ref())
            .map_err(LoadError::new)?
            .remove(0);

        match resp {
            SimpleQueryMessage::Row(row) => Ok(row),
            _ => unreachable!(),
        }
    }
}

struct FragmentRunner<'a, 'b>
where
    'b: 'a,
{
    transaction: &'a mut Transaction<'b>,
}

impl<'a, 'b> FragmentRunner<'a, 'b> {
    fn select(&mut self, fragment: &str) -> Result<String, LoadError> {
        let query = format!("SELECT {}", fragment);

        let mut rows = self
            .transaction
            .simple_query(&query)
            .map_err(LoadError::new)?;

        if !matches!(rows[..], [SimpleQueryMessage::Row(_), SimpleQueryMessage::CommandComplete(1)]) {
            panic!("expected single row from SQL fragment `{}`", fragment);
        }

        let row = match rows.remove(0) {
            SimpleQueryMessage::Row(row) => row,
            _ => unreachable!(),
        };

        if row.len() != 1 {
            panic!("expected one column in SQL fragment result `{}`", fragment);
        }

        let value = row.get(0).expect("unreachable");

        // TODO: Using simple query protocol to select value is messy because
        // it requires stringifying selected values, which also means manually
        // trying to escape single quotes by replacing with two single quotes.
        //
        // Using extended query protocol would not; however, it would also complicate
        // selecting and storing values (since they would no longer just be strings
        // for all the things) and also requires rewriting the insert logic to rely
        // on having table column types queried before insert to build the right
        // type conversions in the literal SQL string.
        //
        // That would still most likely be for the best, though.
        //
        // Alternatively, maybe SQL fragments could be converted to CTEs in the
        // insert statement and the inserted values could be selected from the CTE
        // and completely avoid the round-tripping in either protocol, but this
        // would require a rewrite of the insert statement builder.
        let value = format!("'{}'", value.replace("'", "''"));
        println!("VALUE: {}", value);

        Ok(value)
    }
}

struct InsertStatementBuilder<
    'attribute,
    'current_scope,
    'fragment1,
    'fragment2,
    'qualified_table_name,
    'refmap,
>
where
    'fragment2: 'fragment1
{
    attributes: &'attribute [Attribute],
    attribute_indexes: HashMap<&'attribute str, usize>,
    current_scope: &'current_scope str,
    fragment_runner: FragmentRunner<'fragment1, 'fragment2>,
    qualified_table_name: &'qualified_table_name str,
    refmap: Option<&'refmap RefMap>,
}

impl<'a, 'c, 'f1, 'f2, 'q, 'r> InsertStatementBuilder<'a, 'c, 'f1, 'f2, 'q, 'r> {
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

    fn write_value(&mut self, attribute: &Attribute, out: &mut String) -> Result<(), LoadError> {
        match &attribute.value {
            Value::Bool(b) => out.push_str(&b.to_string()),
            Value::Number(n) => out.push_str(n),
            Value::Reference(Reference::ColumnLevel(colref)) => {
                // Column-reference could refer to a literal value, another
                // column reference, or a reference to a different record
                let index = self
                    .attribute_indexes
                    .get(&colref.column.as_ref())
                    .expect("missing column");

                let attribute = &self.attributes[*index];

                // TODO: Probably best to avoid the recursion?
                self.write_value(attribute, out)?;
            }
            Value::Reference(refval) => {
                let val = self.follow_ref(attribute, refval)?;
                out.push_str(&val);
            }
            Value::SqlFragment(s) => {
                let value = self.fragment_runner.select(s)?;
                out.push_str(&value);
            }
            Value::Text(t) => out.push_str(t),
        }

        Ok(())
    }

    fn follow_ref(&self, attribute: &Attribute, refval: &Reference) -> Result<String, LoadError> {
        use ReferencedColumn::*;

        let mut col = &attribute.name;
        let key = match refval {
            Reference::SchemaLevel(s) => {
                if let Explicit(c) = &s.column {
                    col = &c;
                }
                format!("{}.{}.{}", s.schema, s.table, s.record)
            }
            Reference::TableLevel(t) => {
                if let Explicit(c) = &t.column {
                    col = &c;
                }
                format!("{}.{}", t.table, t.record)
            }
            Reference::RecordLevel(r) => {
                if let Explicit(c) = &r.column {
                    col = &c;
                }
                format!("{}.{}", self.current_scope, r.record)
            }
            // Column-references are handled differently, as there is no record in
            // the map to look up
            Reference::ColumnLevel(_) => unreachable!(),
        };

        let row = self.refmap.expect("no refmap set").get(&key).unwrap();
        let val = row.try_get(col.as_str());

        Ok(val
            .unwrap_or_else(|_| panic!("no column '{}' in record {}", col, key))
            .map_or_else(|| "null".to_owned(), |v| format!("'{}'", v)))
    }
}

struct InsertStatement(String);

impl InsertStatement {
    fn build<'f1, 'f2>(t: &'f1 mut Transaction<'f2>) -> InsertStatementBuilder<'static, 'static, 'f1, 'f2, 'static, 'static> {
        InsertStatementBuilder {
            attributes: &[],
            attribute_indexes: HashMap::new(),
            current_scope: "",
            fragment_runner: FragmentRunner { transaction: t },
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
            StructuralNode::Schema(schema) => {
                let identity = schema.identity;
                for table in schema.nodes {
                    loader.load_table(Some(&identity), &table)?;
                }
            }
            StructuralNode::Table(table) => {
                loader.load_table(None, &table)?;
            }
        }
    }

    Ok(())
}
