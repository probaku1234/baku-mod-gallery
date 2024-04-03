use axum::Router;
use mongodb;

mod dao;
mod errors;
mod jwt_auth;
mod posts;
mod router;
mod sync_job;
mod sync_post;
mod test_util;
mod util;

use crate::errors::SetupError;
use anyhow::Error;
use mongodb::{options::ClientOptions, Client, Database};
use router::create_api_router;
use shuttle_runtime::SecretStore;

#[derive(Clone)]
pub struct AppState {
    pub mongo: mongodb::Database,
    pub jwt_key: String,
    pub server_domain: String,
    pub client_domain: String,
    pub patreon_access_token: String,
}

#[shuttle_runtime::main]
async fn main(
    #[shuttle_shared_db::MongoDb] _mongo: Database,
    #[shuttle_runtime::Secrets] secret_store: SecretStore,
) -> shuttle_axum::ShuttleAxum {
    //TODO: handle db connection error
    let (
        jwt_key,
        server_domain,
        client_domain,
        mongo_id,
        mongo_password,
        patreon_access_token,
        db_name,
    ) = grab_secrets(secret_store);

    let db = connect_mongo(mongo_id, mongo_password, db_name).await?;

    let state = AppState {
        mongo: db,
        jwt_key,
        server_domain,
        client_domain,
        patreon_access_token,
    };

    Ok(app(state).into())
}

fn app(state: AppState) -> Router {
    // let state = AppState { mongo, jwt_key };

    let api_router = create_api_router(state);
    Router::new().nest("/api", api_router)
}

fn grab_secrets(
    secrets: SecretStore,
) -> (String, String, String, String, String, String, String) {
    let jwt_key = secrets
        .get("JWT_SECRET")
        .unwrap_or_else(|| "None".to_string());

    let server_domain = secrets
        .get("SERVER_DOMAIN")
        .unwrap_or_else(|| "None".to_string());

    let client_domain = secrets
        .get("CLIENT_DOMAIN")
        .unwrap_or_else(|| "None".to_string());

    let mongo_id = secrets
        .get("MONGO_ID")
        .unwrap_or_else(|| "None".to_string());

    let mongo_password = secrets
        .get("MONGO_PASSWORD")
        .unwrap_or_else(|| "None".to_string());

    let patreon_access_token = secrets
        .get("PATREON_ACCESS_TOKEN")
        .unwrap_or_else(|| "None".to_string());

    let db_name = secrets.get("DB_NAME").unwrap_or_else(|| "None".to_string());

    (
        jwt_key,
        server_domain,
        client_domain,
        mongo_id,
        mongo_password,
        patreon_access_token,
        db_name,
    )
}

