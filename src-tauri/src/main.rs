use std::{fs, path::PathBuf};

use log::{error, info};
mod audio;
mod cmds;
mod dict;
mod ssml;
mod state;

use cmds::{
    current_audio, export_audio, handle_audio_completion, import_epub_command, import_pdf_command,
    import_text, pause_audio, play_audio, stop_audio, CommandError, SpeakCommand, SpeakResponse,
};
use state::AppState;

fn init_logging() {
    let logs_dir = PathBuf::from("logs");
    if let Err(err) = fs::create_dir_all(&logs_dir) {
        eprintln!("Failed to create log directory {logs_dir:?}: {err}");
    }

    if let Err(err) = flexi_logger::Logger::try_with_env_or_str("info").and_then(|logger| {
        logger
            .log_to_file(
                flexi_logger::FileSpec::default()
                    .directory(&logs_dir)
                    .basename("reader")
                    .suffix("log")
                    .suppress_timestamp(),
            )
            .rotate(
                flexi_logger::Criterion::Size(5_000_000),
                flexi_logger::Naming::Numbers,
                flexi_logger::Cleanup::KeepLogFiles(5),
            )
            .duplicate_to_stderr(flexi_logger::Duplicate::Info)
            .start()
    }) {
        eprintln!("Failed to initialise logger: {err}");
    }
}

fn main() {
    init_logging();
    info!("Starting Reader Tauri backend");

    let app_state = match AppState::initialise() {
        Ok(state) => state,
        Err(err) => {
            error!("Failed to initialise application state: {err:?}");
            panic!("Unable to start Reader backend: {err}");
        }
    };

    if let Err(err) = tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            speak,
            import_pdf_command,
            import_epub_command,
            import_text,
            list_voices,
            play_audio,
            pause_audio,
            stop_audio,
            current_audio,
            export_audio,
        ])
        .run(tauri::generate_context!())
    {
        error!("Tauri runtime error: {err:?}");
    }
}

#[tauri::command]
fn list_voices(state: tauri::State<AppState>) -> Vec<cmds::VoiceInfo> {
    state.voices.list()
}

#[tauri::command]
fn speak(
    app_handle: tauri::AppHandle,
    state: tauri::State<AppState>,
    request: SpeakCommand,
) -> Result<SpeakResponse, CommandError> {
    let response = cmds::execute_synthesis(&state, request)?;
    if let Some(playback_id) = response.playback_id {
        handle_audio_completion(&state.audio, &app_handle, playback_id);
    }
    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn logger_creates_directory() {
        let temp_dir = tempfile::tempdir().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();
        init_logging();
        assert!(temp_dir.path().join("logs").exists());
        std::env::set_current_dir(original_dir).unwrap();
    }
}
