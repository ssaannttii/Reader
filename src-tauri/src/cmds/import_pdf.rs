use std::ffi::OsString;
use std::path::PathBuf;
use std::process::Command;

use log::{error, info};
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(tag = "ok")]
pub enum ImportPdfResponse {
    #[serde(rename = "true")]
    Success {
        pages: serde_json::Value,
        meta: serde_json::Value,
    },
    #[serde(rename = "false")]
    Error { code: String, message: String },
}

#[tauri::command]
pub async fn import_pdf(path: String) -> ImportPdfResponse {
    let pdf_path = PathBuf::from(&path);
    if !pdf_path.exists() {
        return ImportPdfResponse::Error {
            code: "PDF_NOT_FOUND".into(),
            message: format!("No se encontró {path}"),
        };
    }

    let script = std::env::var_os("READER_PDF_SCRIPT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("scripts/py/pdf_extract.py"));
    let python_cmd: OsString =
        std::env::var_os("READER_PYTHON_BIN").unwrap_or_else(|| OsString::from("python"));
    let output = match Command::new(&python_cmd)
        .arg(&script)
        .arg(&pdf_path)
        .output()
    {
        Ok(output) => output,
        Err(err) => {
            error!("No se pudo ejecutar pdf_extract: {err}");
            return ImportPdfResponse::Error {
                code: "PDF_SCRIPT_FAIL".into(),
                message: err.to_string(),
            };
        }
    };

    if !output.status.success() {
        let message = String::from_utf8_lossy(&output.stderr).to_string();
        error!("pdf_extract falló: {message}");
        return ImportPdfResponse::Error {
            code: "PDF_SCRIPT_FAIL".into(),
            message,
        };
    }

    match serde_json::from_slice::<serde_json::Value>(&output.stdout) {
        Ok(json) => {
            if json.get("ok") == Some(&serde_json::Value::Bool(true)) {
                let pages = json.get("pages").cloned().unwrap_or_default();
                let meta = json.get("meta").cloned().unwrap_or_default();
                info!(
                    "PDF importado con {} páginas",
                    pages.as_array().map(|arr| arr.len()).unwrap_or(0)
                );
                ImportPdfResponse::Success { pages, meta }
            } else {
                ImportPdfResponse::Error {
                    code: json
                        .get("code")
                        .and_then(|v| v.as_str())
                        .unwrap_or("PDF_PARSE_FAIL")
                        .to_string(),
                    message: json
                        .get("message")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Fallo al importar PDF")
                        .to_string(),
                }
            }
        }
        Err(err) => ImportPdfResponse::Error {
            code: "PDF_PARSE_FAIL".into(),
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
    fn missing_pdf_returns_error() {
        let response = futures::executor::block_on(import_pdf("no-existe.pdf".into()));
        match response {
            ImportPdfResponse::Error { code, .. } => assert_eq!(code, "PDF_NOT_FOUND"),
            other => panic!("Respuesta inesperada: {other:?}"),
        }
    }

    #[test]
    #[serial]
    fn successful_invocation_returns_payload() {
        let temp = TempDir::new().unwrap();
        let pdf_path = temp.path().join("demo.pdf");
        File::create(&pdf_path).unwrap();
        let script = write_stub(
            &temp,
            "pdf_stub.py",
            "import json\nprint(json.dumps({\"ok\": True, \"pages\": [{\"text\": \"Hola\"}], \"meta\": {\"title\": \"Demo\", \"author\": \"Autor\"}}))\n",
        );
        let _env_script = scoped_env("READER_PDF_SCRIPT", script.to_string_lossy().to_string());
        let _env_python = scoped_env("READER_PYTHON_BIN", "python".to_string());

        AssertCommand::new("python")
            .arg(script.to_string_lossy().to_string())
            .arg(pdf_path.to_string_lossy().to_string())
            .assert()
            .success();

        let response =
            futures::executor::block_on(import_pdf(pdf_path.to_string_lossy().to_string()));
        match response {
            ImportPdfResponse::Success { pages, meta } => {
                assert_eq!(pages.as_array().unwrap().len(), 1);
                assert_eq!(meta["title"], "Demo");
            }
            other => panic!("Respuesta inesperada: {other:?}"),
        }
    }

    #[test]
    #[serial]
    fn surfaces_script_failure() {
        let temp = TempDir::new().unwrap();
        let pdf_path = temp.path().join("demo.pdf");
        File::create(&pdf_path).unwrap();
        let script = write_stub(
            &temp,
            "pdf_fail.py",
            "import sys\nsys.stderr.write('fallo')\nsys.exit(1)\n",
        );
        let _env_script = scoped_env("READER_PDF_SCRIPT", script.to_string_lossy().to_string());
        let _env_python = scoped_env("READER_PYTHON_BIN", "python".to_string());

        let response =
            futures::executor::block_on(import_pdf(pdf_path.to_string_lossy().to_string()));
        match response {
            ImportPdfResponse::Error { code, .. } => assert_eq!(code, "PDF_SCRIPT_FAIL"),
            other => panic!("Respuesta inesperada: {other:?}"),
        }
    }
}
