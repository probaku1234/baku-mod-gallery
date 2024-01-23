use axum::Router;
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
    #[shuttle_secrets::Secrets] _secret_store: shuttle_secrets::SecretStore,
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
            find_post_by_id, generate_port_number, get_db_connection_uri, get_mongo_image, populate_test_data
        };
    use ::axum_test::TestServer;
    use mongodb::{bson::Bson, Client};
    use serde_json::json;
    use testcontainers::clients;

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

    #[tokio::test]
    async fn test_get_post_by_id_invalid_id() {
        let docker = clients::Cli::default();
        let port = generate_port_number();
        let mongo_img = get_mongo_image(&port);
        let _c = docker.run(mongo_img);
        let uri = get_db_connection_uri(&port);
        let client = Client::with_uri_str(uri).await.unwrap();

        let test_db = client.database("test_db");

        let app = app(test_db.clone());

        let invalid_id = "invalid id";

        let server = TestServer::new(app).unwrap();

        let response = server
            .post(format!("/api/posts/{}", invalid_id).as_str())
            .await;

        response.assert_status_bad_request();
    }

    #[tokio::test]
    async fn test_get_post_by_id_not_found() {
        let docker = clients::Cli::default();
        let port = generate_port_number();
        let mongo_img = get_mongo_image(&port);
        let _c = docker.run(mongo_img);
        let uri = get_db_connection_uri(&port);
        let client = Client::with_uri_str(uri).await.unwrap();

        let test_db = client.database("test_db");

        let app = app(test_db.clone());

        let invalid_id = "659e79f831f22dc0395699b2";

        let server = TestServer::new(app).unwrap();

        let response = server
            .post(format!("/api/posts/{}", invalid_id).as_str())
            .await;

        response.assert_status_not_found()
    }

    #[tokio::test]
    async fn test_get_post_by_id() {
        // FIXME: figure out how to do this
        let docker = clients::Cli::default();
        let port = generate_port_number();
        let mongo_img = get_mongo_image(&port);
        let _c = docker.run(mongo_img);
        let uri = get_db_connection_uri(&port);
        let client = Client::with_uri_str(uri).await.unwrap();

        let test_db = client.database("test_db");

        let app = app(test_db.clone());

        // let new_post_title = "aa".to_string();
        // let new_post_images_url: Vec<String> = vec![];
        // let new_post_file_url = "aa".to_string();

        // let data = r#"
        //     {
        //         "title": "test title",
        //         "images_url": [],
        //         "file_url": "test url",
        //         "created_at": 1705937213517,
        //         "updated_at": 1705937213517
        //     }
        // "#;
        // let new_post: Post = serde_json::from_str(data).unwrap();
        // let inserted_post_object_id = insert_test_post(test_db.clone(), new_post).await;

        // let response_post = response.json::<Post>();
        // assert_eq!(res)
    }

    #[tokio::test]
    async fn test_edit_post_invalid_id() {
        let docker = clients::Cli::default();
        let port = generate_port_number();
        let mongo_img = get_mongo_image(&port);
        let _c = docker.run(mongo_img);
        let uri = get_db_connection_uri(&port);
        let client = Client::with_uri_str(uri).await.unwrap();

        let test_db = client.database("test_db");

        let app = app(test_db.clone());

        let invalid_id = "invalid id";
        let updated_title = "updated test post".to_string();
        let updated_image_url = vec!["one two three".to_string()];
        let updated_file_url = "updated file url".to_string();

        let server = TestServer::new(app).unwrap();

        let response = server
            .put(format!("/api/posts/{}", invalid_id).as_str())
            .content_type(&"application/json")
            .json(&json!({
                "title": updated_title.clone(),
                "imagesUrl": updated_image_url.clone(),
                "fileUrl": updated_file_url.clone(),
            }))
            .await;

        response.assert_status_bad_request();
    }

    #[tokio::test]
    async fn test_eidt_post_not_found() {
        let docker = clients::Cli::default();
        let port = generate_port_number();
        let mongo_img = get_mongo_image(&port);
        let _c = docker.run(mongo_img);
        let uri = get_db_connection_uri(&port);
        let client = Client::with_uri_str(uri).await.unwrap();

        let test_db = client.database("test_db");

        let app = app(test_db.clone());

        let invalid_id = "659e79f831f22dc0395699b2";
        let updated_title = "updated test post".to_string();
        let updated_image_url = vec!["one two three".to_string()];
        let updated_file_url = "updated file url".to_string();

        let server = TestServer::new(app).unwrap();

        let response = server
            .put(format!("/api/posts/{}", invalid_id).as_str())
            .content_type(&"application/json")
            .json(&json!({
                "title": updated_title.clone(),
                "imagesUrl": updated_image_url.clone(),
                "fileUrl": updated_file_url.clone(),
            }))
            .await;

        response.assert_status_not_found();
    }

    #[tokio::test]
    async fn test_edit_post() {
        let docker = clients::Cli::default();
        let port = generate_port_number();
        let mongo_img = get_mongo_image(&port);
        let _c = docker.run(mongo_img);
        let uri = get_db_connection_uri(&port);
        let client = Client::with_uri_str(uri).await.unwrap();

        let test_db = client.database("test_db");

        let app = app(test_db.clone());

        let new_post_title = "aa".to_string();
        let new_post_images_url: Vec<String> = vec![];
        let new_post_file_url = "aa".to_string();

        let server = TestServer::new(app).unwrap();

        let insert_result = server
            .post("/api/posts/create")
            .content_type(&"application/json")
            .json(&json!({
                "title": new_post_title.clone(),
                "imagesUrl": new_post_images_url.clone(),
                "fileUrl": new_post_file_url.clone(),
            }))
            .await;

        let inserted_post_id = insert_result.json::<Bson>();
        let object_id = inserted_post_id.as_object_id().unwrap();

        let updated_title = "updated test post".to_string();
        let updated_image_url = vec!["one two three".to_string()];
        let updated_file_url = "updated file url".to_string();

        let response = server
            .put(format!("/api/posts/{}", object_id.to_hex()).as_str())
            .content_type(&"application/json")
            .json(&json!({
                "title": updated_title.clone(),
                "imagesUrl": updated_image_url.clone(),
                "fileUrl": updated_file_url.clone(),
            }))
            .await;

        response.assert_status_ok();
    }

    #[tokio::test]
    async fn test_delete_post_invalid_id() {
        let docker = clients::Cli::default();
        let port = generate_port_number();
        let mongo_img = get_mongo_image(&port);
        let _c = docker.run(mongo_img);
        let uri = get_db_connection_uri(&port);
        let client = Client::with_uri_str(uri).await.unwrap();

        let test_db = client.database("test_db");

        let app = app(test_db.clone());

        let invalid_id = "invalid id";

        let server = TestServer::new(app).unwrap();

        let response = server
            .delete(format!("/api/posts/{}", invalid_id).as_str())
            .await;

        response.assert_status_bad_request();
    }

    #[tokio::test]
    async fn test_delete_post_not_found() {
        let docker = clients::Cli::default();
        let port = generate_port_number();
        let mongo_img = get_mongo_image(&port);
        let _c = docker.run(mongo_img);
        let uri = get_db_connection_uri(&port);
        let client = Client::with_uri_str(uri).await.unwrap();

        let test_db = client.database("test_db");

        let app = app(test_db.clone());

        let invalid_id = "659e79f831f22dc0395699b2";

        let server = TestServer::new(app).unwrap();

        let response = server
            .delete(format!("/api/posts/{}", invalid_id).as_str())
            .await;

        response.assert_status_not_found();
    }

    #[tokio::test]
    async fn test_delete_post() {
        let docker = clients::Cli::default();
        let port = generate_port_number();
        let mongo_img = get_mongo_image(&port);
        let _c = docker.run(mongo_img);
        let uri = get_db_connection_uri(&port);
        let client = Client::with_uri_str(uri).await.unwrap();

        let test_db = client.database("test_db");

        let app = app(test_db.clone());

        let new_post_title = "aa".to_string();
        let new_post_images_url: Vec<String> = vec![];
        let new_post_file_url = "aa".to_string();

        let server = TestServer::new(app).unwrap();

        let insert_result = server
            .post("/api/posts/create")
            .content_type(&"application/json")
            .json(&json!({
                "title": new_post_title.clone(),
                "imagesUrl": new_post_images_url.clone(),
                "fileUrl": new_post_file_url.clone(),
            }))
            .await;

        let inserted_post_id = insert_result.json::<Bson>();
        let object_id = inserted_post_id.as_object_id().unwrap();

        let delete_result = server
            .delete(format!("/api/posts/{}", object_id.clone().to_hex()).as_str())
            .await;

        delete_result.assert_status_ok();

        let find_result = find_post_by_id(test_db, object_id).await;

        assert!(find_result.is_none());
    }

    #[tokio::test]
    async fn test_create_new_post() {
        let docker = clients::Cli::default();
        let port = generate_port_number();
        let mongo_img = get_mongo_image(&port);
        let _c = docker.run(mongo_img);
        let uri = get_db_connection_uri(&port);
        let client = Client::with_uri_str(uri).await.unwrap();

        let test_db = client.database("test_db");

        let app = app(test_db.clone());

        let new_post_title = "aa".to_string();
        let new_post_images_url: Vec<String> = vec![];
        let new_post_file_url = "aa".to_string();

        let server = TestServer::new(app).unwrap();

        let insert_result = server
            .post("/api/posts/create")
            .content_type(&"application/json")
            .json(&json!({
                "title": new_post_title.clone(),
                "imagesUrl": new_post_images_url.clone(),
                "fileUrl": new_post_file_url.clone(),
            }))
            .await;

        insert_result.assert_status_ok();

        let inserted_post_id = insert_result.json::<Bson>();
        let object_id = inserted_post_id.as_object_id().unwrap();

        let find_result = find_post_by_id(test_db, object_id).await;

        assert!(find_result.is_some());
    }
}
