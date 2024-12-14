use std::fmt::Display;
use anyhow::{anyhow, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerType {
    Vanilla,
    Paper
}

impl ServerType {
    pub fn from_string(server_type_string: String) -> Result<Self> {
        match server_type_string.as_str() {
            "paper" => Ok(ServerType::Paper),
            "vanilla" => Ok(ServerType::Vanilla),
            _ => Err(anyhow!("Invalid server type: {}", server_type_string)),
        }
    }
}

impl Display for ServerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            ServerType::Vanilla => "vanilla".to_string(),
            ServerType::Paper => "paper".to_string()
        };
        write!(f, "{}", str)
    }
}