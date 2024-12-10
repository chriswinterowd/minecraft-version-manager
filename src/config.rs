use std::env;
use std::path::PathBuf;
use anyhow::anyhow;
use dirs::home_dir;

pub async fn initialize_home_dir() -> anyhow::Result<()> {
    if let Some(home_dir) = home_dir() {
        env::set_var("HOME_DIR", &home_dir);
        Ok(())
    } else {
        Err(anyhow!("Failed to retrieve the user's home directory."))
    }
}

pub async fn get_home_dir() -> anyhow::Result<PathBuf> {
    let home_dir = env::var("HOME_DIR")
        .map(PathBuf::from)
        .map_err(|err| anyhow!("HOME_DIR variable is not set: {}", err))?;
    Ok(home_dir)
}