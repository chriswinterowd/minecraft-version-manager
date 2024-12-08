mod version_manager;
mod models;
use clap::{Parser, Subcommand};
use crate::version_manager::download_server_jar;
use anyhow::{anyhow, Context, Result};

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
    Which {
        #[arg(default_value = "latest")]
        version: String
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::r#Use {version}) => {
            let version = version.ok_or_else(|| anyhow!("No version provided, please specify a version."))?;
            version_manager::use_version(&version)
                .await
                .context(format!("Error using version '{}'", version))?;
        }

        Some(Commands::Install { version}) => {
            let download_info = version_manager::get_version_download(&version)
                .await?
                .ok_or_else(|| anyhow!("Version: '{}' not found!;", version))?;

            println!("Found version, downloading...");

            download_server_jar(download_info.url, &version)
                .await
                .context("Error downloading server jar")?;
        }

        Some(Commands::Which {version}) => {
            let path = version_manager::get_version(&version)
                .await
                .context(format!("Error getting version '{}'", version))?;
            println!("{}", path);
        }
        None => {
            println!("Unknown command: {:?}", cli.command);
        }
    }


    Ok(())
}
