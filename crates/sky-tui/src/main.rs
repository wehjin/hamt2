use crate::routes::app::App;
use ratatui_kit::prelude::*;

#[tokio::main]
async fn main() {
    element!(App)
        .fullscreen()
        .await
        .expect("failed to run the application");
}

mod routes;
