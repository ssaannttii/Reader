use std::path::PathBuf;

use tauri::State;

use crate::audio::{AudioPlayer, AudioPlayerError};

fn map_error(err: AudioPlayerError) -> String {
    err.to_string()
}

/// Inicia la reproducción del fichero indicado por `path`.
#[tauri::command]
pub fn play_audio(state: State<'_, AudioPlayer>, path: String) -> Result<(), String> {
    let path = PathBuf::from(path);
    state
        .play(path)
        .map_err(map_error)
}

/// Detiene cualquier reproducción en curso.
#[tauri::command]
pub fn stop_audio(state: State<'_, AudioPlayer>) -> Result<(), String> {
    state.stop().map_err(map_error)
}

/// Devuelve `true` si el backend sigue reproduciendo audio.
#[tauri::command]
pub fn status_audio(state: State<'_, AudioPlayer>) -> Result<bool, String> {
    Ok(state.is_playing())
}
