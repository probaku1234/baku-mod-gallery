use crate::posts::{create_new_post, get_all_posts, get_post_by_id};
use crate::AppState;
use axum::{
    http::{self},
    middleware::{self},
    routing::{get, post},
    Router,
};

pub fn create_api_router(state: AppState) -> Router {
    let posts_router = Router::new()
        .route("/", get(get_all_posts))
        .route("/:id", post(get_post_by_id))
        .route("/create", post(create_new_post));

    Router::new()
        .nest("/posts", posts_router)
        // .layer(middleware::from_extractor_with_state(
        //     state.clone()
        // ))
        .route("/health_check", get(hello_world))
        .with_state(state)
}

pub async fn hello_world() -> &'static str {
    "Hello, world!"
}
