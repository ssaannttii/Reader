use std::{
    collections::HashSet,
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
};

use log::{debug, warn};
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
    language: Option<LanguageMetadata>,
    dataset: Option<String>,
    audio: Option<AudioMetadata>,
    reader: Option<ReaderMetadata>,
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

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
struct ReaderMetadata {
    label: Option<String>,
}

fn voices_root() -> PathBuf {
    PathBuf::from("assets/voices")
}

fn read_directory(path: &Path) -> Result<Option<fs::ReadDir>, CommandError> {
    match fs::read_dir(path) {
        Ok(read_dir) => Ok(Some(read_dir)),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(CommandError::new(
            ERROR_DISCOVERY_IO,
            format!("Failed to read voices directory at {}", path.display()),
            Some(err.to_string()),
        )),
    }
}

fn is_voice_metadata(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("json"))
        .unwrap_or(false)
        && path
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| name.ends_with(".onnx.json"))
            .unwrap_or(false)
}

fn is_voice_model(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("onnx"))
        .unwrap_or(false)
}

fn metadata_path_for_model(path: &Path) -> PathBuf {
    let mut os_string: OsString = path.as_os_str().to_os_string();
    os_string.push(".json");
    PathBuf::from(os_string)
}

fn voice_basename(path: &Path) -> Option<String> {
    let file_name = path.file_name()?.to_str()?;
    if let Some(name) = file_name.strip_suffix(".onnx.json") {
        return Some(name.to_string());
    }
    if let Some(name) = file_name.strip_suffix(".onnx") {
        return Some(name.to_string());
    }
    None
}

fn voice_id_from_basename(basename: &str) -> String {
    basename.replace('_', "-")
}

fn read_metadata(path: &Path) -> Result<VoiceMetadata, CommandError> {
    let content = fs::read_to_string(path).map_err(|err| {
        CommandError::new(
            ERROR_DISCOVERY_IO,
            format!("Failed to read metadata at {}", path.display()),
            Some(err.to_string()),
        )
    })?;

    serde_json::from_str(&content).map_err(|err| {
        CommandError::new(
            ERROR_METADATA_INVALID,
            format!("Invalid metadata for voice at {}", path.display()),
            Some(err.to_string()),
        )
    })
}

fn format_language_label(metadata: &VoiceMetadata) -> Option<String> {
    metadata.language.as_ref().map(|language| {
        let code = language.code.as_deref().map(|code| code.replace('_', "-"));
        let name = language
            .name_native
            .as_ref()
            .filter(|value| !value.trim().is_empty())
            .or_else(|| language.name_english.as_ref());

        match (code, name) {
            (Some(code), Some(name)) => format!("{code} - {name}"),
            (Some(code), None) => code,
            (None, Some(name)) => name.to_string(),
            (None, None) => String::new(),
        }
    })
}

