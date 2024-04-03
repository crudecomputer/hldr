use std::{error::Error, fs, path::PathBuf};

use serde::Deserialize;

mod v1;
mod v3;

pub use v1::error;
pub use v1::lex;
pub use v1::load;
pub use v1::parse;
pub use v1::validate;

pub use error::{HldrError, HldrErrorKind};

#[derive(Clone, Default, Debug, Deserialize)]
pub struct Options {
    #[serde(default)]
    pub commit: bool,

    #[serde(default = "default_data_file")]
    pub data_file: PathBuf,

    #[serde(default)]
    pub database_conn: String,
}

impl Options {
    pub fn new(filepath: &PathBuf) -> Result<Option<Self>, String> {
        if !filepath.exists() {
            return Ok(None);
        }

        if !filepath.is_file() {
            return Err(format!("{} is not a file", filepath.display()));
        }

        let contents = fs::read_to_string(&filepath)
            .map_err(|e| e.to_string())?;

        Ok(Some(toml::from_str(&contents)
            .map_err(|e| e.to_string())?))
    }
}

fn default_data_file() -> PathBuf {
    PathBuf::from("place.hldr")
}

pub fn v3_place(options: &Options) -> Result<(), Box<dyn Error>> {
    let input = fs::read_to_string(&options.data_file)?;

    let tokens = v3::lexer::tokenize(input.chars()).unwrap();
    let parse_tree = v3::parser::parse(tokens.into_iter()).unwrap();
    let parse_tree = v3::analyzer::analyze(parse_tree).unwrap();

    let mut client = v3::loader::new_client(&options.database_conn)?;
    let mut transaction = client.transaction()?;

    v3::loader::load(&mut transaction, parse_tree)?;

    if options.commit {
        println!("Committing changes");
        transaction.commit()?;
    } else {
        println!("Rolling back changes, pass `--commit` to apply")
    }

    Ok(())
}

pub fn place(options: &Options, commit: bool) -> Result<(), Box<dyn Error>> {
    let content = fs::read_to_string(&options.data_file)?;
    let tokens = lex::lex(&content)?;
    let schemas = parse::parse(tokens)?;
    let validated = validate::validate(schemas)?;

    let mut client = load::new_client(&options.database_conn)?;
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

/*
#[cfg(test)]
mod root_tests {
    use std::env;

    use super::{place, Options, PathBuf};

    #[test]
    fn it_works() {
        let options = Options {
            data_file: PathBuf::from("test/fixtures/place.hldr".to_owned()),
            database_conn: env::var("HLDR_TEST_DATABASE_URL").unwrap().clone(),
        };

        place(&options, false).unwrap();
    }
}
*/
