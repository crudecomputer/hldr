use std::path::PathBuf;

use clap::{Parser, crate_version};

/// Easy PostgreSQL data seeding
#[derive(Parser, Debug)]
#[clap(version = crate_version!())]
struct Opts {
    /// Database connection string - for allowed formats see https://docs.rs/postgres/0.19.2/postgres/config/struct.Config.html
    #[clap(short = 'd', long = "database-conn", name = "CONN")]
    database_connstr: String,

    /// Path to the data file to load
    #[clap(short = 'f', long = "file", name = "FILE")]
    filepath: PathBuf,

    /// Commits the transaction, which is rolled back by default
    #[clap(long = "commit")]
    commit: bool
}

fn main() {
    let opts: Opts = Opts::parse();
    hldr::seed(&opts.database_connstr, &opts.filepath, opts.commit);
}
