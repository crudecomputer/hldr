use std::path::PathBuf;

use clap::{crate_version, Parser};

/// Placeholder: Easy PostgreSQL data seeding
#[derive(Parser, Debug)]
#[clap(version = crate_version!())]
struct Command {
    /// Commit the transaction, rolled back by default
    #[clap(long = "commit")]
    commit: bool,

    /// Path to the data file to load
    #[clap(short = 'f', long = "file", name = "FILE")]
    file: Option<PathBuf>,

    /// Database connection string - for allowed formats see: https://docs.rs/postgres/0.19.2/postgres/config/struct.Config.html
    #[clap(short = 'c', long = "database-conn", name = "CONN")]
    database_conn: Option<String>,
}

fn main() {
    let cmd = Command::parse();
    let settings = {
        let mut settings = hldr::Settings::new("hldr.toml");

        if let Some(f) = cmd.file {
            settings.file = f.clone();
        }

        if let Some(dc) = cmd.database_conn {
            settings.database_conn = dc.clone();
        }

        settings
    };

    hldr::place(&settings, cmd.commit).unwrap();
}
