mod error;

use std::{collections::HashMap, str::FromStr, time::Duration};

use postgres::{config::Config, Client, NoTls, SimpleQueryMessage, SimpleQueryRow, Transaction};

use super::{parse::Value, validate::{Item, ValidatedSchemas, ValidatedAttributes}};
pub use error::{ClientError, LoadError};

pub fn new_client(connstr: &str) -> Result<Client, ClientError> {
    let mut config = Config::from_str(connstr).map_err(ClientError::config_error)?;

    config.application_name("hldr");

    if config.get_connect_timeout().is_none() {
        config.connect_timeout(Duration::new(30, 0));
    }

    config.connect(NoTls).map_err(ClientError::connection_error)
}

struct Loader<'a, 'b>
where 'b: 'a {
    named_records: HashMap<String, SimpleQueryRow>,
    transaction: &'a mut Transaction<'b>,
}

impl<'a, 'b> Loader<'a, 'b> {
    fn new(transaction: &'a mut Transaction<'b>) -> Self {
        Self {
            named_records: HashMap::new(),
            transaction,
        }
    }

    fn load(&mut self, validated: &ValidatedSchemas) -> Result<(), LoadError> {
        for schema in validated.schemas().items() {
            for table in schema.tables().items() {
                for record in table.named_records().items() {
                    let qualified_name = format!("{}.{}@{}", schema.name(), table.name(), record.name());
                    let row = self.insert(
                        schema.name(),
                        table.name(),
                        record.attributes(),
                    )?;
                    self.named_records.insert(qualified_name, row);
                }

                for record in table.anonymous_records().items() {
                    self.insert(
                        schema.name(),
                        table.name(),
                        record.attributes(),
                    )?;
                }
            }
        }

        Ok(())
    }

    fn insert(&mut self, schema: &str, table: &str, attrs: &ValidatedAttributes) -> Result<SimpleQueryRow, LoadError> {
        let mut columns = Vec::new();
        let mut values = Vec::new();

        for attr in attrs.items() {
            columns.push(attr.name());
            values.push(attr.value());
        }

        let columns_string = columns
            .into_iter()
            .map(|c| format!("\"{}\"", c))
            .collect::<Vec<String>>()
            .join(", ");

        let values_string = values
            .into_iter()
            .map(|v| self.literal_value(v))
            .collect::<Vec<String>>()
            .join(", ");

        let statement = format!(
            r#"
            INSERT INTO "{}"."{}" ({}) VALUES ({})
            RETURNING *
        "#,
            schema, table, columns_string, values_string,
        );

        let resp = self.transaction
            .simple_query(&statement)
            .map_err(LoadError::new)?
            .remove(0);

        match resp {
            SimpleQueryMessage::Row(row) => Ok(row),
            _ => unreachable!(),
        }
    }


    fn literal_value(&self, v: &Value) -> String {
        match v {
            Value::Boolean(b) => b.to_string(),
            Value::Number(n) => n.clone(),
            Value::Text(t) => format!("'{}'", t),
            Value::Reference(refval) => {
                let col = refval.column.as_str();
                
                let qualified_name = format!("{}.{}@{}", refval.schema, refval.table, refval.record);
                let row = self.named_records.get(&qualified_name).unwrap();
                let val = row.try_get(col);

                val
                    .expect(&format!("no column '{}' in record {}", col, qualified_name))
                    .map_or_else(|| "null".to_owned(), |v| format!("'{}'", v))
            },
        }
    }
}

pub fn load(transaction: &mut Transaction, validated: &ValidatedSchemas) -> Result<(), LoadError> {
    Loader::new(transaction).load(validated)

}

#[cfg(test)]
mod load_tests {
    use std::env;

    use chrono::prelude::*;

    use super::super::parse::{Attribute, Record, Schema, Table, Value, ReferenceValue};
    use super::super::validate::validate;
    use super::*;

