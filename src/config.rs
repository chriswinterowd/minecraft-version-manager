use std::env;
use std::path::PathBuf;
use anyhow::{anyhow, Result};
use dirs::home_dir;

pub async fn set_dir(path: Option<&PathBuf>) -> Result<()> {
    if let Some(path) = path {
        env::set_var("MVM_DIR", path);
        return Ok(())
    }

    if let Some(home_dir) = home_dir() {
        env::set_var("MVM_DIR", &home_dir);
        Ok(())
    } else {
        Err(anyhow!("Failed to retrieve the user's home directory"))
    }
}

pub async fn get_dir() -> anyhow::Result<PathBuf> {
    let mvm_dir = env::var("MVM_DIR")
        .map(PathBuf::from)
        .map_err(|err| anyhow!("HOME_DIR variable is not set: {}", err))?;
    Ok(mvm_dir)
}