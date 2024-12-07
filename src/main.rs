mod version_manager;
mod models;
use clap::{Parser, Subcommand};
use crate::version_manager::download_server_jar;

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
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::r#Use {version}) => {
            match version {
                Some(version) => {
                    if let Err(err) = version_manager::use_version(&version).await {
                        println!("Error using version {}: {}", &version, err);
                    }
                }
                None => {
                    println!("No version provided, please specify a version.");
                }
            }

        }

        Some(Commands::Install { version}) => {
            match version_manager::get_version_download(&version).await {
                Ok(Some(download_info)) =>  {
                    println!("Found version, downloading..");
                    if let Err(err) = download_server_jar(download_info.url, &version).await {
                        println!("Error downloading server jar: {}", err);
                    }
                },
                Ok(None) => println!("Version not found!"),
                Err(err) => {
                    println!("Error: {}", err);
                }
            }
        }

        Some(Commands::Which {version}) => {
            match version_manager::get_version(&version).await {
                Ok(path) => {
                    println!("{}", path);
                }
                Err(err) => {
                    println!("Error: {}", err);
                }
            }
        }
        None => {
            println!("Unknown command: {:?}", cli.command);
        }
    }

}
