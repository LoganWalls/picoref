mod citekey;
mod config;
mod entry;
mod fetch;
mod ops;
mod regex;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use anyhow::Result;
use clap::{command, Parser, Subcommand, ValueHint};

use self::entry::EntryData;
use self::ops::read_entry;

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

    /// Downloads a pdf
    Pdf {
        /// The citekey of the reference to fetch
        key: String,
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
            let key = citekey::get_key(&entry.data)?;
            ops::update_metadata(&mut entry.data, &key)?;
            ops::write_entry(&root, &key, &entry.data)?;
        }
        Command::Import { path } => {
            let file = File::open(path)?;
            let reader = BufReader::new(file);
            let mut entries: Vec<EntryData> = serde_json::from_reader(reader)?;
            for data in entries.iter_mut() {
                let key = citekey::get_key(data)?;
                ops::update_metadata(data, &key)?;
                ops::write_entry(&root, &key, data)?;
            }
        }
        Command::Pdf { key } => {
            let source = read_entry(&root, &key)?.source;
            let path = ops::pdf_path(&root, &key);
            if path.exists() {
                panic!("A file already exists at: {}", path.to_string_lossy());
            }
            let pdf_url = fetch::fetch_pdf_url(&source, &conf.email)?;
            let mut pdf_data = ureq::get(&pdf_url).call()?.into_reader();
            let mut file = File::create(path)?;
            std::io::copy(&mut pdf_data, &mut file)?;
        }
    }
    Ok(())
}
