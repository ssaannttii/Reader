pub mod audio_controls;
pub mod import_epub;
pub mod import_pdf;
pub mod import_text;
pub mod speak;
pub mod voices;

pub use audio_controls::{current_audio, export_audio, pause_audio, play_audio, stop_audio};
pub use import_epub::{import_epub, import_epub_command};
pub use import_pdf::{import_pdf, import_pdf_command};
pub use import_text::{import_text, ImportTextRequest};
pub use speak::{
    execute_synthesis, handle_audio_completion, CommandError, SpeakCommand, SpeakResponse,
};

pub use import_epub::ImportEpubRequest;
pub use import_pdf::ImportPdfRequest;
pub use speak::CommandFailure;
pub use voices::{VoiceError, VoiceInfo, VoiceLibrary};
