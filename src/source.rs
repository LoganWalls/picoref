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
