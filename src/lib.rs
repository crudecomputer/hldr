use std::{fs, path::PathBuf};

pub mod lex;
pub mod load;
pub mod parse;
pub mod validate;

pub fn seed(connstr: &str, filepath: &PathBuf, commit: bool) {
    let text = fs::read_to_string(&filepath).unwrap();
    let tokens = lex::lex(&text);
    let schemas = parse::parse(tokens);
    let validated = validate::validate(schemas);

    let mut client = load::new_client(connstr);
    let mut transaction = client.transaction().unwrap();

    load::load(&mut transaction, &validated);

    if commit {
        println!("Committing changes");
        transaction.commit().unwrap();
    } else {
        println!("Rolling back changes, pass `--commit` to apply")
    }
}
