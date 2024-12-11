mod version_manager;
mod models;
mod config;

use clap::{Parser, Subcommand};
use crate::version_manager::download_server_jar;
use crate::models::ServerType;
use anyhow::{anyhow, Result};
use config::{get_dir};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]

struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
#[command(group(
    ArgGroup::new("server_type")
        .args(&["paper"]),
))]
enum Commands {
    r#Use {
        version: Option<String>,
    },
    Install {
        #[arg(default_value = "latest")]
        version: String,

        #[arg(long)]
        paper: bool
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
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::r#Use {version}) => {
            let version = version.ok_or_else(|| anyhow!("No version provided, please specify a version."))?;
            version_manager::use_version(&version, &get_dir().await?)
                .await?;
        }

        Some(Commands::Install { version, paper}) => {
            let server_type = ServerType::determine_server_type(paper);

            let download_info = version_manager::get_version_download(&version, &server_type)
                .await?;

            println!("Found version, downloading...");

            download_server_jar(download_info.url, &version, &get_dir().await?)
                .await?;
        }

        Some(Commands::Uninstall {version}) => {
            let version = version.ok_or_else(|| anyhow!("No version provided, please specify a version."))?;

            version_manager::delete_server_jar(&version, &get_dir().await?)
                .await?;
        }

        Some(Commands::Which {version}) => {
            let path = version_manager::get_version(&version, &get_dir().await?)
                .await?;

            println!("{}", path);
        }
        None => {
            println!("Unknown command: {:?}", cli.command);
        }
    }


    Ok(())
}
