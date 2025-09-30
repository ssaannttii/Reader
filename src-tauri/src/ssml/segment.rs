use regex::Regex;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Segment {
    pub text: String,
    pub pause_ms: u32,
    pub length_scale: f32,
    pub noise_scale: f32,
}

impl Segment {
    fn new(text: String) -> Self {
        Self {
            text,
            pause_ms: 0,
            length_scale: 1.0,
            noise_scale: 0.5,
        }
    }
}

const ABBREVIATIONS: &[&str] = &["Sr.", "Sra.", "Dr.", "Dra.", "etc."];

pub fn segment(input: &str) -> Vec<Segment> {
    let break_re = Regex::new(r#"(?i)<break\s+time="(\d+)ms"\s*/?>"#).unwrap();
    let emphasis_re = Regex::new(r"(?is)<emphasis>(.*?)</emphasis>").unwrap();
    let mut normalized = input.replace("\r\n", "\n");
    normalized = break_re
        .replace_all(&normalized, "\n[[BREAK:$1]]\n")
        .to_string();
    normalized = emphasis_re
        .replace_all(&normalized, |caps: &regex::Captures| {
            format!("[[EMPH:{}]]", caps[1].trim())
        })
        .to_string();

    let mut segments = Vec::new();
    let mut pending_pause = 0;

    for block in normalized.split("\n\n") {
        let block = block.trim();
        if block.is_empty() {
            continue;
        }

        for part in split_sentences(block) {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }
            if part.starts_with("[[BREAK:") && part.ends_with(']') {
                if let Some(value) = part.trim_matches(&['[', ']'][..]).strip_prefix("BREAK:") {
                    pending_pause = value.parse().unwrap_or(pending_pause);
                }
                continue;
            }

            if part.starts_with("[[EMPH:") && part.ends_with(']') {
                if let Some(inner) = part.trim_matches(&['[', ']'][..]).strip_prefix("EMPH:") {
                    let mut segment = Segment::new(inner.to_string());
                    segment.length_scale = 0.9;
                    segment.noise_scale = 0.4;
                    segment.pause_ms = pending_pause;
                    pending_pause = 0;
                    segments.push(segment);
                }
                continue;
            }

            let mut segment = Segment::new(part.to_string());
            segment.pause_ms = pending_pause;
            pending_pause = 0;
            segments.push(segment);
        }
    }

    segments
}

fn split_sentences(paragraph: &str) -> Vec<String> {
    let sentence_re = Regex::new(r"(?<=[.!?])\s+(?=[A-ZÁÉÍÓÚÑ])").unwrap();
    let mut sentences = Vec::new();
    let mut start = 0;
    for mat in sentence_re.find_iter(paragraph) {
        let candidate = &paragraph[start..mat.end()].trim();
        if !is_abbreviation(candidate) {
            sentences.push(candidate.to_string());
            start = mat.end();
        }
    }
    let tail = paragraph[start..].trim();
    if !tail.is_empty() {
        sentences.push(tail.to_string());
    }
    sentences
}

fn is_abbreviation(sentence: &str) -> bool {
    ABBREVIATIONS.iter().any(|abbr| sentence.ends_with(abbr))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn segments_paragraphs_and_sentences() {
        let text = "Hola mundo. Esto es una prueba.\n\nNueva línea";
        let segments = segment(text);
        assert_eq!(segments.len(), 3);
        assert_eq!(segments[0].text, "Hola mundo.");
        assert_eq!(segments[1].text, "Esto es una prueba.");
        assert_eq!(segments[2].text, "Nueva línea");
    }

    #[test]
    fn respects_abbreviations() {
        let text = "El Sr. López llegó. Hola.";
        let segments = segment(text);
        assert_eq!(segments.len(), 2);
    }

    #[test]
    fn handles_break_and_emphasis() {
        let text = "Hola<break time=\"500ms\"/><emphasis>Importante</emphasis>";
        let segments = segment(text);
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].pause_ms, 0);
        assert_eq!(segments[1].pause_ms, 500);
        assert!((segments[1].length_scale - 0.9).abs() < f32::EPSILON);
    }
}
