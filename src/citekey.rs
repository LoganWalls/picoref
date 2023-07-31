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

/// Generates a citekey based on the metadata contained in `reference`
pub fn get_key(entry: &serde_json::Map<String, serde_json::Value>) -> Result<String> {
    let name = entry
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
        .as_ref()
        .map(|s| UNSUPPORTED_CHAR_RE.replace_all(s, " "))
        .map(|s| WHITESPACE_RE.replace_all(&s, " ").trim().replace(' ', "-"));
    let year = entry.get("issued").and_then(|issued| {
        Some(
            issued
                .get("date-parts")?
                .get(0)?
                .get(0)?
                .as_u64()?
                .to_string(),
        )
    });

    // Filters out stopwords, then takes all of the remaining complete words
    // separated by underscores, up to a total length of 15 characters.
    let short_title = entry
        .get("title_short")
        .or_else(|| entry.get("title"))
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
                        .join("-")
                })
                .as_ref()
                .map(|s| s.split('-').take(2).collect::<Vec<_>>().join("-"))
        });

    Ok([name, year, short_title]
        .into_iter()
        .flatten()
        .collect::<Vec<String>>()
        .join("_"))
}
