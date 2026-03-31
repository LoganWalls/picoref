use anyhow::Result;
use serde::Deserialize;

#[derive(Deserialize)]
struct Response {
    records: Vec<Record>,
}

#[derive(Deserialize)]
struct Record {
    pmcid: Option<String>,
}

pub fn pdf_url(doi: &str) -> Result<Option<String>> {
    let response =
        ureq::get("https://pmc.ncbi.nlm.nih.gov/tools/idconv/api/v1/articles/")
            .query("ids", doi)
            .query("format", "json")
            .call()?
            .into_json::<Response>()?;
    let pmcid = response
        .records
        .into_iter()
        .find_map(|r| r.pmcid);
    Ok(pmcid.map(|id| format!("https://pmc.ncbi.nlm.nih.gov/articles/{id}/pdf/")))
}
