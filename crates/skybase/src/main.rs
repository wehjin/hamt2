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
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::Init => println!("init"),
    }
}
