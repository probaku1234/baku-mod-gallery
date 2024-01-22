use axum::{routing::get, Router};
use mongodb;

mod posts;
mod router;
mod test_util;

use router::create_api_router;

#[derive(Clone)]
pub struct AppState {
    pub mongo: mongodb::Database,
}

#[shuttle_runtime::main]
async fn main(
    #[shuttle_shared_db::MongoDb] mongo: mongodb::Database,
    #[shuttle_secrets::Secrets] secret_store: shuttle_secrets::SecretStore,
) -> shuttle_axum::ShuttleAxum {
    // let state = AppState {
    //     mongo
    // };

    // let api_router = create_api_router(state);
    // let router = Router::new().nest("/api", api_router);

    Ok(app(mongo.clone()).into())
}

fn app(mongo: mongodb::Database) -> Router {
    let state = AppState { mongo };

    let api_router = create_api_router(state);
    Router::new().nest("/api", api_router)
}

// TODO: test endpoint
#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::test_util::{
        generate_port_number, get_db_connection_uri, get_mongo_image, populate_test_data,
    };
    use mongodb::Client;
    use serde_json::{json, Value};
    use testcontainers::clients;
    use ::axum_test::TestServer;

    #[tokio::test]
    async fn test_hello_world() {
        let docker = clients::Cli::default();
        let port = generate_port_number();
        let mongo_img = get_mongo_image(&port);
        let _c = docker.run(mongo_img);
        let uri = get_db_connection_uri(&port);
        let client = Client::with_uri_str(uri).await.unwrap();

        let test_db = client.database("test_db");

        let app = app(test_db);

        let server = TestServer::new(app).unwrap();

        let response = server.get("/api/health_check").await;

        assert_eq!(response.text(), "Hello, world!");
    }

    #[tokio::test]
    async fn test_get_all_posts() {
        let docker = clients::Cli::default();
        let port = generate_port_number();
        let mongo_img = get_mongo_image(&port);
        let _c = docker.run(mongo_img);
        populate_test_data(&port);
        let uri = get_db_connection_uri(&port);
        let client = Client::with_uri_str(uri).await.unwrap();

        let test_db = client.database("test_db");

        let app = app(test_db);

        let server = TestServer::new(app).unwrap();

        let response = server.get("/api/posts").await;

        response.assert_status_ok();
        let response_posts = response.json::<Vec<posts::Post>>();
        assert_eq!(response_posts.len(), 2);
        // let k = response.as_bytes();
        // let kkk: Value = serde_json::from_slice(k).unwrap();
        // assert_eq!(kkk, [json!({})]);
    }
}
