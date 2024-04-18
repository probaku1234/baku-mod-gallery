use crate::jwt_auth::auth_jwt;
use crate::posts::{
    create_new_post, delete_all_posts, delete_post, edit_post, get_all_posts, get_post_by_id,
    sync_posts,
};
use crate::{redis_pubsub, AppState};
use axum::extract::State;
use axum::http::StatusCode;
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
use tracing::error;

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
        .route("/:id", post(get_post_by_id))
        .route("/sync", get(sync_posts));

    Router::new()
        .nest("/posts", posts_router)
        // .layer(middleware::from_extractor_with_state(
        //     state.clone()
        // ))
        .route("/health_check", get(hello_world))
        .route("/pubsub_test", get(pubsub_test))
        .with_state(state)
        .layer(cors)
}

pub async fn hello_world() -> &'static str {
    "Hello, world!"
}

pub async fn pubsub_test(State(state): State<AppState>) -> StatusCode {
    let redis = state.redis.clone();

    let result = redis_pubsub::pubsub::publish_message(
        redis.clone(),
        redis_pubsub::message::Message::new(String::from("test message")),
    );

    if result.is_err() {
        error!("Failed to publish message: {}", result.unwrap_err());
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    StatusCode::OK
}
