use axum::Extension;
use axum::Router;
use axum::extract::FromRef;
use axum::extract::ws::WebSocketUpgrade;
use axum::response::IntoResponse;
use axum::routing::get;
use leptos::prelude::*;
use leptos_axum::{ErrorHandler, LeptosRoutes, generate_route_list, site_pkg_dir_service};
use sky_server::server::axum_ws;
use sky_server::server::storage::StorageService;
use skybase::app::{App, shell};

#[derive(Clone)]
pub struct AppState {
    pub leptos_options: LeptosOptions,
    pub storage: StorageService,
}

impl FromRef<AppState> for LeptosOptions {
    fn from_ref(state: &AppState) -> Self {
        state.leptos_options.clone()
    }
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    Extension(storage): Extension<StorageService>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| async move {
        axum_ws::serve_websocket(socket, storage).await;
    })
}

pub async fn serve() {
    use leptos::logging::log;

    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;
    let routes = generate_route_list(App);

    let storage = StorageService::start(["skybase/version"])
        .await
        .expect("start a storage service");
    let state = AppState {
        leptos_options: conf.leptos_options,
        storage: storage.clone(),
    };
    let app = Router::new()
        .leptos_routes_with_context(
            &state,
            routes,
            {
                // let db = state.db.clone();
                move || {
                    //                    provide_context(db.clone());
                }
            },
            {
                let state = state.clone();
                move || shell(state.leptos_options.clone())
            },
        )
        .fallback_service(
            site_pkg_dir_service(&state.leptos_options)
                .fallback(ErrorHandler::new(shell, state.leptos_options.clone())),
        )
        .route("/ws", get(ws_handler))
        .with_state(state)
        .layer(Extension(storage));

    log!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
