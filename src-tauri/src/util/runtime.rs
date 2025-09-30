use std::path::PathBuf;

pub fn runtime_dir() -> PathBuf {
    if let Ok(path) = std::env::var("READER_RUNTIME_DIR") {
        let candidate = PathBuf::from(path);
        if candidate.is_absolute() || candidate.components().next().is_some() {
            return candidate;
        }
    }
    PathBuf::from("runtime")
}
