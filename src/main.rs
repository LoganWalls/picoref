mod citekey;
mod config;
mod fetch;
mod ops;
mod regex;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use anyhow::Result;
use clap::{command, Parser, Subcommand, ValueHint};

type Entry = serde_json::Map<String, serde_json::Value>;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct CliArgs {
    #[command(subcommand)]
    pub command: Command,

    /// Path to the configuration file that should be used
    #[arg(short, long, value_name = "FILE", value_hint = ValueHint::FilePath)]
    pub config: Option<PathBuf>,

    /// Path to the root of the library to use (defaults to path specified in config)
    #[arg(short, long, value_name = "FILE", value_hint = ValueHint::FilePath)]
    pub root: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Adds a reference to your library
    Add {
        /// The DOI of the reference to fetch
        doi: String,
    },

    /// Import all entries from a file
    Import {
        /// The file to import
        #[arg(value_name = "FILE", value_hint = ValueHint::FilePath)]
        path: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli_args = CliArgs::parse();
    let conf = config::load(cli_args.config)?;
    let root = cli_args.root.unwrap_or(conf.root);

    match cli_args.command {
        Command::Add { doi } => {
            let mut entry = fetch::fetch_metadata(&doi)?;
            let key = citekey::get_key(&entry)?;
            ops::update_metadata(&mut entry, &key)?;
            ops::write_entry(&root, &key, &entry)?;
        }
        Command::Import { path } => {
            let file = File::open(path)?;
            let reader = BufReader::new(file);
            let mut entries: Vec<Entry> = serde_json::from_reader(reader)?;
            for entry in entries.iter_mut() {
                let key = citekey::get_key(entry)?;
                ops::update_metadata(entry, &key)?;
                ops::write_entry(&root, &key, entry)?;
            }
        }
    }
    Ok(())
}
