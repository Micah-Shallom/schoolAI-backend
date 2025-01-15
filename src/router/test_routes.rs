use crate::controllers::test_controller::hello_world;
use crate::controllers::test_controller::protected_hello;
use crate::middleware::auth::auth_middleware;
use axum::{middleware, routing::get, Router};

use super::AppState;

pub fn routes(state: AppState) -> Router<AppState> {
    let public_routes = Router::new()
        .route("/", get(hello_world))
        .route("/test", get(hello_world));

    let protected_routes = Router::new()
        .route("/protected", get(protected_hello))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    Router::new()
        .nest(
            state.version.as_str(),
            public_routes.merge(protected_routes),
        )
        .with_state(state)
}
