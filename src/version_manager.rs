use reqwest;
use crate::models::{VersionDownloads, Versions};
use crate::models::Latest;
use crate::models::DownloadLink;
use dirs::home_dir;
use tokio::fs::{self, File};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use futures_util::stream::StreamExt;
use anyhow::{anyhow, Context, Result};

pub async fn get_latest_version() -> Result<Latest> {
    let response = reqwest::get("https://launchermeta.mojang.com/mc/game/version_manifest.json")
        .await
        .context("Error fetching the latest version")?
        .json::<Versions>()
        .await
        .context("Failed to parse the latest version JSON")?;
    Ok(response.latest)
}

pub async fn get_version_download(version_to_find: &str) -> Result<Option<DownloadLink>> {
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
        .context("Failed to parse the version manifest JSON")?;

    let find_version = versions_list.versions.into_iter().find(|version| version.id == version_id);

    if let Some(find_version) = find_version {
        let version_info = reqwest::get(find_version.url)
            .await
            .context("Failed to fetch the version details")?
            .json::<VersionDownloads>()
            .await
            .context("Failed to parse version details JSON")?;

        let download_url = version_info.downloads.server;

        return Ok(Some(download_url))
    }

    Ok(None)
}

pub async fn get_version(version: &str) -> Result<String> {
    let home_dir = home_dir().ok_or_else(|| anyhow!("Could not find home directory"))?;

    let version_to_get = if version == "latest" {
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

pub async fn download_server_jar(file_url: String, version: &str) -> Result<()> {
    let response = reqwest::get(file_url)
        .await
        .context("Failed to send request to download server jar")?;

    let home_dir = home_dir().ok_or_else(|| anyhow!("Could not find home directory"))?;

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

pub async fn use_version(version: &str) -> Result<()> {
    let home_dir = home_dir().ok_or_else(|| anyhow!("Could not find home directory"))?;
    let version_path = home_dir.join(".mvm").join("versions").join(version).join("server.jar");

    if !version_path.exists() {
        let download_info = get_version_download(version)
           .await?
           .ok_or_else(|| anyhow!("Version '{}' not found", version))?;
        println!("Found version, downloading...");
        download_server_jar(download_info.url, version)
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