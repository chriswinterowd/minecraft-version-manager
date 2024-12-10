use crate::config::{get_dir, set_dir};
use crate::models::DownloadLink;
use crate::models::Latest;
use crate::models::{VersionDownloads, Versions};
use anyhow::{anyhow, Context, Result};
use futures_util::stream::StreamExt;
use reqwest;
use std::path::PathBuf;
use tokio::fs::{self, File};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

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
    let mvm_dir = path;
    let version_to_get = if version == "recent" {
        let config_path = mvm_dir.join(".mvm").join("config.txt");
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

    let version_path = mvm_dir.join(".mvm").join("versions").join(&version_to_get).join("server.jar");

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

    let mvm_dir = path;
    let version_dir = mvm_dir.join(".mvm/versions/").join(version);

    if !version_dir.exists() {
        fs::create_dir_all(&version_dir)
            .await
            .context("Failed to create directory for the version")?;
    }

    let server_jar_path = version_dir.join("server.jar");

    if response.status().is_success() {
        let mut file = File::create(&server_jar_path)
            .await
            .context("Failed to create server.jar file")?;
        let mut stream = response.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.context("Failed to read chunk from response")?;
            file.write_all(&chunk)
                .await
                .context("Failed to write chunk for server.jar file")?;
        }

        println!("File downloaded to {:?}", &server_jar_path);
    }
    Ok(())
}

pub async fn delete_server_jar(version: &str, path: &PathBuf) -> Result<()> {
    let mvm_dir = path;

    let version_dir = mvm_dir.join(".mvm/versions/").join(version);

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
    let mvm_dir = path;
    let server_jar_path = mvm_dir.join(".mvm").join("versions").join(version).join("server.jar");

    if !server_jar_path.exists() {
        let download_info = get_version_download(version)
           .await?;
        println!("Found version, downloading...");
        download_server_jar(download_info.url, version, &mvm_dir)
            .await
            .context("Failed to download server jar")?;
    }

    let config_path = mvm_dir.join(".mvm").join("config.txt");

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
    async fn test_get_version() -> Result<()> {
        let test_home_dir= PathBuf::from("./test_data");

        set_dir(Some(&test_home_dir)).await?;

        let result = get_version("1.21", &get_dir().await?).await;

        if let Err(ref err) = result {
            eprintln!("Test failed with error: {:?}", err);
        }

        assert!(result.is_ok());

        Ok(())
    }

    #[tokio::test]
    async fn test_get_version_recent() -> Result<()> {
        let test_home_dir = PathBuf::from("./test_data");

        set_dir(Some(&test_home_dir)).await?;

        let result = get_version("recent", &get_dir().await?).await;

        if let Err(ref err) = result {
            eprintln!("Test failed with error: {:?}", err);
        }

        assert!(result.is_ok());

        Ok(())
    }

    #[tokio::test]
    async fn test_get_version_nonexistent_version() -> Result<()> {
        let test_home_dir= PathBuf::from("./test_data");

        set_dir(Some(&test_home_dir)).await?;

        let version = "nonexistent version";
        let result = get_version(version, &get_dir().await?).await;

        if let Err(ref err) = result {
            eprintln!("Test failed with error: {:?}", err);
        }

        assert!(result.is_err(), "Expected an error for a nonexistent version");

        Ok(())
    }

    #[tokio::test]
    async fn test_download_server_jar() -> Result<()> {
        let test_home_dir = PathBuf::from("./test_data");

        set_dir(Some(&test_home_dir)).await?;

        let version = "1.20.2";
        let download_info = get_version_download(&version).await?;

        let result = download_server_jar(download_info.url, version, &get_dir().await?).await;

        if let Err(ref err) = result {
            eprintln!("Test failed with error: {:?}", err);
        }

        assert!(result.is_ok(), "Failed to download server jar!");

        let downloaded_file = test_home_dir.join(".mvm/versions/1.20.2/server.jar");
        assert!(
            downloaded_file.exists(),
            "Server jar was not downloaded to the expected location!"
        );

        tokio::fs::remove_dir_all(test_home_dir.join(".mvm/versions/1.20.2"))
            .await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_delete_server_jar() -> Result<()> {
        let test_home_dir = PathBuf::from("./test_data");
        set_dir(Some(&test_home_dir)).await?;

        let test_dir = PathBuf::from("./test_data/.mvm/versions/1.17");
        let test_file = test_dir.join("server.jar");

        fs::create_dir_all(&test_dir).await?;
        fs::write(&test_file, "dummy content").await?;

        assert!(test_dir.exists());
        assert!(test_file.exists());

        let version = "1.17";

        let result = delete_server_jar(version, &get_dir().await?).await;

        if let Err(ref err) = result {
            eprintln!("Test failed with error: {:?}", err);
        }

        assert!(!test_dir.exists(), "Test directory was not deleted");

        Ok(())
    }

    #[tokio::test]
    async fn test_delete_server_jar_nonexistent_version() -> Result<()> {
        let test_home_dir = PathBuf::from("./test_data");

        set_dir(Some(&test_home_dir)).await?;

        let version = "nonexistent version";

        let result = delete_server_jar(version, &get_dir().await?).await;

        if let Err(ref err) = result {
            eprintln!("Test failed with error: {:?}", err);
        }

        assert!(result.is_err(), "Expected an error for a nonexistent version");

        Ok(())
    }

    #[tokio::test]
    async fn test_use_version() -> Result<()> {
        let test_home_dir = PathBuf::from("./test_data");

        set_dir(Some(&test_home_dir)).await?;

        let test_dir = PathBuf::from("./test_data/.mvm/versions/1.17");
        let test_file = test_dir.join("server.jar");

        let version = "1.17";

        let result = use_version(version, &get_dir().await?).await;

        if let Err(ref err) = result {
            eprintln!("Test failed with error: {:?}", err);
        }

        assert!(result.is_ok());

        assert!(test_file.exists());

        tokio::fs::remove_dir_all(test_dir).await?;

        let config_path = test_home_dir.join(".mvm").join("config.txt");
        let contents = tokio::fs::read_to_string(&config_path)
            .await
            .expect("Failed to read config.txt");
        assert_eq!(contents, version);

        // set the test environment's config version back to the default value
        let test_version = "1.21";
        let mut file = File::create(&config_path)
            .await
            .context("Failed to create config.txt file")?;
        file.write_all(test_version.as_bytes())
            .await
            .context("Failed to write to config.txt")?;

        Ok(())
    }

    #[tokio::test]
    async fn test_use_version_nonexistent_version() -> Result<()> {
        let test_home_dir = PathBuf::from("./test_data");

        set_dir(Some(&test_home_dir)).await?;

        let version = "nonexistent version";

        let result = use_version(version, &get_dir().await?).await;

        if let Err(ref err) = result {
            eprintln!("Test failed with error: {:?}", err);
        }

        assert!(result.is_err(), "Expected an error for a nonexistent version");

        Ok(())
    }

}