async fn connect_mongo(
    mongo_id: String,
    mongo_password: String,
    db_name: String,
) -> anyhow::Result<Database> {
    let mongo_connect_str = format!("mongodb+srv://{}:{}@my-mod-gallery-cluter0.nkvlp6a.mongodb.net/?retryWrites=true&w=majority", mongo_id, mongo_password);
    let client_options_result = ClientOptions::parse(mongo_connect_str).await;

    if client_options_result.is_err() {
        let error = client_options_result.err().unwrap().to_string();
        return Err(Error::from(SetupError(error)));
    }
    let client_options = client_options_result.unwrap();

    let client_result = Client::with_options(client_options);
    if client_result.is_err() {
        let error = client_result.err().unwrap().to_string();
        return Err(Error::from(SetupError(error)));
    }
    let client = client_result.unwrap();

    Ok(client.database(&db_name))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::test_util::{count_all_posts, create_test_state, find_post_by_id, generate_port_number, generate_test_jwt_token, get_db_connection_uri, get_mongo_image, insert_test_post, populate_test_data};
    use ::axum_test::TestServer;
    use axum::http::{HeaderName, HeaderValue};
    use mongodb::{bson::Bson, Client};
    use mongodb::bson::to_document;
    use serde_json::json;
    use testcontainers::clients;
    use crate::posts::Post;

    #[tokio::test]
    async fn test_hello_world() {
        let docker = clients::Cli::default();
        let port = generate_port_number();
        let mongo_img = get_mongo_image(&port);
        let _c = docker.run(mongo_img);
        let uri = get_db_connection_uri(&port);
        let client = Client::with_uri_str(uri).await.unwrap();

        let test_db = client.database("test_db");
        let state = create_test_state(test_db);
        let app = app(state);

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
        let state = create_test_state(test_db);
        let app = app(state);

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
        let state = create_test_state(test_db);
        let app = app(state);

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
        let state = create_test_state(test_db);
        let app = app(state);

        let invalid_id = "659e79f831f22dc0395699b2";

        let server = TestServer::new(app).unwrap();

        let response = server
            .post(format!("/api/posts/{}", invalid_id).as_str())
            .await;

        response.assert_status_not_found()
    }

    #[tokio::test]
    async fn test_get_post_by_id() {
        let docker = clients::Cli::default();
        let port = generate_port_number();
        let mongo_img = get_mongo_image(&port);
        let _c = docker.run(mongo_img);
        let uri = get_db_connection_uri(&port);
        let client = Client::with_uri_str(uri).await.unwrap();

        let test_db = client.database("test_db");
        let state = create_test_state(test_db.clone());
        let app = app(state);

        let data = r#"
            {
                "_id": "659e79f831f22dc0395699b2",
                "patreon_post_id": "123123",
                "title": "test title",
                "content": "qweqwe",
                "images_url": [],
                "file_url": "test url",
                "mod_type": "qweqwe",
                "created_at": "2024-01-23T13:48:06.761Z",
                "updated_at": "2024-01-23T13:48:06.761Z",
                "synced_at": "2024-01-23T13:48:06.761Z"
            }
        "#;
        let new_post: Post = serde_json::from_str(data).unwrap();
        let inserted_post_object_id = insert_test_post(test_db.clone(), new_post).await;

        let server = TestServer::new(app).unwrap();

        let response = server
            .post(format!("/api/posts/{}", inserted_post_object_id.to_hex()).as_str())
            .await;

        response.assert_status_ok();

        let response_post = response.json::<Post>();
        let post_doc = to_document(&response_post).unwrap();
        assert_eq!(post_doc.get_str("patreon_post_id").unwrap(), "123123");
        assert_eq!(post_doc.get_str("title").unwrap(), "test title");
    }

    #[tokio::test]
    async fn test_edit_post_unauthorized() {}

    #[tokio::test]
    async fn test_edit_post_invalid_id() {
        let docker = clients::Cli::default();
        let port = generate_port_number();
        let mongo_img = get_mongo_image(&port);
        let _c = docker.run(mongo_img);
        let uri = get_db_connection_uri(&port);
        let client = Client::with_uri_str(uri).await.unwrap();

        let test_db = client.database("test_db");
        let state = create_test_state(test_db);
        let app = app(state);

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

        response.assert_status_unauthorized();
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
        let state = create_test_state(test_db);
        let app = app(state);

        let invalid_id = "659e79f831f22dc0395699b2";
        let updated_title = "updated test post".to_string();
        let updated_content = "content".to_string();
        let updated_image_url = vec!["one two three".to_string()];
        let updated_file_url = "updated file url".to_string();
        let updated_mod_type = "mod".to_string();

        let server = TestServer::new(app).unwrap();

        let header_name = HeaderName::from_lowercase(b"authorization").unwrap();
        let header_value =
            HeaderValue::from_str(&format!("Bearer {}", generate_test_jwt_token())).unwrap();
        let response = server
            .put(format!("/api/posts/{}", invalid_id).as_str())
            .content_type(&"application/json")
            .json(&json!({
                "title": updated_title.clone(),
                "content": updated_content.clone(),
                "imagesUrl": updated_image_url.clone(),
                "fileUrl": updated_file_url.clone(),
                "modType": updated_mod_type.clone(),
            }))
            .add_header(header_name, header_value)
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
        let state = create_test_state(test_db);
        let app = app(state);

        let new_post_title = "aa".to_string();
        let new_post_content = "content".to_string();
        let new_post_images_url: Vec<String> = vec![];
        let new_post_file_url = "aa".to_string();
        let new_post_mod_type = "preset".to_string();

        let server = TestServer::new(app).unwrap();

        let header_name = HeaderName::from_lowercase(b"authorization").unwrap();
        let header_value =
            HeaderValue::from_str(&format!("Bearer {}", generate_test_jwt_token())).unwrap();
        let insert_result = server
            .post("/api/posts/create")
            .content_type(&"application/json")
            .json(&json!({
                "title": new_post_title.clone(),
                "content": new_post_content.clone(),
                "imagesUrl": new_post_images_url.clone(),
                "fileUrl": new_post_file_url.clone(),
                "modType": new_post_mod_type.clone(),
            }))
            .add_header(header_name.clone(), header_value.clone())
            .await;

        let inserted_post_id = insert_result.json::<Bson>();
        let object_id = inserted_post_id.as_object_id().unwrap();

        let updated_title = "updated test post".to_string();
        let updated_content = "content".to_string();
        let updated_image_url = vec!["one two three".to_string()];
        let updated_file_url = "updated file url".to_string();
        let updated_mod_type = "mod".to_string();

        let response = server
            .put(format!("/api/posts/{}", object_id.to_hex()).as_str())
            .content_type(&"application/json")
            .json(&json!({
                "title": updated_title.clone(),
                "content": updated_content.clone(),
                "imagesUrl": updated_image_url.clone(),
                "fileUrl": updated_file_url.clone(),
                "modType": updated_mod_type.clone(),
            }))
            .add_header(header_name.clone(), header_value.clone())
            .await;

        response.assert_status_ok();
    }

    #[tokio::test]
    async fn test_delete_post_unauthorized() {
        let docker = clients::Cli::default();
        let port = generate_port_number();
        let mongo_img = get_mongo_image(&port);
        let _c = docker.run(mongo_img);
        let uri = get_db_connection_uri(&port);
        let client = Client::with_uri_str(uri).await.unwrap();

        let test_db = client.database("test_db");
        let state = create_test_state(test_db);
        let app = app(state);

        let server = TestServer::new(app).unwrap();

        let response = server
            .delete(format!("/api/posts/{}", "invalid_id").as_str())
            .await;

        response.assert_status_unauthorized();
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
        let state = create_test_state(test_db);
        let app = app(state);

        let invalid_id = "invalid id";

        let server = TestServer::new(app).unwrap();

        let header_name = HeaderName::from_lowercase(b"authorization").unwrap();
        let header_value =
            HeaderValue::from_str(&format!("Bearer {}", generate_test_jwt_token())).unwrap();
        let response = server
            .delete(format!("/api/posts/{}", invalid_id).as_str())
            .add_header(header_name, header_value)
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
        let state = create_test_state(test_db);
        let app = app(state);

        let invalid_id = "659e79f831f22dc0395699b2";

        let server = TestServer::new(app).unwrap();

        let header_name = HeaderName::from_lowercase(b"authorization").unwrap();
        let header_value =
            HeaderValue::from_str(&format!("Bearer {}", generate_test_jwt_token())).unwrap();
        let response = server
            .delete(format!("/api/posts/{}", invalid_id).as_str())
            .add_header(header_name, header_value)
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
        let state = create_test_state(test_db.clone());
        let app = app(state);

        let new_post_title = "aa".to_string();
        let new_post_content = "content".to_string();
        let new_post_images_url: Vec<String> = vec![];
        let new_post_file_url = "aa".to_string();
        let new_post_mod_type = "preset".to_string();

        let server = TestServer::new(app).unwrap();

        let header_name = HeaderName::from_lowercase(b"authorization").unwrap();
        let header_value =
            HeaderValue::from_str(&format!("Bearer {}", generate_test_jwt_token())).unwrap();
        let insert_result = server
            .post("/api/posts/create")
            .content_type(&"application/json")
            .json(&json!({
                "title": new_post_title.clone(),
                "content": new_post_content.clone(),
                "imagesUrl": new_post_images_url.clone(),
                "fileUrl": new_post_file_url.clone(),
                "modType": new_post_mod_type.clone(),
            }))
            .add_header(header_name.clone(), header_value.clone())
            .await;

        let inserted_post_id = insert_result.json::<Bson>();
        let object_id = inserted_post_id.as_object_id().unwrap();

        let delete_result = server
            .delete(format!("/api/posts/{}", object_id.clone().to_hex()).as_str())
            .add_header(header_name, header_value)
            .await;

        delete_result.assert_status_ok();

        let find_result = find_post_by_id(test_db, object_id).await;

        assert!(find_result.is_none());
    }

    #[tokio::test]
    async fn test_create_new_post_unauthorized() {
        let docker = clients::Cli::default();
        let port = generate_port_number();
        let mongo_img = get_mongo_image(&port);
        let _c = docker.run(mongo_img);
        let uri = get_db_connection_uri(&port);
        let client = Client::with_uri_str(uri).await.unwrap();

        let test_db = client.database("test_db");
        let state = create_test_state(test_db.clone());
        let app = app(state);

        let server = TestServer::new(app).unwrap();

        let insert_result = server
            .post("/api/posts/create")
            .content_type(&"application/json")
            .json(&json!({
                "title": "new_post_title",
                "imagesUrl": [],
                "fileUrl": "new_post_file_url",
            }))
            .await;

        insert_result.assert_status_unauthorized();
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
        let state = create_test_state(test_db.clone());
        let app = app(state);

        let new_post_title = "aa".to_string();
        let new_post_content = "content".to_string();
        let new_post_images_url: Vec<String> = vec![];
        let new_post_file_url = "aa".to_string();
        let new_post_mod_type = "preset".to_string();

        let server = TestServer::new(app).unwrap();

        let header_name = HeaderName::from_lowercase(b"authorization").unwrap();
        let header_value =
            HeaderValue::from_str(&format!("Bearer {}", generate_test_jwt_token())).unwrap();
        let insert_result = server
            .post("/api/posts/create")
            .content_type(&"application/json")
            .json(&json!({
                "title": new_post_title.clone(),
                "content": new_post_content.clone(),
                "imagesUrl": new_post_images_url.clone(),
                "fileUrl": new_post_file_url.clone(),
                "modType": new_post_mod_type.clone(),
            }))
            .add_header(header_name, header_value)
            .await;

        insert_result.assert_status_ok();

        let inserted_post_id = insert_result.json::<Bson>();
        let object_id = inserted_post_id.as_object_id().unwrap();

        let find_result = find_post_by_id(test_db, object_id).await;

        assert!(find_result.is_some());
    }

    #[tokio::test]
    async fn test_delete_all_posts_unauthorized() {
        let docker = clients::Cli::default();
        let port = generate_port_number();
        let mongo_img = get_mongo_image(&port);
        let _c = docker.run(mongo_img);
        populate_test_data(&port);
        let uri = get_db_connection_uri(&port);
        let client = Client::with_uri_str(uri).await.unwrap();

        let test_db = client.database("test_db");
        let state = create_test_state(test_db);
        let app = app(state);

        let server = TestServer::new(app).unwrap();

        let response = server.delete("/api/posts").await;

        response.assert_status_unauthorized();
    }

    #[tokio::test]
    async fn test_delete_all_posts() {
        let docker = clients::Cli::default();
        let port = generate_port_number();
        let mongo_img = get_mongo_image(&port);
        let _c = docker.run(mongo_img);
        populate_test_data(&port);
        let uri = get_db_connection_uri(&port);
        let client = Client::with_uri_str(uri).await.unwrap();

        let test_db = client.database("test_db");
        let state = create_test_state(test_db.clone());
        let app = app(state);

        let server = TestServer::new(app).unwrap();

        let header_name = HeaderName::from_lowercase(b"authorization").unwrap();
        let header_value =
            HeaderValue::from_str(&format!("Bearer {}", generate_test_jwt_token())).unwrap();
        let response = server
            .delete("/api/posts")
            .add_header(header_name, header_value)
            .await;

        response.assert_status_ok();

        let count = count_all_posts(test_db).await;

        assert_eq!(count, 0);
    }
}
