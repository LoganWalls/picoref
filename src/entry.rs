use anyhow::{Context, Result};
use serde::Deserialize;

use crate::fetch::{arxiv, biorxiv, osf};
use crate::regex::cap_as_str;

#[derive(Debug)]
pub enum Source {
    Arxiv(String),
    Biorxiv(String),
    Osf(String),
    Other(String),
}

impl From<String> for Source {
    fn from(value: String) -> Self {
        for (t, re) in [
            (Self::Arxiv as fn(String) -> Self, &arxiv::RE),
            (Self::Biorxiv as fn(String) -> Self, &biorxiv::RE),
            (Self::Osf as fn(String) -> Self, &osf::RE),
        ] {
            if let Some(id) = cap_as_str(re, &value, 1) {
                return t(id.to_string());
            }
        }
        Self::Other(value)
    }
}

impl From<&str> for Source {
    fn from(value: &str) -> Self {
        value.to_string().into()
    }
}

pub type EntryData = serde_json::Map<String, serde_json::Value>;

#[derive(Debug, Deserialize)]
#[serde(try_from = "EntryData")]
pub struct Entry {
    pub source: Source,
    pub data: EntryData,
}

impl TryFrom<EntryData> for Entry {
    type Error = anyhow::Error;
    fn try_from(data: EntryData) -> Result<Self> {
        let source = data
            .get("DOI")
            .context("No DOI found")?
            .as_str()
            .context("Could not convert DOI to string")?
            .into();
        Ok(Self { source, data })
    }
}
