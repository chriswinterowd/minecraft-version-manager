//! The submodule includes utilities to parse a server type from a string and convert it to a displayable format.

use std::fmt::Display;
use anyhow::{anyhow, Result};

/// Represents the type of Minecraft server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerType {
    Vanilla,
    Paper
}

/// Converts a string into a `ServerType` enum.
///
///
/// # Arguments
/// - `server_type_string`: A `String` representing the type of server.
///
/// # Returns
/// A ServerType for the String if successful
impl ServerType {
    pub fn from_string(server_type_string: String) -> Result<Self> {
        match server_type_string.as_str() {
            "paper" => Ok(ServerType::Paper),
            "vanilla" => Ok(ServerType::Vanilla),
            _ => Err(anyhow!("Invalid server type: {}", server_type_string)),
        }
    }
}

/// Converts a `ServerType` into a displayable string.
impl Display for ServerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            ServerType::Vanilla => "vanilla".to_string(),
            ServerType::Paper => "paper".to_string()
        };
        write!(f, "{}", str)
    }
}