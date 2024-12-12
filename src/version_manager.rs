use crate::config::get_dir;
use crate::server::vanilla::{VanillaDownloadLink, Latest, VersionDownloads, VanillaVersions};
use crate::server::paper::{PaperVersions, PaperVersion, PaperVersionBuilds, PaperDownloadLink};
use crate::server::server_types::ServerType;
use crate::server::toml_config::VersionConfig;
use anyhow::{anyhow, Context, Result};
use futures_util::stream::StreamExt;
use reqwest;
use std::path::PathBuf;
use tokio::fs::{self, File};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use toml;

pub async fn get_version_download(version_to_find: &str, server_type: &ServerType) -> Result<String> {
    match server_type {
        ServerType::Vanilla => get_vanilla_download_url(&version_to_find).await,
        ServerType::Paper => get_paper_download_url(&version_to_find).await,
    }
}

pub async fn get_latest_vanilla_version() -> Result<Latest> {
    let response = reqwest::get("https://launchermeta.mojang.com/mc/game/version_manifest.json")
        .await
        .context("Error fetching the latest vanilla version")?
        .json::<VanillaVersions>()
        .await
        .context("Failed to parse the latest vanilla version JSON")?;
    Ok(response.latest)
}

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

pub async fn get_version(version: &str, server_type: &ServerType, path: &PathBuf) -> Result<String> {
    let mvm_dir = path;
    let config_path = mvm_dir.join("config.toml");
    if !config_path.exists() {
        return Err(anyhow!(format!("No version has been set! path: {:?}", config_path)));
    }

    let version_to_get = if version == "recent" {
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
        version.to_string()
    };

    let server_type_dir = mvm_dir.join(server_type.get_server_path());
    let version_path = server_type_dir.join("versions").join(&version_to_get).join("server.jar");

    if !version_path.exists() {
        return Err(anyhow!("Version '{}' not found", &version_to_get));
    }

    let path_str = version_path.to_str().ok_or_else(|| anyhow!("Invalid path"))?.to_string();

    Ok(path_str)

}

pub async fn download_server_jar(file_url: String, version: &str, server_type: &ServerType, path: &PathBuf) -> Result<()> {
    let response = reqwest::get(&file_url)
        .await
        .context(format!("Failed to send request to download server jar! Download link: {}", &file_url))?;

    let mvm_dir = path;

    let server_type_dir = mvm_dir.join(server_type.get_server_path());
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

pub async fn delete_server_jar(version: &str, server_type: &ServerType, path: &PathBuf) -> Result<()> {
    let mvm_dir = path;
    let server_type_dir = mvm_dir.join(server_type.get_server_path());
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

pub async fn use_version(version: &str, server_type: &ServerType, path: &PathBuf) -> Result<()> {
    let mvm_dir = path;
    let server_type_dir = mvm_dir.join(server_type.get_server_path());
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


#[cfg(test)]
mod tests {
    use std::env;
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
        let test_home_dir= PathBuf::from("./test_data/.mvm");

        env::set_var("MVM_HOME", &test_home_dir);

        let result = get_version("1.21", &get_dir().await?).await;

        if let Err(ref err) = result {
            eprintln!("Test failed with error: {:?}", err);
        }

        assert!(result.is_ok());

        Ok(())
    }

    #[tokio::test]
    async fn test_get_version_recent() -> Result<()> {
        let test_home_dir = PathBuf::from("./test_data/.mvm");

        env::set_var("MVM_HOME", &test_home_dir);

        let result = get_version("recent", &get_dir().await?).await;

        if let Err(ref err) = result {
            eprintln!("Test failed with error: {:?}", err);
        }

        assert!(result.is_ok());

        Ok(())
    }

    #[tokio::test]
    async fn test_get_version_nonexistent_version() -> Result<()> {
        let test_home_dir= PathBuf::from("./test_data/.mvm");

        env::set_var("MVM_HOME", &test_home_dir);

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
        let test_home_dir = PathBuf::from("./test_data/.mvm");

        env::set_var("MVM_HOME", &test_home_dir);

        let version = "1.20.2";
        let download_info = get_version_download(&version).await?;

        let result = download_server_jar(download_info.url, version, &get_dir().await?).await;

        if let Err(ref err) = result {
            eprintln!("Test failed with error: {:?}", err);
        }

        assert!(result.is_ok(), "Failed to download server jar!");

        let downloaded_file = test_home_dir.join("versions/1.20.2/server.jar");
        assert!(
            downloaded_file.exists(),
            "Server jar was not downloaded to the expected location!"
        );

        tokio::fs::remove_dir_all(test_home_dir.join("versions/1.20.2"))
            .await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_delete_server_jar() -> Result<()> {
        let test_home_dir = PathBuf::from("./test_data/.mvm");
        env::set_var("MVM_HOME", &test_home_dir);

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
        let test_home_dir = PathBuf::from("./test_data/.mvm");

        env::set_var("MVM_HOME", &test_home_dir);

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
        let test_home_dir = PathBuf::from("./test_data/.mvm");

        env::set_var("MVM_HOME", &test_home_dir);

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

        let config_path = test_home_dir.join("config.txt");
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
        let test_home_dir = PathBuf::from("./test_data/.mvm");

        env::set_var("MVM_HOME", &test_home_dir);

        let version = "nonexistent version";

        let result = use_version(version, &get_dir().await?).await;

        if let Err(ref err) = result {
            eprintln!("Test failed with error: {:?}", err);
        }

        assert!(result.is_err(), "Expected an error for a nonexistent version");

        Ok(())
    }

}

