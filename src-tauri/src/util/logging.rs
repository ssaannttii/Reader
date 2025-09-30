use std::path::PathBuf;

use flexi_logger::{Age, Cleanup, Criterion, Duplicate, FileSpec, Logger, Naming};
use once_cell::sync::OnceCell;

static LOGGER: OnceCell<()> = OnceCell::new();

pub fn init() -> anyhow::Result<()> {
    LOGGER.get_or_try_init(|| {
        let log_dir = log_dir();
        std::fs::create_dir_all(&log_dir)?;
        Logger::try_with_str("info")?
            .duplicate_to_stdout(Duplicate::Info)
            .log_to_file(FileSpec::default().directory(&log_dir).basename("reader"))
            .rotate(
                Criterion::AgeOrSize(Age::Day, 10_000_000),
                Naming::Numbers,
                Cleanup::KeepLogFiles(7),
            )
            .start()?;
        Ok(())
    })?;
    Ok(())
}

fn log_dir() -> PathBuf {
    PathBuf::from("logs")
}

pub struct LogCleanupPlugin;

impl tauri::plugin::Plugin for LogCleanupPlugin {
    type Builder = Self;

    fn name(&self) -> &'static str {
        "reader-log-cleanup"
    }

    fn build(self, _app: &tauri::AppHandle) -> tauri::plugin::Result<Self> {
        Ok(self)
    }
}
