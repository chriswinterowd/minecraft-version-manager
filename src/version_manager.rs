use reqwest;
use crate::models::{VersionDownloads, Versions};
use crate::models::Latest;
use crate::models::DownloadLink;
use dirs::home_dir;
use tokio::fs::{self, File};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use futures_util::stream::StreamExt;
use anyhow::{anyhow, Context, Result};
use tempfile::tempdir;
use std::env;
use std::path::PathBuf;

pub async fn get_latest_version() -> Result<Latest> {
    let response = reqwest::get("https://launchermeta.mojang.com/mc/game/version_manifest.json")
        .await
        .context("Error fetching the latest version")?
        .json::<Versions>()
        .await
        .context("Failed to parse the latest version JSON")?;
    Ok(response.latest)
}

pub async fn get_version_download(version_to_find: &str) -> Result<DownloadLink> {
    let version_id = if version_to_find == "latest" {
        let latest_version = get_latest_version()
            .await
            .context("Failed to get the latest version")?;
        latest_version.release
    } else {
        version_to_find.to_string()
    };

    let versions_list = reqwest::get("https://launchermeta.mojang.com/mc/game/version_manifest.json")
        .await
        .context("Failed to fetch the version manifest")?
        .json::<Versions>()
        .await
        .context("Failed to retrieve versions json")?;

    let find_version = versions_list.versions.into_iter().find(|version| version.id == version_id);

    if let Some(find_version) = find_version {
        let version_info = reqwest::get(find_version.url)
            .await
            .context("Failed to fetch the version details")?
            .json::<VersionDownloads>()
            .await
            .context("Failed to parse version details JSON")?;

        let download_url = version_info.downloads.server;

        return Ok(download_url)
    }

    Err(anyhow!("Version {} not found!", &version_id))
}

pub async fn get_version(version: &str, path: &PathBuf) -> Result<String> {
    let home_dir = path;
    let version_to_get = if version == "recent" {
        let config_path = home_dir.join(".mvm").join("config.txt");
        let mut file = File::open(&config_path)
            .await
            .context(format!("Failed to open config file at{:?}", &config_path))?;

        let mut contents = String::new();

        file.read_to_string(&mut contents)
            .await
            .context("Failed to read contents of config file")?;
        contents
    } else {
        version.to_string()
    };

    let version_path = home_dir.join(".mvm").join("versions").join(&version_to_get).join("server.jar");

    if !version_path.exists() {
        return Err(anyhow!("Version '{}' not found", &version_to_get));
    }

    let path_str = version_path.to_str().ok_or_else(|| anyhow!("Invalid path"))?.to_string();

    Ok(path_str)

}

pub async fn download_server_jar(file_url: String, version: &str, path: &PathBuf) -> Result<()> {
    let response = reqwest::get(file_url)
        .await
        .context("Failed to send request to download server jar")?;

    let home_dir = path;

    let mvm_dir = home_dir.join(".mvm/versions/").join(version);

    if !mvm_dir.exists() {
        fs::create_dir_all(&mvm_dir)
            .await
            .context("Failed to create directory for the version")?;
    }

    let path = mvm_dir.join("server.jar");

    if response.status().is_success() {
        let mut file = File::create(&path)
            .await
            .context("Failed to create server.jar file")?;
        let mut stream = response.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.context("Failed to read chunk from response")?;
            file.write_all(&chunk)
                .await
                .context("Failed to write chunk for server.jar file")?;
        }

        println!("File downloaded to {:?}", &path);
    }
    Ok(())
}

pub async fn delete_server_jar(version: &str, path: &PathBuf) -> Result<()> {
    let home_dir = path;

    let version_dir = home_dir.join(".mvm/versions/").join(version);

    if !version_dir.exists() {
        return Err(anyhow!("Version not found"));
    }

    fs::remove_dir_all(version_dir)
        .await
        .context(format!("Failed to delete version {}", version))?;

    println!("Version {} successfully deleted", version);

    Ok(())
}

