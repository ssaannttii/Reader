use std::{
    fs,
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
    time::Instant,
};

use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use shlex::Shlex;
use thiserror::Error;

use crate::dict::PhoneticDictionary;

const ERROR_VOICE_NOT_FOUND: &str = "VOICE_NOT_FOUND";
const ERROR_PROCESS_FAILED: &str = "PROCESS_FAILED";
const ERROR_IO: &str = "IO_ERROR";
const ERROR_INTERNAL: &str = "INTERNAL_ERROR";

#[derive(Debug, Error)]
pub enum CommandFailure {
    #[error("voice model not found at {0}")]
    VoiceNotFound(PathBuf),
    #[error("failed to spawn Piper process: {0}")]
    SpawnFailure(#[from] std::io::Error),
    #[error("Piper exited with status {status}: {stderr}")]
    PiperFailure { status: i32, stderr: String },
    #[error("{0}")]
    Other(String),
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct CommandError {
    pub code: String,
    pub message: String,
    pub details: Option<String>,
}

impl CommandError {
    pub fn new(code: &str, message: impl Into<String>, details: Option<String>) -> Self {
        Self {
            code: code.to_string(),
            message: message.into(),
            details,
        }
    }
}

impl From<CommandFailure> for CommandError {
    fn from(value: CommandFailure) -> Self {
        match value {
            CommandFailure::VoiceNotFound(path) => CommandError::new(
                ERROR_VOICE_NOT_FOUND,
                format!("Voice model not found: {}", path.display()),
                None,
            ),
            CommandFailure::SpawnFailure(err) => {
                CommandError::new(ERROR_IO, "Failed to launch Piper", Some(err.to_string()))
            }
            CommandFailure::PiperFailure { status, stderr } => CommandError::new(
                ERROR_PROCESS_FAILED,
                format!("Piper exited with status {status}"),
                Some(stderr),
            ),
            CommandFailure::Other(message) => CommandError::new(ERROR_INTERNAL, message, None),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SpeakRequest {
    pub text: String,
    pub model_path: PathBuf,
    pub output_path: PathBuf,
    #[serde(default)]
    pub speaker: Option<String>,
    #[serde(default)]
    pub length_scale: Option<f32>,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct SpeakResponse {
    pub output_path: PathBuf,
    pub duration_ms: u128,
    pub stderr: Option<String>,
}

trait PiperInvoker {
    fn invoke(&self, request: &SpeakRequest) -> Result<SpeakResponse, CommandFailure>;
}

struct DefaultPiperInvoker;

impl DefaultPiperInvoker {
    fn build_command(request: &SpeakRequest) -> Result<Command, CommandFailure> {
        if !request.model_path.exists() {
            return Err(CommandFailure::VoiceNotFound(request.model_path.clone()));
        }

        if let Some(parent) = request
            .output_path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent).map_err(|err| {
                CommandFailure::Other(format!(
                    "Unable to create output directory {}: {err}",
                    parent.display()
                ))
            })?;
        }

        if let Some(raw_command) = std::env::var_os("READER_PIPER_COMMAND") {
            let raw_command = raw_command.to_string_lossy().into_owned();
            let mut shlex = Shlex::new(&raw_command);
            let mut parts: Vec<String> = shlex.collect();
            if parts.is_empty() {
                return Err(CommandFailure::Other(
                    "READER_PIPER_COMMAND is empty".to_string(),
                ));
            }
            let program = parts.remove(0);
            let mut command = Command::new(program);
            for part in parts {
                command.arg(part);
            }
            Ok(command)
        } else if cfg!(target_os = "windows") {
            Ok(Command::new("runtime/piper/piper.exe"))
        } else {
            let mut command = Command::new("python");
            command.args(["-m", "piper"]);
            Ok(command)
        }
    }

    fn command_arguments(command: &mut Command, request: &SpeakRequest) {
        command.arg("--model");
        command.arg(&request.model_path);
        command.arg("--output_file");
        command.arg(&request.output_path);
        if let Some(speaker) = &request.speaker {
            command.arg("--speaker");
            command.arg(speaker);
        }
        if let Some(scale) = request.length_scale {
            command.arg("--length_scale");
            command.arg(scale.to_string());
        }
    }
}

impl PiperInvoker for DefaultPiperInvoker {
    fn invoke(&self, request: &SpeakRequest) -> Result<SpeakResponse, CommandFailure> {
        let start = Instant::now();
        let mut command = Self::build_command(request)?;
        Self::command_arguments(&mut command, request);
        let mut child = command
            .stdin(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(CommandFailure::SpawnFailure)?;
        {
            let stdin = child
                .stdin
                .as_mut()
                .ok_or_else(|| CommandFailure::Other("Failed to access Piper stdin".into()))?;
            stdin
                .write_all(request.text.as_bytes())
                .map_err(|err| CommandFailure::Other(err.to_string()))?;
        }
        let output = child
            .wait_with_output()
            .map_err(|err| CommandFailure::Other(err.to_string()))?;
        let duration_ms = start.elapsed().as_millis();
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

        if !output.status.success() {
            let code = output.status.code().unwrap_or_default();
            error!("Piper command exited with status {code}: {}", stderr);
            return Err(CommandFailure::PiperFailure {
                status: code,
                stderr,
            });
        }

        if !request.output_path.exists() {
            warn!(
                "Piper succeeded but the expected output {:?} was not created",
                request.output_path
            );
        }

        Ok(SpeakResponse {
            output_path: request.output_path.clone(),
            duration_ms,
            stderr: if stderr.is_empty() {
                None
            } else {
                Some(stderr)
            },
        })
    }
}

pub fn speak(mut request: SpeakRequest) -> Result<SpeakResponse, CommandError> {
    info!(
        "Invoking Piper for model {} writing to {}",
        request.model_path.display(),
        request.output_path.display()
    );

    match PhoneticDictionary::load_from_env() {
        Ok(dictionary) => {
            request.text = dictionary.transform_text(&request.text);
        }
        Err(err) => {
            warn!("Failed to load phonetic dictionary: {err}");
        }
    }

    DefaultPiperInvoker.invoke(&request).map_err(|err| {
        error!("Speak command failed: {err}");
        CommandError::from(err)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::TempDir;

    struct EnvVarGuard {
        key: &'static str,
    }

    impl EnvVarGuard {
        fn set(key: &'static str, value: String) -> Self {
            std::env::set_var(key, value);
            Self { key }
        }
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            std::env::remove_var(self.key);
        }
    }

    fn write_mock_piper_script(temp: &TempDir, content: &str) -> EnvVarGuard {
        let script_path = temp.path().join("mock_piper.py");
        fs::write(&script_path, content).unwrap();
        EnvVarGuard::set(
            "READER_PIPER_COMMAND",
            format!("python3 {}", script_path.display()),
        )
    }

    fn make_request(temp: &TempDir, model_exists: bool) -> SpeakRequest {
        let model_path = temp.path().join("voice.onnx");
        if model_exists {
            fs::write(&model_path, b"voice").unwrap();
        }
        let output_path = temp.path().join("output.wav");
        SpeakRequest {
            text: "hola".into(),
            model_path,
            output_path,
            speaker: None,
            length_scale: None,
        }
    }

    #[test]
    fn speak_success_creates_audio() {
        let temp = TempDir::new().unwrap();
        let _guard = write_mock_piper_script(
            &temp,
            r#"import argparse
import sys
parser = argparse.ArgumentParser()
parser.add_argument('--model')
parser.add_argument('--output_file')
args = parser.parse_args()
text = sys.stdin.read()
with open(args.output_file, 'w', encoding='utf-8') as f:
    f.write('WAV:' + text)
"#,
        );
        let request = make_request(&temp, true);
        let response = speak(request).unwrap();
        assert!(response.duration_ms > 0);
        let output = fs::read_to_string(temp.path().join("output.wav")).unwrap();
        assert_eq!(output, "WAV:hola");
    }

    #[test]
    fn speak_missing_voice_returns_error() {
        let temp = TempDir::new().unwrap();
        let _guard = write_mock_piper_script(&temp, "import sys; sys.exit(0)");
        let request = make_request(&temp, false);
        let error = speak(request).unwrap_err();
        assert_eq!(error.code, ERROR_VOICE_NOT_FOUND);
    }

    #[test]
    fn speak_process_failure_returns_error() {
        let temp = TempDir::new().unwrap();
        let _guard = write_mock_piper_script(
            &temp,
            r#"import sys
sys.stderr.write('boom')
sys.exit(2)
"#,
        );
        let request = make_request(&temp, true);
        let error = speak(request).unwrap_err();
        assert_eq!(error.code, ERROR_PROCESS_FAILED);
        assert_eq!(error.details.unwrap(), "boom");
    }
}
