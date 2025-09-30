use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Instant;

use log::{error, info};
use serde::{Deserialize, Serialize};

use crate::audio::player::Player;
use crate::dict::lexicon::apply_replacements;
use crate::util::piper_path::{command_to_args, resolve};
use crate::util::runtime::runtime_dir;

#[derive(Debug, Deserialize)]
pub struct SpeakArgs {
    pub text: String,
    pub voice_path: String,
    #[serde(default)]
    pub out_path: Option<String>,
    #[serde(default = "default_sentence_break")]
    pub sentence_break: u32,
    #[serde(default = "default_length_scale")]
    pub length_scale: f32,
    #[serde(default = "default_noise_scale")]
    pub noise_scale: f32,
    #[serde(default = "default_noise_w")]
    pub noise_w: f32,
    #[serde(default)]
    pub play_after: bool,
}

fn default_sentence_break() -> u32 {
    550
}

fn default_length_scale() -> f32 {
    1.0
}

fn default_noise_scale() -> f32 {
    0.5
}

fn default_noise_w() -> f32 {
    0.9
}

#[derive(Debug, Serialize)]
#[serde(tag = "ok")]
pub enum SpeakResponse {
    #[serde(rename = "true")]
    Success {
        #[serde(rename = "outPath")]
        out_path: String,
        #[serde(rename = "elapsedMs")]
        elapsed_ms: u128,
    },
    #[serde(rename = "false")]
    Error {
        code: SpeakErrorCode,
        message: String,
    },
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SpeakErrorCode {
    VoiceNotFound,
    PiperNotFound,
    SynthFail,
}

#[tauri::command]
pub async fn speak(args: SpeakArgs) -> SpeakResponse {
    info!("Solicitando síntesis de {} bytes", args.text.len());

    let voice_path = PathBuf::from(&args.voice_path);
    if !voice_path.exists() {
        error!("Voz no encontrada: {:?}", voice_path);
        return SpeakResponse::Error {
            code: SpeakErrorCode::VoiceNotFound,
            message: format!("No se encontró la voz en {:?}", voice_path),
        };
    }

    let runtime_dir = runtime_dir();
    if let Err(err) = fs::create_dir_all(&runtime_dir) {
        error!("No se pudo preparar runtime: {err}");
        return SpeakResponse::Error {
            code: SpeakErrorCode::SynthFail,
            message: format!("No se pudo preparar runtime: {err}"),
        };
    }

    let piper_command = match resolve(&runtime_dir) {
        Ok(cmd) => cmd,
        Err(err) => {
            error!("No se encontró Piper: {err}");
            return SpeakResponse::Error {
                code: SpeakErrorCode::PiperNotFound,
                message: err.to_string(),
            };
        }
    };

    let out_path = args
        .out_path
        .clone()
        .unwrap_or_else(|| runtime_dir.join("out.wav").to_string_lossy().to_string());
    if let Some(parent) = PathBuf::from(&out_path).parent() {
        if let Err(err) = fs::create_dir_all(parent) {
            error!("No se pudo preparar la carpeta de salida: {err}");
            return SpeakResponse::Error {
                code: SpeakErrorCode::SynthFail,
                message: format!("No se pudo preparar la carpeta de salida: {err}"),
            };
        }
    }

    let (binary, mut extra_args) = command_to_args(&piper_command);
    extra_args.extend(
        [
            "-m".into(),
            voice_path.into_os_string(),
            "-f".into(),
            PathBuf::from(&out_path).into_os_string(),
            "--sentence-break".into(),
            args.sentence_break.to_string().into(),
            "--length-scale".into(),
            args.length_scale.to_string().into(),
            "--noise-scale".into(),
            args.noise_scale.to_string().into(),
            "--noise-w".into(),
            args.noise_w.to_string().into(),
        ]
        .into_iter(),
    );

    let mut command = Command::new(binary);
    command.args(extra_args);
    command.stdin(Stdio::piped());
    command.stdout(Stdio::inherit());
    command.stderr(Stdio::inherit());

    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(err) => {
            error!("No se pudo iniciar Piper: {err}");
            return SpeakResponse::Error {
                code: SpeakErrorCode::SynthFail,
                message: err.to_string(),
            };
        }
    };

    let normalized_text = apply_replacements(args.text.trim());
    let start = Instant::now();

    if let Some(mut stdin) = child.stdin.take() {
        use std::io::Write;
        if let Err(err) = stdin.write_all(normalized_text.as_bytes()) {
            error!("No se pudo enviar texto a Piper: {err}");
            return SpeakResponse::Error {
                code: SpeakErrorCode::SynthFail,
                message: err.to_string(),
            };
        }
    }

