use anyhow::Result;
use once_cell::sync::Lazy;
use regex::Regex;

use super::cap_as_str;

static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)10.48550/arxiv\.(.+)").unwrap());
static RESP_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)<arxiv:doi.*?>(.+?)</arxiv:doi>").unwrap());

pub fn published_doi(doi: &str) -> Result<Option<String>> {
    Ok(if let Some(arxiv_id) = cap_as_str(&RE, doi, 1) {
        let response = ureq::get(&format!(
            "http://export.arxiv.org/api/query?id_list={arxiv_id}"
        ))
        .set("Accept", "application/xml; charset=utf-8")
        .call()?
        .into_string()?;
        // HACK: Using a regex here to avoid pulling in an XML parser dependency
        // this should be replaced with proper parsing if we add such a dependency
        // in the future.
        cap_as_str(&RESP_RE, &response, 1).map(|s| s.trim().to_string())
    } else {
        None
    })
}