fn title_case(value: &str) -> String {
    value
        .split(['-', '_', ' '])
        .filter(|part| !part.is_empty())
        .map(|part| {
            if part.chars().all(|c| c.is_ascii_uppercase()) {
                part.to_string()
            } else {
                let mut chars = part.chars();
                match chars.next() {
                    Some(first) => {
                        format!("{}{}", first.to_uppercase(), chars.as_str().to_lowercase())
                    }
                    None => String::new(),
                }
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn build_label(basename: &str, metadata: Option<&VoiceMetadata>) -> String {
    if let Some(metadata) = metadata {
        if let Some(label) = metadata
            .reader
            .as_ref()
            .and_then(|reader| reader.label.as_ref())
            .filter(|label| !label.trim().is_empty())
        {
            return label.trim().to_string();
        }

        let mut parts = Vec::new();
        if let Some(language_label) =
            format_language_label(metadata).filter(|value| !value.trim().is_empty())
        {
            parts.push(language_label);
        }

        if let Some(dataset) = metadata
            .dataset
            .as_ref()
            .filter(|value| !value.trim().is_empty())
        {
            parts.push(title_case(dataset));
        }

        if let Some(quality) = metadata
            .audio
            .as_ref()
            .and_then(|audio| audio.quality.as_ref())
            .filter(|value| !value.trim().is_empty())
        {
            parts.push(title_case(quality));
        }

        if !parts.is_empty() {
            return parts.join(" - ");
        }
    }

    title_case(&basename.replace('_', " "))
}

fn register_voice(
    voices: &mut Vec<VoicePreference>,
    seen: &mut HashSet<String>,
    basename: String,
    metadata: Option<VoiceMetadata>,
) {
    let id = voice_id_from_basename(&basename);
    if seen.contains(&id) {
        debug!("Skipping duplicate voice id {id}");
        return;
    }
    let label = build_label(&basename, metadata.as_ref());
    voices.push(VoicePreference {
        id: id.clone(),
        label,
    });
    seen.insert(id);
}

fn collect_voices_from(dir: &Path, voices: &mut Vec<VoicePreference>) -> Result<(), CommandError> {
    let mut seen = HashSet::new();
    collect_voices_recursive(dir, voices, &mut seen)
}

fn collect_voices_recursive(
    dir: &Path,
    voices: &mut Vec<VoicePreference>,
    seen: &mut HashSet<String>,
) -> Result<(), CommandError> {
    let Some(read_dir) = read_directory(dir)? else {
        return Ok(());
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
            collect_voices_recursive(&path, voices, seen)?;
            continue;
        }

        if is_voice_metadata(&path) {
            if let Some(basename) = voice_basename(&path) {
                let metadata = read_metadata(&path)?;
                register_voice(voices, seen, basename, Some(metadata));
            } else {
                warn!(
                    "Metadata file {} does not follow expected naming convention",
                    path.display()
                );
            }
            continue;
        }

        if is_voice_model(&path) {
            let metadata_path = metadata_path_for_model(&path);
            if metadata_path.exists() {
                continue;
            }
            if let Some(basename) = voice_basename(&path) {
                register_voice(voices, seen, basename, None);
            }
        }
    }

    Ok(())
}

fn discover_voices_from(base: &Path) -> Result<Vec<VoicePreference>, CommandError> {
    let mut voices = Vec::new();
    collect_voices_from(base, &mut voices)?;
    voices.sort_by(|a, b| a.label.cmp(&b.label).then_with(|| a.id.cmp(&b.id)));
    Ok(voices)
}

#[tauri::command]
pub fn list_voices() -> Result<Vec<VoicePreference>, CommandError> {
    discover_voices_from(&voices_root())
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::TempDir;

    fn write_file(path: &Path, content: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, content).unwrap();
    }

    #[test]
    fn discovers_voices_from_metadata() {
        let temp = TempDir::new().unwrap();
        let base = temp.path().join("voices");
        write_file(
            &base.join("es_ES/es_ES-carlfm-x_low.onnx.json"),
            r#"{
    "language": { "code": "es_ES", "name_native": "Español" },
    "dataset": "carlfm",
    "audio": { "quality": "x_low" }
}"#,
        );

        let voices = discover_voices_from(&base).unwrap();
        assert_eq!(voices.len(), 1);
        assert_eq!(voices[0].id, "es-ES-carlfm-x-low");
        assert!(voices[0].label.contains("es-ES"));
        assert!(voices[0].label.contains("Español") || voices[0].label.contains("CARLFM"));
    }

    #[test]
    fn handles_missing_directory() {
        let temp = TempDir::new().unwrap();
        let base = temp.path().join("missing");
        let voices = discover_voices_from(&base).unwrap();
        assert!(voices.is_empty());
    }

    #[test]
    fn invalid_metadata_reports_error() {
        let temp = TempDir::new().unwrap();
        let base = temp.path().join("voices");
        write_file(&base.join("broken/bad.onnx.json"), "{ invalid json");

        let error = discover_voices_from(&base).unwrap_err();
        assert_eq!(error.code, ERROR_METADATA_INVALID);
        assert!(error.message.contains("Invalid metadata"));
    }
}
