use crate::{
    controllers::{
        content_controller::generate_academic_content, mcq_controller::generate_mcq_content,
        presentation_controller::generate_presentation_content,
    },
    middleware::auth::auth_middleware,
};

use super::AppState;
use axum::{middleware, routing::post, Router};

pub fn routes(state: AppState) -> Router<AppState> {
    let protected_routes = Router::new()
        .route("/academic-content-gen", post(generate_academic_content))
        .route("/mcq-gen", post(generate_mcq_content))
        .route("/presentation-gen", post(generate_presentation_content))
        //middleware
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    Router::new()
        .nest(state.version.as_str(), protected_routes)
        .with_state(state)
}
