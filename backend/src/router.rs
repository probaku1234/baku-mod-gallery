use crate::posts::{create_new_post, delete_post, edit_post, get_all_posts, get_post_by_id, delete_all_posts};
use crate::AppState;
use axum::routing::delete;
use axum::{
    http::{self},
    middleware::{self},
    routing::{get, post, put},
    Router,
};
use http::header::{ACCEPT, AUTHORIZATION, ORIGIN};
use http::Method;
use tower_http::cors::CorsLayer;
use crate::jwt_auth::auth_jwt;

pub fn create_api_router(state: AppState) -> Router {
    let origins = [
        state.server_domain.parse().unwrap(),
        state.client_domain.parse().unwrap(),
    ];

    let cors = CorsLayer::new()
        .allow_credentials(true)
        .allow_methods(vec![Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(vec![ORIGIN, AUTHORIZATION, ACCEPT])
        .allow_origin(origins);

    let posts_router = Router::new()
        .route("/:id", put(edit_post).delete(delete_post))
        .route("/create", post(create_new_post))
        .route("/", delete(delete_all_posts))
        .layer(middleware::from_fn_with_state(state.clone(), auth_jwt))
        .route("/", get(get_all_posts))
        .route(
            "/:id",
            post(get_post_by_id)
        );

    Router::new()
        .nest("/posts", posts_router)
        // .layer(middleware::from_extractor_with_state(
        //     state.clone()
        // ))
        .route("/health_check", get(hello_world))
        .with_state(state)
        .layer(cors)
}

pub async fn hello_world() -> &'static str {
    "Hello, world!"
}
