use std::path::PathBuf;

use log::info;

use super::{import_pdf, CommandError};
use import_pdf::ImportResponse;

#[derive(Debug, serde::Deserialize)]
pub struct ImportEpubRequest {
    pub path: PathBuf,
}

pub fn import_epub(request: ImportEpubRequest) -> Result<ImportResponse, CommandError> {
    info!("Importing EPUB {}", request.path.display());
    import_pdf::run_importer(
        "READER_IMPORT_EPUB_COMMAND",
        "scripts/py/import_epub.py",
        &request.path,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::TempDir;
    use std::fs;

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
            "READER_IMPORT_EPUB_COMMAND",
            format!("python3 {}", script_path.display()),
        )
    }

    fn sample_request(temp: &TempDir) -> ImportEpubRequest {
        let epub = temp.path().join("book.epub");
        fs::write(&epub, b"epub").unwrap();
        ImportEpubRequest { path: epub }
    }

    #[test]
    fn successful_import_parses_json() {
        let temp = TempDir::new().unwrap();
        let _guard = write_mock_importer(
            &temp,
            r#"import json, sys
print(json.dumps({
    "title": "Libro",
    "language": "es",
    "sections": [
        {"heading": "Capítulo 1", "content": "Hola"}
    ]
}))
"#,
        );
        let request = sample_request(&temp);
        let response = import_epub(request).unwrap();
        assert_eq!(response.document.title.as_deref(), Some("Libro"));
    }

    #[test]
    fn importer_failure_is_reported() {
        let temp = TempDir::new().unwrap();
        let _guard = write_mock_importer(
            &temp,
            "import sys\nsys.stderr.write('kaput')\nsys.exit(2)",
        );
        let request = sample_request(&temp);
        let error = import_epub(request).unwrap_err();
        assert_eq!(error.code, import_pdf::ERROR_SCRIPT_FAILED);
        assert_eq!(error.details.unwrap(), "kaput");
    }

    #[test]
    fn missing_file_returns_error() {
        let temp = TempDir::new().unwrap();
        let request = ImportEpubRequest {
            path: temp.path().join("missing.epub"),
        };
        let error = import_epub(request).unwrap_err();
        assert_eq!(error.code, import_pdf::ERROR_IO);
    }
}
