use std::collections::BTreeMap;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use log::debug;
use serde::{Deserialize, Serialize};

use super::speak::CommandError;

const ERROR_DISCOVERY_IO: &str = "VOICES_IO_ERROR";
const ERROR_METADATA_INVALID: &str = "VOICES_METADATA_INVALID";

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct VoicePreference {
    pub id: String,
    pub label: String,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
struct VoiceMetadata {
    reader: Option<ReaderMetadata>,
    language: Option<LanguageMetadata>,
    dataset: Option<String>,
    audio: Option<AudioMetadata>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
struct ReaderMetadata {
    label: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
struct LanguageMetadata {
    code: Option<String>,
    name_native: Option<String>,
    name_english: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
struct AudioMetadata {
    quality: Option<String>,
}

fn voices_root() -> PathBuf {
    PathBuf::from("assets/voices")
}

fn discover_voices_from(base: &Path) -> Result<Vec<VoicePreference>, CommandError> {
    let mut voices = BTreeMap::new();
    visit_directory(base, &mut voices)?;

    let mut preferences: Vec<VoicePreference> = voices
        .into_iter()
        .map(|(id, label)| VoicePreference { id, label })
        .collect();
    preferences.sort_by(|a, b| a.label.cmp(&b.label).then_with(|| a.id.cmp(&b.id)));
    Ok(preferences)
}

fn visit_directory(dir: &Path, voices: &mut BTreeMap<String, String>) -> Result<(), CommandError> {
    let read_dir = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(err) if err.kind() == ErrorKind::NotFound => return Ok(()),
        Err(err) => {
            return Err(CommandError::new(
                ERROR_DISCOVERY_IO,
                format!("Failed to read voices directory at {}", dir.display()),
                Some(err.to_string()),
            ))
        }
    };

    for entry in read_dir {
        let entry = entry.map_err(|err| {
            CommandError::new(
                ERROR_DISCOVERY_IO,
                format!("Failed to inspect entry inside {}", dir.display()),
                Some(err.to_string()),
            )
        })?;

        let path = entry.path();
        if path.is_dir() {
            visit_directory(&path, voices)?;
            continue;
        }

        if is_metadata_file(&path) {
            register_voice_from_metadata(&path, voices)?;
            continue;
        }

        if is_voice_model(&path) {
            register_voice_from_model(&path, voices);
        }
    }

    Ok(())
}

fn is_voice_model(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|ext| ext.to_str()),
        Some(ext) if ext.eq_ignore_ascii_case("onnx")
    )
}

fn is_metadata_file(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.ends_with(".onnx.json"))
        .unwrap_or(false)
}

fn register_voice_from_model(path: &Path, voices: &mut BTreeMap<String, String>) {
    if let Some(basename) = model_basename(path) {
        let id = voice_id_from_basename(&basename);
        voices
            .entry(id)
            .or_insert_with(|| default_label_from_basename(&basename));
    }
}

fn register_voice_from_metadata(
    path: &Path,
    voices: &mut BTreeMap<String, String>,
) -> Result<(), CommandError> {
    let Some(basename) = metadata_basename(path) else {
        debug!(
            "Skipping metadata file {} because the name is not recognised",
            path.display()
        );
        return Ok(());
    };

    let metadata = read_metadata(path)?;
    let id = voice_id_from_basename(&basename);
    let label = build_label(&basename, &metadata);
    voices.insert(id, label);
    Ok(())
}

fn model_basename(path: &Path) -> Option<String> {
    let file_name = path.file_name()?.to_str()?;
    file_name
        .strip_suffix(".onnx")
        .map(|value| value.to_string())
}

fn metadata_basename(path: &Path) -> Option<String> {
    let file_name = path.file_name()?.to_str()?;
    file_name
        .strip_suffix(".onnx.json")
        .map(|value| value.to_string())
}

fn voice_id_from_basename(basename: &str) -> String {
    basename.replace('_', "-")
}

fn read_metadata(path: &Path) -> Result<VoiceMetadata, CommandError> {
    let contents = fs::read_to_string(path).map_err(|err| {
        CommandError::new(
            ERROR_DISCOVERY_IO,
            format!("Failed to read metadata at {}", path.display()),
            Some(err.to_string()),
        )
    })?;

    serde_json::from_str(&contents).map_err(|err| {
        CommandError::new(
            ERROR_METADATA_INVALID,
            format!("Invalid metadata for voice at {}", path.display()),
            Some(err.to_string()),
        )
    })
}

