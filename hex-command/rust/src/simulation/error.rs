//! Typed failures from loading, planning, recording, replay and worker execution.
use std::{error::Error, fmt, io};
#[derive(Debug)]
/// Native harness failure with an underlying cause.
pub enum SimulationError {
    InvalidConfig(String),
    Content(crate::data::ContentError),
    Planning(crate::data::CommandError),
    Io(io::Error),
    Json(serde_json::Error),
    Replay(String),
    Worker(String),
}
impl fmt::Display for SimulationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidConfig(message) | Self::Replay(message) | Self::Worker(message) => {
                f.write_str(message)
            }
            Self::Content(error) => write!(f, "{error}"),
            Self::Planning(error) => write!(f, "Policy command failed: {error}"),
            Self::Io(error) => write!(f, "{error}"),
            Self::Json(error) => write!(f, "Invalid JSON: {error}"),
        }
    }
}
impl Error for SimulationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Content(error) => Some(error),
            Self::Planning(error) => Some(error),
            Self::Io(error) => Some(error),
            Self::Json(error) => Some(error),
            _ => None,
        }
    }
}
impl From<io::Error> for SimulationError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}
impl From<serde_json::Error> for SimulationError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}
impl From<String> for SimulationError {
    fn from(message: String) -> Self {
        Self::InvalidConfig(message)
    }
}
impl From<&str> for SimulationError {
    fn from(message: &str) -> Self {
        Self::InvalidConfig(message.into())
    }
}

impl From<crate::data::ContentError> for SimulationError {
    fn from(error: crate::data::ContentError) -> Self {
        Self::Content(error)
    }
}
impl From<crate::data::CommandError> for SimulationError {
    fn from(error: crate::data::CommandError) -> Self {
        Self::Planning(error)
    }
}
