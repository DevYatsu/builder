use thiserror::Error;

#[derive(Error, Debug)]
pub enum YbuildError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Shell error: {0}")]
    Shell(#[from] xshell::Error),

    #[error("Config error: {0}")]
    Config(String),

    #[error("JSON error: {0}")]
    Json(String),

    #[error("Build error: {0}")]
    Build(String),

    #[error("No supported build system found in this directory")]
    NoBuildSystem,

    #[error("Selection canceled by user")]
    SelectionCanceled,

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, YbuildError>;
