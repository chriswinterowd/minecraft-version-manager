use reqwest;
use std::error::Error;
use crate::models::{VersionDownloads, Versions};
use crate::models::Latest;
use crate::models::DownloadLink;
use dirs::home_dir;
use std::fs;
use std::fs::File;
use std::io::Write;
use futures_util::stream::StreamExt;

pub async fn get_latest_version() -> Result<Latest, Box<dyn Error>> {
    let response = reqwest::get("https://launchermeta.mojang.com/mc/game/version_manifest.json")
        .await?.json::<Versions>()
        .await?;
    Ok(response.latest)
}

pub async fn get_version_download(version_to_find: &str) -> Result<Option<DownloadLink>, Box<dyn Error>> {
    let version_id = if version_to_find == "latest" {
        let latest_version = get_latest_version().await?;
        latest_version.release
    } else {
        version_to_find.to_string()
    };

    let versions_list = reqwest::get("https://launchermeta.mojang.com/mc/game/version_manifest.json")
        .await?.json::<Versions>()
        .await?;

    let find_version = versions_list.versions.into_iter().find(|version| version.id == version_id);

    if let Some(find_version) = find_version {
        let version_info = reqwest::get(find_version.url)
            .await?.json::<VersionDownloads>()
            .await?;

        let download_url = version_info.downloads.server;

        return Ok(Some(download_url))
    }

    Ok(None)
}

pub async fn download_server_jar(file_url: String, version: &str) -> Result<(), Box<dyn Error>> {
    let response = reqwest::get(file_url).await?;

    let home_dir = home_dir().ok_or("Could not find home directory")?;

    let mvm_dir = home_dir.join(".mvm/versions/").join(version);

    if !mvm_dir.exists() {
        fs::create_dir_all(&mvm_dir)?;
    }

    let path = mvm_dir.join("server.jar");

    if response.status().is_success() {
        let mut file = File::create(&path)?;
        let mut stream = response.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk)?;
        }

        println!("File downloaded to {:?}", &path);

    }
    Ok(())
}