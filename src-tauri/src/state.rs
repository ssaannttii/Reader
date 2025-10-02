use std::{fs, path::PathBuf};

use anyhow::{Context, Result};

use crate::{audio::AudioManager, cmds::voices::VoiceLibrary, dict::Dictionary};

pub struct AppState {
    pub audio: AudioManager,
    pub dictionary: Dictionary,
    pub voices: VoiceLibrary,
    output_dir: PathBuf,
}

impl AppState {
    pub fn initialise() -> Result<Self> {
        let audio = AudioManager::new().context("failed to initialise audio manager")?;

        let dictionary_path = std::env::var("READER_DICTIONARY")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("runtime/dictionary.json"));
        let dictionary = Dictionary::load_or_default(dictionary_path)
            .context("failed to load pronunciation dictionary")?;

        let voices_dir = std::env::var("READER_VOICES_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("assets/voices"));
        let voices = VoiceLibrary::new(voices_dir);

        let output_dir = std::env::var("READER_OUTPUT_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("runtime/output"));
        fs::create_dir_all(&output_dir).with_context(|| {
            format!("unable to create output directory {}", output_dir.display())
        })?;

        Ok(Self {
            audio,
            dictionary,
            voices,
            output_dir,
        })
    }

    pub fn output_path(&self, filename: &str) -> PathBuf {
        self.output_dir.join(filename)
    }

    pub fn output_dir(&self) -> &std::path::Path {
        &self.output_dir
    }
}
