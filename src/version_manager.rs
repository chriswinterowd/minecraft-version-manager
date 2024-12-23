//! Handles version management for Minecraft servers.
//! It provides utilities for retrieving and processing server versions.

use crate::server::vanilla::{VanillaDownloadLink, Latest, VersionDownloads, VanillaVersions};
use crate::server::paper::{PaperVersions, PaperVersion, PaperVersionBuilds, PaperDownloadLink};
use crate::server::server_types::ServerType;
use crate::server::toml_config::VersionConfig;
use anyhow::{anyhow, Context, Result};
use futures_util::stream::StreamExt;
use reqwest;
use std::path::PathBuf;
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;
use toml;

/// Fetches the download link for a specified version of the given type of minecraft server
///
/// # Arguments
/// - `version_to_find`: A reference to the version string to fetch the download link
/// - `server_type`: The type of server for the requested download link
///
/// # Returns
/// A `Result` containing the download link as a String if successful
pub async fn get_version_download(version_to_find: &str, server_type: &ServerType) -> Result<String> {
    match server_type {
        ServerType::Vanilla => get_vanilla_download_url(&version_to_find).await,
        ServerType::Paper => get_paper_download_url(&version_to_find).await,
    }
}

/// Returns a Result containing the latest vanilla server version as a String if successful
///
/// # Returns
/// A `Result` containing the vanilla Minecraft version as a String if successful
pub async fn get_latest_vanilla_version() -> Result<Latest> {
    let response = reqwest::get("https://launchermeta.mojang.com/mc/game/version_manifest.json")
        .await
        .context("Error fetching the latest vanilla version")?
        .json::<VanillaVersions>()
        .await
        .context("Failed to parse the latest vanilla version JSON")?;
    Ok(response.latest)
}

/// Fetches the download link for a specific vanilla Minecraft server given the version
///
/// If the version to find is "latest", it retrieves the most recent version automatically
///
///# Arguments
/// - `version_to_find`: A reference to the version string to fetch the download link
///
/// # Returns
/// A `Result` containing the download link for the vanilla Minecraft server version as a String if successful
pub async fn get_vanilla_download_url(version_to_find: &str) -> Result<VanillaDownloadLink> {
    let version_id = if version_to_find == "latest" {
        let latest_version = get_latest_vanilla_version()
            .await
            .context("Failed to get the latest version")?;
        latest_version.release
    } else {
        version_to_find.to_string()
    };

    let versions_list = reqwest::get("https://launchermeta.mojang.com/mc/game/version_manifest.json")
        .await
        .context("Failed to fetch the version manifest")?
        .json::<VanillaVersions>()
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

        let download_url = version_info.downloads.server.url;

        return Ok(download_url)
    }

    Err(anyhow!("Version {} not found!", &version_id))
}


/// Returns a Result containing the latest Paper server version as a String if successful
pub async fn get_latest_paper_version() -> Result<PaperVersion> {
    let mut response = reqwest::get("https://api.papermc.io/v2/projects/paper")
        .await
        .context("Error fetching the latest paper version")?
        .json::<PaperVersions>()
        .await
        .context("Failed to parse the latest paper version JSON")?;

    if let Some(latest) = response.versions.pop() {
        Ok(latest)
    } else {
        Err(anyhow!("Failed to retrieve the latest paper version from array."))
    }

}

/// Fetches the download link for a specific Paper Minecraft server given the version
/// If the version to find is "latest", it retrieves the most recent version automatically
///
/// # Arguments
/// - `version_to_find`: A reference to the version string to fetch the download link
///
/// # Returns
/// A `Result` containing the download link for the vanilla Minecraft server version as a String if successful
pub async fn get_paper_download_url(version_to_find: &str) -> Result<PaperDownloadLink> {
    let version_id = if version_to_find == "latest" {
        let latest_version = get_latest_paper_version()
            .await?;
        latest_version
    } else {
        version_to_find.to_string()
    };

    let response = reqwest::get(format!("https://api.papermc.io/v2/projects/paper/versions/{}", version_id))
        .await
        .context("Version not found!")?
        .json::<PaperVersionBuilds>()
        .await
        .context("Failed to parse the paper version builds JSON")?;

    if let Some(latest_build) = response.builds.last() {
        let jar_name = format!("paper-{}-{}.jar", version_id, latest_build);
        let download_url = format!("https://api.papermc.io/v2/projects/paper/versions/{}/builds/{}/downloads/{}", version_id, latest_build, jar_name);
        Ok(download_url)
    } else {
        Err(anyhow!(format!("Failed to reteive the latest build for paper version {}", version_id)))
    }


}