    match child.wait() {
        Ok(status) if status.success() => {
            let elapsed = start.elapsed().as_millis();
            info!("Síntesis completada en {} ms", elapsed);
            if args.play_after {
                if let Err(err) = Player::global().play(&out_path) {
                    error!("No se pudo reproducir el audio: {err}");
                }
            }
            SpeakResponse::Success {
                out_path,
                elapsed_ms: elapsed,
            }
        }
        Ok(status) => {
            let message = format!("Piper terminó con código {status}");
            error!("{message}");
            SpeakResponse::Error {
                code: SpeakErrorCode::SynthFail,
                message,
            }
        }
        Err(err) => {
            error!("Error esperando Piper: {err}");
            SpeakResponse::Error {
                code: SpeakErrorCode::SynthFail,
                message: err.to_string(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::Path;
    use tempfile::TempDir;

    struct EnvGuard {
        key: &'static str,
        previous: Option<String>,
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            if let Some(value) = &self.previous {
                std::env::set_var(self.key, value);
            } else {
                std::env::remove_var(self.key);
            }
        }
    }

    fn scoped_env(key: &'static str, value: &Path) -> EnvGuard {
        let previous = std::env::var(key).ok();
        std::env::set_var(key, value);
        EnvGuard { key, previous }
    }

    fn scoped_env_str(key: &'static str, value: String) -> EnvGuard {
        let previous = std::env::var(key).ok();
        std::env::set_var(key, &value);
        EnvGuard { key, previous }
    }

    fn create_voice(temp: &TempDir) -> PathBuf {
        let path = temp.path().join("voice.onnx");
        File::create(&path).unwrap();
        path
    }

    fn create_stub_piper(dir: &Path) -> PathBuf {
        fs::create_dir_all(dir).unwrap();
        let name = if cfg!(windows) { "piper.exe" } else { "piper" };
        let path = dir.join(name);
        let mut file = File::create(&path).unwrap();
        #[cfg(windows)]
        {
            writeln!(file, "@echo off").unwrap();
            writeln!(file, "set OUTPUT=").unwrap();
            writeln!(file, "goto parse").unwrap();
        }
        #[cfg(not(windows))]
        {
            writeln!(file, "#!/bin/sh").unwrap();
            writeln!(file, "OUTPUT=\"\"").unwrap();
            writeln!(file, "while [ \"$1\" != \"\" ]; do").unwrap();
            writeln!(file, "  if [ \"$1\" = \"-f\" ]; then").unwrap();
            writeln!(file, "    shift").unwrap();
            writeln!(file, "    OUTPUT=\"$1\"").unwrap();
            writeln!(file, "  fi").unwrap();
            writeln!(file, "  shift").unwrap();
            writeln!(file, "done").unwrap();
            writeln!(file, "cat > \"$OUTPUT\"").unwrap();
        }
        #[cfg(windows)]
        {
            writeln!(file, ":parse").unwrap();
            writeln!(file, "if \"%1\"==\"\" goto synth").unwrap();
            writeln!(file, "if /i \"%1\"==\"-f\" (").unwrap();
            writeln!(file, "    shift").unwrap();
            writeln!(file, "    set OUTPUT=%1").unwrap();
            writeln!(file, ")").unwrap();
            writeln!(file, "shift").unwrap();
            writeln!(file, "goto parse").unwrap();
            writeln!(file, ":synth").unwrap();
            writeln!(file, "setlocal enabledelayedexpansion").unwrap();
            writeln!(file, "set CONTENT=").unwrap();
            writeln!(
                file,
                "for /f \"usebackq tokens=* delims=\" %%L in (`more`) do ("
            )
            .unwrap();
            writeln!(file, "    if defined CONTENT (").unwrap();
            writeln!(file, "        set CONTENT=!CONTENT!%%L").unwrap();
            writeln!(file, "    ) else (").unwrap();
            writeln!(file, "        set CONTENT=%%L").unwrap();
            writeln!(file, "    )").unwrap();
            writeln!(file, ")").unwrap();
            writeln!(file, "if not defined OUTPUT exit /b 1").unwrap();
            writeln!(file, "echo(!CONTENT!)> \"%OUTPUT%\"").unwrap();
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
    fn voice_missing_returns_error() {
        let response = futures::executor::block_on(speak(SpeakArgs {
            text: "Hola".into(),
            voice_path: "no existe".into(),
            out_path: None,
            sentence_break: default_sentence_break(),
            length_scale: default_length_scale(),
            noise_scale: default_noise_scale(),
            noise_w: default_noise_w(),
            play_after: false,
        }));
        match response {
            SpeakResponse::Error { code, .. } => {
                assert!(matches!(code, SpeakErrorCode::VoiceNotFound))
            }
            _ => panic!("Se esperaba error"),
        }
    }

    #[test]
    #[serial]
    fn synthesizes_with_runtime_executable() {
        let temp = TempDir::new().unwrap();
        let runtime = temp.path().join("runtime");
        let _env = scoped_env("READER_RUNTIME_DIR", &runtime);
        let voice = create_voice(&temp);
        let piper_dir = runtime.join("piper");
        create_stub_piper(&piper_dir);
        let out_path = temp.path().join("out.wav");

        let response = futures::executor::block_on(speak(SpeakArgs {
            text: "Hola mundo".into(),
            voice_path: voice.to_string_lossy().to_string(),
            out_path: Some(out_path.to_string_lossy().to_string()),
            sentence_break: default_sentence_break(),
            length_scale: default_length_scale(),
            noise_scale: default_noise_scale(),
            noise_w: default_noise_w(),
            play_after: false,
        }));

        match response {
            SpeakResponse::Success { out_path, .. } => {
                assert!(Path::new(&out_path).exists());
            }
            other => panic!("Respuesta inesperada: {other:?}"),
        }
    }

    #[test]
    #[serial]
    fn falls_back_to_python_module() {
        let temp = TempDir::new().unwrap();
        let runtime = temp.path().join("runtime");
        let _env_runtime = scoped_env("READER_RUNTIME_DIR", &runtime);
        let voice = create_voice(&temp);
        let module_dir = temp.path().join("py_mod").join("piper");
        fs::create_dir_all(&module_dir).unwrap();
        let mut module = File::create(module_dir.join("__main__.py")).unwrap();
        writeln!(
            module,
            "import sys\nfrom pathlib import Path\n\nargs = sys.argv[1:]\nout = None\nfor idx, arg in enumerate(args):\n    if arg == '-f' and idx + 1 < len(args):\n        out = args[idx + 1]\nif out is None:\n    sys.exit(1)\ntext = sys.stdin.read() or 'demo'\nPath(out).write_text(text, encoding='utf-8')\n"
        )
        .unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(module_dir.join("__main__.py"))
                .unwrap()
                .permissions();
            perms.set_mode(0o644);
            fs::set_permissions(module_dir.join("__main__.py"), perms).unwrap();
        }
        let pythonpath = temp.path().join("py_mod");
        let _env_py = scoped_env_str("PYTHONPATH", pythonpath.to_string_lossy().to_string());
        let out_path = temp.path().join("python-out.wav");

        let response = futures::executor::block_on(speak(SpeakArgs {
            text: "Texto de prueba".into(),
            voice_path: voice.to_string_lossy().to_string(),
            out_path: Some(out_path.to_string_lossy().to_string()),
            sentence_break: default_sentence_break(),
            length_scale: default_length_scale(),
            noise_scale: default_noise_scale(),
            noise_w: default_noise_w(),
            play_after: false,
        }));

        match response {
            SpeakResponse::Success { out_path, .. } => {
                let contents = fs::read_to_string(out_path).unwrap();
                assert!(contents.contains("Texto de prueba"));
            }
            other => panic!("Respuesta inesperada: {other:?}"),
        }
    }

    #[test]
    #[serial]
    fn reports_error_when_piper_fails() {
        let temp = TempDir::new().unwrap();
        let runtime = temp.path().join("runtime");
        let _env = scoped_env("READER_RUNTIME_DIR", &runtime);
        let voice = create_voice(&temp);
        let piper_dir = runtime.join("piper");
        fs::create_dir_all(&piper_dir).unwrap();
        let name = if cfg!(windows) { "piper.exe" } else { "piper" };
        let mut file = File::create(piper_dir.join(name)).unwrap();
        #[cfg(not(windows))]
        {
            writeln!(file, "#!/bin/sh").unwrap();
            writeln!(file, "exit 1").unwrap();
        }
        #[cfg(windows)]
        {
            writeln!(file, "@echo off").unwrap();
            writeln!(file, "exit /b 1").unwrap();
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(piper_dir.join(name)).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(piper_dir.join(name), perms).unwrap();
        }

        let response = futures::executor::block_on(speak(SpeakArgs {
            text: "Hola".into(),
            voice_path: voice.to_string_lossy().to_string(),
            out_path: None,
            sentence_break: default_sentence_break(),
            length_scale: default_length_scale(),
            noise_scale: default_noise_scale(),
            noise_w: default_noise_w(),
            play_after: false,
        }));

        match response {
            SpeakResponse::Error { code, .. } => {
                assert!(matches!(code, SpeakErrorCode::SynthFail));
            }
            other => panic!("Respuesta inesperada: {other:?}"),
        }
    }
}
