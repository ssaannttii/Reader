use std::path::{Path, PathBuf};

use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Serialize)]
#[serde(tag = "kind", content = "value")]
pub enum PiperCommand {
    Executable(PathBuf),
    PythonModule,
}

#[derive(Debug, Error)]
pub enum PiperPathError {
    #[error("no se encontró piper.exe en runtime/piper")]
    ExecutableMissing,
    #[error("python no está disponible en el PATH para ejecutar 'python -m piper'")]
    PythonUnavailable,
}

pub fn resolve(runtime_dir: &Path) -> Result<PiperCommand, PiperPathError> {
    let candidate =
        runtime_dir
            .join("piper")
            .join(if cfg!(windows) { "piper.exe" } else { "piper" });
    if candidate.exists() {
        return Ok(PiperCommand::Executable(candidate));
    }

    if which::which("python").is_ok() {
        return Ok(PiperCommand::PythonModule);
    }

    Err(PiperPathError::PythonUnavailable)
}

pub fn command_to_args(cmd: &PiperCommand) -> (std::ffi::OsString, Vec<std::ffi::OsString>) {
    match cmd {
        PiperCommand::Executable(path) => (path.as_os_str().into(), vec![]),
        PiperCommand::PythonModule => (
            std::ffi::OsString::from("python"),
            vec!["-m".into(), "piper".into()],
        ),
    }
}
