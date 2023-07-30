mod citekey;
mod config;
mod fetch;
mod regex;
use std::path::PathBuf;
use anyhow::Result;
use clap::{command, Parser, ValueHint};

#[derive(Parser, Clone, Default, Debug)]
#[command(author, version, about, long_about = None)]
struct CliArgs {
    /// The DOI of the reference to fetch
    #[arg(short, long)]
    pub doi: Option<String>,

    /// Path to the configuration file that should be used
    #[arg(short, long, value_name = "FILE", value_hint = ValueHint::FilePath)]
    pub config: Option<PathBuf>,

    /// Path to the root of the library to use (defaults to path specified in config)
    #[arg(short, long, value_name = "FILE", value_hint = ValueHint::FilePath)]
    pub root: Option<PathBuf>,
}

fn main() -> Result<()> {
    let cli_args = CliArgs::parse();
    let metadata = fetch::fetch_metadata(&cli_args.doi)?;
    println!("{}", serde_yaml::to_string(&metadata)?);
    Ok(())
}
