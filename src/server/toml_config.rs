//! This submodule is primarily used to serialize and deserialize configuration data stored in a TOML file.

use serde::{Deserialize, Serialize};


/// Represents the version configuration for Minecraft servers.
///
/// # Fields
/// - `vanilla`: A `String` representing the current Vanilla server version.
/// - `paper`: A `String` representing the current Paper server version.
#[derive(Debug, Deserialize, Serialize)]
pub struct VersionConfig {
    pub vanilla: String,
    pub paper: String
}