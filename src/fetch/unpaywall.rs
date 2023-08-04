use std::cmp::Ordering;

use anyhow::Result;
use serde::Deserialize;

/// A response from the Unpaywall API
/// See https://unpaywall.org/data-format
#[allow(dead_code)]
#[derive(Deserialize)]
struct Response {
    is_oa: bool,
    best_oa_location: Option<OaLocation>,
    oa_locations: Vec<OaLocation>,
    doi_url: String,
    published_date: Option<String>,
    updated: String,
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct OaLocation {
    is_best: bool,
    url_for_pdf: Option<String>,
    url_for_landing_page: String,
    updated: String,
    version: Version,
}

#[derive(Deserialize, PartialEq, Eq, Clone, Copy)]
enum Version {
    #[serde(rename = "submittedVersion")]
    Submitted,
    #[serde(rename = "acceptedVersion")]
    Accepted,
    #[serde(rename = "publishedVersion")]
    Published,
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            _ if self == other => Ordering::Equal,
            (Self::Published, _) | (Self::Accepted, Self::Submitted) => Ordering::Greater,
            (Self::Submitted, _) | (Self::Accepted, Self::Published) => Ordering::Less,
            _ => unreachable!(),
        }
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

pub fn pdf_url(doi: &str, email: &str) -> Result<Option<String>> {
    let response = ureq::get(&format!("https://api.unpaywall.org/v2/{doi}"))
        .set("Accept", "application/json; charset=utf-8")
        .query("email", email)
        .call()?
        .into_json::<Response>()?;
    let result = response
        .oa_locations
        .into_iter()
        .filter(|loc| loc.url_for_pdf.is_some())
        .max_by_key(|loc| loc.version);

    if matches!(&result, Some(loc) if loc.version != Version::Published) {
        eprintln!("WARNING: Published version of PDF not available for automatic download.");
        if let Some(OaLocation {
            version: Version::Published,
            ref url_for_landing_page,
            ..
        }) = response.best_oa_location
        {
            eprintln!("Published version available for manual download at: {url_for_landing_page}",);
        }
    }
    Ok(result.and_then(|loc| loc.url_for_pdf))
}