/// Retrieves the path to the specified server version's 'server.jar' file
/// If the version is set to "recent," it fetches the version from the config file.
///
/// # Arguments
/// - `version_to_find`: A reference to the version string to fetch the download link
/// - `server_type`: The type of server for the requested version
/// - `path`: The root directory of server installations
/// # Returns
/// A result containing the path as a String if successful
pub async fn get_version(version_to_find: &str, server_type: &ServerType, path: &PathBuf) -> Result<String> {
    let mvm_dir = path;
    let config_path = mvm_dir.join("config.toml");
    if !config_path.exists() {
        return Err(anyhow!(format!("No version has been set! path: {:?}", config_path)));
    }

    let version = if version_to_find == "recent" {
        let toml_content = fs::read_to_string(config_path)
            .await
            .context("Failed to read config.toml")?;
        let version_config = toml::from_str::<VersionConfig>(&toml_content)
            .context("Failed to deserialize version config")?;
        match server_type {
            ServerType::Vanilla => version_config.vanilla,
            ServerType::Paper => version_config.paper
        }
    } else {
        version_to_find.to_string()
    };

    let server_type_dir = mvm_dir.join(server_type.to_string());
    let version_path = server_type_dir.join("versions").join(&version).join("server.jar");

    if !version_path.exists() {
        return Err(anyhow!("Version '{}' not found", &version));
    }

    let path_str = version_path.to_str().ok_or_else(|| anyhow!("Invalid path"))?.to_string();

    Ok(path_str)

}


///Fetches the download link for a specific vanilla Minecraft server given the version
///
/// If the version to find is "latest", it retrieves the most recent version automatically
///
///# Arguments
/// - `file_url`: Represents the URL from where the JAR file should be downloaded from
/// - `version`: A reference to the version of the minecraft server that it is downloading.
/// - `server_type`: The type of server for the requested version
/// - `path`: The root directory of server installations

pub async fn download_server_jar(file_url: String, version_to_download: &str, server_type: &ServerType, path: &PathBuf) -> Result<()> {
    let response = reqwest::get(&file_url)
        .await
        .context(format!("Failed to send request to download server jar! Download link: {}", &file_url))?;

    let version = if version_to_download == "latest" {
        let latest_version = get_latest_paper_version()
            .await?;
        latest_version
    } else {
        version_to_download.to_string()
    };

    let mvm_dir = path;

    let server_type_dir = mvm_dir.join(server_type.to_string());
    let version_dir = server_type_dir.join("versions").join(version);

    if !version_dir.exists() {
        fs::create_dir_all(&version_dir)
            .await
            .context(format!("Failed to create directory for the version, path: {:?}", &version_dir))?;
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


/// Deletes the server JAR for a specified Minecraft server version
///
/// # Arguments
/// - `version`: A reference to the version of the minecraft server to delete
/// - `server_type`: The type of server
/// - `path`: The root directory of server installations

pub async fn delete_server_jar(version: &str, server_type: &ServerType, path: &PathBuf) -> Result<()> {
    let mvm_dir = path;
    let server_type_dir = mvm_dir.join(server_type.to_string());
    let version_dir = server_type_dir.join("versions").join(version);

    if !version_dir.exists() {
        return Err(anyhow!("Version not found"));
    }

    fs::remove_dir_all(version_dir)
        .await
        .context(format!("Failed to delete version {}", version))?;

    println!("Version {} successfully deleted", version);

    Ok(())
}

/// Sets the specified version of the Minecraft server as the current version.
/// If the version to find is "latest", it retrieves the most recent version automatically
///
/// # Arguments
/// - `version`: A reference to the version of the minecraft server to be set
/// - `server_type`: The type of server
/// - `path`: The root directory of server installations
///
/// # Notes
/// - If the server jar for the specified version does not exist, it is downloaded automatically.
/// - Updates the `config.toml` file to the new current version.
pub async fn use_version(version: &str, server_type: &ServerType, path: &PathBuf) -> Result<()> {
    let mvm_dir = path;
    let server_type_dir = mvm_dir.join(server_type.to_string());
    let version_dir = server_type_dir.join("versions").join(version);
    let server_jar_path = version_dir.join("server.jar");

    if !server_jar_path.exists() {
        let download_info = get_version_download(version, server_type)
           .await?;
        println!("Found version, downloading...");
        download_server_jar(download_info, version, server_type, &mvm_dir)
            .await
            .context("Failed to download server jar")?;
    }

    let config_path = mvm_dir.join("config.toml");

    let mut versions = if config_path.exists() {
        let toml_content = fs::read_to_string(&config_path)
            .await
            .context("Failed to read config.toml")?;
        toml::from_str::<VersionConfig>(&toml_content)
            .context("Failed to deserialize version config")?
    } else {
        VersionConfig {
            vanilla: "".to_string(),
            paper: "".to_string()
        }
    };

    match server_type {
        ServerType::Vanilla => versions.vanilla = version.to_string(),
        ServerType::Paper => versions.paper = version.to_string()
    }

    let toml_string = toml::to_string_pretty(&versions)
        .context("Failed to serialize version config")?;

    fs::write(&config_path, toml_string)
        .await
        .context("Failed to write to config.toml file")?;

    println!("Now using version: {}", version);
    Ok(())
}



