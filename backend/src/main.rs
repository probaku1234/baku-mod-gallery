use axum::{routing::get, Router};
use mongodb;

async fn hello_world() -> &'static str {
    "Hello, world!"
}

#[shuttle_runtime::main]
async fn main(
    #[shuttle_shared_db::MongoDb] mongo: mongodb::Database,
    #[shuttle_secrets::Secrets] secret_store: shuttle_secrets::SecretStore
) -> shuttle_axum::ShuttleAxum {
    let router = Router::new().route("/", get(hello_world));

    Ok(router.into())
}
