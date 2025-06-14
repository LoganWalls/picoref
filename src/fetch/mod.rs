pub mod arxiv;
pub mod biorxiv;
pub mod osf;
pub mod unpaywall;

use anyhow::{Context, Result};
use biblatex::{Bibliography, Entry};

use crate::source::Source;

/// Checks if there is a published version of this paper (e.g. if the provided DOI is a pre-print)
/// and returns the DOI of the published version, if available.
pub fn published_doi(source: &Source) -> Result<Option<String>> {
    match source {
        Source::Arxiv(id) => arxiv::published_doi(id),
        Source::Biorxiv(id) => biorxiv::published_doi(id),
        Source::Osf(id) => osf::published_doi(id),
        Source::Other(_) => Ok(None),
    }
}

pub fn fetch_metadata(doi: &str) -> Result<Entry> {
    let source: Source = doi.into();
    let url = format!(
        "https://dx.doi.org/{}",
        published_doi(&source)?.unwrap_or(doi.to_string())
    );
    let response = ureq::get(&url)
        .set("Accept", "text/bibliography; charset=utf-8")
        .call()?
        .into_string()?;
    Ok(Bibliography::parse(&response)
        .expect("response not valid bibtex")
        .into_iter()
        .next()
        .expect("no bib entries in response"))
}

pub fn fetch_pdf_url(source: &Source, email: &str) -> Result<String> {
    match source {
        Source::Arxiv(id) => Ok(arxiv::pdf_url(id)),
        Source::Biorxiv(id) => Ok(biorxiv::pdf_url(id)),
        Source::Osf(id) => osf::pdf_url(id),
        Source::Other(doi) => {
            Ok(unpaywall::pdf_url(doi, email)?.context("No open access PDF found")?)
        }
    }
}
