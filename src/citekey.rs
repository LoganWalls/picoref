use anyhow::Result;
use deunicode::deunicode_with_tofu;
use once_cell::sync::Lazy;
use regex::Regex;

use crate::regex::cap_as_str;

const STOPWORDS: [&str; 22] = [
    "of", "the", "and", "in", "for", "a", "on", "with", "to", "from", "an", "at", "by", "as",
    "its", "is", "via", "using", "through", "their", "some", "are",
];

static RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^((https?://)?www[.\-])?(?P<t>[A-Za-z0-9]{1,10})").unwrap());
static WHITESPACE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s+").unwrap());
static UNSUPPORTED_CHAR_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(<.*?>)|[^\w\-\s]").unwrap());
static SHORT_TITLE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^_?(\w{1,15})(_|$)").unwrap());

/// Generates a citekey based on the metadata contained in `reference`
pub fn get_key(reference: &serde_json::Map<String, serde_json::Value>) -> Result<String> {
    let name = reference
        .get("author")
        .and_then(|authors| authors.get(0))
        .and_then(|author| {
            author
                .get("family")?
                .as_str()
                .map(|s| deunicode_with_tofu(s, "").to_lowercase())
                .or_else(|| {
                    Some(
                        author
                            .get("literal")?
                            .as_str()
                            .map(|s| deunicode_with_tofu(s, "").to_lowercase())?
                            .split_whitespace()
                            .last()?
                            .to_string(),
                    )
                })
                .or_else(|| {
                    author
                        .get("source")
                        .and_then(|s| cap_as_str(&RE, s.as_str()?, 1))
                        .map(|s| s.to_string())
                })
        })
        .unwrap_or("noauthor".to_string());

    let year = reference
        .get("issued")
        .and_then(|issued| {
            Some(
                issued
                    .get("date-parts")?
                    .get(0)?
                    .get(0)?
                    .as_u64()?
                    .to_string(),
            )
        })
        .unwrap_or("".to_string());

    // Filters out stopwords, then takes all of the remaining complete words
    // separated by underscores, up to a total length of 15 characters.
    let short_title = reference
        .get("title_short")
        .or_else(|| reference.get("title"))
        .and_then(|title| {
            title
                .as_str()
                .map(|s| WHITESPACE_RE.replace_all(s, " "))
                .map(|s| deunicode_with_tofu(&s, ""))
                .map(|s| {
                    UNSUPPORTED_CHAR_RE
                        .replace_all(&s, "")
                        .to_lowercase()
                        .split_whitespace()
                        .filter(|w| !STOPWORDS.contains(w))
                        .collect::<Vec<_>>()
                        .join("_")
                })
                .as_ref()
                .and_then(|s| cap_as_str(&SHORT_TITLE_RE, s, 1).or_else(|| s.split('_').next()))
                .map(|s| s.to_string())
        })
        .unwrap_or("".to_string());

    Ok(format!("{name}{year}_{short_title}").to_string())
}
