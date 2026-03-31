use std::fs::File;
use std::path::Path;
use std::time::Duration;

use anyhow::Result;
use indicatif::ProgressBar;

use crate::entry::Source;
use crate::fetch;

pub fn download_pdf(source: &Source, email: &str, dest: &Path) -> Result<()> {
    let mut pb = ProgressBar::new_spinner().with_message("Searching for PDF");
    pb.enable_steady_tick(Duration::from_millis(100));
    let pdf_url = fetch::fetch_pdf_url(source, email)?;
    pb.finish_with_message("PDF found");
    let pdf_response = ureq::get(&pdf_url).call()?;
    let content_type = pdf_response.content_type().to_owned();
    if content_type != "application/pdf" {
        eprintln!("Found a URL but it is not a direct PDF link:");
        eprintln!("  {pdf_url}");
        eprintln!();
        eprintln!("Download the PDF manually, then run:");
        eprintln!("  picoref pdf <key> --file <path-to-pdf>");
        anyhow::bail!("Could not automatically download PDF");
    }
    let mut pdf_data = pdf_response.into_reader();
    pb = ProgressBar::new_spinner().with_message("Downloading");
    pb.enable_steady_tick(Duration::from_millis(100));
    let mut new_file = File::create(dest)?;
    std::io::copy(&mut pdf_data, &mut new_file)?;
    pb.finish_with_message("Download complete");
    Ok(())
}
