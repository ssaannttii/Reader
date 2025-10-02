//! Herramientas de reproducción de audio para el backend Tauri.
//!
//! Este módulo expone [`AudioPlayer`], un contenedor thread-safe que gestiona un
//! `rodio::OutputStream` y reproduce ficheros locales mediante los métodos
//! [`AudioPlayer::play`], [`AudioPlayer::stop`] e [`AudioPlayer::is_playing`].
//! El reproductor es inyectable, por lo que sus pruebas pueden utilizar un
//! backend simulado que evite depender de un dispositivo real.

pub mod player;

pub use player::{AudioEngine, AudioPlayer, AudioPlayerError, ManagedSink, RodioEngine};
