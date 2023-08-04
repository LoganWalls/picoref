use anyhow::{Context, Result};
use once_cell::sync::Lazy;
use regex::Regex;

pub static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)10.\d{4,9}/osf.io/(.+)").unwrap());

pub fn published_doi(osf_id: &str) -> Result<Option<String>> {
    Ok(
        ureq::get(&format!("https://api.osf.io/v2/preprints/{osf_id}"))
            .set("Accept", "application/vnd.api+json; charset=utf-8")
            .call()?
            .into_json::<serde_json::Value>()?
            .get("data")
            .and_then(|data| data.get("attributes"))
            .and_then(|attrs| attrs.get("doi"))
            .and_then(|value| value.as_str())
            .map(|s| s.to_string()),
    )
}

pub fn pdf_url(osf_id: &str) -> Result<String> {
    Ok(ureq::get(&format!(
        "https://api.osf.io/v2/preprints/{osf_id}/files/osfstorage/"
    ))
    .set("Accept", "application/vnd.api+json; charset=utf-8")
    .call()?
    .into_json::<serde_json::Value>()?["data"][0]["links"]["download"]
        .as_str()
        .context("Could not find download link")?
        .to_string())
}
