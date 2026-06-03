use anyhow::{Context, Result};
use arboard::Clipboard;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_HISTORY_ENTRIES: usize = 100;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: String,
    pub created_at_ms: u64,
    pub text: String,
}

pub fn path() -> Result<PathBuf> {
    let dir = dirs::data_dir()
        .context("Could not find Application Support directory")?
        .join("whispy");
    fs::create_dir_all(&dir)?;
    Ok(dir.join("history.json"))
}

pub fn list() -> Result<Vec<HistoryEntry>> {
    let path = path()?;
    if !path.exists() {
        return Ok(Vec::new());
    }

    let data = fs::read_to_string(&path)?;
    let mut entries: Vec<HistoryEntry> = serde_json::from_str(&data)?;
    entries.sort_by(|a, b| b.created_at_ms.cmp(&a.created_at_ms));
    entries.truncate(MAX_HISTORY_ENTRIES);
    Ok(entries)
}

pub fn save_transcription(text: &str) -> Result<()> {
    let text = text.trim();
    if text.is_empty() {
        return Ok(());
    }

    let mut entries = list()?;
    let created_at_ms = now_ms();
    entries.insert(
        0,
        HistoryEntry {
            id: unique_id(created_at_ms, &entries),
            created_at_ms,
            text: text.to_string(),
        },
    );
    entries.truncate(MAX_HISTORY_ENTRIES);
    save_entries(&entries)
}

pub fn copy_entry(id: &str) -> Result<()> {
    let entries = list()?;
    let entry = entries
        .iter()
        .find(|entry| entry.id == id)
        .context("History entry not found")?;

    let mut clipboard = Clipboard::new()?;
    clipboard.set_text(&entry.text)?;
    Ok(())
}

pub fn clear() -> Result<()> {
    save_entries(&[])
}

fn save_entries(entries: &[HistoryEntry]) -> Result<()> {
    let data = serde_json::to_string_pretty(entries)?;
    fs::write(path()?, data)?;
    Ok(())
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn unique_id(created_at_ms: u64, entries: &[HistoryEntry]) -> String {
    let mut suffix = 0;
    loop {
        let id = if suffix == 0 {
            created_at_ms.to_string()
        } else {
            format!("{created_at_ms}-{suffix}")
        };
        if entries.iter().all(|entry| entry.id != id) {
            return id;
        }
        suffix += 1;
    }
}
