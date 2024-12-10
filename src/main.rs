mod version_manager;
mod models;
mod config;

use clap::{Parser, Subcommand};
use crate::version_manager::download_server_jar;
use anyhow::{anyhow, Result};
use config::{initialize_home_dir, get_home_dir};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]

struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    r#Use {
        version: Option<String>,
    },
    Install {
        #[arg(default_value = "latest")]
        version: String
    },
    Uninstall {
        version: Option<String>,
    },
    Which {
        #[arg(default_value = "recent")]
        version: String
    }
}


#[tokio::main]
async fn main() -> Result<()> {
    initialize_home_dir().await?;
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::r#Use {version}) => {
            let version = version.ok_or_else(|| anyhow!("No version provided, please specify a version."))?;
            version_manager::use_version(&version, &get_home_dir().await?)
                .await?;
        }

        Some(Commands::Install { version}) => {
            let download_info = version_manager::get_version_download(&version)
                .await?;

            println!("Found version, downloading...");

            download_server_jar(download_info.url, &version, &get_home_dir().await?)
                .await?;
        }

        Some(Commands::Uninstall {version}) => {
            let version = version.ok_or_else(|| anyhow!("No version provided, please specify a version."))?;

            version_manager::delete_server_jar(&version, &get_home_dir().await?)
                .await?;
        }

        Some(Commands::Which {version}) => {
            let path = version_manager::get_version(&version, &get_home_dir().await?)
                .await?;

            println!("{}", path);
        }
        None => {
            println!("Unknown command: {:?}", cli.command);
        }
    }


    Ok(())
}
