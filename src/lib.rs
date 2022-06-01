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
    #[serde(default)]
    pub commit: bool,
    pub database: DatabaseConfig,

    #[serde(default = "default_data_file")]
    pub data_file: PathBuf,
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


pub fn place(config: &Config) -> Result<(), Box<dyn Error>> {
    let text = fs::read_to_string(&config.data_file)?;
    let tokens = lex::lex(&text)?;
    let schemas = parse::parse(tokens)?;
    let validated = validate::validate(schemas)?;

    let mut client = load::new_client(&config.database.url)?;
    let mut transaction = client.transaction()?;

    load::load(&mut transaction, &validated)?;

    if config.commit {
        println!("Committing changes");
        transaction.commit()?;
    } else {
        println!("Rolling back changes, pass `--commit` to apply")
    }

    Ok(())
}
