use biblatex::{Date, DateValue, PermissiveType};
use once_cell::sync::Lazy;
use regex::Regex;

static YEAR_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"([0-9]{4})").unwrap());

pub fn get_year(date: Option<PermissiveType<Date>>) -> String {
    date.and_then(|d| match d {
        PermissiveType::Typed(d) => Some(match d.value {
            DateValue::Between(_, t) => t.year,
            DateValue::At(t) | DateValue::After(t) | DateValue::Before(t) => t.year,
        }),
        PermissiveType::Chunks(chunks) => chunks.into_iter().find_map(|c| {
            YEAR_RE
                .find(c.v.get())
                .map(|m| m.as_str().parse::<i32>().unwrap())
        }),
    })
    .map(|year| {
        if year < 0 {
            format!("{year}-bc")
        } else {
            year.to_string()
        }
    })
    .unwrap_or_else(|| "xxxx".to_string())
}
