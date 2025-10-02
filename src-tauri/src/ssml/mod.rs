//! Text segmentation and SSML helpers.
//!
//! Build utilities here to turn raw paragraphs into SSML annotated
//! segments (<break>, <emphasis/>, etc.) for richer speech synthesis.

use std::fmt;

use thiserror::Error;

/// Represents an emphasis level supported by SSML.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmphasisLevel {
    Reduced,
    Moderate,
    Strong,
}

impl EmphasisLevel {
    fn as_str(self) -> &'static str {
        match self {
            EmphasisLevel::Reduced => "reduced",
            EmphasisLevel::Moderate => "moderate",
            EmphasisLevel::Strong => "strong",
        }
    }
}

/// A pause inserted either explicitly by the user or inferred from
/// punctuation while segmenting a paragraph.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pause {
    pub duration_ms: u32,
    pub kind: PauseKind,
}

impl Pause {
    fn explicit(duration_ms: u32) -> Self {
        Self {
            duration_ms,
            kind: PauseKind::Explicit,
        }
    }

    fn sentence(duration_ms: u32) -> Self {
        Self {
            duration_ms,
            kind: PauseKind::Sentence,
        }
    }
}

/// Distinguishes pauses inserted by the engine from explicit pauses in
/// the text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PauseKind {
    Explicit,
    Sentence,
}

/// A segment produced during paragraph segmentation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Segment {
    Text(String),
    Break(Pause),
    Emphasis {
        level: EmphasisLevel,
        children: Vec<Segment>,
    },
}

impl Segment {
    fn is_empty(&self) -> bool {
        matches!(self, Segment::Text(text) if text.trim().is_empty())
    }
}

/// Errors produced while interpreting paragraph markup.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum SsmlError {
    #[error("invalid pause directive: {0}")]
    InvalidPause(String),
}

/// Convert a queue paragraph into SSML ready for Piper.
pub fn render_paragraph(paragraph: &str) -> Result<String, SsmlError> {
    let segments = segment_paragraph(paragraph)?;
    Ok(render_segments(&segments))
}

/// Convert the queue of paragraphs into SSML fragments.
pub fn render_queue<I, S>(queue: I) -> Result<Vec<String>, SsmlError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    queue
        .into_iter()
        .map(|paragraph| render_paragraph(paragraph.as_ref()))
        .collect()
}

/// Break a paragraph into SSML friendly segments. The resulting segments
/// include automatically inferred pauses based on punctuation and any
/// explicit cues found in the paragraph.
pub fn segment_paragraph(paragraph: &str) -> Result<Vec<Segment>, SsmlError> {
    let parsed = parse_markup(paragraph)?;
    Ok(insert_sentence_breaks(parsed))
}

fn parse_markup(input: &str) -> Result<Vec<Segment>, SsmlError> {
    let mut root: Vec<Segment> = Vec::new();
    let mut frames: Vec<Frame> = Vec::new();
    let mut buffer = String::new();
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '\\' => {
                if let Some(next) = chars.next() {
                    buffer.push(next);
                }
            }
            '[' => {
                flush_buffer(&mut buffer, &mut frames, &mut root);
                let mut directive = String::new();
                let mut closed = false;
                while let Some(next) = chars.next() {
                    if next == ']' {
                        closed = true;
                        break;
                    }
                    directive.push(next);
                }
                if !closed {
                    buffer.push('[');
                    buffer.push_str(&directive);
                    continue;
                }
                match parse_directive(&directive)? {
                    Some(segment) => push_segment(segment, &mut frames, &mut root),
                    None => {
                        buffer.push('[');
                        buffer.push_str(&directive);
                        buffer.push(']');
                    }
                }
            }
            '*' => {
                let is_double = matches!(chars.peek(), Some('*'));
                if is_double {
                    chars.next();
                }
                flush_buffer(&mut buffer, &mut frames, &mut root);
                let level = if is_double {
                    EmphasisLevel::Strong
                } else {
                    EmphasisLevel::Moderate
                };
                toggle_emphasis(level, &mut frames, &mut root);
            }
            _ => buffer.push(ch),
        }
    }

    flush_buffer(&mut buffer, &mut frames, &mut root);

    while let Some(frame) = frames.pop() {
        if !frame.segments.is_empty() {
            push_segment(
                Segment::Emphasis {
                    level: frame.level,
                    children: frame.segments,
                },
                &mut frames,
                &mut root,
            );
        } else {
            buffer.push_str(frame.opening_marker());
        }
    }

    if !buffer.is_empty() {
        push_segment(Segment::Text(buffer), &mut frames, &mut root);
    }

    Ok(root)
}

struct Frame {
    level: EmphasisLevel,
    segments: Vec<Segment>,
}

impl Frame {
    fn opening_marker(&self) -> &'static str {
        match self.level {
            EmphasisLevel::Moderate => "*",
            EmphasisLevel::Strong => "**",
            EmphasisLevel::Reduced => "",
        }
    }
}

