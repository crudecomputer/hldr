use std::{
    env,
    str::FromStr,
    time::Duration,
};

use postgres::{config::Config, Client, NoTls};

use super::{
    parse::Value,
    validate::ValidatedSchemas,
};

fn new_client() -> Client {
    let url: String = env::var("HLDR_DATABASE_URL").expect("HLDR_DATABASE_URL not set");
    let mut config = Config::from_str(&url).unwrap();

    config.application_name("hldr");

    if config.get_connect_timeout().is_none() {
        config.connect_timeout(Duration::new(30, 0));
    }

    let client = config.connect(NoTls).unwrap();

    client
}

pub fn literal_value(v: &Value) -> String {
    match v {
        Value::Boolean(b) => b.to_string(),
        Value::Number(n) => n.clone(),
        Value::Text(t) => format!("'{}'", t),
    }
}

pub fn load(validated: &ValidatedSchemas, commit: bool) {
    let mut client = new_client();
    let mut transaction = client.transaction().unwrap();

    for schema in validated.schemas() {
        for table in &schema.tables {
            for record in &table.records {
                let mut columns = Vec::new();
                let mut values = Vec::new();

                for attr in &record.attributes {
                    columns.push(&attr.name);
                    values.push(&attr.value);
                }

                let columns_string = columns.into_iter()
                    .map(|c| format!("\"{}\"", c))
                    .collect::<Vec<String>>()
                    .join(", ");

                let values_string = values.into_iter()
                    .map(literal_value)
                    .collect::<Vec<String>>()
                    .join(", ");

                let statement = format!(r#"
                    INSERT INTO "{}"."{}" ({}) VALUES ({})
                "#,
                    schema.name,
                    table.name,
                    columns_string,
                    values_string,
                );

                transaction.execute(&statement, &[]).unwrap();
            }
        }
    }

    if commit {
        transaction.commit().unwrap();
    }
}

#[cfg(test)]
mod load_tests {
    use super::*;
    use super::super::validate::validate;
    use super::super::parse::{Schema, Table, Record, Attribute, Value};

    #[test]
    fn loads() {
        let v = validate(vec![
            Schema {
                name: "schema1".to_owned(),
                tables: vec![
                    Table {
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
                                ]
                            }
                        ]
                    }
                ]
            }
        ]);

        load(&v, true);
    }
}
