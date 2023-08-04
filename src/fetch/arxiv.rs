use anyhow::Result;
use once_cell::sync::Lazy;
use regex::Regex;

use crate::regex::cap_as_str;

pub static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)10.48550/arxiv\.(.+)").unwrap());
static RESP_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)<arxiv:doi.*?>(.+?)</arxiv:doi>").unwrap());

pub fn published_doi(arxiv_id: &str) -> Result<Option<String>> {
    let response = ureq::get(&format!(
        "http://export.arxiv.org/api/query?id_list={arxiv_id}"
    ))
    .set("Accept", "application/xml; charset=utf-8")
    .call()?
    .into_string()?;
    // HACK: Using a regex here to avoid pulling in an XML parser dependency
    // this should be replaced with proper parsing if we add such a dependency
    // in the future.
    Ok(cap_as_str(&RESP_RE, &response, 1).map(|s| s.trim().to_string()))
}

pub fn pdf_url(arxiv_id: &str) -> String {
    format!("https://arxiv.org/pdf/{arxiv_id}")
}
