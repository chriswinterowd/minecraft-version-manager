use reqwest;
use std::error::Error;
use crate::models::{VersionDownloads, Versions};
use crate::models::Latest;
use crate::models::DownloadLink;
use dirs::home_dir;
use tokio::fs::{self, File};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
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

pub async fn get_version(version: &str) -> Result<String, Box<dyn Error>> {
    let home_dir = home_dir().ok_or("Could not find home directory")?;
    let version_to_get = if version == "latest" {
        let config_path = home_dir.join(".mvm").join("config.txt");
        let mut file = File::open(config_path).await?;

        let mut contents = String::new();

        file.read_to_string(&mut contents).await?;
        contents
    } else {
        version.to_string()
    };

    let version_path = home_dir.join(".mvm").join("versions").join(version_to_get).join("server.jar");

    if !version_path.exists() {
        return Err("Version not found".into());
    }

    let path_str = version_path.to_str().ok_or("Invalid path")?.to_string();

    Ok(path_str)

}

pub async fn download_server_jar(file_url: String, version: &str) -> Result<(), Box<dyn Error>> {
    let response = reqwest::get(file_url).await?;

    let home_dir = home_dir().ok_or("Could not find home directory")?;

    let mvm_dir = home_dir.join(".mvm/versions/").join(version);

    if !mvm_dir.exists() {
        fs::create_dir_all(&mvm_dir).await?;
    }

    let path = mvm_dir.join("server.jar");

    if response.status().is_success() {
        let mut file = File::create(&path).await?;
        let mut stream = response.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk).await?;
        }

        println!("File downloaded to {:?}", &path);
    }
    Ok(())
}

pub async fn use_version(version: &str) -> Result<(), Box<dyn Error>> {
    let home_dir = home_dir().ok_or("Could not find home directory")?;
    let version_path = home_dir.join(".mvm").join("versions").join(version).join("server.jar");

    if !version_path.exists() {
        match get_version_download(version).await {
            Ok(Some(download_info)) => {
                println!("Found version, downloading...");
                download_server_jar(download_info.url, version).await?;
            }
            Ok(None) => {
                return Err("Version not found".into());
            }
            Err(err) => {
                println!("Error: {}", err);
                return Err(err);
            }
        }
    }

    let config_path = home_dir.join(".mvm").join("config.txt");

    let mut file = File::create(&config_path).await?;
    file.write_all(version.as_bytes()).await?;

    println!("Now using version: {}", version);
    Ok(())
}