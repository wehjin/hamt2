#[cfg(feature = "ssr")]
mod server;

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use clap::Parser;

    #[derive(Parser)]
    #[command(
        name = "skybase",
        version = "0.1",
        about = "A tool for managing sky-db."
    )]
    pub struct Cli {}

    let _cli = Cli::parse();
    server::serve().await;
}

#[cfg(not(feature = "ssr"))]
fn main() {
    panic!("ssr feature is not enabled");
}
