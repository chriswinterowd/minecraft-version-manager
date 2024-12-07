mod downloader;
mod models;

use clap::{Parser, Subcommand};
use crate::downloader::download_server_jar;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]

struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    r#Use,
    Install {
        #[arg(default_value = "latest")]
        version: String
    }
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::r#Use) => {
            println!("Placeholder for the use command");
        }

        Some(Commands::Install { version}) => {
            match downloader::get_version_download(&version).await {
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

        None => {
            println!("Unknown command: {:?}", cli.command);
        }
    }

}
