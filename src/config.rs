//! This module provides functionality for retrieving the working directory of the Minecraft Version Manager
//!
//! It first checks for the `MVM_HOME` environment variable to determine the directory.
//! If the variable is not set or invalid, it defaults to using the user's home directory and appending `.mvm`.

use std::env;
use std::path::PathBuf;
use anyhow::{anyhow, Result};
use dirs::home_dir;

/// Retrieves the working directory for the Minecraft Version Manager (MVM).
/// It checks for the `MVM_HOME` environment variable to determine the directory.
/// If the variable is not set or invalid, it defaults to using the user's home directory and appending `.mvm`.
///
/// # Returns
/// A Result containing the path to the MVM working directory if successful
pub async fn get_dir() -> Result<PathBuf> {
    if let Ok(env_value) = env::var("MVM_HOME") {
        let mvm_dir = PathBuf::from(&env_value);
        if mvm_dir.exists() && mvm_dir.is_dir() {
           return Ok(mvm_dir)
        }

    }

    let Some(home_dir) = home_dir() else {
        return Err(anyhow!("Failed to retrieve the mvm directory"))
    };

    Ok(home_dir.join(".mvm"))

}