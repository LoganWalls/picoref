use once_cell::sync::Lazy;
use regex::Regex;

pub fn cap_as_str<'a>(re: &Lazy<Regex>, haystack: &'a str, group: usize) -> Option<&'a str> {
    re.captures(haystack)
        .and_then(|c| c.get(group))
        .map(|m| m.as_str())
}
