use biblatex::{Chunk, Entry, Spanned};

pub const TAGS_FIELD: &str = "picoref_tags";

pub fn set_tags(entry: &mut Entry, tags: impl IntoIterator<Item = impl ToString>) {
    entry.set(
        TAGS_FIELD,
        tags.into_iter()
            .map(|s| Spanned::detached(Chunk::Normal(s.to_string())))
            .collect(),
    );
}

pub fn get_tags(entry: &Entry) -> Vec<String> {
    entry
        .get(TAGS_FIELD)
        .unwrap_or(&[])
        .iter()
        .map(|chunks| chunks.v.get().to_owned())
        .collect()
}
