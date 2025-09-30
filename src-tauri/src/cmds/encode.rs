use std::path::PathBuf;
use std::process::Command;

use log::{error, info};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct EncodeArgs {
    pub source_path: String,
    pub target_path: Option<String>,
    #[serde(default = "default_format")]
    pub format: String,
}

fn default_format() -> String {
    "mp3".to_string()
}

#[derive(Debug, Serialize)]
#[serde(tag = "ok")]
pub enum EncodeResponse {
    #[serde(rename = "true")]
    Success { path: String },
    #[serde(rename = "false")]
    Error { code: String, message: String },
}

fn locate_ffmpeg() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("READER_FFMPEG_PATH") {
        let candidate = PathBuf::from(path);
        if candidate.exists() {
            return Some(candidate);
        }
    }
    which::which("ffmpeg").ok()
}

#[tauri::command]
pub async fn encode_audio(args: EncodeArgs) -> EncodeResponse {
    let source = PathBuf::from(&args.source_path);
    if !source.exists() {
        return EncodeResponse::Error {
            code: "ENCODE_SOURCE_MISSING".into(),
            message: format!("No se encontró {source:?}"),
        };
    }

    let format = args.format.to_lowercase();
    let target = args.target_path.unwrap_or_else(|| {
        let mut path = source.clone();
        path.set_extension(&format);
        path.to_string_lossy().to_string()
    });
    let target_path = PathBuf::from(&target);

    if format == "mp3" {
        match locate_ffmpeg() {
            Some(ffmpeg) => {
                info!("Codificando WAV→MP3 con ffmpeg");
                let input = source.to_string_lossy().to_string();
                let output = target_path.to_string_lossy().to_string();
                let status = Command::new(ffmpeg)
                    .args([
                        "-y", "-i", &input, "-vn", "-ar", "22050", "-ac", "1", &output,
                    ])
                    .status();
                match status {
                    Ok(status) if status.success() => EncodeResponse::Success { path: target },
                    Ok(status) => EncodeResponse::Error {
                        code: "ENCODE_FAIL".into(),
                        message: format!("ffmpeg retornó {status}"),
                    },
                    Err(err) => EncodeResponse::Error {
                        code: "ENCODE_FAIL".into(),
                        message: err.to_string(),
                    },
                }
            }
            None => {
                error!("ffmpeg no disponible, devolviendo WAV original");
                EncodeResponse::Error {
                    code: "FFMPEG_MISSING".into(),
                    message: "ffmpeg no está disponible en PATH. Exporta WAV o instala ffmpeg."
                        .into(),
                }
            }
        }
    } else {
        match std::fs::copy(&source, &target_path) {
            Ok(_) => EncodeResponse::Success { path: target },
            Err(err) => EncodeResponse::Error {
                code: "ENCODE_COPY_FAIL".into(),
                message: err.to_string(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::TempDir;

    struct EnvGuard {
        previous: Option<String>,
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            if let Some(value) = &self.previous {
                std::env::set_var("READER_FFMPEG_PATH", value);
            } else {
                std::env::remove_var("READER_FFMPEG_PATH");
            }
        }
    }

    fn scoped_ffmpeg(path: &str) -> EnvGuard {
        let previous = std::env::var("READER_FFMPEG_PATH").ok();
        std::env::set_var("READER_FFMPEG_PATH", path);
        EnvGuard { previous }
    }

    fn create_source(temp: &TempDir) -> PathBuf {
        let path = temp.path().join("input.wav");
        let mut file = File::create(&path).unwrap();
        file.write_all(b"RIFF....WAVEdata").unwrap();
        path
    }

    fn create_stub_ffmpeg(temp: &TempDir) -> PathBuf {
        let path = temp.path().join("ffmpeg");
        let mut file = File::create(&path).unwrap();
        #[cfg(not(windows))]
        {
            writeln!(file, "#!/bin/sh").unwrap();
            writeln!(file, "IN=\"\"; OUT=\"\"").unwrap();
            writeln!(file, "while [ \"$1\" != \"\" ]; do").unwrap();
            writeln!(file, "  if [ \"$1\" = \"-i\" ]; then").unwrap();
            writeln!(file, "    shift").unwrap();
            writeln!(file, "    IN=\"$1\"").unwrap();
            writeln!(file, "  fi").unwrap();
            writeln!(file, "  OUT=\"$1\"").unwrap();
            writeln!(file, "  shift").unwrap();
            writeln!(file, "done").unwrap();
            writeln!(file, "cat \"$IN\" > \"$OUT\"").unwrap();
        }
        #[cfg(windows)]
        {
            writeln!(file, "@echo off").unwrap();
            writeln!(file, "set IN=").unwrap();
            writeln!(file, ":loop").unwrap();
            writeln!(file, "if \"%1\"==\"\" goto end").unwrap();
            writeln!(file, "if /i \"%1\"==\"-i\" (").unwrap();
            writeln!(file, "    shift").unwrap();
            writeln!(file, "    set IN=%1").unwrap();
            writeln!(file, ")").unwrap();
            writeln!(file, "set OUT=%1").unwrap();
            writeln!(file, "shift").unwrap();
            writeln!(file, "goto loop").unwrap();
            writeln!(file, ":end").unwrap();
            writeln!(file, "copy /y %IN% %OUT% >nul").unwrap();
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&path).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&path, perms).unwrap();
        }
        path
    }

    #[test]
    #[serial]
    fn fails_when_source_missing() {
        let response = futures::executor::block_on(encode_audio(EncodeArgs {
            source_path: "nope.wav".into(),
            target_path: None,
            format: "mp3".into(),
        }));
        match response {
            EncodeResponse::Error { code, .. } => assert_eq!(code, "ENCODE_SOURCE_MISSING"),
            other => panic!("Respuesta inesperada: {other:?}"),
        }
    }

    #[test]
    #[serial]
    fn mp3_uses_ffmpeg_when_available() {
        let temp = TempDir::new().unwrap();
        let source = create_source(&temp);
        let stub = create_stub_ffmpeg(&temp);
        let _guard = scoped_ffmpeg(stub.to_string_lossy().as_ref());
        let target = temp.path().join("out.mp3");

        let response = futures::executor::block_on(encode_audio(EncodeArgs {
            source_path: source.to_string_lossy().to_string(),
            target_path: Some(target.to_string_lossy().to_string()),
            format: "mp3".into(),
        }));

        match response {
            EncodeResponse::Success { path } => {
                assert!(PathBuf::from(path).exists());
            }
            other => panic!("Respuesta inesperada: {other:?}"),
        }
    }

    #[test]
    #[serial]
    fn mp3_reports_missing_ffmpeg() {
        std::env::remove_var("READER_FFMPEG_PATH");
        let temp = TempDir::new().unwrap();
        let source = create_source(&temp);
        let response = futures::executor::block_on(encode_audio(EncodeArgs {
            source_path: source.to_string_lossy().to_string(),
            target_path: None,
            format: "mp3".into(),
        }));
        match response {
            EncodeResponse::Error { code, .. } => assert_eq!(code, "FFMPEG_MISSING"),
            other => panic!("Respuesta inesperada: {other:?}"),
        }
    }

    #[test]
    #[serial]
    fn non_mp3_formats_copy_file() {
        let temp = TempDir::new().unwrap();
        let source = create_source(&temp);
        let target = temp.path().join("copy.wav");
        let response = futures::executor::block_on(encode_audio(EncodeArgs {
            source_path: source.to_string_lossy().to_string(),
            target_path: Some(target.to_string_lossy().to_string()),
            format: "wav".into(),
        }));
        match response {
            EncodeResponse::Success { path } => {
                assert!(PathBuf::from(path).exists());
            }
            other => panic!("Respuesta inesperada: {other:?}"),
        }
    }
}
