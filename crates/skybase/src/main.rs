use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "skybase")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Init,
    #[cfg(feature = "ssr")]
    Serve,
}

#[cfg(feature = "ssr")]
mod server;

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::Init => println!("init"),
        Command::Serve => server::serve().await,
    }
}

#[cfg(not(feature = "ssr"))]
fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::Init => println!("init"),
    }
}
