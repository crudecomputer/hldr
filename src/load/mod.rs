mod error;

use std::{str::FromStr, time::Duration};

use postgres::{config::Config, Client, NoTls, Transaction};

use super::{parse::Value, validate::ValidatedSchemas};
pub use error::{ClientError, LoadError};

pub fn new_client(connstr: &str) -> Result<Client, ClientError> {
    let mut config = Config::from_str(connstr)
        .map_err(ClientError::config_error)?;

    config.application_name("hldr");

    if config.get_connect_timeout().is_none() {
        config.connect_timeout(Duration::new(30, 0));
    }

    config.connect(NoTls)
        .map_err(ClientError::connection_error)
}

pub fn load(transaction: &mut Transaction, validated: &ValidatedSchemas) -> Result<(), LoadError> {
    for schema in validated.schemas() {
        for table in &schema.tables {
            for record in &table.records {
                let mut columns = Vec::new();
                let mut values = Vec::new();

                for attr in &record.attributes {
                    columns.push(&attr.name);
                    values.push(&attr.value);
                }

                let columns_string = columns
                    .into_iter()
                    .map(|c| format!("\"{}\"", c))
                    .collect::<Vec<String>>()
                    .join(", ");

                let values_string = values
                    .into_iter()
                    .map(literal_value)
                    .collect::<Vec<String>>()
                    .join(", ");

                let statement = format!(
                    r#"
                    INSERT INTO "{}"."{}" ({}) VALUES ({})
                "#,
                    schema.name, table.name, columns_string, values_string,
                );

                transaction.execute(&statement, &[])
                    .map_err(LoadError::new)?;
            }
        }
    }

    Ok(())
}

fn literal_value(v: &Value) -> String {
    match v {
        Value::Boolean(b) => b.to_string(),
        Value::Number(n) => n.clone(),
        Value::Text(t) => format!("'{}'", t),
    }
}

#[cfg(test)]
mod load_tests {
    use std::env;

    use chrono::prelude::*;

    use super::super::parse::{Attribute, Record, Schema, Table, Value};
    use super::super::validate::validate;
    use super::*;

    #[test]
    fn loads() {
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
                        name: Some("record".to_owned()),
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
            }],
        }])
        .unwrap();

        let mut client = new_client(&env::var("HLDR_TEST_DATABASE_URL").unwrap()).unwrap();
        let mut transaction = client.transaction().unwrap();

        transaction
            .execute(
                "
            CREATE SCHEMA hldr_test_schema
        ",
                &[],
            )
            .unwrap();
        transaction
            .execute(
                "
            CREATE TABLE  hldr_test_schema.table1  (
                id serial primary key,
                column1 bool,
                column2 int,
                column3 timestamptz
            )
        ",
                &[],
            )
            .unwrap();

        load(&mut transaction, &validated).unwrap();

        let rows = transaction
            .query(
                "
            SELECT column1, column2, column3 FROM hldr_test_schema.table1
            ORDER BY id DESC
        ",
                &[],
            )
            .unwrap();

        assert_eq!(rows.len(), 2);

        assert!(!rows[0].get::<&str, bool>("column1"));
        assert!(rows[1].get::<&str, bool>("column1"));

        assert_eq!(rows[0].get::<&str, i32>("column2"), 12345);
        assert_eq!(rows[1].get::<&str, i32>("column2"), 13); // Decimals were truncated

        assert_eq!(
            rows[0].get::<&str, DateTime<Utc>>("column3"),
            Utc.ymd(2021, 11, 30).and_hms(5, 0, 0)
        );
        assert_eq!(
            rows[1].get::<&str, DateTime<Utc>>("column3"),
            Utc.ymd(2021, 11, 28).and_hms(17, 0, 0)
        );
    }
}
