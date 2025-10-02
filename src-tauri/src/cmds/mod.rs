//! Command module stubs for the Tauri backend.
//!
//! Each subcommand will bridge the UI with platform capabilities such as
//! Piper synthesis and document importers. Replace the placeholder
//! functions with real implementations as the MVP evolves.

/// Audio related commands that wrap [`crate::audio::AudioPlayer`].
pub mod audio;

/// Placeholder module for speech synthesis commands.
pub mod speak {
    //! Spawn Piper subprocesses and manage streaming playback here.
}

/// Placeholder module for document import commands.
pub mod import {
    //! Wire Python helpers for EPUB/PDF extraction here.
}
