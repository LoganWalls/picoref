use anyhow::Result;

use std::fs::{create_dir_all, File};
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};

use crate::entry::{Entry, EntryData};

pub fn entry_root_path(root: &Path, key: &str) -> PathBuf {
    root.join(key)
}

pub fn data_path(root: &Path, key: &str) -> PathBuf {
    entry_root_path(root, key).join(format!("{key}.yaml"))
}

pub fn pdf_path(root: &Path, key: &str) -> PathBuf {
    entry_root_path(root, key).join(format!("{key}.pdf"))
}

pub fn read_entry(root: &Path, key: &str) -> Result<Entry> {
    let path = data_path(root, key);
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    Ok(serde_yaml::from_reader(reader)?)
}

pub fn write_entry(root: &Path, key: &str, data: &EntryData) -> Result<()> {
    let dir = entry_root_path(root, key);
    let path = data_path(root, key);
    if path.exists() {
        println!(
            "Entry already exists at: {} (skipping)",
            path.to_string_lossy()
        );
        return Ok(());
    }
    create_dir_all(dir)?;
    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    serde_yaml::to_writer(writer, &data)?;
    Ok(())
}

pub fn update_metadata(data: &mut EntryData, key: &str) -> Result<()> {
    let old_id = data.insert("id".to_string(), key.clone().into());
    if let Some(i) = old_id {
        data.insert("legacy-id".to_string(), i);
    }
    let old_key = data.insert("citation-key".to_string(), key.clone().into());
    if let Some(k) = old_key {
        data.insert("legacy-citation-key".to_string(), k);
    }
    Ok(())
}
