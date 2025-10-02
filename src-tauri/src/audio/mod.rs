use std::{
    fs,
    io::BufReader,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use parking_lot::Mutex;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use serde::Serialize;
use thiserror::Error;

/// Errors that can occur when controlling audio playback.
#[derive(Debug, Error)]
pub enum AudioError {
    #[error("no audio device available: {0}")]
    Device(#[from] rodio::StreamError),
    #[error("failed to open audio file {0}: {1}")]
    Io(PathBuf, #[source] std::io::Error),
    #[error("failed to decode audio stream {0}: {1}")]
    Decode(PathBuf, #[source] rodio::decoder::DecoderError),
    #[error("no audio is currently loaded")]
    NoAudioLoaded,
    #[error("no active playback sink")]
    NoSink,
}

/// Result returned by playback commands.
#[derive(Debug, Serialize)]
pub struct PlaybackStatus {
    pub is_playing: bool,
    pub current_path: Option<PathBuf>,
}

/// Centralised audio playback manager used by the Tauri commands.
pub struct AudioManager {
    stream: Mutex<Option<OutputStream>>,
    handle: OutputStreamHandle,
    sink: Mutex<Option<Sink>>,
    last_audio: Mutex<Option<PathBuf>>,
    playback_id: AtomicU64,
}

impl AudioManager {
    /// Initialise the audio backend, connecting to the default output device.
    pub fn new() -> Result<Self, AudioError> {
        let (stream, handle) = OutputStream::try_default()?;
        Ok(Self {
            stream: Mutex::new(Some(stream)),
            handle,
            sink: Mutex::new(None),
            last_audio: Mutex::new(None),
            playback_id: AtomicU64::new(0),
        })
    }

    /// Play a freshly generated audio file. Returns a playback identifier that
    /// can be used to correlate completion events.
    pub fn play_file(&self, path: &Path, volume: f32) -> Result<u64, AudioError> {
        if !path.exists() {
            return Err(AudioError::Io(
                path.to_path_buf(),
                std::io::Error::from(std::io::ErrorKind::NotFound),
            ));
        }

        let file = fs::File::open(path).map_err(|err| AudioError::Io(path.to_path_buf(), err))?;
        let source = Decoder::new(BufReader::new(file))
            .map_err(|err| AudioError::Decode(path.to_path_buf(), err))?;

        let sink = Sink::try_new(&self.handle)?;
        sink.set_volume(volume.clamp(0.0, 1.0));
        sink.append(source);
        sink.play();

        if let Some(existing) = self.sink.lock().replace(sink.clone()) {
            existing.stop();
        }

        *self.last_audio.lock() = Some(path.to_path_buf());

        let id = self.playback_id.fetch_add(1, Ordering::SeqCst) + 1;
        Ok(id)
    }

    /// Pause the active playback sink, keeping the buffer in memory.
    pub fn pause(&self) -> Result<(), AudioError> {
        let guard = self.sink.lock();
        if let Some(sink) = guard.as_ref() {
            sink.pause();
            Ok(())
        } else {
            Err(AudioError::NoSink)
        }
    }

    /// Resume playback from the paused sink.
    pub fn resume(&self) -> Result<(), AudioError> {
        let guard = self.sink.lock();
        if let Some(sink) = guard.as_ref() {
            sink.play();
            Ok(())
        } else {
            Err(AudioError::NoSink)
        }
    }

    /// Stop the current sink and release the audio buffer.
    pub fn stop(&self) -> Result<(), AudioError> {
        let mut guard = self.sink.lock();
        if let Some(mut sink) = guard.take() {
            sink.stop();
            Ok(())
        } else {
            Err(AudioError::NoSink)
        }
    }

    /// Retrieve a clone of the active sink if available.
    pub fn current_sink(&self) -> Option<Sink> {
        self.sink.lock().as_ref().map(Sink::clone)
    }

    /// Return a snapshot of the playback status.
    pub fn status(&self) -> PlaybackStatus {
        let guard = self.sink.lock();
        let last = self.last_audio.lock();
        let is_playing = guard
            .as_ref()
            .map(|sink| !sink.is_paused() && !sink.empty())
            .unwrap_or(false);
        PlaybackStatus {
            is_playing,
            current_path: last.clone(),
        }
    }

    /// Obtain the identifier associated with the most recent playback.
    pub fn current_playback_id(&self) -> u64 {
        self.playback_id.load(Ordering::SeqCst)
    }

    /// Copy the last generated audio file to the destination path.
    pub fn export_last_audio(&self, destination: &Path) -> Result<PathBuf, AudioError> {
        let Some(source) = self.last_audio.lock().clone() else {
            return Err(AudioError::NoAudioLoaded);
        };
        let parent = destination
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .ok_or_else(|| {
                AudioError::Io(
                    destination.to_path_buf(),
                    std::io::Error::from(std::io::ErrorKind::NotFound),
                )
            })?;
        fs::create_dir_all(parent).map_err(|err| AudioError::Io(parent.to_path_buf(), err))?;
        fs::copy(&source, destination)
            .map_err(|err| AudioError::Io(destination.to_path_buf(), err))?;
        Ok(destination.to_path_buf())
    }

    /// Return the path to the last generated audio if available.
    pub fn last_audio_path(&self) -> Option<PathBuf> {
        self.last_audio.lock().clone()
    }
}

unsafe impl Send for AudioManager {}
unsafe impl Sync for AudioManager {}
