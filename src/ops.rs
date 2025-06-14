use anyhow::Result;
use biblatex::{Chunk, Entry, Spanned};

use std::fs::{create_dir_all, File};
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};

pub fn entry_root_path(root: &Path, key: &str) -> PathBuf {
    root.join(key)
}

pub fn data_path(root: &Path, key: &str) -> PathBuf {
    entry_root_path(root, key).join(format!("{key}.yaml"))
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
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    Ok(serde_yaml::from_reader(reader)?)
}

pub fn write_entry(root: &Path, key: &str, data: &Entry, overwrite: bool) -> Result<()> {
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
    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    serde_yaml::to_writer(writer, &data)?;
    Ok(())
}

pub fn update_metadata(data: &mut Entry, key: &str) -> Result<()> {
    let old_id = data.get("id");
    if let Some(i) = old_id {
        data.set("legacy-id", i.to_vec());
    }
    let old_key = data.get("citation-key");
    if let Some(k) = old_key {
        data.set("legacy-citation-key", k.to_vec());
    }
    let key_chunks = vec![Spanned::detached(Chunk::Normal(key.to_string()))];
    data.set("id", key_chunks.clone());
    data.set("citation-key", key_chunks);
    Ok(())
}
