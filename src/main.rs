use std::path::PathBuf;

use clap::{crate_version, Parser};

/// Placeholder: Easy PostgreSQL data seeding
#[derive(Parser, Debug)]
#[clap(version = crate_version!())]
struct Command {
    /// Database connection string - for allowed formats see: https://docs.rs/postgres/0.19.2/postgres/config/struct.Config.html

    #[clap(short = 'd', long = "database-conn", name = "CONN")]
    database_conn: Option<String>,

    /// Path to the data file to load
    #[clap(short = 'f', long = "data-file", name = "FILE")]
    data_file: Option<PathBuf>,

    /// Commit the transaction, rolled back by default
    #[clap(long = "commit")]
    commit: bool,
}

struct Vars {
    database_conn: Option<String>,
    data_file: Option<PathBuf>,
}

impl Vars {
    fn empty() -> Self {
        Self {
            database_conn: None,
            data_file: None,
        }
    }
}

fn main() {
    let cmd = Command::parse();

    match (cmd.database_conn, cmd.data_file) {
        (Some(database_conn), Some(data_file)) => {
            hldr::place(&database_conn, &data_file, cmd.commit).unwrap();
        }
        (dc, df) => {
            let vars = vars_from_file();
            hldr::place(
                &dc.unwrap_or_else(|| vars.database_conn.expect("database_conn not found in file")),
                &df.or(vars.data_file)
                    .unwrap_or_else(|| PathBuf::from("place.hldr")),
                cmd.commit,
            ).unwrap()
        }
    }
}

fn vars_from_file() -> Vars {
    let varfile = PathBuf::from(".placehldr");

    if !varfile.exists() {
        panic!(".placehldr file is missing");
    }

    if !varfile.is_file() {
        panic!(".placehldr is not a file");
    }

    let mut vars = Vars::empty();

    for item in dotenv::from_path_iter(&varfile).unwrap() {
        let (key, val) = item.unwrap();

        match key.as_ref() {
            "database_conn" => vars.database_conn = Some(val),
            "data_file" => vars.data_file = Some(PathBuf::from(&val)),
            _ => panic!("Unexpected variable: {}", key),
        }
    }

    vars
}