fn build_label(basename: &str, metadata: &VoiceMetadata) -> String {
    if let Some(label) = metadata
        .reader
        .as_ref()
        .and_then(|reader| reader.label.as_deref())
        .and_then(|label| non_empty(label))
    {
        return label;
    }

    let mut label_parts = Vec::new();

    if let Some(language) = metadata.language.as_ref().and_then(language_identifier) {
        label_parts.push(language);
    }

    if let Some(dataset) = metadata
        .dataset
        .as_deref()
        .and_then(non_empty)
        .map(title_case)
    {
        label_parts.push(dataset);
    }

    let mut label = if label_parts.is_empty() {
        default_label_from_basename(basename)
    } else {
        label_parts.join(" - ")
    };

    if let Some(quality) = metadata
        .audio
        .as_ref()
        .and_then(|audio| audio.quality.as_deref())
        .and_then(non_empty)
        .map(title_case)
    {
        label = format!("{label} ({quality})");
    }

    label
}

fn language_identifier(metadata: &LanguageMetadata) -> Option<String> {
    metadata
        .code
        .as_deref()
        .and_then(non_empty)
        .map(|code| code.replace('_', "-"))
        .or_else(|| {
            metadata
                .name_native
                .as_deref()
                .and_then(non_empty)
                .map(|name| name.to_string())
        })
        .or_else(|| {
            metadata
                .name_english
                .as_deref()
                .and_then(non_empty)
                .map(|name| name.to_string())
        })
}

fn default_label_from_basename(basename: &str) -> String {
    title_case(&basename.replace('_', " "))
}

fn title_case(value: &str) -> String {
    value
        .split_whitespace()
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => {
                    let mut cased = first.to_uppercase().collect::<String>();
                    cased.push_str(&chars.as_str().to_lowercase());
                    cased
                }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn non_empty(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

#[tauri::command]
pub fn list_voices() -> Result<Vec<VoicePreference>, CommandError> {
    discover_voices_from(&voices_root())
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::TempDir;

    fn write_file(path: &Path, contents: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, contents).unwrap();
    }

    #[test]
    fn discovers_voices_from_metadata() {
        let temp = TempDir::new().unwrap();
        let base = temp.path();
        let metadata_path = base.join("es_ES/es_ES-carlfm-x_low.onnx.json");
        write_file(
            &metadata_path,
            r#"{
    "reader": { "label": "CarlFM" },
    "language": { "code": "es_ES" },
    "dataset": "carlfm",
    "audio": { "quality": "x_low" }
}"#,
        );
        write_file(&base.join("es_ES/es_ES-carlfm-x_low.onnx"), "dummy-model");

        let voices = discover_voices_from(base).unwrap();
        assert_eq!(voices.len(), 1);
        assert_eq!(voices[0].id, "es-ES-carlfm-x-low");
        assert_eq!(voices[0].label, "CarlFM");
    }

    #[test]
    fn returns_empty_when_directory_missing() {
        let temp = TempDir::new().unwrap();
        let base = temp.path().join("missing");
        let voices = discover_voices_from(&base).unwrap();
        assert!(voices.is_empty());
    }

    #[test]
    fn invalid_metadata_returns_error() {
        let temp = TempDir::new().unwrap();
        let base = temp.path();
        write_file(&base.join("broken/voice.onnx.json"), "{ not valid");

        let error = discover_voices_from(base).unwrap_err();
        assert_eq!(error.code, ERROR_METADATA_INVALID);
        assert!(error.message.contains("Invalid metadata"));
    }

    #[test]
    fn discovers_model_without_metadata() {
        let temp = TempDir::new().unwrap();
        let base = temp.path();
        write_file(&base.join("fr_FR/fr_sample.onnx"), "model");

        let voices = discover_voices_from(base).unwrap();
        assert_eq!(voices.len(), 1);
        assert_eq!(voices[0].id, "fr-FR-fr-sample");
        assert_eq!(voices[0].label, "Fr Fr Fr Sample");
    }
}
