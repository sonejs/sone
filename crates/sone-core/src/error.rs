use std::fmt;

#[derive(Debug, thiserror::Error)]
pub enum SoneError {
    #[error("IR parse error at {pointer}: {message}")]
    Ir { pointer: String, message: String },
    #[error("asset error ({src}): {message}")]
    Asset { src: String, message: String },
    #[error("font error ({family}): {message}")]
    Font { family: String, message: String },
    #[error("layout error: {0}")]
    Layout(String),
    #[error("render error: {0}")]
    Render(String),
    #[error("unsupported: {0}")]
    Unsupported(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, SoneError>;

impl SoneError {
    pub fn ir(pointer: impl fmt::Display, message: impl fmt::Display) -> Self {
        SoneError::Ir {
            pointer: pointer.to_string(),
            message: message.to_string(),
        }
    }
    /// CLI exit code contract: 2 IR parse / 3 asset / 4 render.
    pub fn exit_code(&self) -> i32 {
        match self {
            SoneError::Ir { .. } => 2,
            SoneError::Asset { .. } | SoneError::Io(_) => 3,
            _ => 4,
        }
    }
}
