mod version_manager;
mod config;
mod server;

use clap::{Parser, Subcommand};
use crate::version_manager::download_server_jar;
use crate::server::server_types::ServerType;
use anyhow::{anyhow, Result};
use config::{get_dir};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]

struct Cli {
    #[arg(long, global = true)]
    paper: bool,
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    r#Use {
        version: Option<String>,

        #[arg(long)]
        paper: bool
    },
    Install {
        #[arg(default_value = "latest")]
        version: String,

        #[arg(long)]
        paper: bool
    },
    Uninstall {
        version: Option<String>,

        #[arg(long)]
        paper: bool
    },
    Which {
        #[arg(default_value = "recent")]
        version: String,

        #[arg(long)]
        paper: bool
    }
}


#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::r#Use {version, paper}) => {
            let server_type_string = bool_to_string(paper);
            let server_type = ServerType::from_string(server_type_string)?;
            let version = version.ok_or_else(|| anyhow!("No version provided, please specify a version."))?;
            version_manager::use_version(&version, &server_type, &get_dir().await?)
                .await?;
        }

        Some(Commands::Install { version, paper}) => {
            let server_type_string = bool_to_string(paper);
            let server_type = ServerType::from_string(server_type_string)?;

            let download_url = version_manager::get_version_download(&version, &server_type)
                .await?;

            println!("Found version, downloading...");

            download_server_jar(download_url, &version, &server_type,&get_dir().await?)
                .await?;
        }

        Some(Commands::Uninstall {version, paper}) => {
            let version = version.ok_or_else(|| anyhow!("No version provided, please specify a version."))?;
            let server_type_string = bool_to_string(paper);
            let server_type = ServerType::from_string(server_type_string)?;
            version_manager::delete_server_jar(&version, &server_type, &get_dir().await?)
                .await?;
        }

        Some(Commands::Which {version, paper}) => {
            let server_type_string = bool_to_string(paper);
            let server_type = ServerType::from_string(server_type_string)?;
            let path = version_manager::get_version(&version, &server_type, &get_dir().await?)
                .await?;

            println!("{}", path);
        }
        None => {
            println!("Unknown command: {:?}", cli.command);
        }
    }


    Ok(())
}

fn bool_to_string(paper: bool) -> String {
    match paper {
        true => "paper".to_string(),
        false => "vanilla".to_string()
    }
}
