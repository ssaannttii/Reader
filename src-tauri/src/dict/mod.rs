use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use parking_lot::RwLock;
use regex::Regex;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DictionaryError {
    #[error("failed to read dictionary file {0}: {1}")]
    Io(PathBuf, #[source] std::io::Error),
    #[error("failed to parse dictionary JSON {0}: {1}")]
    Parse(PathBuf, #[source] serde_json::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DictionaryEntry {
    pub word: String,
    pub replacement: String,
}

#[derive(Default)]
pub struct Dictionary {
    path: PathBuf,
    entries: RwLock<HashMap<String, String>>,
}

impl Dictionary {
    pub fn load_or_default(path: PathBuf) -> Result<Self, DictionaryError> {
        if !path.exists() {
            return Ok(Self {
                path,
                entries: RwLock::new(HashMap::new()),
            });
        }

        let data =
            fs::read_to_string(&path).map_err(|err| DictionaryError::Io(path.clone(), err))?;
        let parsed: Vec<DictionaryEntry> =
            serde_json::from_str(&data).map_err(|err| DictionaryError::Parse(path.clone(), err))?;
        let mut map = HashMap::new();
        for entry in parsed {
            if !entry.word.trim().is_empty() {
                map.insert(entry.word.to_lowercase(), entry.replacement);
            }
        }
        Ok(Self {
            path,
            entries: RwLock::new(map),
        })
    }

    pub fn entries(&self) -> Vec<DictionaryEntry> {
        self.entries
            .read()
            .iter()
            .map(|(word, replacement)| DictionaryEntry {
                word: word.clone(),
                replacement: replacement.clone(),
            })
            .collect()
    }

    pub fn update(&self, entries: Vec<DictionaryEntry>) -> Result<(), DictionaryError> {
        let mut map = HashMap::new();
        for entry in entries {
            if entry.word.trim().is_empty() {
                continue;
            }
            map.insert(entry.word.to_lowercase(), entry.replacement);
        }

        let serialisable: Vec<DictionaryEntry> = map
            .iter()
            .map(|(word, replacement)| DictionaryEntry {
                word: word.clone(),
                replacement: replacement.clone(),
            })
            .collect();

        let json = serde_json::to_string_pretty(&serialisable)
            .map_err(|err| DictionaryError::Parse(self.path.clone(), err))?;
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)
                .map_err(|err| DictionaryError::Io(parent.to_path_buf(), err))?;
        }
        fs::write(&self.path, json).map_err(|err| DictionaryError::Io(self.path.clone(), err))?;

        *self.entries.write() = map;
        Ok(())
    }

    pub fn apply(&self, input: &str) -> String {
        let entries = self.entries.read();
        if entries.is_empty() {
            return input.to_string();
        }

        let mut output = input.to_string();
        for (word, replacement) in entries.iter() {
            let pattern = Regex::new(&format!(r"(?i)\b{}\b", regex::escape(word)))
                .unwrap_or_else(|_| Regex::new(&regex::escape(word)).unwrap());
            output = pattern
                .replace_all(&output, replacement.as_str())
                .into_owned();
        }
        output
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::prelude::*;

    #[test]
    fn loads_dictionary_entries() {
        let file = assert_fs::NamedTempFile::new("dict.json").unwrap();
        file.write_str(r#"[{"word":"AI","replacement":"ei"}]"#)
            .unwrap();
        let dict = Dictionary::load_or_default(file.path().to_path_buf()).unwrap();
        assert_eq!(dict.entries().len(), 1);
    }

    #[test]
    fn applies_replacements_case_insensitively() {
        let dict = Dictionary::load_or_default(PathBuf::from("missing.json")).unwrap();
        dict.update(vec![DictionaryEntry {
            word: "hola".into(),
            replacement: "ola".into(),
        }])
        .unwrap();
        let result = dict.apply("Hola mundo hola");
        assert_eq!(result, "ola mundo ola");
    }
}
