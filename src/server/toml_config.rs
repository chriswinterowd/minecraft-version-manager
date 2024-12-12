use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct VersionConfig {
    pub vanilla: String,
    pub paper: String
}