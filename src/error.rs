use thiserror::Error;

#[derive(Error, Debug)]
pub enum BuildError {
    #[error("Build system execution failed: {0}")]
    Execution(#[from] xshell::Error),
    #[error("Directory not found: {0}")]
    NotFound(String),
    #[error("No supported build system found")]
    NoSystemFound,
    #[error("Interactive input failed: {0}")]
    Input(#[from] std::io::Error),
    #[error("Argument parsing failed: {0}")]
    Arguments(#[from] pico_args::Error),
}

pub type Result<T> = std::result::Result<T, BuildError>;
