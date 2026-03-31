use anyhow::Result;
use serde::Deserialize;

#[derive(Deserialize)]
struct Response {
    open_access: OpenAccess,
    best_oa_location: Option<Location>,
    locations: Vec<Location>,
}

#[derive(Deserialize)]
struct OpenAccess {
    oa_url: Option<String>,
}

#[derive(Deserialize)]
struct Location {
    pdf_url: Option<String>,
}

pub fn pdf_url(doi: &str, email: &str) -> Result<Option<String>> {
    let response = ureq::get(&format!("https://api.openalex.org/works/doi:{doi}"))
        .query("mailto", email)
        .call()?
        .into_json::<Response>()?;
    let url = response
        .best_oa_location
        .and_then(|loc| loc.pdf_url)
        .or_else(|| {
            response
                .locations
                .into_iter()
                .find_map(|loc| loc.pdf_url)
        })
        .or(response.open_access.oa_url);
    Ok(url)
}
