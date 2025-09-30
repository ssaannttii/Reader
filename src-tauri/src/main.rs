//! Punto de entrada del backend Tauri.

#![cfg_attr(all(not(test), not(debug_assertions)), windows_subsystem = "windows")]

mod audio;
mod cmds;
mod dict;
mod ssml;
mod util;

use crate::util::logging;
use tauri::Manager;

fn main() {
    logging::init().expect("failed to initialize logging");

    tauri::Builder::default()
        .plugin(logging::LogCleanupPlugin)
        .invoke_handler(tauri::generate_handler![
            cmds::speak::speak,
            cmds::import_pdf::import_pdf,
            cmds::import_epub::import_epub,
            cmds::encode::encode_audio,
            dict::lexicon::list_entries,
            dict::lexicon::upsert_entry,
            dict::lexicon::delete_entry,
            dict::lexicon::apply_preview
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
