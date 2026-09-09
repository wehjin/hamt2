#[cfg(feature = "ssr")]
mod server;

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use clap::{Parser, Subcommand};

    #[derive(Parser)]
    #[command(name = "skybase")]
    pub struct Cli {
        #[command(subcommand)]
        pub command: Command,
    }

    #[derive(Subcommand)]
    pub enum Command {
        Init,
        #[cfg(feature = "ssr")]
        Serve,
    }
    let cli = Cli::parse();
    match cli.command {
        Command::Init => println!("init"),
        Command::Serve => server::serve().await,
    }
}

#[cfg(not(feature = "ssr"))]
fn main() {
    panic!("ssr feature is not enabled");
}
