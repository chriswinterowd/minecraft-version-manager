use std::env;
use std::path::PathBuf;
use anyhow::{anyhow, Result};
use dirs::home_dir;

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