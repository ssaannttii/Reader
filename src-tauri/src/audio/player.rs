use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::{fs::File, io::BufReader};

use rodio::{decoder::Decoder, OutputStream, OutputStreamHandle, Sink};

/// Errors returned by [`AudioPlayer`].
#[derive(Debug)]
pub enum AudioPlayerError {
    /// The audio device could not be opened or is unavailable.
    Device(rodio::StreamError),
    /// The requested audio file does not exist.
    NotFound(PathBuf),
    /// A generic I/O error occurred while opening the file.
    Io {
        /// Path that triggered the error.
        path: PathBuf,
        /// Source [`std::io::Error`].
        source: std::io::Error,
    },
    /// The file could not be decoded by the backend.
    Decode {
        /// Path that triggered the error.
        path: PathBuf,
        /// Source [`rodio::decoder::DecoderError`].
        source: rodio::decoder::DecoderError,
    },
    /// Backend specific error.
    Backend(String),
}

impl std::fmt::Display for AudioPlayerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AudioPlayerError::Device(err) => {
                write!(f, "no se pudo inicializar el dispositivo de audio: {err}")
            }
            AudioPlayerError::NotFound(path) => {
                write!(f, "el fichero de audio '{path}' no existe")
            }
            AudioPlayerError::Io { path, source } => {
                write!(f, "error de E/S al abrir '{path}': {source}")
            }
            AudioPlayerError::Decode { path, source } => {
                write!(f, "no se pudo decodificar '{path}': {source}")
            }
            AudioPlayerError::Backend(msg) => f.write_str(msg),
        }
    }
}

impl std::error::Error for AudioPlayerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AudioPlayerError::Device(err) => Some(err),
            AudioPlayerError::Io { source, .. } => Some(source),
            AudioPlayerError::Decode { source, .. } => Some(source),
            AudioPlayerError::Backend(_) | AudioPlayerError::NotFound(_) => None,
        }
    }
}

/// Trait describing the required behaviour of a sink managed by [`AudioPlayer`].
pub trait ManagedSink: Send + Sync + 'static {
    /// Stop playback immediately.
    fn stop(&self);
    /// Returns whether the sink is still playing audio.
    fn is_playing(&self) -> bool;
}

/// Trait representing the audio backend used by [`AudioPlayer`].
pub trait AudioEngine: Send + Sync + 'static {
    /// Concrete sink type produced by the backend.
    type ActiveSink: ManagedSink;

    /// Creates a new backend instance.
    fn init() -> Result<Self, AudioPlayerError>
    where
        Self: Sized;

    /// Starts playback of the file located at `path` and returns the sink used.
    fn start_stream(&self, path: &Path) -> Result<Self::ActiveSink, AudioPlayerError>;
}

/// Default backend powered by `rodio`.
pub struct RodioEngine {
    _stream: OutputStream,
    handle: OutputStreamHandle,
}

impl AudioEngine for RodioEngine {
    type ActiveSink = RodioSink;

    fn init() -> Result<Self, AudioPlayerError> {
        let (stream, handle) = OutputStream::try_default()
            .map_err(AudioPlayerError::Device)?;
        Ok(Self { _stream: stream, handle })
    }

    fn start_stream(&self, path: &Path) -> Result<Self::ActiveSink, AudioPlayerError> {
        let path_buf = PathBuf::from(path);
        let file = File::open(&path_buf).map_err(|err| match err.kind() {
            std::io::ErrorKind::NotFound => AudioPlayerError::NotFound(path_buf.clone()),
            _ => AudioPlayerError::Io {
                path: path_buf.clone(),
                source: err,
            },
        })?;

        let decoder = Decoder::new(BufReader::new(file)).map_err(|source| AudioPlayerError::Decode {
            path: path_buf.clone(),
            source,
        })?;

        let sink = Sink::try_new(&self.handle).map_err(AudioPlayerError::Device)?;
        sink.append(decoder);
        sink.play();

        Ok(RodioSink { sink })
    }
}

struct RodioSink {
    sink: Sink,
}

impl ManagedSink for RodioSink {
    fn stop(&self) {
        self.sink.stop();
    }

    fn is_playing(&self) -> bool {
        !self.sink.empty()
    }
}

