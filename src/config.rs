use std::fs::read_to_string;
use std::path::PathBuf;

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Config {
    /// Path to library
    pub root: PathBuf,
}

pub fn load(path: Option<PathBuf>) -> Result<Config> {
    let fallback_path = || {
        let mut fb_path = std::env::var("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                [
                    std::env::var("HOME").expect("No config paths found"),
                    ".config".to_string(),
                ]
                .iter()
                .collect()
            });
        fb_path.push("picoref");
        fb_path.push("config.toml");
        fb_path
    };

    Ok(toml::from_str(&read_to_string(
        path.unwrap_or_else(fallback_path),
    )?)?)
}
