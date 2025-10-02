use std::{fs, path::PathBuf};

use log::{error, info};
mod cmds;
mod ssml;

use cmds::{import_epub, import_pdf, speak};

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

    if let Err(err) = tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![speak, import_pdf, import_epub])
        .run(tauri::generate_context!())
    {
        error!("Tauri runtime error: {err:?}");
    }
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
