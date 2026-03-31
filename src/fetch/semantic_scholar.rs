use anyhow::Result;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Response {
    title: Option<String>,
    open_access_pdf: Option<OpenAccessPdf>,
}

#[derive(Deserialize)]
struct OpenAccessPdf {
    url: String,
}

pub fn pdf_url(doi: &str, expected_title: Option<&str>) -> Result<Option<String>> {
    let response = ureq::get(&format!(
        "https://api.semanticscholar.org/graph/v1/paper/DOI:{doi}"
    ))
    .query("fields", "title,openAccessPdf")
    .call()?
    .into_json::<Response>()?;
    if let (Some(expected), Some(actual)) = (expected_title, response.title.as_deref()) {
        if expected.trim().to_lowercase() != actual.trim().to_lowercase() {
            eprintln!(
                "WARNING: Semantic Scholar returned a different paper title for this DOI, skipping."
            );
            return Ok(None);
        }
    }
    Ok(response
        .open_access_pdf
        .map(|pdf| pdf.url)
        .filter(|url| !url.is_empty()))
}
