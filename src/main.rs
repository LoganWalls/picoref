mod fetch;
use anyhow::Result;
use clap::{command, Parser};

#[derive(Parser, Clone, Default, Debug)]
#[command(author, version, about, long_about = None)]
struct CliArgs {
    /// The DOI of the reference to fetch
    #[arg(short, long)]
    pub doi: String,
}

fn main() -> Result<()> {
    let cli_args = CliArgs::parse();
    let metadata = fetch::fetch_metadata(&cli_args.doi)?;
    println!("{}", serde_yaml::to_string(&metadata)?);
    Ok(())
}
