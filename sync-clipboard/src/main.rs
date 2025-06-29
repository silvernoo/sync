mod client;
mod crypto;
mod protocol;
mod server;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Runs the clipboard sync server
    Server {
        /// The address to bind to
        #[arg(short, long, default_value = "0.0.0.0")]
        address: String,

        /// The port to listen on
        #[arg(short, long, default_value_t = 7878)]
        port: u16,

        /// The secret key for encryption
        #[arg(short, long)]
        key: String,
    },
    /// Runs the clipboard sync client
    Client {
        /// The address of the server to connect to
        #[arg(short, long, default_value = "127.0.0.1")]
        address: String,

        /// The port of the server to connect to
        #[arg(short, long, default_value_t = 7878)]
        port: u16,

        /// The secret key for encryption
        #[arg(short, long)]
        key: String,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Server {
            address,
            port,
            key,
        } => {
            if let Err(e) = server::run(address, *port, key).await {
                eprintln!("Server error: {}", e);
            }
        }
        Commands::Client {
            address,
            port,
            key,
        } => {
            if let Err(e) = client::run(address, *port, key).await {
                eprintln!("Client error: {}", e);
            }
        }
    }
}