pub async fn use_version(version: &str, path: &PathBuf) -> Result<()> {
    let home_dir = path;
    let version_path = home_dir.join(".mvm").join("versions").join(version).join("server.jar");

    if !version_path.exists() {
        let download_info = get_version_download(version)
           .await?;
        println!("Found version, downloading...");
        download_server_jar(download_info.url, version, &home_dir)
            .await
            .context("Failed to download server jar")?;
    }

    let config_path = home_dir.join(".mvm").join("config.txt");

    let mut file = File::create(&config_path)
        .await
        .context("Failed to create config.txt file")?;
    file.write_all(version.as_bytes())
        .await
        .context("Failed to write to config.txt")?;

    println!("Now using version: {}", version);
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_latest_version() {
        let result = get_latest_version().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_version_download_latest() {
        let result = get_version_download("latest").await;
        assert!(result.is_ok(), "Expected to fetch the download link for the latest version");
    }

    #[tokio::test]
    async fn test_get_version_download_specific_version() {
        let result = get_version_download("1.21").await;
        assert!(result.is_ok(), "Expected to fetch the download link for version 1.21");
    }

    #[tokio::test]
    async fn test_get_version_download_nonexistent_version() {
        let result = get_version_download("nonexistent_version").await;
        assert!(result.is_err(), "Expected an error for a nonexistent version");
    }

    #[tokio::test]
    async fn test_get_version_with_temp_dir() -> Result<()> {
        // Create a temporary directory for testing
        let temp_dir = tempdir()?;
        let test_home_dir = temp_dir.path();

        // Set TEST_HOME_DIR environment variable
        env::set_var("TEST_HOME_DIR", test_home_dir);

        // Mock directory structure
        let version_dir = test_home_dir.join(".mvm/versions/1.21");
        fs::create_dir_all(&version_dir).await?;
        let server_jar_path = version_dir.join("server.jar");

        // Create a dummy "server.jar" file
        fs::write(&server_jar_path, "dummy content").await?;

        // Call the function
        let result = get_version("1.21").await;

        if let Err(ref err) = result {
            eprintln!("Test failed with error: {:?}", err);
        }

        // Assert the result
        assert!(result.is_ok());
        assert_eq!(&result?, server_jar_path.to_str().unwrap());

        // Clean up environment variable
        env::remove_var("TEST_HOME_DIR");

        Ok(())
    }

    #[tokio::test]
    async fn test_get_version_recent_with_temp_dir() -> Result<()> {
        let temp_dir = tempdir()?; // Temporary directory
        let test_home_dir = temp_dir.path();

        // Set TEST_HOME_DIR environment variable
        std::env::set_var("HOME_DIR", test_home_dir);

        // Create .mvm/config.txt and write the version
        let config_path = test_home_dir.join(".mvm/config.txt");
        eprintln!("Config path exists: {}", config_path.exists());
        tokio::fs::create_dir_all(config_path.parent().unwrap())
            .await
            .context("Failed to create config directory")?;
        tokio::fs::write(&config_path, "1.21")
            .await
            .context("Failed to write to config file")?;
        eprintln!("Config path exists: {}", config_path.exists());
        // Create the version directory and server.jar file
        let version_dir = test_home_dir.join(".mvm/versions/1.21");
        tokio::fs::create_dir_all(&version_dir)
            .await
            .context("Failed to create version directory")?;
        tokio::fs::write(version_dir.join("server.jar"), "dummy content")
            .await
            .context("Failed to write server.jar")?;

        // Call the function to test
        let result = get_version("recent").await;

        // Log the error if it exists
        if let Err(ref err) = result {
            eprintln!("Test failed with error: {:?}", err);
        }

        // Assert the result
        assert!(result.is_ok());
        assert_eq!(
            result?,
            version_dir.join("server.jar").to_str().unwrap()
        );

        // Clean up environment variable
        env::remove_var("TEST_HOME_DIR");

        Ok(())
    }


}