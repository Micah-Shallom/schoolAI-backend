use super::AppState;
use crate::controllers::auth_controller::register_user;
use axum::{routing::post, Router};

pub fn routes(_state: AppState) -> Router<AppState> {
    let public_routes = Router::new().route("/register", post(register_user));
    // .route("/login", post(login_user))

    public_routes
}
