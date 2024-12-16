use mvm::server::server_types::ServerType;
use mvm::version_manager::*;
use mvm::config::{get_dir};
use anyhow::{Context, Result};
use std::path::PathBuf;
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;
use toml;


#[cfg(test)]
mod tests {
    use std::env;
    use toml::Value;
    use super::*;

    #[tokio::test]
    async fn test_get_version_download_vanilla_latest() {
        let result = get_version_download("latest", &ServerType::Vanilla).await;
        assert!(result.is_ok(), "Expected to fetch the download link for the latest version");
    }

    #[tokio::test]
    async fn test_get_version_download_vanilla_specific_version() {
        let result = get_version_download("1.21", &ServerType::Vanilla).await;
        assert!(result.is_ok(), "Expected to fetch the download link for version 1.21");
    }

    #[tokio::test]
    async fn test_get_version_download_vanilla_nonexistent_version() {
        let result = get_version_download("nonexistent_version", &ServerType::Vanilla).await;
        assert!(result.is_err(), "Expected an error for a nonexistent version");
    }

    #[tokio::test]
    async fn test_get_version_download_paper_latest() {
        let result = get_version_download("latest", &ServerType::Paper).await;
        assert!(result.is_ok(), "Expected to fetch the download link for the latest version");
    }

    #[tokio::test]
    async fn test_get_version_download_paper_specific_version() {
        let result = get_version_download("1.21", &ServerType::Paper).await;
        assert!(result.is_ok(), "Expected to fetch the download link for version 1.21");
    }

    #[tokio::test]
    async fn test_get_version_download_paper_nonexistent_version() {
        let result = get_version_download("nonexistent_version", &ServerType::Paper).await;
        assert!(result.is_err(), "Expected an error for a nonexistent version");
    }

    #[tokio::test]
    async fn test_get_version_vanilla() -> Result<()> {
        let test_home_dir= PathBuf::from("./tests/test_data/.mvm");

        env::set_var("MVM_HOME", &test_home_dir);

        let result = get_version("1.21", &ServerType::Vanilla, &get_dir().await?).await;

        if let Err(ref err) = result {
            eprintln!("Test failed with error: {:?}", err);
        }

        assert!(result.is_ok());

        Ok(())
    }

    #[tokio::test]
    async fn test_get_version_vanilla_recent() -> Result<()> {
        let test_home_dir = PathBuf::from("./tests/test_data/.mvm");

        env::set_var("MVM_HOME", &test_home_dir);

        let result = get_version("recent", &ServerType::Vanilla, &get_dir().await?).await;

        if let Err(ref err) = result {
            eprintln!("Test failed with error: {:?}", err);
        }

        assert!(result.is_ok());

        Ok(())
    }

    #[tokio::test]
    async fn test_get_version_vanilla_nonexistent_version() -> Result<()> {
        let test_home_dir= PathBuf::from("./tests/test_data/.mvm");

        env::set_var("MVM_HOME", &test_home_dir);

        let version = "nonexistent version";
        let result = get_version(version, &ServerType::Vanilla,&get_dir().await?).await;

        if let Err(ref err) = result {
            eprintln!("Test failed with error: {:?}", err);
        }

        assert!(result.is_err(), "Expected an error for a nonexistent version");

        Ok(())
    }

    #[tokio::test]
    async fn test_get_version_paper() -> Result<()> {
        let test_home_dir= PathBuf::from("./tests/test_data/.mvm");

        env::set_var("MVM_HOME", &test_home_dir);

        let result = get_version("1.21", &ServerType::Paper, &get_dir().await?).await;

        if let Err(ref err) = result {
            eprintln!("Test failed with error: {:?}", err);
        }

        assert!(result.is_ok());

        Ok(())
    }

    #[tokio::test]
    async fn test_get_version_paper_recent() -> Result<()> {
        let test_home_dir = PathBuf::from("./tests/test_data/.mvm");

        env::set_var("MVM_HOME", &test_home_dir);

        let result = get_version("recent", &ServerType::Paper, &get_dir().await?).await;

        if let Err(ref err) = result {
            eprintln!("Test failed with error: {:?}", err);
        }

        assert!(result.is_ok());

        Ok(())
    }

