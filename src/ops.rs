use anyhow::Result;

use std::fs::{create_dir_all, File};
use std::io::BufWriter;
use std::path::Path;

use crate::Entry;

pub fn write_entry(root: &Path, key: &str, entry: &Entry) -> Result<()> {
    let dir = root.join(key);
    let path = dir.join(format!("{key}.yaml"));
    if path.exists() {
        println!(
            "Entry already exists at: {} (skipping)",
            path.to_string_lossy()
        );
        return Ok(());
    }
    create_dir_all(&dir)?;
    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    serde_yaml::to_writer(writer, &entry)?;
    Ok(())
}

pub fn update_metadata(entry: &mut Entry, key: &str) -> Result<()> {
    let old_id = entry.insert("id".to_string(), key.clone().into());
    if let Some(i) = old_id {
        entry.insert("legacy-id".to_string(), i);
    }
    let old_key = entry.insert("citation_key".to_string(), key.clone().into());
    if let Some(k) = old_key {
        entry.insert("legacy-citation-key".to_string(), k);
    }
    Ok(())
}
