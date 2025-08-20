//! Error module: Defines custom error types for the P2P Chat application.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ChatError {
    #[error("Network error: {0}")]
    Network(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl From<std::io::Error> for ChatError {
    fn from(e: std::io::Error) -> Self {
        ChatError::Network(e.to_string())
    }
}

impl From<serde_json::Error> for ChatError {
    fn from(e: serde_json::Error) -> Self {
        ChatError::Serialization(e.to_string())
    }
}
