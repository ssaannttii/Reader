use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::RwLock;

use once_cell::sync::Lazy;
use regex::RegexBuilder;
use serde::{Deserialize, Serialize};
use thiserror::Error;

static LEXICON: Lazy<RwLock<Lexicon>> =
    Lazy::new(|| RwLock::new(Lexicon::load().unwrap_or_default()));

#[derive(Debug, Error)]
pub enum LexiconError {
    #[error("no se pudo leer el lexicón: {0}")]
    Io(String),
    #[error("formato de lexicón inválido: {0}")]
    Invalid(String),
    #[error("entrada duplicada para '{0}'")]
    Duplicate(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LexiconEntry {
    pub text: String,
    pub phonemes: String,
    #[serde(default)]
    pub case_sensitive: bool,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Lexicon {
    pub entries: Vec<LexiconEntry>,
}

impl Lexicon {
    fn load() -> Result<Self, LexiconError> {
        let path = lexicon_path();
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = fs::read_to_string(&path).map_err(|err| LexiconError::Io(err.to_string()))?;
        serde_json::from_str(&content).map_err(|err| LexiconError::Invalid(err.to_string()))
    }

    fn save(&self) -> Result<(), LexiconError> {
        let path = lexicon_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|err| LexiconError::Io(err.to_string()))?;
        }
        let data = serde_json::to_string_pretty(self)
            .map_err(|err| LexiconError::Invalid(err.to_string()))?;
        fs::write(path, data).map_err(|err| LexiconError::Io(err.to_string()))
    }
}

fn lexicon_path() -> PathBuf {
    PathBuf::from("assets/lexicon.es.json")
}

#[derive(Debug, Serialize)]
pub struct LexiconListResponse {
    pub ok: bool,
    pub entries: Vec<LexiconEntry>,
}

#[derive(Debug, Serialize)]
pub struct LexiconErrorResponse {
    pub ok: bool,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct UpsertPayload {
    pub text: String,
    pub phonemes: String,
    #[serde(default)]
    pub case_sensitive: bool,
}

#[derive(Debug, Deserialize)]
pub struct DeletePayload {
    pub text: String,
}

#[derive(Debug, Deserialize)]
pub struct PreviewPayload {
    pub sample: String,
}

#[derive(Debug, Serialize)]
pub struct PreviewResponse {
    pub ok: bool,
    pub transformed: String,
}

#[tauri::command]
pub fn list_entries() -> Result<LexiconListResponse, LexiconErrorResponse> {
    let guard = LEXICON.read().unwrap();
    Ok(LexiconListResponse {
        ok: true,
        entries: guard.entries.clone(),
    })
}

#[tauri::command]
pub fn upsert_entry(payload: UpsertPayload) -> Result<LexiconListResponse, LexiconErrorResponse> {
    let mut guard = LEXICON.write().unwrap();
    if guard
        .entries
        .iter()
        .any(|entry| entry.text == payload.text && entry.phonemes == payload.phonemes)
    {
        return Err(LexiconErrorResponse {
            ok: false,
            message: format!(
                "La entrada '{0}' ya existe con los mismos valores",
                payload.text
            ),
        });
    }

    if let Some(existing) = guard
        .entries
        .iter_mut()
        .find(|entry| entry.text == payload.text)
    {
        existing.phonemes = payload.phonemes.clone();
        existing.case_sensitive = payload.case_sensitive;
    } else {
        guard.entries.push(LexiconEntry {
            text: payload.text.clone(),
            phonemes: payload.phonemes.clone(),
            case_sensitive: payload.case_sensitive,
        });
    }

    if let Err(err) = guard.save() {
        return Err(LexiconErrorResponse {
            ok: false,
            message: err.to_string(),
        });
    }

    Ok(LexiconListResponse {
        ok: true,
        entries: guard.entries.clone(),
    })
}

#[tauri::command]
pub fn delete_entry(payload: DeletePayload) -> Result<LexiconListResponse, LexiconErrorResponse> {
    let mut guard = LEXICON.write().unwrap();
    guard.entries.retain(|entry| entry.text != payload.text);
    if let Err(err) = guard.save() {
        return Err(LexiconErrorResponse {
            ok: false,
            message: err.to_string(),
        });
    }
    Ok(LexiconListResponse {
        ok: true,
        entries: guard.entries.clone(),
    })
}

#[tauri::command]
pub fn apply_preview(payload: PreviewPayload) -> Result<PreviewResponse, LexiconErrorResponse> {
    let transformed = apply_replacements(&payload.sample);
    Ok(PreviewResponse {
        ok: true,
        transformed,
    })
}

pub fn apply_replacements(input: &str) -> String {
    let guard = LEXICON.read().unwrap();
    let mut result = input.to_string();
    for entry in &guard.entries {
        if entry.case_sensitive {
            result = result.replace(&entry.text, &entry.phonemes);
        } else {
            let pattern = regex::escape(&entry.text);
            let regex = RegexBuilder::new(&pattern)
                .case_insensitive(true)
                .build()
                .expect("regex válida");
            result = regex
                .replace_all(&result, entry.phonemes.as_str())
                .into_owned();
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replaces_case_insensitive() {
        let mut guard = LEXICON.write().unwrap();
        guard.entries = vec![LexiconEntry {
            text: "Hola".into(),
            phonemes: "O LA".into(),
            case_sensitive: false,
        }];
        assert_eq!(apply_replacements("hola mundo"), "O LA mundo");
    }
}
