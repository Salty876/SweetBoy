use std::fmt;

/// Errors that can occur during emulator operations.
#[derive(Debug)]
pub enum EmulatorError {
    /// The ROM data is invalid or unsupported.
    InvalidRom(String),
    /// Failed to serialize emulator state.
    SaveStateFailed(String),
    /// Failed to deserialize or restore emulator state.
    LoadStateFailed(String),
}

impl fmt::Display for EmulatorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EmulatorError::InvalidRom(msg) => write!(f, "Invalid ROM: {}", msg),
            EmulatorError::SaveStateFailed(msg) => write!(f, "Save state failed: {}", msg),
            EmulatorError::LoadStateFailed(msg) => write!(f, "Load state failed: {}", msg),
        }
    }
}

impl std::error::Error for EmulatorError {}
