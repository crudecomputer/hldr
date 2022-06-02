use std::{error::Error, fs, path::PathBuf};

use serde::Deserialize;

pub mod error;
pub mod lex;
pub mod load;
pub mod parse;
pub mod validate;

pub use error::{HldrError, HldrErrorKind};

#[derive(Clone, Debug, Default, Deserialize)]
pub struct DatabaseConfig {
    pub search_path: Option<String>,

    #[serde(default)]
    pub url: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    #[serde(default = "default_data_file")]
    pub data_file: PathBuf,

    pub database: DatabaseConfig,
}

impl Config {
    pub fn new(filepath: &str) -> Self {
        let path = PathBuf::from(filepath);

        if !path.exists() {
            panic!("{} file is missing", filepath);
        }

        if !path.is_file() {
            panic!("{} is not a file", filepath);
        }

        let contents = fs::read_to_string(&path).unwrap();

        toml::from_str(&contents).unwrap()
    }
}

fn default_data_file() -> PathBuf {
    PathBuf::from("place.hldr")
}

pub fn place(config: &Config, commit: bool) -> Result<(), Box<dyn Error>> {
    let content = fs::read_to_string(&config.data_file)?;
    let tokens = lex::lex(&content)?;
    let schemas = parse::parse(tokens)?;
    let validated = validate::validate(schemas)?;

    let mut client = load::new_client(&config.database.url)?;

    if let Some(sp) = &config.database.search_path {
        client.simple_query(&format!("SET search_path TO {}", sp))?;
    }

    let mut transaction = client.transaction()?;

    load::load(&mut transaction, &validated)?;

    if commit {
        println!("Committing changes");
        transaction.commit()?;
    } else {
        println!("Rolling back changes, pass `--commit` to apply")
    }

    Ok(())
}

#[cfg(test)]
mod root_tests {
    use std::env;

    use super::{place, Config, DatabaseConfig, PathBuf};

    #[test]
    #[should_panic]
    fn panics_without_search_path() {
        let database_url = env::var("HLDR_TEST_DATABASE_URL").unwrap();
        let config = Config {
            data_file: PathBuf::from("test/fixtures/place.hldr".to_owned()),
            database: DatabaseConfig {
                search_path: None,
                url: database_url.clone(),
            }
        };

        place(&config, false).unwrap();
    }

    #[test]
    fn respects_search_path() {
        let config = Config {
            data_file: PathBuf::from("test/fixtures/place.hldr".to_owned()),
            database: DatabaseConfig {
                search_path: Some("schema1, schema2".to_owned()),
                url: env::var("HLDR_TEST_DATABASE_URL").unwrap().clone(),
            }
        };

        place(&config, false).unwrap();
    }
}
