use std::path::PathBuf;

use tauri::State;

use crate::{audio::PlaybackStatus, state::AppState};

use super::CommandError;

#[tauri::command]
pub fn play_audio(state: State<AppState>) -> Result<PlaybackStatus, CommandError> {
    state.audio.resume()?;
    Ok(state.audio.status())
}

#[tauri::command]
pub fn pause_audio(state: State<AppState>) -> Result<PlaybackStatus, CommandError> {
    state.audio.pause()?;
    Ok(state.audio.status())
}

#[tauri::command]
pub fn stop_audio(state: State<AppState>) -> Result<PlaybackStatus, CommandError> {
    state.audio.stop()?;
    Ok(state.audio.status())
}

#[tauri::command]
pub fn current_audio(state: State<AppState>) -> PlaybackStatus {
    state.audio.status()
}

#[tauri::command]
pub fn export_audio(state: State<AppState>, destination: PathBuf) -> Result<PathBuf, CommandError> {
    state
        .audio
        .export_last_audio(&destination)
        .map_err(Into::into)
}
