mod citekey;
mod config;
mod entry;
mod fetch;
mod ops;
mod regex;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{command, Parser, Subcommand, ValueHint};
use itertools::Itertools;

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
enum TagCommand {
    /// List all of the tags the currently exist in your library
    List,

    /// Add a tag to an entry
    Add {
        /// The tag to be modified
        tag: String,

        /// The citekey of the reference to change
        key: String,
    },

    /// Remove a tag from an entry
    Remove {
        /// The tag to be modified
        tag: String,

        /// The citekey of the reference to change
        key: String,
    },
}

#[derive(Subcommand, Debug)]
enum Command {
    /// List the references in your library
    List {
        /// List entries with a specific tag
        #[arg(short, long)]
        tag: Option<String>,
    },

    /// Add a reference to your library
    Add {
        /// The DOI of the reference to fetch
        doi: String,
    },

    /// Add a pdf to an entry in your library
    Pdf {
        /// The citekey of the reference to fetch
        key: String,
        /// Copy an exisitng file into your library instead of fetching a PDF from the internet
        #[arg(short, long, value_name = "FILE", value_hint = ValueHint::FilePath)]
        file: Option<PathBuf>,
    },

    /// Import all entries from a file
    Import {
        /// The file to import from
        #[arg(value_name = "FILE", value_hint = ValueHint::FilePath)]
        path: PathBuf,
    },

    /// Export from your library
    Export {
        /// The path to export to
        #[arg(value_name = "FILE", value_hint = ValueHint::FilePath)]
        path: PathBuf,

        /// The citekey of the reference to export (if not provided, all references are exported)
        #[arg(short, long)]
        key: Option<String>,
    },

    /// Modify (add / remove) a tag from an entry
    Tag {
        #[command(subcommand)]
        action: TagCommand,
    },
}

fn main() -> Result<()> {
    let cli_args = CliArgs::parse();
    let conf = config::load(cli_args.config)?;
    let root = cli_args.root.unwrap_or(conf.root);

    match cli_args.command {
        Command::List { tag } => {
            let mut paths = ops::entry_paths(&root)?;
            if let Some(t) = tag {
                paths.retain(|p| {
                    ops::read_entry(&ops::data_path(
                        &root,
                        &p.file_name().unwrap().to_string_lossy(),
                    ))
                    .unwrap()
                    .data
                    .custom
                    .tags
                    .contains(&t)
                })
            }
            for path in paths {
                println!("{}", path.file_name().unwrap().to_str().unwrap())
            }
        }
        Command::Add { doi } => {
            let mut entry = fetch::fetch_metadata(&doi)?;
            let key = citekey::get_key(&entry.data)?;
            ops::update_metadata(&mut entry.data, &key)?;
            ops::write_entry(&root, &key, &entry.data, false)?;
            println!("{key}");
        }
        Command::Tag { action } => match &action {
            TagCommand::List => ops::entry_paths(&root)?
                .into_iter()
                .flat_map(|p| {
                    ops::read_entry(&ops::data_path(
                        &root,
                        &p.file_name().unwrap().to_string_lossy(),
                    ))
                    .unwrap()
                    .data
                    .custom
                    .tags
                })
                .unique()
                .sorted()
                .for_each(|t| println!("{t}")),
            TagCommand::Add { tag, key } | TagCommand::Remove { tag, key } => {
                let path = ops::data_path(&root, key);
                let mut entry = ops::read_entry(&path)?;
                match action {
                    TagCommand::Add { .. } => entry.data.custom.tags.push(tag.to_string()),
                    TagCommand::Remove { .. } => entry.data.custom.tags.retain(|t| *t != *tag),
                    _ => unreachable!(),
                }
                ops::write_entry(&root, key, &entry.data, true)?;
            }
        },
        Command::Import { path } => {
            let file = File::open(path)?;
            let reader = BufReader::new(file);
            let mut entries: Vec<EntryData> = serde_json::from_reader(reader)?;
            for data in entries.iter_mut() {
                let key = citekey::get_key(data)?;
                ops::update_metadata(data, &key)?;
                ops::write_entry(&root, &key, data, false)?;
            }
        }
        Command::Export { path, key } => {
            let paths = if let Some(k) = key {
                vec![ops::entry_root_path(&root, &k)]
            } else {
                ops::entry_paths(&root)?
            };
            let content = paths
                .into_iter()
                .map(|p| {
                    read_entry(&ops::data_path(
                        &root,
                        p.file_name()
                            .context("Not a valid file name")?
                            .to_str()
                            .context("Path contains unicode")?,
                    ))
                    .map(|e| e.data)
                })
                .collect::<Result<Vec<EntryData>>>()?;
            let file = File::create(path)?;
            let writer = BufWriter::new(file);
            serde_json::to_writer(writer, &content)?;
        }
        Command::Pdf { key, file } => {
            let source = read_entry(&ops::data_path(&root, &key))?.source;
            let path = ops::pdf_path(&root, &key);
            if path.exists() {
                panic!("A file already exists at: {}", path.to_string_lossy());
            }
            if let Some(source_path) = file {
                std::fs::copy(source_path, path)?;
            } else {
                let pdf_url = fetch::fetch_pdf_url(
                    &source.expect("Cannot fetch PDF for reference with no DOI"),
                    &conf.email,
                )?;
                let mut pdf_data = ureq::get(&pdf_url).call()?.into_reader();
                let mut new_file = File::create(path)?;
                std::io::copy(&mut pdf_data, &mut new_file)?;
            };
        }
    }
    Ok(())
}
