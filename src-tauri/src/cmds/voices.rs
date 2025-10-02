use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use parking_lot::RwLock;
use serde::Serialize;
use serde_json::Value;
use thiserror::Error;
use walkdir::WalkDir;

#[derive(Debug, Error)]
pub enum VoiceError {
    #[error("voice '{0}' not found")]
    NotFound(String),
    #[error("failed to read metadata {0}: {1}")]
    Metadata(PathBuf, #[source] std::io::Error),
    #[error("failed to parse metadata {0}: {1}")]
    MetadataParse(PathBuf, #[source] serde_json::Error),
}

#[derive(Debug, Clone, Serialize)]
pub struct VoiceInfo {
    pub id: String,
    pub label: String,
    pub language: Option<String>,
    pub quality: Option<String>,
    pub model_path: String,
    pub config_path: Option<String>,
}

#[derive(Default)]
pub struct VoiceLibrary {
    base_dir: PathBuf,
    voices: RwLock<HashMap<String, VoiceInfo>>,
}

impl VoiceLibrary {
    pub fn new(base_dir: PathBuf) -> Self {
        let library = Self {
            base_dir,
            voices: RwLock::new(HashMap::new()),
        };
        library.refresh().ok();
        library
    }

    pub fn refresh(&self) -> std::io::Result<()> {
        let mut discovered = HashMap::new();
        if self.base_dir.exists() {
            for entry in WalkDir::new(&self.base_dir)
                .into_iter()
                .filter_map(Result::ok)
            {
                if !entry.file_type().is_file() {
                    continue;
                }
                let path = entry.path();
                if path.extension().and_then(|ext| ext.to_str()) != Some("onnx") {
                    continue;
                }
                if let Some(info) = build_voice_info(path) {
                    discovered.insert(info.id.clone(), info);
                }
            }
        }
        *self.voices.write() = discovered;
        Ok(())
    }

    pub fn list(&self) -> Vec<VoiceInfo> {
        let mut voices: Vec<_> = self.voices.read().values().cloned().collect();
        voices.sort_by(|a, b| a.label.cmp(&b.label));
        voices
    }

    pub fn get(&self, id: &str) -> Result<VoiceInfo, VoiceError> {
        self.voices
            .read()
            .get(id)
            .cloned()
            .ok_or_else(|| VoiceError::NotFound(id.to_string()))
    }

    pub fn base_dir(&self) -> &Path {
        &self.base_dir
    }
}

fn build_voice_info(path: &Path) -> Option<VoiceInfo> {
    let id = path.file_stem()?.to_string_lossy().to_string();
    let metadata_path = metadata_path_for(path);
    let metadata = metadata_path
        .as_ref()
        .and_then(|path| match fs::read_to_string(path) {
            Ok(contents) => serde_json::from_str::<Value>(&contents)
                .map_err(|err| {
                    log::warn!("Failed to parse metadata {}: {err}", path.display());
                    err
                })
                .ok(),
            Err(err) => {
                log::warn!("Failed to read metadata {}: {err}", path.display());
                None
            }
        });

    let label = metadata
        .as_ref()
        .and_then(|value| value.get("language"))
        .and_then(|lang| lang.get("name_native").or_else(|| lang.get("name")))
        .and_then(Value::as_str)
        .map(|lang| format!("{lang} · {id}"))
        .unwrap_or_else(|| id.clone());

    let quality = metadata
        .as_ref()
        .and_then(|value| value.get("audio"))
        .and_then(|audio| audio.get("quality"))
        .and_then(Value::as_str)
        .map(|s| s.to_string());

    Some(VoiceInfo {
        id,
        label,
        language: metadata
            .as_ref()
            .and_then(|value| value.get("language"))
            .and_then(|lang| lang.get("code"))
            .and_then(Value::as_str)
            .map(|s| s.to_string()),
        quality,
        model_path: path.to_string_lossy().to_string(),
        config_path: metadata_path.map(|path| path.to_string_lossy().to_string()),
    })
}

fn metadata_path_for(path: &Path) -> Option<PathBuf> {
    let mut metadata_path = path.to_path_buf();
    metadata_path.set_extension("onnx.json");
    if metadata_path.exists() {
        Some(metadata_path)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::prelude::*;

    #[test]
    fn discovers_voices_in_directory() {
        let temp = assert_fs::TempDir::new().unwrap();
        let model = temp.child("voice.onnx");
        model.touch().unwrap();
        VoiceLibrary::new(temp.path().to_path_buf())
            .refresh()
            .unwrap();
    }

    #[test]
    fn build_voice_info_returns_label() {
        let temp = assert_fs::TempDir::new().unwrap();
        let model = temp.child("demo.onnx");
        model.touch().unwrap();
        let meta = temp.child("demo.onnx.json");
        meta.write_str(r#"{"language":{"name_native":"Español"},"audio":{"quality":"high"}}"#)
            .unwrap();
        let info = build_voice_info(model.path()).unwrap();
        assert!(info.label.contains("Español"));
        assert_eq!(info.quality.as_deref(), Some("high"));
    }
}
