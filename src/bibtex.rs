use biblatex::{
    Chunk, Date, DateValue, Datetime, Entry, EntryType, PermissiveType, Person, Spanned,
};
use serde_json::Value;

use crate::entry::EntryData;

fn chunks(s: &str) -> Vec<Spanned<Chunk>> {
    vec![Spanned::detached(Chunk::Normal(s.to_string()))]
}

fn map_entry_type(csl_type: &str) -> EntryType {
    match csl_type {
        "article-journal" | "article" | "article-magazine" | "article-newspaper" => {
            EntryType::Article
        }
        "book" => EntryType::Book,
        "chapter" => EntryType::InCollection,
        "paper-conference" => EntryType::InProceedings,
        "report" => EntryType::Report,
        "thesis" => EntryType::Thesis,
        "webpage" | "post-weblog" => EntryType::Online,
        "dataset" => EntryType::Dataset,
        _ => EntryType::Misc,
    }
}

fn parse_persons(value: &Value) -> Option<Vec<Person>> {
    let arr = value.as_array()?;
    let persons: Vec<_> = arr
        .iter()
        .filter_map(|p| {
            let family = p.get("family")?.as_str()?;
            let given = p.get("given").and_then(|g| g.as_str()).unwrap_or("");
            Some(Person {
                name: family.to_string(),
                given_name: given.to_string(),
                prefix: String::new(),
                suffix: String::new(),
            })
        })
        .collect();
    if persons.is_empty() {
        None
    } else {
        Some(persons)
    }
}

fn persons_to_bib_string(persons: &[Person]) -> String {
    persons
        .iter()
        .map(|p| {
            if p.given_name.is_empty() {
                p.name.clone()
            } else {
                format!("{}, {}", p.name, p.given_name)
            }
        })
        .collect::<Vec<_>>()
        .join(" and ")
}

fn parse_date(value: &Value) -> Option<PermissiveType<Date>> {
    let parts = value.get("date-parts")?.as_array()?.first()?.as_array()?;
    let year = parts.first()?.as_i64()? as i32;
    let month = parts
        .get(1)
        .and_then(|m| m.as_u64())
        .map(|m| m.saturating_sub(1) as u8);
    let day = parts.get(2).and_then(|d| d.as_u64()).map(|d| d as u8);
    Some(PermissiveType::Typed(Date {
        value: DateValue::At(Datetime {
            year,
            month,
            day,
            time: None,
        }),
        uncertain: false,
        approximate: false,
    }))
}

/// Normalize page ranges: collapse any run of hyphens to "--".
fn normalize_pages(pages: &str) -> String {
    pages
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("--")
}

pub fn to_bibtex(entries: impl IntoIterator<Item = EntryData>) -> anyhow::Result<String> {
    let mut output = Vec::new();

    for entry_data in entries {
        let fields = &entry_data.fields;

        let csl_type = fields
            .get("type")
            .and_then(|t| t.as_str())
            .unwrap_or("misc");
        let entry_type = map_entry_type(csl_type);
        let is_article = matches!(entry_type, EntryType::Article);

        let key = fields
            .get("citation-key")
            .or_else(|| fields.get("id"))
            .and_then(|k| k.as_str())
            .unwrap_or("unknown")
            .to_string();

        let mut entry = Entry::new(key, entry_type);

        if let Some(title) = fields.get("title").and_then(|t| t.as_str()) {
            entry.set_title(chunks(title));
        }

        if let Some(persons) = fields.get("author").and_then(parse_persons) {
            entry.set_author(persons);
        }

        if let Some(persons) = fields.get("editor").and_then(parse_persons) {
            entry.set("editor", chunks(&persons_to_bib_string(&persons)));
        }

        if let Some(container) = fields.get("container-title").and_then(|t| t.as_str()) {
            if is_article {
                entry.set_journal_title(chunks(container));
            } else {
                entry.set_book_title(chunks(container));
            }
        }

        if let Some(doi) = fields.get("DOI").and_then(|d| d.as_str()) {
            entry.set_doi(doi.to_string());
        }

        if let Some(url) = fields.get("URL").and_then(|u| u.as_str()) {
            entry.set_url(url.to_string());
        }

        if let Some(date) = fields.get("issued").and_then(parse_date) {
            entry.set_date(date);
        }

        if let Some(vol) = fields.get("volume") {
            if let Some(n) = vol.as_i64() {
                entry.set_volume(PermissiveType::Typed(n));
            } else if let Some(s) = vol.as_str() {
                match s.parse::<i64>() {
                    Ok(n) => entry.set_volume(PermissiveType::Typed(n)),
                    Err(_) => entry.set_volume(PermissiveType::Chunks(chunks(s))),
                }
            }
        }

        if let Some(num) = fields.get("issue").or_else(|| fields.get("number")) {
            let s = num
                .as_str()
                .map(|s| s.to_string())
                .or_else(|| num.as_i64().map(|n| n.to_string()));
            if let Some(s) = s {
                entry.set_number(chunks(&s));
            }
        }

        if let Some(pages) = fields.get("page").and_then(|p| p.as_str()) {
            entry.set_pages(PermissiveType::Chunks(chunks(&normalize_pages(pages))));
        }

        if let Some(publisher) = fields.get("publisher").and_then(|p| p.as_str()) {
            entry.set_publisher(vec![chunks(publisher)]);
        }

        if let Some(issn) = fields.get("ISSN") {
            let s = issn
                .as_str()
                .map(|s| s.to_string())
                .or_else(|| issn.as_array()?.first()?.as_str().map(|s| s.to_string()));
            if let Some(s) = s {
                entry.set_issn(chunks(&s));
            }
        }

        if let Some(isbn) = fields.get("ISBN") {
            let s = isbn
                .as_str()
                .map(|s| s.to_string())
                .or_else(|| isbn.as_array()?.first()?.as_str().map(|s| s.to_string()));
            if let Some(s) = s {
                entry.set_isbn(chunks(&s));
            }
        }

        if let Some(abs) = fields.get("abstract").and_then(|a| a.as_str()) {
            entry.set_abstract_(chunks(abs));
        }

        if let Some(ed) = fields.get("edition") {
            if let Some(n) = ed.as_i64() {
                entry.set_edition(PermissiveType::Typed(n));
            } else if let Some(s) = ed.as_str() {
                match s.parse::<i64>() {
                    Ok(n) => entry.set_edition(PermissiveType::Typed(n)),
                    Err(_) => entry.set_edition(PermissiveType::Chunks(chunks(s))),
                }
            }
        }

        output.push(entry.to_biblatex_string());
    }

    Ok(output.join("\n"))
}
