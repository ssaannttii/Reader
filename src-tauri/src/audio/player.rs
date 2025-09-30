use std::fs::File;
use std::io::BufReader;
use std::sync::{Arc, Mutex};

use once_cell::sync::Lazy;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use serde::Serialize;
use thiserror::Error;

static PLAYER: Lazy<Player> = Lazy::new(|| Player::new());

#[derive(Debug, Error, Serialize)]
pub enum AudioError {
    #[error("no se pudo abrir el archivo de audio: {0}")]
    Io(String),
    #[error("el archivo de audio está corrupto: {0}")]
    Decode(String),
    #[error("no se encontró dispositivo de reproducción")]
    Device,
}

#[derive(Default)]
struct InnerPlayer {
    _stream: Option<OutputStream>,
    handle: Option<OutputStreamHandle>,
    sink: Option<Sink>,
}

#[derive(Default)]
pub struct Player {
    inner: Arc<Mutex<InnerPlayer>>,
}

impl Player {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(InnerPlayer::default())),
        }
    }

    pub fn global() -> &'static Player {
        &PLAYER
    }

    pub fn play(&self, path: &str) -> Result<(), AudioError> {
        let file = File::open(path).map_err(|err| AudioError::Io(err.to_string()))?;
        let reader = BufReader::new(file);
        let decoder = Decoder::new(reader).map_err(|err| AudioError::Decode(err.to_string()))?;

        let (stream, handle) = OutputStream::try_default().map_err(|_| AudioError::Device)?;
        let sink = Sink::try_new(&handle).map_err(|err| AudioError::Io(err.to_string()))?;
        sink.append(decoder);
        sink.play();

        let mut state = self.inner.lock().unwrap();
        state._stream = Some(stream);
        state.handle = Some(handle);
        state.sink = Some(sink);
        Ok(())
    }

    pub fn stop(&self) {
        if let Some(sink) = self.inner.lock().unwrap().sink.take() {
            sink.stop();
        }
    }

    pub fn is_playing(&self) -> bool {
        self.inner
            .lock()
            .unwrap()
            .sink
            .as_ref()
            .map(|sink| !sink.empty())
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn returns_error_for_missing_file() {
        let result = Player::global().play("/no/existe.wav");
        assert!(matches!(result, Err(AudioError::Io(_))));
    }

    #[test]
    fn fails_for_invalid_wav() {
        let mut tmp = NamedTempFile::new().unwrap();
        writeln!(tmp, "not a wav").unwrap();
        let result = Player::global().play(tmp.path().to_str().unwrap());
        assert!(matches!(result, Err(AudioError::Decode(_))));
    }
}