fn push_segment(segment: Segment, frames: &mut Vec<Frame>, root: &mut Vec<Segment>) {
    if segment.is_empty() {
        return;
    }
    if let Some(frame) = frames.last_mut() {
        frame.segments.push(segment);
    } else {
        root.push(segment);
    }
}

fn flush_buffer(buffer: &mut String, frames: &mut Vec<Frame>, root: &mut Vec<Segment>) {
    if !buffer.is_empty() {
        let text = std::mem::take(buffer);
        push_segment(Segment::Text(text), frames, root);
    }
}

fn toggle_emphasis(level: EmphasisLevel, frames: &mut Vec<Frame>, root: &mut Vec<Segment>) {
    if frames.last().map(|frame| frame.level) == Some(level) {
        if let Some(frame) = frames.pop() {
            push_segment(
                Segment::Emphasis {
                    level,
                    children: frame.segments,
                },
                frames,
                root,
            );
        }
    } else {
        frames.push(Frame {
            level,
            segments: Vec::new(),
        });
    }
}

fn parse_directive(directive: &str) -> Result<Option<Segment>, SsmlError> {
    let directive = directive.trim();
    if directive.is_empty() {
        return Ok(None);
    }

    if let Some(rest) = directive.strip_prefix("pause") {
        return parse_pause(rest).map(|pause| Some(Segment::Break(pause)));
    }
    if let Some(rest) = directive.strip_prefix("break") {
        return parse_pause(rest).map(|pause| Some(Segment::Break(pause)));
    }

    Ok(None)
}

fn parse_pause(rest: &str) -> Result<Pause, SsmlError> {
    let descriptor = rest.trim();
    if descriptor.is_empty() {
        return Ok(Pause::explicit(420));
    }

    let descriptor = descriptor.strip_prefix(':').unwrap_or(descriptor).trim();

    let pause = match descriptor {
        "short" | "corta" => Pause::explicit(220),
        "medium" | "media" => Pause::explicit(420),
        "long" | "larga" => Pause::explicit(680),
        value if let Some(ms) = value.strip_suffix("ms") => {
            let number = ms.trim();
            let duration = number
                .parse::<u32>()
                .map_err(|_| SsmlError::InvalidPause(value.to_string()))?;
            Pause::explicit(duration)
        }
        value if let Some(seconds) = value.strip_suffix('s') => {
            let number = seconds.trim();
            let secs = number
                .parse::<f32>()
                .map_err(|_| SsmlError::InvalidPause(value.to_string()))?;
            Pause::explicit((secs * 1_000.0).round() as u32)
        }
        other => return Err(SsmlError::InvalidPause(other.to_string())),
    };

    Ok(pause)
}

fn insert_sentence_breaks(segments: Vec<Segment>) -> Vec<Segment> {
    let mut output = Vec::new();
    for segment in segments {
        output.extend(expand_segment(segment));
    }
    output
}

fn expand_segment(segment: Segment) -> Vec<Segment> {
    match segment {
        Segment::Text(text) => split_text_with_breaks(&text),
        Segment::Break(pause) => vec![Segment::Break(pause)],
        Segment::Emphasis { level, children } => {
            let mut expanded_children = Vec::new();
            for child in children {
                expanded_children.extend(expand_segment(child));
            }

            if expanded_children.is_empty() {
                return Vec::new();
            }

            let mut result = Vec::new();
            let mut buffer: Vec<Segment> = Vec::new();
            for child in expanded_children {
                match child {
                    Segment::Break(pause) => {
                        if !buffer.is_empty() {
                            result.push(Segment::Emphasis {
                                level,
                                children: buffer,
                            });
                            buffer = Vec::new();
                        }
                        result.push(Segment::Break(pause));
                    }
                    other => buffer.push(other),
                }
            }
            if !buffer.is_empty() {
                result.push(Segment::Emphasis {
                    level,
                    children: buffer,
                });
            }

            result
        }
    }
}

fn split_text_with_breaks(text: &str) -> Vec<Segment> {
    let mut result = Vec::new();
    let mut last_index = 0;
    let mut iter = text.char_indices().peekable();

    while let Some((idx, ch)) = iter.next() {
        if let Some((pause, consumed)) = sentence_boundary(ch, idx, text, &mut iter) {
            let end = idx + consumed;
            if let Some(segment) = make_text_segment(&text[last_index..end]) {
                result.push(segment);
            }
            result.push(Segment::Break(pause));
            last_index = end;
            skip_whitespace(text, &mut last_index, &mut iter);
        }
    }

    if last_index < text.len() {
        if let Some(segment) = make_text_segment(&text[last_index..]) {
            result.push(segment);
        }
    }

    result
}

