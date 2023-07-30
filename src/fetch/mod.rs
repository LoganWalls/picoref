mod arxiv;
mod biorxiv;
mod osf;

use anyhow::Result;

/// Checks if there is a published version of this paper (e.g. if the provided DOI is a pre-print)
/// and returns the DOI of the published version, if available.
pub fn published_doi(doi: &str) -> Result<Option<String>> {
    for f in [
        arxiv::published_doi,
        biorxiv::published_doi,
        osf::published_doi,
    ] {
        let result = f(doi)?;
        if result.is_some() {
            return Ok(result);
        }
    }
    Ok(None)
}

pub fn fetch_metadata(doi: &str) -> Result<serde_json::Map<String, serde_json::Value>> {
    let url = if let Some(published) = published_doi(doi)? {
        format!("https://dx.doi.org/{published}")
    } else {
        format!("https://dx.doi.org/{doi}")
    };
    Ok(ureq::get(&url)
        .set("Accept", "application/citeproc+json; charset=utf-8")
        .call()?
        .into_json()?)
}
