use anyhow::Result;
use once_cell::sync::Lazy;
use regex::Regex;

use crate::regex::cap_as_str;

static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)10.\d{4,9}/osf.io/(.+)").unwrap());

pub fn published_doi(doi: &str) -> Result<Option<String>> {
    Ok(if let Some(osf_id) = cap_as_str(&RE, doi, 1) {
        ureq::get(&format!("https://api.osf.io/v2/preprints/{osf_id}"))
            .set("Accept", "application/vnd.api+json; charset=utf-8")
            .call()?
            .into_json::<serde_json::Value>()?
            .get("data")
            .and_then(|data| data.get("attributes"))
            .and_then(|attrs| attrs.get("doi"))
            .and_then(|value| value.as_str())
            .map(|s| s.to_string())
    } else {
        None
    })
}
