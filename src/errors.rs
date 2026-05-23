use std::path::PathBuf;

use thiserror::Error;

pub type Result<T> = std::result::Result<T, ViewBsError>;

#[derive(Debug, Error)]
pub enum ViewBsError {
    #[error("invalid input {path}: {message}")]
    InvalidInput { path: PathBuf, message: String },

    #[error("parse error in {path} at line {line:?}: {message}")]
    ParseError {
        path: PathBuf,
        line: Option<u64>,
        message: String,
    },

    #[error("indexed query error in {path} for {region}: {message}")]
    IndexedQueryError {
        path: PathBuf,
        region: String,
        message: String,
    },

    #[error("plot error: {message}")]
    PlotError { message: String },

    #[error("I/O error for {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("CSV/TSV error for {path}: {source}")]
    Csv {
        path: PathBuf,
        #[source]
        source: csv::Error,
    },

    #[error("{command} is not implemented in the Rust rewrite yet")]
    NotImplemented { command: String },

    #[error("operation cancelled")]
    Cancelled,
}

impl ViewBsError {
    pub fn invalid_input(path: impl Into<PathBuf>, message: impl Into<String>) -> Self {
        Self::InvalidInput {
            path: path.into(),
            message: message.into(),
        }
    }

    pub fn parse_error(
        path: impl Into<PathBuf>,
        line: Option<u64>,
        message: impl Into<String>,
    ) -> Self {
        Self::ParseError {
            path: path.into(),
            line,
            message: message.into(),
        }
    }

    pub fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::Io {
            path: path.into(),
            source,
        }
    }

    pub fn csv(path: impl Into<PathBuf>, source: csv::Error) -> Self {
        Self::Csv {
            path: path.into(),
            source,
        }
    }
}
