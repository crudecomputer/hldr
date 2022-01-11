use std::{error:: Error, fs, path::Path};

pub mod error;
pub mod lex;
pub mod load;
pub mod parse;
pub mod validate;

pub use error::{HldrError, HldrErrorKind};

pub fn place(connstr: &str, filepath: &Path, commit: bool) -> Result<(), Box<dyn Error>> {
    let text = fs::read_to_string(&filepath)?;
    let tokens = lex::lex(&text)?;
    let schemas = parse::parse(tokens)?;
    let validated = validate::validate(schemas)?;

    let mut client = load::new_client(connstr)?;
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