fn sentence_boundary<'a, I>(
    ch: char,
    idx: usize,
    text: &str,
    iter: &mut std::iter::Peekable<I>,
) -> Option<(Pause, usize)>
where
    I: Iterator<Item = (usize, char)>,
{
    match ch {
        '.' => {
            let remaining = &text[idx..];
            if remaining.as_bytes().starts_with(b"...") {
                iter.next();
                iter.next();
                Some((Pause::sentence(720), 3))
            } else if next_is_whitespace_or_end(text, idx + ch.len_utf8(), iter) {
                Some((Pause::sentence(420), ch.len_utf8()))
            } else {
                None
            }
        }
        '!' | '?' => {
            if next_is_whitespace_or_end(text, idx + ch.len_utf8(), iter) {
                Some((Pause::sentence(520), ch.len_utf8()))
            } else {
                None
            }
        }
        '…' => Some((Pause::sentence(720), ch.len_utf8())),
        ',' => {
            if next_is_whitespace_or_end(text, idx + ch.len_utf8(), iter) {
                Some((Pause::sentence(240), ch.len_utf8()))
            } else {
                None
            }
        }
        ';' | ':' => {
            if next_is_whitespace_or_end(text, idx + ch.len_utf8(), iter) {
                Some((Pause::sentence(360), ch.len_utf8()))
            } else {
                None
            }
        }
        _ => None,
    }
}

fn next_is_whitespace_or_end<I>(
    text: &str,
    next_index: usize,
    iter: &std::iter::Peekable<I>,
) -> bool
where
    I: Iterator<Item = (usize, char)>,
{
    if next_index >= text.len() {
        return true;
    }
    if let Some(&(_, next_ch)) = iter.peek() {
        next_ch.is_whitespace()
    } else {
        false
    }
}

fn skip_whitespace<I>(text: &str, last_index: &mut usize, iter: &mut std::iter::Peekable<I>)
where
    I: Iterator<Item = (usize, char)>,
{
    while let Some(&(peek_idx, peek_ch)) = iter.peek() {
        if peek_ch.is_whitespace() {
            *last_index = peek_idx + peek_ch.len_utf8();
            iter.next();
        } else {
            break;
        }
    }
}

fn make_text_segment(slice: &str) -> Option<Segment> {
    let trimmed = slice.trim_matches(|c: char| c.is_whitespace());
    if trimmed.is_empty() {
        None
    } else {
        Some(Segment::Text(trimmed.to_string()))
    }
}

fn render_segments(segments: &[Segment]) -> String {
    let mut content = String::new();
    for segment in segments {
        match segment {
            Segment::Text(text) => content.push_str(&escape_text(text)),
            Segment::Break(pause) => {
                fmt::write(
                    &mut content,
                    format_args!("<break time=\"{}ms\"/>", pause.duration_ms),
                )
                .ok();
            }
            Segment::Emphasis { level, children } => {
                if children.is_empty() {
                    continue;
                }
                content.push_str(&format!(
                    "<emphasis level=\"{}\">{}</emphasis>",
                    level.as_str(),
                    render_segments(children)
                ));
            }
        }
    }

    format!("<speak xml:lang=\"es-ES\"><p>{content}</p></speak>")
}

fn escape_text(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&apos;"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_basic_paragraph_to_ssml() {
        let ssml = render_paragraph("Hola mundo. ¿Cómo estás?").unwrap();
        assert!(ssml.contains("<speak"));
        assert!(ssml.contains("Hola mundo."));
        assert!(ssml.contains("¿Cómo estás?"));
        assert!(ssml.contains("<break"));
    }

    #[test]
    fn segments_nested_emphasis() {
        let segments = segment_paragraph("**Muy *importante* aviso.**").unwrap();
        assert!(matches!(
            segments[0],
            Segment::Emphasis {
                level: EmphasisLevel::Strong,
                ..
            }
        ));
        let rendered = render_segments(&segments);
        assert!(rendered.contains("<emphasis level=\"strong\">"));
        assert!(rendered.contains("<emphasis level=\"moderate\">importante"));
    }

    #[test]
    fn parses_explicit_pause_directive() {
        let segments = segment_paragraph("Respira [pause:long] ahora.").unwrap();
        assert!(segments.iter().any(|segment| matches!(
            segment,
            Segment::Break(Pause {
                duration_ms: 680,
                ..
            })
        )));
        let ssml = render_segments(&segments);
        assert!(ssml.contains("<break time=\"680ms\"/>"));
    }

    #[test]
    fn rejects_invalid_pause() {
        let error = segment_paragraph("Esto [pause:foo] falla").unwrap_err();
        assert_eq!(error, SsmlError::InvalidPause("foo".into()))
    }

    #[test]
    fn gracefully_handles_unmatched_markers() {
        let segments = segment_paragraph("*Hola mundo").unwrap();
        assert!(render_segments(&segments).contains("Hola mundo"));
    }
}
