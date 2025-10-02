use std::{
    fs,
    path::PathBuf,
    process::{Command, Stdio},
    time::Instant,
};

use log::{error, info};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use shlex::Shlex;

use super::CommandError;

pub const ERROR_SCRIPT_FAILED: &str = "SCRIPT_FAILED";
pub const ERROR_INVALID_JSON: &str = "INVALID_JSON";
pub const ERROR_INVALID_RESPONSE: &str = "INVALID_RESPONSE";
pub const ERROR_IO: &str = "IO_ERROR";

#[derive(Debug, Deserialize)]
pub struct ImportPdfRequest {
    pub path: PathBuf,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct DocumentSection {
    pub id: Option<String>,
    pub heading: Option<String>,
    pub content: String,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct ImportedDocument {
    pub title: Option<String>,
    pub language: Option<String>,
    pub sections: Vec<DocumentSection>,
    pub metadata: Value,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct ImportResponse {
    pub document: ImportedDocument,
    pub warnings: Vec<String>,
    pub duration_ms: u128,
}

#[derive(Debug, Deserialize)]
struct RawImport {
    title: Option<String>,
    language: Option<String>,
    sections: Vec<RawSection>,
    #[serde(default)]
    metadata: Option<Map<String, Value>>,
    #[serde(default)]
    warnings: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct RawSection {
    id: Option<String>,
    heading: Option<String>,
    content: Option<String>,
}

fn build_command(env_key: &str, default_script: &str) -> Result<Command, CommandError> {
    if let Some(raw_command) = std::env::var_os(env_key) {
        let raw_command = raw_command.to_string_lossy().into_owned();
        let mut shlex = Shlex::new(&raw_command);
        let mut parts: Vec<String> = shlex.collect();
        if parts.is_empty() {
            return Err(CommandError::new(
                ERROR_IO,
                format!("{env_key} is empty"),
                None,
            ));
        }
        let program = parts.remove(0);
        let mut command = Command::new(program);
        for part in parts {
            command.arg(part);
        }
        Ok(command)
    } else {
        let mut command = Command::new("python");
        command.arg(default_script);
        Ok(command)
    }
}

pub(crate) fn run_importer(
    env_key: &str,
    default_script: &str,
    path: &PathBuf,
) -> Result<ImportResponse, CommandError> {
    if !path.exists() {
        return Err(CommandError::new(
            ERROR_IO,
            format!("File not found: {}", path.display()),
            None,
        ));
    }

    let mut command = build_command(env_key, default_script)?;
    command.arg(path);
    command.stdin(Stdio::null());
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());

    let start = Instant::now();
    let output = command.output().map_err(|err| {
        CommandError::new(ERROR_IO, "Failed to run importer", Some(err.to_string()))
    })?;
    let duration_ms = start.elapsed().as_millis();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    if !output.status.success() {
        error!(
            "Importer {env_key} failed with status {:?}: {}",
            output.status.code(),
            stderr
        );
        return Err(CommandError::new(
            ERROR_SCRIPT_FAILED,
            format!("Importer exited with status {:?}", output.status.code()),
            if stderr.is_empty() {
                None
            } else {
                Some(stderr)
            },
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let raw: RawImport = serde_json::from_str(&stdout).map_err(|err| {
        CommandError::new(
            ERROR_INVALID_JSON,
            "Importer returned invalid JSON",
            Some(err.to_string()),
        )
    })?;

    let sections = raw
        .sections
        .into_iter()
        .map(|section| {
            let content = section.content.ok_or_else(|| {
                CommandError::new(
                    ERROR_INVALID_RESPONSE,
                    "Section is missing the content field",
                    None,
                )
            })?;
            Ok(DocumentSection {
                id: section.id,
                heading: section.heading,
                content,
            })
        })
        .collect::<Result<Vec<_>, CommandError>>()?;

    let metadata = raw.metadata.map(Value::Object).unwrap_or(Value::Null);

    Ok(ImportResponse {
        document: ImportedDocument {
            title: raw.title,
            language: raw.language,
            sections,
            metadata,
        },
        warnings: raw.warnings,
        duration_ms,
    })
}

pub fn import_pdf(request: ImportPdfRequest) -> Result<ImportResponse, CommandError> {
    info!("Importing PDF {}", request.path.display());
    run_importer(
        "READER_IMPORT_PDF_COMMAND",
        "scripts/py/import_pdf.py",
        &request.path,
    )
}

#[tauri::command]
pub fn import_pdf_command(path: PathBuf) -> Result<Vec<String>, CommandError> {
    import_pdf(ImportPdfRequest { path }).map(|response| {
        response
            .document
            .sections
            .into_iter()
            .map(|section| section.content)
            .collect()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::TempDir;

    struct EnvGuard {
        key: &'static str,
    }

    impl EnvGuard {
        fn set(key: &'static str, value: String) -> Self {
            std::env::set_var(key, value);
            Self { key }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            std::env::remove_var(self.key);
        }
    }

    fn write_mock_importer(temp: &TempDir, body: &str) -> EnvGuard {
        let script_path = temp.path().join("importer.py");
        fs::write(&script_path, body).unwrap();
        EnvGuard::set(
            "READER_IMPORT_PDF_COMMAND",
            format!("python3 {}", script_path.display()),
        )
    }

    fn sample_request(temp: &TempDir) -> ImportPdfRequest {
        let pdf = temp.path().join("doc.pdf");
        fs::write(&pdf, b"pdf").unwrap();
        ImportPdfRequest { path: pdf }
    }

    #[test]
    fn successful_import_parses_json() {
        let temp = TempDir::new().unwrap();
        let _guard = write_mock_importer(
            &temp,
            r#"import json, sys
print(json.dumps({
    "title": "Doc",
    "language": "es",
    "sections": [
        {"id": "1", "heading": "Intro", "content": "Hola"}
    ],
    "metadata": {"pages": 3},
    "warnings": ["minor"]
}))
"#,
        );
        let request = sample_request(&temp);
        let response = import_pdf(request).unwrap();
        assert_eq!(response.document.title.as_deref(), Some("Doc"));
        assert_eq!(response.document.sections.len(), 1);
        assert_eq!(response.warnings, vec!["minor".to_string()]);
    }

    #[test]
    fn import_pdf_command_returns_content_only() {
        let temp = TempDir::new().unwrap();
        let _guard = write_mock_importer(
            &temp,
            r#"import json, sys
print(json.dumps({
    "sections": [
        {"id": "1", "heading": "Intro", "content": "Hola"},
        {"id": "2", "heading": "Capítulo", "content": "Mundo"}
    ]
}))
"#,
        );
        let request = sample_request(&temp);
        let sections = import_pdf_command(request.path).unwrap();
        assert_eq!(sections, vec!["Hola".to_string(), "Mundo".to_string()]);
    }

    #[test]
    fn importer_error_returns_script_error() {
        let temp = TempDir::new().unwrap();
        let _guard =
            write_mock_importer(&temp, "import sys\nsys.stderr.write('fail')\nsys.exit(1)");
        let request = sample_request(&temp);
        let error = import_pdf(request).unwrap_err();
        assert_eq!(error.code, ERROR_SCRIPT_FAILED);
        assert_eq!(error.details.unwrap(), "fail");
    }

    #[test]
    fn import_pdf_command_propagates_errors() {
        let temp = TempDir::new().unwrap();
        let _guard =
            write_mock_importer(&temp, "import sys\nsys.stderr.write('boom')\nsys.exit(3)");
        let request = sample_request(&temp);
        let error = import_pdf_command(request.path).unwrap_err();
        assert_eq!(error.code, ERROR_SCRIPT_FAILED);
        assert_eq!(error.details.as_deref(), Some("boom"));
    }

    #[test]
    fn invalid_json_returns_error() {
        let temp = TempDir::new().unwrap();
        let _guard = write_mock_importer(&temp, "print('not json')");
        let request = sample_request(&temp);
        let error = import_pdf(request).unwrap_err();
        assert_eq!(error.code, ERROR_INVALID_JSON);
    }

    #[test]
    fn missing_content_is_invalid_response() {
        let temp = TempDir::new().unwrap();
        let _guard = write_mock_importer(
            &temp,
            r#"import json
print(json.dumps({"sections": [{}]}))
"#,
        );
        let request = sample_request(&temp);
        let error = import_pdf(request).unwrap_err();
        assert_eq!(error.code, ERROR_INVALID_RESPONSE);
    }

    #[test]
    fn missing_file_returns_io_error() {
        let temp = TempDir::new().unwrap();
        let request = ImportPdfRequest {
            path: temp.path().join("missing.pdf"),
        };
        let error = import_pdf(request).unwrap_err();
        assert_eq!(error.code, ERROR_IO);
    }
}
