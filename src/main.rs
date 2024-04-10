use std::path::PathBuf;

use clap::{crate_version, Parser};

/// Placeholder: Easy PostgreSQL data seeding
#[derive(Parser, Debug)]
#[clap(version = crate_version!())]
struct Command {
    /// Commit the transaction
    #[clap(long = "commit")]
    commit: Option<bool>,

    /// Path to the .hldr data file to load [default: place.hldr if not specified in options file]
    #[clap(short = 'f', long = "data-file", name = "DATA-FILE")]
    file: Option<PathBuf>,

    /// Path to the optional .toml options file
    #[clap(
        short = 'o',
        long = "opts-file",
        name = "OPTS-FILE",
        default_value = "hldr-opts.toml"
    )]
    opts_file: PathBuf,

    /// Database connection string, either key/value pair or URI style
    #[clap(short = 'c', long = "database-conn", name = "CONN")]
    database_conn: Option<String>,
}

fn main() {
    let cmd = Command::parse();
    let options = {
        let mut options = hldr::Options::new(&cmd.opts_file)
            .unwrap() // consume result
            .unwrap_or_default();

        // The options file can specify the data file and connection string,
        // which should be overridden by command-line options
        if let Some(f) = cmd.file {
            options.data_file = f.clone();
        }

        if let Some(dc) = cmd.database_conn {
            options.database_conn = dc.clone();
        }

        if let Some(commit) = cmd.commit {
            options.commit = commit;
        }

        options
    };

    if let Err(e) = hldr::place(&options) {
        eprintln!("Error: {}", e);
    }
}
