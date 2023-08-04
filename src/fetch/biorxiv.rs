use anyhow::Result;
use once_cell::sync::Lazy;
use regex::Regex;

pub static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)(10.1101/.+)").unwrap());

pub fn published_doi(biorxiv_id: &str) -> Result<Option<String>> {
    fn try_fetch(url: &str) -> Result<Option<String>> {
        Ok(ureq::get(url)
            .set("Accept", "application/json; charset=utf-8")
            .call()?
            .into_json::<serde_json::Value>()?
            .get("collection")
            .and_then(|collection| collection.get(0))
            .and_then(|metadata| metadata.get("published_doi"))
            .and_then(|value| value.as_str())
            .map(|s| s.to_string()))
    }

    let mut result = try_fetch(&format!(
        "https://api.biorxiv.org/pubs/biorxiv/{biorxiv_id}"
    ))?;
    if result.is_none() {
        result = try_fetch(&format!(
            "https://api.biorxiv.org/pubs/medrxiv/{biorxiv_id}"
        ))?;
    }
    Ok(result)
}

pub fn pdf_url(biorxiv_id: &str) -> String {
    format!("https://www.biorxiv.org/content/{biorxiv_id}.full.pdf")
}
