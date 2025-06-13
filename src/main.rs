mod citekey;
mod config;
mod entry;
mod fetch;
mod ops;
mod regex;
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::path::PathBuf;
use std::time::Duration;

use anyhow::{Context, Result};
use clap::{command, Parser, Subcommand, ValueHint};
use indicatif::ProgressBar;
use itertools::Itertools;

use self::entry::EntryData;
use self::ops::read_entry;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct CliArgs {
    #[command(subcommand)]
    pub command: Command,

    /// Path to the configuration file that should be used (defaults to "$XDG_CONFIG_HOME/picoref/config.toml")
    #[arg(short, long, value_name = "FILE", value_hint = ValueHint::FilePath)]
    pub config: Option<PathBuf>,

    /// Path to the root of the library to use (defaults to path specified in config)
    #[arg(short, long, value_name = "FILE", value_hint = ValueHint::FilePath)]
    pub root: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
enum TagsCommand {
    /// List all of the tags the currently exist in your library
    #[clap(alias = "ls")]
    List,

    /// Add a tag to an entry
    Add {
        /// The tag(s) to be added
        #[arg(short, long, num_args(1..))]
        tags: Vec<String>,

        /// The citekey(s) of the entri(es) to add the tag to.
        #[arg(short, long, num_args(1..))]
        keys: Vec<String>,

        /// Add the tag(s) to all entries in your library
        #[arg(long, conflicts_with = "keys", default_value_t = false)]
        all_entries: bool,
    },

    /// Remove a tag from an entry
    Remove {
        /// The tag(s) to be removed
        #[arg(short, long, num_args(1..))]
        tags: Vec<String>,

        /// Remove all tags from the entries with
        #[arg(long, conflicts_with = "tags", default_value_t = false)]
        all_tags: bool,

        /// The citekey(s) of the entri(es) to remove the tag from
        #[arg(short, long, num_args(1..))]
        keys: Vec<String>,

        /// Remove the tag(s) from all entries in your library
        #[arg(long, conflicts_with = "keys", default_value_t = false)]
        all_entries: bool,
    },
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Print the path to your library's root
    Root,

    /// List the entries in your library
    #[clap(alias = "ls")]
    List {
        /// List entries with a specific tag
        #[arg(long, num_args(1..), conflicts_with = "all_tags")]
        any_tag: Option<Vec<String>>,

        #[arg(long, num_args(1..), conflicts_with = "any_tag")]
        all_tags: Option<Vec<String>>,
    },

    /// Add a reference to your library
    Add {
        /// The DOI of the reference to fetch
        doi: String,

        /// Tags to add to the new entry
        #[arg(short, long, num_args(1..))]
        tags: Option<Vec<String>>,
    },

    /// Add a pdf to an entry in your library
    Pdf {
        /// The citekey of the reference to fetch
        key: String,
        /// Copy an existing file into your library instead of fetching a PDF from the internet
        #[arg(short, long, value_name = "FILE", value_hint = ValueHint::FilePath)]
        file: Option<PathBuf>,
    },

    /// Generate a markdown file with the entry's metadata
    #[clap(alias = "md")]
    Markdown {
        /// The citekey of the reference to render
        key: String,
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

        /// The citekey of the reference to export (if not provided, all entries are exported)
        #[arg(short, long)]
        key: Option<String>,
    },

    /// Modify (add / remove) or list tags
    Tags {
        #[command(subcommand)]
        action: TagsCommand,
    },
}

fn main() -> Result<()> {
    let cli_args = CliArgs::parse();
    let conf = config::load(cli_args.config)?;
    let root = cli_args.root.unwrap_or(conf.root);

    match cli_args.command {
        Command::Root => println!("{}", root.to_str().expect("path to be valid unicode")),
        Command::List { any_tag, all_tags } => {
            let mut paths = ops::all_entry_paths(&root)?;
            if any_tag.is_some() || all_tags.is_some() {
                paths.retain(|p| {
                    let entry_tags = ops::read_entry(&ops::data_path(
                        &root,
                        &p.file_name().unwrap().to_string_lossy(),
                    ))
                    .unwrap()
                    .data
                    .custom
                    .tags;
                    match (&any_tag, &all_tags) {
                        (Some(tags), None) => tags.iter().any(|t| entry_tags.contains(t)),
                        (None, Some(tags)) => tags.iter().all(|t| entry_tags.contains(t)),
                        _ => unreachable!(),
                    }
                })
            }
            for path in paths {
                println!("{}", ops::key_from_path(&path))
            }
        }
        Command::Add { doi, tags } => {
            let mut entry = fetch::fetch_metadata(&doi)?;
            let key = citekey::get_key(&entry.data)?;
            ops::update_metadata(&mut entry.data, &key)?;
            if let Some(t) = tags {
                entry.data.custom.tags.extend(t);
            }
            ops::write_entry(&root, &key, &entry.data, false)?;
            println!("{key}");
        }
        Command::Tags { action } => match &action {
            TagsCommand::List => ops::all_entry_paths(&root)?
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
            TagsCommand::Add {
                tags,
                keys,
                all_entries,
            } => {
                let keys = if *all_entries {
                    ops::all_keys(&root)?
                } else {
                    keys.to_vec()
                };
                for key in keys.iter() {
                    let mut entry = ops::read_entry(&ops::data_path(&root, key))?;
                    entry.data.custom.tags.extend(tags.clone());
                    ops::write_entry(&root, key, &entry.data, true)?;
                }
            }
            TagsCommand::Remove {
                tags,
                all_tags,
                keys,
                all_entries,
            } => {
                let keys = if *all_entries {
                    ops::all_keys(&root)?
                } else {
                    keys.to_vec()
                };
                for key in keys.iter() {
                    let mut entry = ops::read_entry(&ops::data_path(&root, key))?;
                    if *all_tags {
                        entry.data.custom.tags.clear();
                    } else {
                        entry.data.custom.tags.retain(|t| !tags.contains(t));
                    }
                    ops::write_entry(&root, key, &entry.data, true)?;
                }
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
                ops::all_entry_paths(&root)?
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
                let mut pb = ProgressBar::new_spinner().with_message("Searching for PDF");
                pb.enable_steady_tick(Duration::from_millis(100));
                let pdf_url = fetch::fetch_pdf_url(
                    &source.expect("Cannot fetch PDF for reference with no DOI"),
                    &conf.email,
                )?;
                pb.finish_with_message("PDF found");
                let mut pdf_data = ureq::get(&pdf_url).call()?.into_reader();
                pb = ProgressBar::new_spinner().with_message("Downloading");
                pb.enable_steady_tick(Duration::from_millis(100));
                let mut new_file = File::create(path)?;
                std::io::copy(&mut pdf_data, &mut new_file)?;
                pb.finish_with_message("Download complete");
            };
        }
        Command::Markdown { key } => {
            let data = read_entry(&ops::data_path(&root, &key))?.data;
            let stdout = std::io::stdout().lock();
            let mut writer = BufWriter::new(stdout);

            if let Some(title) = data.standard_fields.get("title").and_then(|t| t.as_str()) {
                writer.write_all("# ".as_bytes())?;
                writer.write_all(title.as_bytes())?;
                writer.write_all("\n".as_bytes())?;
            }

            let by_line = data
                .standard_fields
                .get("author")
                .and_then(|a| {
                    a.as_array()?
                        .iter()
                        .map(|a| {
                            let given = a.get("given")?.as_str()?;
                            let family = a.get("family")?.as_str()?;
                            Some(format!("{given} {family}"))
                        })
                        .collect::<Option<Vec<String>>>()
                })
                .map(|mut v| {
                    if v.len() > 1 {
                        let last_i = v.len() - 1;
                        let new_last_entry = format!("& {}", v[last_i]);
                        let _ = std::mem::replace(&mut v[last_i], new_last_entry);
                    }
                    v.join(if v.len() > 2 { ", " } else { " " })
                })
                .or_else(|| Some(data.standard_fields.get("source")?.as_str()?.to_string()));
            if let Some(by) = &by_line {
                writer.write_all(by.as_bytes())?;
                writer.write_all("\n".as_bytes())?;
            }

            if let Some(container) = data
                .standard_fields
                .get("container-title")
                .and_then(|c| c.as_str())
            {
                writer.write_all("*".as_bytes())?;
                writer.write_all(container.as_bytes())?;
                writer.write_all("*".as_bytes())?;
            }

            if let Some(year) = data
                .standard_fields
                .get("issued")
                .and_then(|i| i.get("date-parts")?.get(0)?.get(0)?.as_u64())
            {
                writer.write_all(" (".as_bytes())?;
                writer.write_all(year.to_string().as_bytes())?;
                writer.write_all(")".as_bytes())?;
            }

            writer.write_all("\n\n".as_bytes())?;

            if let Some(entry_abstract) = data
                .standard_fields
                .get("abstract")
                .and_then(|a| a.as_str())
            {
                writer.write_all(entry_abstract.as_bytes())?;
                writer.write_all("\n\n".as_bytes())?;
            }

            let tags = data.custom.tags;
            if !tags.is_empty() {
                writer.write_all("Tags: ".as_bytes())?;
                writer.write_all(tags.join(", ").as_bytes())?;
                writer.write_all("\n".as_bytes())?;
            }

            writer.flush()?;
        }
    }
    Ok(())
}
