use std::ffi::OsString;
use std::path::PathBuf;
use std::process::Command;

use log::{error, info};
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(tag = "ok")]
pub enum ImportEpubResponse {
    #[serde(rename = "true")]
    Success { chapters: serde_json::Value },
    #[serde(rename = "false")]
    Error { code: String, message: String },
}

#[tauri::command]
pub async fn import_epub(path: String) -> ImportEpubResponse {
    let epub_path = PathBuf::from(&path);
    if !epub_path.exists() {
        return ImportEpubResponse::Error {
            code: "EPUB_NOT_FOUND".into(),
            message: format!("No se encontró {path}"),
        };
    }

    let script = std::env::var_os("READER_EPUB_SCRIPT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("scripts/py/epub_extract.py"));
    let python_cmd: OsString =
        std::env::var_os("READER_PYTHON_BIN").unwrap_or_else(|| OsString::from("python"));
    let output = match Command::new(&python_cmd)
        .arg(&script)
        .arg(&epub_path)
        .output()
    {
        Ok(output) => output,
        Err(err) => {
            error!("No se pudo ejecutar epub_extract: {err}");
            return ImportEpubResponse::Error {
                code: "EPUB_SCRIPT_FAIL".into(),
                message: err.to_string(),
            };
        }
    };

    if !output.status.success() {
        let message = String::from_utf8_lossy(&output.stderr).to_string();
        error!("epub_extract falló: {message}");
        return ImportEpubResponse::Error {
            code: "EPUB_SCRIPT_FAIL".into(),
            message,
        };
    }

    match serde_json::from_slice::<serde_json::Value>(&output.stdout) {
        Ok(json) => {
            if json.get("ok") == Some(&serde_json::Value::Bool(true)) {
                let chapters = json.get("chapters").cloned().unwrap_or_default();
                info!(
                    "EPUB importado con {} capítulos",
                    chapters.as_array().map(|arr| arr.len()).unwrap_or(0)
                );
                ImportEpubResponse::Success { chapters }
            } else {
                ImportEpubResponse::Error {
                    code: json
                        .get("code")
                        .and_then(|v| v.as_str())
                        .unwrap_or("EPUB_PARSE_FAIL")
                        .to_string(),
                    message: json
                        .get("message")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Fallo al importar EPUB")
                        .to_string(),
                }
            }
        }
        Err(err) => ImportEpubResponse::Error {
            code: "EPUB_PARSE_FAIL".into(),
            message: err.to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_cmd::Command as AssertCommand;
    use serial_test::serial;
    use std::fs::File;
    use std::io::Write;
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

    fn scoped_env(key: &'static str, value: String) -> EnvGuard {
        let previous = std::env::var(key).ok();
        std::env::set_var(key, &value);
        EnvGuard { key, previous }
    }

    fn write_stub(temp: &TempDir, name: &str, body: &str) -> PathBuf {
        let path = temp.path().join(name);
        let mut file = File::create(&path).unwrap();
        file.write_all(body.as_bytes()).unwrap();
        path
    }

    #[test]
    #[serial]
    fn missing_epub_returns_error() {
        let response = futures::executor::block_on(import_epub("no-existe.epub".into()));
        match response {
            ImportEpubResponse::Error { code, .. } => assert_eq!(code, "EPUB_NOT_FOUND"),
            other => panic!("Respuesta inesperada: {other:?}"),
        }
    }

    #[test]
    #[serial]
    fn successful_invocation_returns_chapters() {
        let temp = TempDir::new().unwrap();
        let epub_path = temp.path().join("demo.epub");
        File::create(&epub_path).unwrap();
        let script = write_stub(
            &temp,
            "epub_stub.py",
            "import json\nprint(json.dumps({\"ok\": True, \"chapters\": [{\"title\": \"Cap 1\", \"paragraphs\": [\"Hola\"]}]}))\n",
        );
        let _env_script = scoped_env("READER_EPUB_SCRIPT", script.to_string_lossy().to_string());
        let _env_python = scoped_env("READER_PYTHON_BIN", "python".to_string());

        AssertCommand::new("python")
            .arg(script.to_string_lossy().to_string())
            .arg(epub_path.to_string_lossy().to_string())
            .assert()
            .success();

        let response =
            futures::executor::block_on(import_epub(epub_path.to_string_lossy().to_string()));
        match response {
            ImportEpubResponse::Success { chapters } => {
                assert_eq!(chapters.as_array().unwrap().len(), 1);
            }
            other => panic!("Respuesta inesperada: {other:?}"),
        }
    }

    #[test]
    #[serial]
    fn propagates_script_failure() {
        let temp = TempDir::new().unwrap();
        let epub_path = temp.path().join("demo.epub");
        File::create(&epub_path).unwrap();
        let script = write_stub(
            &temp,
            "epub_fail.py",
            "import sys\nsys.stderr.write('fallo')\nsys.exit(1)\n",
        );
        let _env_script = scoped_env("READER_EPUB_SCRIPT", script.to_string_lossy().to_string());
        let _env_python = scoped_env("READER_PYTHON_BIN", "python".to_string());

        let response =
            futures::executor::block_on(import_epub(epub_path.to_string_lossy().to_string()));
        match response {
            ImportEpubResponse::Error { code, .. } => assert_eq!(code, "EPUB_SCRIPT_FAIL"),
            other => panic!("Respuesta inesperada: {other:?}"),
        }
    }
}
