use std::{fs, path::PathBuf};

use clap::{crate_version, Parser};

/// Placeholder: Easy PostgreSQL data seeding
#[derive(Parser, Debug)]
#[clap(version = crate_version!())]
struct Command {
    /// Commit the transaction, rolled back by default
    #[clap(long = "commit")]
    commit: bool,

    /// Path to the data file to load
    #[clap(short = 'f', long = "data-file", name = "FILE")]
    data_file: Option<PathBuf>,

    /// Database connection string - for allowed formats see: https://docs.rs/postgres/0.19.2/postgres/config/struct.Config.html
    #[clap(short = 'd', long = "database-url", name = "URL")]
    database_url: Option<String>,

    /// Search path if overriding the default
    #[clap(short = 's', long = "search-path", name = "PATH")]
    search_path: Option<String>,
}

fn main() {
    let cmd = Command::parse();
    let config = {
        let mut config = hldr::Config::new("hldr.toml");

        if let Some(df) = cmd.data_file {
            config.data_file = df.clone();
        }

        if let Some(url) = cmd.database_url {
            config.database.url = url.clone();
        }

        if let Some(sp) = cmd.search_path {
            config.database.search_path = Some(sp.clone());
        }

        config.commit = cmd.commit;
        config
    };

    hldr::place(&config).unwrap();
}