    #[test]
    fn loads() {
        let mut client = new_client(&env::var("HLDR_TEST_DATABASE_URL").unwrap()).unwrap();
        let mut transaction = client.transaction().unwrap();

        transaction.simple_query("
            CREATE SCHEMA hldr_test_schema;

            CREATE TABLE hldr_test_schema.table1  (
                id serial primary key,
                column1 bool,
                column2 int,
                column3 timestamptz
            );

            CREATE TABLE hldr_test_schema.table2  (
                id serial primary key,
                table1_id int,
                column2 int
            );
        ").unwrap();

        let validated = validate(vec![Schema {
            name: "hldr_test_schema".to_owned(),
            tables: vec![Table {
                name: "table1".to_owned(),
                records: vec![
                    Record {
                        name: None,
                        attributes: vec![
                            Attribute {
                                name: "column1".to_owned(),
                                value: Value::Boolean(true),
                            },
                            Attribute {
                                name: "column2".to_owned(),
                                value: Value::Number("13.37".to_owned()),
                            },
                            Attribute {
                                name: "column3".to_owned(),
                                value: Value::Text("2021-11-28T12:00:00-05:00".to_owned()),
                            },
                        ],
                    },
                    Record {
                        name: Some("record1".to_owned()),
                        attributes: vec![
                            Attribute {
                                name: "column1".to_owned(),
                                value: Value::Boolean(false),
                            },
                            Attribute {
                                name: "column2".to_owned(),
                                value: Value::Number("12345".to_owned()),
                            },
                            Attribute {
                                name: "column3".to_owned(),
                                value: Value::Text("2021-11-30T00:00:00-5:00".to_owned()),
                            },
                        ],
                    },
                ],
                ..Default::default()
            }, Table {
                name: "table2".to_owned(),
                records: vec![
                    Record {
                        name: None,
                        attributes: vec![
                            Attribute {
                                name: "table1_id".to_owned(),
                                value: Value::Reference(Box::new(ReferenceValue {
                                    schema: "hldr_test_schema".to_owned(),
                                    table: "table1".to_owned(),
                                    record: "record1".to_owned(),
                                    column: "id".to_owned(),
                                })),
                            },
                        ],
                    },
                    Record {
                        name: None,
                        attributes: vec![
                            Attribute {
                                name: "column2".to_owned(),
                                value: Value::Reference(Box::new(ReferenceValue {
                                    schema: "hldr_test_schema".to_owned(),
                                    table: "table1".to_owned(),
                                    record: "record1".to_owned(),
                                    column: "column2".to_owned(),
                                })),
                            },
                        ],
                    },
                ],
                ..Default::default()
            }],
        }])
        .unwrap();

        load(&mut transaction, &validated).unwrap();

        // FIXME: Why is this DESC..?
        let rows = transaction.query("
            SELECT id, column1, column2, column3
            FROM hldr_test_schema.table1
            ORDER BY id DESC
        ", &[]).unwrap();

        assert_eq!(rows.len(), 2);

        assert!(rows[0].get::<&str, bool>("column1"));
        assert!(!rows[1].get::<&str, bool>("column1"));

        assert_eq!(rows[0].get::<&str, i32>("column2"), 13); // Decimals were truncated
        assert_eq!(rows[1].get::<&str, i32>("column2"), 12345);

        assert_eq!(
            rows[0].get::<&str, DateTime<Utc>>("column3"),
            Utc.ymd(2021, 11, 28).and_hms(17, 0, 0)
        );
        assert_eq!(
            rows[1].get::<&str, DateTime<Utc>>("column3"),
            Utc.ymd(2021, 11, 30).and_hms(5, 0, 0)
        );

        let record1_id = rows[1].get::<&str, i32>("id");

        let rows = transaction.query("
            SELECT table1_id, column2
            FROM hldr_test_schema.table2
            ORDER BY id ASC
        ", &[]).unwrap();

        assert_eq!(
            rows[0].get::<&str, i32>("table1_id"),
            record1_id,
        );
        assert_eq!(
            rows[1].get::<&str, i32>("column2"),
            12345,
        );
    }
}
