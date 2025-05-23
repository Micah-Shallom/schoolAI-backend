use super::AppState;
use crate::controllers::auth_controller::{login_user, logout_user, register_user};
use axum::{routing::post, Router};

pub fn routes(state: AppState) -> Router<AppState> {
    let public_routes = Router::new()
        .route("/register", post(register_user))
        .route("/login", post(login_user))
        .route("/logout", post(logout_user));

    Router::new()
        .nest(state.version.as_str(), public_routes)
        .with_state(state)
}
