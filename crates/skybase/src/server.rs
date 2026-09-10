use axum::Router;
use axum::extract::FromRef;
use leptos::prelude::*;
use leptos_axum::{ErrorHandler, LeptosRoutes, generate_route_list, site_pkg_dir_service};

use skybase::app::{App, shell};

#[derive(Clone)]
pub struct AppState {
    pub leptos_options: LeptosOptions,
}

impl FromRef<AppState> for LeptosOptions {
    fn from_ref(state: &AppState) -> Self {
        state.leptos_options.clone()
    }
}

pub async fn serve() {
    use leptos::logging::log;

    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;
    let routes = generate_route_list(App);

    let state = AppState {
        leptos_options: conf.leptos_options,
    };

    let app = Router::new()
        .leptos_routes(
            &state,
            routes,
            {
                let state = state.clone();
                move || shell(state.leptos_options.clone())
            },
        )
        .fallback_service(
            site_pkg_dir_service(&state.leptos_options)
                .fallback(ErrorHandler::new(shell, state.leptos_options.clone())),
        )
        .with_state(state);

    log!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
