use std::path::{Path, PathBuf};

pub fn with_extension(path: impl AsRef<Path>, extension: &str) -> PathBuf {
    path.as_ref().with_extension(extension)
}
