use axum::{routing::get, Router};
use mongodb;

mod posts;
mod router;
mod test_util;

use router::create_api_router;

#[derive(Clone)]
pub struct AppState {
    pub mongo: mongodb::Database
}

#[shuttle_runtime::main]
async fn main(
    #[shuttle_shared_db::MongoDb] mongo: mongodb::Database,
    #[shuttle_secrets::Secrets] secret_store: shuttle_secrets::SecretStore,
) -> shuttle_axum::ShuttleAxum {
    let state = AppState {
        mongo
    };

    let api_router = create_api_router(state);
    let router = Router::new().nest("/api", api_router);

    Ok(router.into())
}
