use std::{fs, path::Path};

pub mod lex;
pub mod load;
pub mod parse;
pub mod validate;

pub fn place(connstr: &str, filepath: &Path, commit: bool) {
    let text = fs::read_to_string(&filepath).unwrap();
    let tokens = lex::lex(&text).unwrap();
    let schemas = parse::parse(tokens).unwrap();
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
