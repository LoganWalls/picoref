use anyhow::Result;

use std::fs::create_dir_all;
use std::path::{Path, PathBuf};

use crate::entry::{Entry, EntryData};

pub fn entry_root_path(root: &Path, key: &str) -> PathBuf {
    root.join(key)
}

pub fn data_path(root: &Path, key: &str) -> PathBuf {
    entry_root_path(root, key).join(format!("{key}.toml"))
}

pub fn pdf_path(root: &Path, key: &str) -> PathBuf {
    entry_root_path(root, key).join(format!("{key}.pdf"))
}

pub fn key_from_path(path: &Path) -> &str {
    path.file_name().unwrap().to_str().unwrap()
}

pub fn all_entry_paths(root: &Path) -> Result<Vec<PathBuf>> {
    Ok(std::fs::read_dir(root)?
        .filter_map(|p| {
            let p = p.unwrap().path();
            let key = p.file_name().unwrap().to_str().unwrap();
            if p.is_dir() && data_path(root, key).exists() {
                Some(p)
            } else {
                None
            }
        })
        .collect())
}

pub fn all_keys(root: &Path) -> Result<Vec<String>> {
    Ok(all_entry_paths(root)?
        .iter()
        .map(|p| key_from_path(p).to_string())
        .collect())
}

pub fn read_entry(path: &Path) -> Result<Entry> {
    let content = std::fs::read_to_string(path)?;
    Ok(toml::from_str(&content)?)
}

pub fn write_entry(root: &Path, key: &str, data: &EntryData, overwrite: bool) -> Result<()> {
    let dir = entry_root_path(root, key);
    let path = data_path(root, key);
    if path.exists() && !overwrite {
        println!(
            "Entry already exists at: {} (skipping)",
            path.to_string_lossy()
        );
        return Ok(());
    }
    create_dir_all(dir)?;
    let content = toml::to_string_pretty(&data)?;
    std::fs::write(path, content)?;
    Ok(())
}

pub fn update_metadata(data: &mut EntryData, key: &str) -> Result<()> {
    let old_id = data.fields.insert("id".to_string(), key.into());
    if let Some(i) = old_id {
        data.fields.insert("legacy-id".to_string(), i);
    }
    let old_key = data.fields.insert("citation-key".to_string(), key.into());
    if let Some(k) = old_key {
        data.fields.insert("legacy-citation-key".to_string(), k);
    }
    Ok(())
}
