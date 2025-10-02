use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ImportTextRequest {
    pub text: String,
}

pub fn extract_paragraphs(text: &str) -> Vec<String> {
    let mut paragraphs = Vec::new();
    let mut current = Vec::new();

    for line in text.lines() {
        if line.trim().is_empty() {
            if !current.is_empty() {
                paragraphs.push(current.join(" "));
                current.clear();
            }
            continue;
        }
        current.push(line.trim().to_string());
    }

    if !current.is_empty() {
        paragraphs.push(current.join(" "));
    }

    paragraphs
}

#[tauri::command]
pub fn import_text(request: ImportTextRequest) -> Vec<String> {
    extract_paragraphs(&request.text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_paragraphs_on_blank_lines() {
        let text = "Linea uno\nLinea dos\n\nLinea tres";
        let paragraphs = extract_paragraphs(text);
        assert_eq!(paragraphs, vec!["Linea uno Linea dos", "Linea tres"]);
    }

    #[test]
    fn trims_extra_whitespace() {
        let text = "  Hola   mundo  \n\n \t Otro parrafo \n";
        let paragraphs = extract_paragraphs(text);
        assert_eq!(paragraphs, vec!["Hola mundo", "Otro parrafo"]);
    }

    #[test]
    fn ignores_multiple_blank_lines() {
        let text = "Uno\n\n\nDos";
        let paragraphs = extract_paragraphs(text);
        assert_eq!(paragraphs, vec!["Uno", "Dos"]);
    }
}
