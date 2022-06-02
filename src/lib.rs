use std::{error::Error, fs, path::PathBuf};

use serde::Deserialize;

pub mod error;
pub mod lex;
pub mod load;
pub mod parse;
pub mod validate;

pub use error::{HldrError, HldrErrorKind};

#[derive(Clone, Debug, Deserialize)]
pub struct Settings {
    #[serde(default = "default_data_file")]
    pub file: PathBuf,

    #[serde(default)]
    pub database_conn: String,
}

impl Settings {
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

pub fn place(settings: &Settings, commit: bool) -> Result<(), Box<dyn Error>> {
    let content = fs::read_to_string(&settings.file)?;
    let tokens = lex::lex(&content)?;
    let schemas = parse::parse(tokens)?;
    let validated = validate::validate(schemas)?;

    let mut client = load::new_client(&settings.database_conn)?;
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

    use super::{place, Settings, PathBuf};

    #[test]
    fn it_works() {
        let settings = Settings {
            file: PathBuf::from("test/fixtures/place.hldr".to_owned()),
            database_conn: env::var("HLDR_TEST_DATABASE_URL").unwrap().clone(),
        };

        place(&settings, false).unwrap();
    }
}
