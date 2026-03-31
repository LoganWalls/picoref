use anyhow::Result;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Response {
    open_access_pdf: Option<OpenAccessPdf>,
}

#[derive(Deserialize)]
struct OpenAccessPdf {
    url: String,
}

pub fn pdf_url(doi: &str) -> Result<Option<String>> {
    let response = ureq::get(&format!(
        "https://api.semanticscholar.org/graph/v1/paper/DOI:{doi}"
    ))
    .query("fields", "openAccessPdf")
    .call()?
    .into_json::<Response>()?;
    Ok(response
        .open_access_pdf
        .map(|pdf| pdf.url)
        .filter(|url| !url.is_empty()))
}
