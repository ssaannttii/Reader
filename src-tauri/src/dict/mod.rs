//! Pronunciation dictionary utilities.
//!
//! Persist custom lexicon entries (JSON/SQLite) and expose helpers for
//! mapping words to phonetic sequences compatible with Piper.

use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use thiserror::Error;

const DEFAULT_DICTIONARY_PATH: &str = "runtime/phonetic_dictionary.json";
const DICTIONARY_PATH_ENV: &str = "READER_DICTIONARY_PATH";

fn canonical_key(word: &str) -> String {
    word.trim().to_lowercase()
}

fn is_token_character(ch: char) -> bool {
    ch.is_alphanumeric() || matches!(ch, '\'' | '-')
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DictionaryEntry {
    pub word: String,
    pub phonemes: String,
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
struct DictionaryFile {
    entries: Vec<DictionaryEntry>,
}

#[derive(Debug, Error)]
pub enum DictionaryError {
    #[error("failed to read dictionary at {path:?}: {source}")]
    ReadFailure {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("failed to parse dictionary at {path:?}: {source}")]
    ParseFailure {
        path: PathBuf,
        source: serde_json::Error,
    },
    #[error("failed to write dictionary at {path:?}: {source}")]
    WriteFailure {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("failed to serialise dictionary for {path:?}: {source}")]
    SerialiseFailure {
        path: PathBuf,
        source: serde_json::Error,
    },
}

#[derive(Debug, Clone)]
pub struct PhoneticDictionary {
    path: PathBuf,
    entries: HashMap<String, DictionaryEntry>,
}

impl PhoneticDictionary {
    pub fn load(path: impl Into<PathBuf>) -> Result<Self, DictionaryError> {
        let path = path.into();
        let entries = if path.exists() {
            let contents =
                fs::read_to_string(&path).map_err(|source| DictionaryError::ReadFailure {
                    path: path.clone(),
                    source,
                })?;

            if contents.trim().is_empty() {
                Vec::new()
            } else {
                let file: DictionaryFile = serde_json::from_str(&contents).map_err(|source| {
                    DictionaryError::ParseFailure {
                        path: path.clone(),
                        source,
                    }
                })?;
                file.entries
            }
        } else {
            Vec::new()
        };

        Ok(Self {
            path,
            entries: entries
                .into_iter()
                .map(|entry| (canonical_key(&entry.word), entry))
                .collect(),
        })
    }

    pub fn load_from_env() -> Result<Self, DictionaryError> {
        let path = std::env::var_os(DICTIONARY_PATH_ENV)
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(DEFAULT_DICTIONARY_PATH));
        Self::load(path)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn save(&self) -> Result<(), DictionaryError> {
        if let Some(parent) = self.path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent).map_err(|source| DictionaryError::WriteFailure {
                    path: parent.to_path_buf(),
                    source,
                })?;
            }
        }

        let mut entries: Vec<_> = self.entries.values().cloned().collect();
        entries.sort_by(|a, b| a.word.to_lowercase().cmp(&b.word.to_lowercase()));

        let file = DictionaryFile { entries };
        let payload = serde_json::to_string_pretty(&file).map_err(|source| {
            DictionaryError::SerialiseFailure {
                path: self.path.clone(),
                source,
            }
        })?;

        fs::write(&self.path, payload).map_err(|source| DictionaryError::WriteFailure {
            path: self.path.clone(),
            source,
        })
    }

    pub fn get(&self, word: &str) -> Option<&DictionaryEntry> {
        self.entries.get(&canonical_key(word))
    }

    pub fn get_phonemes(&self, word: &str) -> Option<&str> {
        self.get(word).map(|entry| entry.phonemes.as_str())
    }

    pub fn insert(
        &mut self,
        word: impl Into<String>,
        phonemes: impl Into<String>,
    ) -> Option<DictionaryEntry> {
        let entry = DictionaryEntry {
            word: word.into(),
            phonemes: phonemes.into(),
        };
        self.entries.insert(canonical_key(&entry.word), entry)
    }

    pub fn upsert_entry(&mut self, entry: DictionaryEntry) -> Option<DictionaryEntry> {
        self.entries.insert(canonical_key(&entry.word), entry)
    }

    pub fn remove(&mut self, word: &str) -> Option<DictionaryEntry> {
        self.entries.remove(&canonical_key(word))
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn entries(&self) -> impl Iterator<Item = &DictionaryEntry> {
        self.entries.values()
    }

    pub fn transform_text(&self, text: &str) -> String {
        if self.entries.is_empty() {
            return text.to_string();
        }

        let mut result = String::with_capacity(text.len());
        let mut token = String::new();

        for ch in text.chars() {
            if is_token_character(ch) {
                token.push(ch);
            } else {
                self.flush_token(&mut token, &mut result);
                result.push(ch);
            }
        }

        self.flush_token(&mut token, &mut result);
        result
    }

    fn flush_token(&self, token: &mut String, output: &mut String) {
        if token.is_empty() {
            return;
        }

        if let Some(entry) = self.get(token) {
            output.push_str(&entry.phonemes);
        } else {
            output.push_str(token);
        }

        token.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_missing_dictionary_creates_empty_store() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("missing.json");
        let dictionary = PhoneticDictionary::load(&path).unwrap();
        assert!(dictionary.is_empty());
        assert_eq!(dictionary.path(), path.as_path());
    }

    #[test]
    fn persist_and_reload_entries() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("lexicon.json");
        let mut dictionary = PhoneticDictionary::load(&path).unwrap();
        dictionary.insert("gif", "JH IH1 F");
        dictionary.insert("SQL", "S Q L");
        dictionary.save().unwrap();

        let reloaded = PhoneticDictionary::load(&path).unwrap();
        assert_eq!(reloaded.get_phonemes("gif"), Some("JH IH1 F"));
        assert_eq!(reloaded.get_phonemes("sql"), Some("S Q L"));
    }

    #[test]
    fn transform_text_replaces_tokens() {
        use std::collections::HashMap;

        let mut dictionary = PhoneticDictionary {
            path: PathBuf::from("memory"),
            entries: HashMap::new(),
        };
        dictionary.insert("gif", "JH IH1 F");
        dictionary.insert("Na", "N EY1");
        let text = "Play that GIF, Na!";
        let transformed = dictionary.transform_text(text);
        assert_eq!(transformed, "Play that JH IH1 F, N EY1!");
    }
}