/// Audio player abstraction that wraps an [`AudioEngine`].
pub struct AudioPlayer<B: AudioEngine = RodioEngine> {
    backend: Arc<B>,
    current: Mutex<Option<B::ActiveSink>>,
}

impl<B: AudioEngine> AudioPlayer<B> {
    /// Creates a new player using the backend's default configuration.
    pub fn new() -> Result<Self, AudioPlayerError> {
        let backend = B::init()?;
        Ok(Self::with_backend(backend))
    }

    /// Creates a player from an already initialised backend. Mainly used for tests.
    pub fn with_backend(backend: B) -> Self {
        Self {
            backend: Arc::new(backend),
            current: Mutex::new(None),
        }
    }

    /// Starts playback of the WAV/MP3 file at `path`, replacing any running sink.
    pub fn play<P: AsRef<Path>>(&self, path: P) -> Result<(), AudioPlayerError> {
        let sink = self.backend.start_stream(path.as_ref())?;
        let mut guard = self
            .current
            .lock()
            .expect("audio sink mutex poisoned during play");

        if let Some(previous) = guard.replace(sink) {
            previous.stop();
        }

        Ok(())
    }

    /// Stops the current playback, if any.
    pub fn stop(&self) -> Result<(), AudioPlayerError> {
        let mut guard = self
            .current
            .lock()
            .expect("audio sink mutex poisoned during stop");
        if let Some(sink) = guard.take() {
            sink.stop();
        }
        Ok(())
    }

    /// Returns whether the player is actively reproducing audio.
    pub fn is_playing(&self) -> bool {
        let mut guard = self
            .current
            .lock()
            .expect("audio sink mutex poisoned during status");
        if let Some(sink) = guard.as_ref() {
            if sink.is_playing() {
                return true;
            }
        }
        guard.take();
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Mutex as StdMutex;
    use std::thread;

    #[derive(Clone)]
    struct MockBackend {
        playing: Arc<AtomicBool>,
        log: Arc<StdMutex<VecDeque<&'static str>>>,
    }

    impl MockBackend {
        fn new() -> Self {
            Self {
                playing: Arc::new(AtomicBool::new(false)),
                log: Arc::new(StdMutex::new(VecDeque::new())),
            }
        }
    }

    impl AudioEngine for MockBackend {
        type ActiveSink = MockSink;

        fn init() -> Result<Self, AudioPlayerError> {
            Ok(Self::new())
        }

        fn start_stream(&self, _path: &Path) -> Result<Self::ActiveSink, AudioPlayerError> {
            self.log
                .lock()
                .unwrap()
                .push_back("play");
            self.playing.store(true, Ordering::SeqCst);
            Ok(MockSink {
                playing: self.playing.clone(),
                log: self.log.clone(),
            })
        }
    }

    struct MockSink {
        playing: Arc<AtomicBool>,
        log: Arc<StdMutex<VecDeque<&'static str>>>,
    }

    impl ManagedSink for MockSink {
        fn stop(&self) {
            self.log.lock().unwrap().push_back("stop");
            self.playing.store(false, Ordering::SeqCst);
        }

        fn is_playing(&self) -> bool {
            self.playing.load(Ordering::SeqCst)
        }
    }

    #[test]
    fn play_and_stop_updates_state() {
        let backend = MockBackend::new();
        let player = AudioPlayer::with_backend(backend.clone());

        assert!(!player.is_playing());
        player.play("dummy").unwrap();
        assert!(player.is_playing());
        player.stop().unwrap();
        assert!(!player.is_playing());

        let log: Vec<_> = backend.log.lock().unwrap().iter().copied().collect();
        assert_eq!(log, vec!["play", "stop"]);
    }

    #[test]
    fn concurrent_playback_requests_are_serialised() {
        let backend = MockBackend::new();
        let player = Arc::new(AudioPlayer::with_backend(backend.clone()));

        let mut handles = Vec::new();
        for _ in 0..8 {
            let player = Arc::clone(&player);
            handles.push(thread::spawn(move || {
                player.play("dummy").unwrap();
                player.is_playing();
                player.stop().unwrap();
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert!(!player.is_playing());
        let log: Vec<_> = backend.log.lock().unwrap().iter().copied().collect();
        assert!(log.len() >= 8);
    }
}
