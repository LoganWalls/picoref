use anyhow::Result;
use biblatex::{ChunksExt, Entry};
use deunicode::deunicode_with_tofu;
use once_cell::sync::Lazy;
use regex::Regex;

use crate::date::get_year;
use crate::regex::cap_as_str;

const STOPWORDS: [&str; 22] = [
    "of", "the", "and", "in", "for", "a", "on", "with", "to", "from", "an", "at", "by", "as",
    "its", "is", "via", "using", "through", "their", "some", "are",
];

static RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^((https?://)?www[.\-])?(?P<t>[A-Za-z0-9]{1,10})").unwrap());
static WHITESPACE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s+").unwrap());
static UNSUPPORTED_CHAR_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(<.*?>)|[^\w\-\s]").unwrap());

/// Generates a citekey based on the metadata contained in `data`
pub fn get_key(data: &Entry) -> Result<String> {
    let name = data
        .author()
        .map(|authors| {
            let author = authors.first().expect("should have at least one author");
            if author.name.is_empty() {
                author.given_name.to_owned()
            } else {
                author.name.to_owned()
            }
        })
        .map(|s| deunicode_with_tofu(&s, "").to_lowercase())
        .map(|s| cap_as_str(&RE, &s, 1).unwrap_or(&s).to_owned())
        .map(|s| UNSUPPORTED_CHAR_RE.replace_all(&s, " ").to_string())
        .map(|s| WHITESPACE_RE.replace_all(&s, " ").trim().replace(' ', "-"))
        .unwrap_or("unkown".to_string());

    let year = get_year(data.date().ok());

    // Filters out stopwords, then takes all of the remaining complete words
    // separated by underscores, up to a total length of 15 characters.
    let short_title = data
        .short_title()
        .or_else(|_| data.title())
        .map(|chunks| chunks.to_biblatex_string(false))
        .map(|s| deunicode_with_tofu(&WHITESPACE_RE.replace_all(&s, " "), ""))
        .map(|s| {
            UNSUPPORTED_CHAR_RE
                .replace_all(&s, "")
                .to_lowercase()
                .split_whitespace()
                .filter(|w| !STOPWORDS.contains(w))
                .collect::<Vec<_>>()
                .join("-")
        })
        .as_ref()
        .map(|s| s.split('-').take(2).collect::<Vec<_>>().join("-"))
        .unwrap_or("no-title".to_string());

    Ok([name, year, short_title].join("_"))
}