    #[tokio::test]
    async fn test_get_version_paper_nonexistent_version() -> Result<()> {
        let test_home_dir= PathBuf::from("./tests/test_data/.mvm");

        env::set_var("MVM_HOME", &test_home_dir);

        let version = "nonexistent version";
        let result = get_version(version, &ServerType::Paper,&get_dir().await?).await;

        if let Err(ref err) = result {
            eprintln!("Test failed with error: {:?}", err);
        }

        assert!(result.is_err(), "Expected an error for a nonexistent version");

        Ok(())
    }

    #[tokio::test]
    async fn test_download_vanilla_server_jar() -> Result<()> {
        let test_home_dir = PathBuf::from("./tests/test_data/.mvm");

        env::set_var("MVM_HOME", &test_home_dir);

        let version = "1.20.2";
        let download_url = get_version_download(&version, &ServerType::Vanilla).await?;

        let result = download_server_jar(download_url, version, &ServerType::Vanilla, &get_dir().await?).await;

        if let Err(ref err) = result {
            eprintln!("Test failed with error: {:?}", err);
        }

        assert!(result.is_ok(), "Failed to download server jar!");

        let downloaded_file = test_home_dir.join("vanilla/versions/1.20.2/server.jar");
        assert!(
            downloaded_file.exists(),
            "Server jar was not downloaded to the expected location!"
        );

        tokio::fs::remove_dir_all(test_home_dir.join("vanilla/versions/1.20.2"))
            .await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_download_paper_server_jar() -> Result<()> {
        let test_home_dir = PathBuf::from("./tests/test_data/.mvm");

        env::set_var("MVM_HOME", &test_home_dir);

        let version = "1.20.2";
        let download_url = get_version_download(&version, &ServerType::Paper).await?;

        let result = download_server_jar(download_url, version, &ServerType::Paper, &get_dir().await?).await;

        if let Err(ref err) = result {
            eprintln!("Test failed with error: {:?}", err);
        }

        assert!(result.is_ok(), "Failed to download server jar!");

        let downloaded_file = test_home_dir.join("paper/versions/1.20.2/server.jar");
        assert!(
            downloaded_file.exists(),
            "Server jar was not downloaded to the expected location!"
        );

        tokio::fs::remove_dir_all(test_home_dir.join("paper/versions/1.20.2"))
            .await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_delete_vanilla_server_jar() -> Result<()> {
        let test_home_dir = PathBuf::from("./tests/test_data/.mvm");
        env::set_var("MVM_HOME", &test_home_dir);

        let test_dir = PathBuf::from("./tests/test_data/.mvm/vanilla/versions/1.17");
        let test_file = test_dir.join("server.jar");

        fs::create_dir_all(&test_dir).await?;
        fs::write(&test_file, "dummy content").await?;

        assert!(test_dir.exists());
        assert!(test_file.exists());

        let version = "1.17";

        let result = delete_server_jar(version, &ServerType::Vanilla,&get_dir().await?).await;

        if let Err(ref err) = result {
            eprintln!("Test failed with error: {:?}", err);
        }

        assert!(!test_dir.exists(), "Test directory was not deleted");

        Ok(())
    }

    #[tokio::test]
    async fn test_delete_vanilla_server_jar_nonexistent_version() -> Result<()> {
        let test_home_dir = PathBuf::from("./tests/test_data/.mvm");

        env::set_var("MVM_HOME", &test_home_dir);

        let version = "nonexistent version";

        let result = delete_server_jar(version, &ServerType::Vanilla, &get_dir().await?).await;

        if let Err(ref err) = result {
            eprintln!("Test failed with error: {:?}", err);
        }

        assert!(result.is_err(), "Expected an error for a nonexistent version");

        Ok(())
    }

    #[tokio::test]
    async fn test_delete_paper_server_jar() -> Result<()> {
        let test_home_dir = PathBuf::from("./tests/test_data/.mvm");
        env::set_var("MVM_HOME", &test_home_dir);

        let test_dir = PathBuf::from("./tests/test_data/.mvm/paper/versions/1.17");
        let test_file = test_dir.join("server.jar");

        fs::create_dir_all(&test_dir).await?;
        fs::write(&test_file, "dummy content").await?;

        assert!(test_dir.exists());
        assert!(test_file.exists());

        let version = "1.17";

        let result = delete_server_jar(version, &ServerType::Paper,&get_dir().await?).await;

        if let Err(ref err) = result {
            eprintln!("Test failed with error: {:?}", err);
        }

        assert!(!test_dir.exists(), "Test directory was not deleted");

        Ok(())
    }

    #[tokio::test]
    async fn test_delete_paper_server_jar_nonexistent_version() -> Result<()> {
        let test_home_dir = PathBuf::from("./tests/test_data/.mvm");

        env::set_var("MVM_HOME", &test_home_dir);

        let version = "nonexistent version";

        let result = delete_server_jar(version, &ServerType::Paper, &get_dir().await?).await;

        if let Err(ref err) = result {
            eprintln!("Test failed with error: {:?}", err);
        }

        assert!(result.is_err(), "Expected an error for a nonexistent version");

        Ok(())
    }

    #[tokio::test]
    async fn test_use_vanilla_version() -> Result<()> {
        let test_home_dir = PathBuf::from("./tests/test_data/.mvm");

        env::set_var("MVM_HOME", &test_home_dir);

        let test_dir = PathBuf::from("./tests/test_data/.mvm/vanilla/versions/1.17");
        let test_file = test_dir.join("server.jar");

        let version = "1.17";

        let result = use_version(version, &ServerType::Vanilla, &get_dir().await?).await;

        if let Err(ref err) = result {
            eprintln!("Test failed with error: {:?}", err);
        }

        assert!(result.is_ok());

        assert!(test_file.exists());

        tokio::fs::remove_dir_all(test_dir).await?;

        let config_path = test_home_dir.join("config.toml");
        let contents = tokio::fs::read_to_string(&config_path)
            .await
            .expect("Failed to read config.txt");

        let config: Value = toml::from_str(&contents).expect("Failed to parse config.toml");

        assert_eq!(config["vanilla"].as_str(), Some(version));

        let toml_content = "vanilla = \"1.21\"\npaper = \"1.21\"\n";

        let mut file = File::create(&config_path)
            .await
            .context("Failed to create config.txt file")?;

        file.write_all(toml_content.as_bytes())
            .await
            .context("Failed to write to config.txt")?;

        Ok(())
    }

    #[tokio::test]
    async fn test_use_vanilla_version_nonexistent_version() -> Result<()> {
        let test_home_dir = PathBuf::from("./tests/test_data/.mvm");

        env::set_var("MVM_HOME", &test_home_dir);

        let version = "nonexistent version";

        let result = use_version(version, &ServerType::Vanilla,&get_dir().await?).await;

        if let Err(ref err) = result {
            eprintln!("Test failed with error: {:?}", err);
        }

        assert!(result.is_err(), "Expected an error for a nonexistent version");

        Ok(())
    }

    #[tokio::test]
    async fn test_use_paper_version() -> Result<()> {
        let test_home_dir = PathBuf::from("./tests/test_data/.mvm");

        env::set_var("MVM_HOME", &test_home_dir);

        let test_dir = PathBuf::from("./tests/test_data/.mvm/paper/versions/1.17");
        let test_file = test_dir.join("server.jar");

        let version = "1.17";

        let result = use_version(version, &ServerType::Paper, &get_dir().await?).await;

        if let Err(ref err) = result {
            eprintln!("Test failed with error: {:?}", err);
        }

        assert!(result.is_ok());

        assert!(test_file.exists());

        tokio::fs::remove_dir_all(test_dir).await?;

        let config_path = test_home_dir.join("config.toml");
        let contents = tokio::fs::read_to_string(&config_path)
            .await
            .expect("Failed to read config.txt");

        let config: Value = toml::from_str(&contents).expect("Failed to parse config.toml");

        assert_eq!(config["paper"].as_str(), Some(version));

        let toml_content = "vanilla = \"1.21\"\npaper = \"1.21\"\n";

        let mut file = File::create(&config_path)
            .await
            .context("Failed to create config.txt file")?;

        file.write_all(toml_content.as_bytes())
            .await
            .context("Failed to write to config.txt")?;

        Ok(())
    }

    #[tokio::test]
    async fn test_use_paper_version_nonexistent_version() -> Result<()> {
        let test_home_dir = PathBuf::from("./tests/test_data/.mvm");

        env::set_var("MVM_HOME", &test_home_dir);

        let version = "nonexistent version";

        let result = use_version(version, &ServerType::Paper,&get_dir().await?).await;

        if let Err(ref err) = result {
            eprintln!("Test failed with error: {:?}", err);
        }

        assert!(result.is_err(), "Expected an error for a nonexistent version");

        Ok(())
    }

}