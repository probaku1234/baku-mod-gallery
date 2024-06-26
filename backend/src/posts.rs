use std::str::FromStr;
use std::vec;

// use crate::sync_post::sync_post;
use crate::AppState;
use axum::response::IntoResponse;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use mongodb::bson::oid::ObjectId;
use mongodb::bson::serde_helpers::{bson_datetime_as_rfc3339_string, hex_string_as_object_id};
use mongodb::bson::{doc, Bson, DateTime};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Post {
    #[serde(with = "hex_string_as_object_id")]
    _id: String,
    title: String,
    patreon_post_id: String,
    content: String,
    images_url: Vec<String>,
    file_url: String,
    mod_type: String,
    #[serde(with = "bson_datetime_as_rfc3339_string")]
    created_at: DateTime,
    #[serde(with = "bson_datetime_as_rfc3339_string")]
    updated_at: DateTime,
    #[serde(with = "bson_datetime_as_rfc3339_string")]
    synced_at: DateTime,
}

impl Post {
    pub fn new_for_sync(
        patreon_post_id: &str,
        title: &str,
        content: &str,
        date_string: &str,
    ) -> Self {
        Post {
            _id: ObjectId::new().to_hex(),
            patreon_post_id: patreon_post_id.to_string(),
            title: title.to_string(),
            content: content.to_string(),
            images_url: vec![],
            file_url: "".to_string(),
            mod_type: "".to_string(),
            created_at: DateTime::now(),
            updated_at: DateTime::from_chrono(get_chrono_dt_from_string(date_string.to_string())),
            synced_at: DateTime::now(),
        }
    }
}

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct NewPostRequest {
    title: String,
    content: String,
    imagesUrl: Vec<String>,
    fileUrl: String,
    modType: String,
}

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct EditPostRequest {
    title: String,
    content: String,
    imagesUrl: Vec<String>,
    fileUrl: String,
    modType: String,
}

pub async fn get_all_posts(
    State(state): State<AppState>,
) -> Result<Json<Vec<Post>>, impl IntoResponse> {
    match get_all_docs::<Post>(state.mongo).await {
        Ok(posts) => Ok(Json(posts)),
        Err(err) => {
            let error_message = err.to_string();
            error!("{}", error_message.clone());
            Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response())
        }
    }
}

pub async fn get_post_by_id(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Post>, impl IntoResponse> {
    let target_post_object_id_result = ObjectId::from_str(&id);

    if target_post_object_id_result.is_err() {
        let error_message = target_post_object_id_result.unwrap_err().to_string();
        error!(error_message);

        return Err((StatusCode::BAD_REQUEST, error_message).into_response());
    }

    let filter = doc! {
        "_id": target_post_object_id_result.unwrap()
    };

    match find_one_doc::<Post>(state.mongo, filter).await {
        Ok(result) => match result {
            Some(post) => Ok(Json(post)),
            None => {
                error!("The post with id: {} not found!", id);
                Err((
                    StatusCode::NOT_FOUND,
                    format!("The post with id: {} not found!", id),
                )
                    .into_response())
            }
        },
        Err(err) => {
            let error_message = err.to_string();
            error!("{}", error_message.clone());
            Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response())
        }
    }
}

pub async fn create_new_post(
    State(state): State<AppState>,
    Json(req): Json<NewPostRequest>,
) -> Result<Json<Bson>, impl IntoResponse> {
    let new_post = Post {
        _id: ObjectId::new().to_hex(),
        patreon_post_id: "".to_string(),
        title: req.title,
        content: req.content,
        images_url: req.imagesUrl,
        file_url: req.fileUrl,
        mod_type: req.modType,
        created_at: DateTime::now(),
        updated_at: DateTime::now(),
        synced_at: DateTime::now(),
    };

    match insert_one_doc::<Post>(state.mongo, new_post).await {
        Ok(inserted_id) => {
            let object_id = inserted_id.as_object_id().unwrap();
            info!("New Post Created {}", object_id.to_hex());
            Ok(Json(inserted_id))
        }
        Err(err) => {
            let error_message = err.to_string();
            error!("{}", error_message.clone());
            Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response())
        }
    }
}

pub async fn edit_post(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<EditPostRequest>,
) -> Result<Json<Post>, impl IntoResponse> {
    let target_post_object_id_result = ObjectId::from_str(&id);

    if target_post_object_id_result.is_err() {
        let error_message = target_post_object_id_result.unwrap_err().to_string();
        error!(error_message);

        return Err((StatusCode::BAD_REQUEST, error_message).into_response());
    }

    let filter = doc! {
        "_id": target_post_object_id_result.unwrap()
    };
    let update = doc! {
        "$set": doc! {
            "title": req.title,
            "images_url": req.imagesUrl,
            "file_url": req.fileUrl,
            "updated_at": DateTime::now().try_to_rfc3339_string().unwrap()
        },
    };

    match edit_one_doc::<Post>(state.mongo, filter, update).await {
        Ok(result) => match result {
            Some(post) => {
                info!("Post {} edited", post._id);
                Ok(Json(post))
            }
            None => {
                error!("The post with id: {} not found!", id);
                Err((
                    StatusCode::NOT_FOUND,
                    format!("The post with id: {} not found!", id),
                )
                    .into_response())
            }
        },
        Err(err) => {
            let error_message = err.to_string();
            error!("{}", error_message.clone());
            Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response())
        }
    }
}

pub async fn delete_post(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, impl IntoResponse> {
    let target_post_object_id_result = ObjectId::from_str(&id);

    if target_post_object_id_result.is_err() {
        let error_message = target_post_object_id_result.unwrap_err().to_string();
        error!(error_message);

        return Err((StatusCode::BAD_REQUEST, error_message).into_response());
    }

    let filter = doc! {
        "_id": target_post_object_id_result.unwrap()
    };

    match delete_one_doc::<Post>(state.mongo, filter).await {
        Ok(result) => match result {
            Some(_) => Ok(StatusCode::OK),
            None => {
                error!("The post with id: {} not found!", id);
                Err((
                    StatusCode::NOT_FOUND,
                    format!("The post with id: {} not found!", id),
                )
                    .into_response())
            }
        },
        Err(err) => {
            let error_message = err.to_string();
            error!("{}", error_message.clone());
            Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response())
        }
    }
}

pub async fn delete_all_posts(
    State(state): State<AppState>,
) -> Result<StatusCode, impl IntoResponse> {
    match delete_all_docs::<Post>(state.mongo).await {
        Ok(deleted_count) => {
            info!("{} posts deleted", deleted_count);
            Ok(StatusCode::OK)
        }
        Err(err) => {
            let error_message = err.to_string();
            error!("{}", error_message.clone());
            Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response())
        }
    }
}

pub async fn sync_posts(State(state): State<AppState>) -> StatusCode {
    let redis = state.redis.clone();

    let is_running_job_exist_result = check_sync_job_exists(redis.clone());
    if is_running_job_exist_result.is_err() {
        error!("fail to check sync job");
        return StatusCode::OK;
    }

    let is_running_job_exist = is_running_job_exist_result.unwrap();
    if is_running_job_exist {
        info!("there is already running job!");
        return StatusCode::OK;
    }

    // tokio::spawn(async move {
    //     sync_post(x, state.patreon_access_token).await;
    // });
    let publish_result = publish_message(redis, Message::new(String::from("Sync")));

    if publish_result.is_err() {
        error!("Failed to publish message: {}", publish_result.unwrap_err());
    }

    StatusCode::OK
}

use crate::dao::{
    delete_all_docs, delete_one_doc, edit_one_doc, find_one_doc, get_all_docs, insert_one_doc,
};
use crate::redis_pubsub::message::Message;
use crate::redis_pubsub::pubsub::publish_message;
use crate::util::get_chrono_dt_from_string;
#[cfg(test)]
use test_env_helpers::*;
use crate::sync_job::check_sync_job_exists;

#[before_all]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::test_util::{
        count_all_posts, create_test_state, find_post_by_id, generate_port_number,
        get_db_connection_uri, get_mongo_image, get_redis_connection_uri, get_redis_image,
        insert_test_post, populate_test_data,
    };
    use mongodb::Client;
    use testcontainers_modules::{redis::REDIS_PORT, testcontainers::clients};

    async fn before_all() {
        // let docker = clients::Cli::default();
        // let port = generate_port_number();
        // let mongo_img = get_mongo_image(&port);
        // let _c = docker.run(mongo_img);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_get_all_posts() {
        let docker = clients::Cli::default();
        let port = generate_port_number();
        let mongo_img = get_mongo_image(&port);
        let redis_img = get_redis_image();
        let _c = docker.run(mongo_img);
        let redis_node = docker.run(redis_img);
        populate_test_data(&port);
        let uri = get_db_connection_uri(&port);
        let redis_uri = get_redis_connection_uri(&redis_node.get_host_port_ipv4(REDIS_PORT));
        let client = Client::with_uri_str(uri).await.unwrap();
        let redis_client = redis::Client::open(redis_uri.as_ref()).unwrap();

        let test_db = client.database("test_db");

        let state = create_test_state(test_db, redis_client);

        let result = get_all_posts(State(state)).await;

        assert!(result.is_ok());

        // FIXME: currently not working as intended
        // FIXME: how to assert json data
        // let body: Value = serde_json::to_vec(result.unwrap());
        // assert_eq!(result.unwrap()., Json(vec![]));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_get_post_by_id_invalid_id() {
        let docker = clients::Cli::default();
        let port = generate_port_number();
        let mongo_img = get_mongo_image(&port);
        let redis_img = get_redis_image();
        let _c = docker.run(mongo_img);
        let redis_node = docker.run(redis_img);
        populate_test_data(&port);
        let uri = get_db_connection_uri(&port);
        let redis_uri = get_redis_connection_uri(&redis_node.get_host_port_ipv4(REDIS_PORT));
        let client = Client::with_uri_str(uri).await.unwrap();
        let redis_client = redis::Client::open(redis_uri.as_ref()).unwrap();

        let test_db = client.database("test_db");

        let state = create_test_state(test_db, redis_client);

        let result = get_post_by_id(State(state), Path("aaaa".to_string())).await;

        assert!(result.is_err());

        let response = result.unwrap_err();

        assert_eq!(response.into_response().status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_get_post_by_id() {
        let docker = clients::Cli::default();
        let port = generate_port_number();
        let mongo_img = get_mongo_image(&port);
        let redis_img = get_redis_image();
        let _c = docker.run(mongo_img);
        let redis_node = docker.run(redis_img);

        let uri = get_db_connection_uri(&port);
        let redis_uri = get_redis_connection_uri(&redis_node.get_host_port_ipv4(REDIS_PORT));
        let client = Client::with_uri_str(uri).await.unwrap();
        let redis_client = redis::Client::open(redis_uri.as_ref()).unwrap();

        let test_db = client.database("test_db");

        let state = create_test_state(test_db.clone(), redis_client);

        let new_post_title = "aa".to_string();
        let new_post_content = "content".to_string();
        let new_post_images_url: Vec<String> = vec![];
        let new_post_file_url = "aa".to_string();
        let new_post_mod_type = "preset".to_string();

        let new_post = Post {
            _id: ObjectId::new().to_hex(),
            patreon_post_id: "123123".to_string(),
            title: new_post_title.clone(),
            content: new_post_content.clone(),
            images_url: new_post_images_url.clone(),
            file_url: new_post_file_url.clone(),
            mod_type: new_post_mod_type.clone(),
            created_at: DateTime::now(),
            updated_at: DateTime::now(),
            synced_at: DateTime::now(),
        };

        let inserted_post_object_id = insert_test_post(test_db, new_post).await;
        let object_id_string = inserted_post_object_id.to_hex();

        let result = get_post_by_id(State(state), Path(object_id_string)).await;

        assert!(result.is_ok());

        let found_post = result.ok().unwrap().0;

        assert_eq!(found_post.title, new_post_title);
        assert_eq!(found_post.images_url, new_post_images_url);
        assert_eq!(found_post.file_url, new_post_file_url);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_create_new_posts() {
        let docker = clients::Cli::default();
        let port = generate_port_number();
        let mongo_img = get_mongo_image(&port);
        let redis_img = get_redis_image();
        let _c = docker.run(mongo_img);
        let redis_node = docker.run(redis_img);

        let uri = get_db_connection_uri(&port);
        let redis_uri = get_redis_connection_uri(&redis_node.get_host_port_ipv4(REDIS_PORT));
        let client = Client::with_uri_str(uri).await.unwrap();
        let redis_client = redis::Client::open(redis_uri.as_ref()).unwrap();

        let test_db = client.database("test_db");

        let state = create_test_state(test_db.clone(), redis_client);

        let new_post_title = "aa".to_string();
        let new_post_content = "content".to_string();
        let new_post_images_url: Vec<String> = vec![];
        let new_post_file_url = "aa".to_string();
        let new_post_mod_type = "preset".to_string();

        let new_post_request = NewPostRequest {
            title: new_post_title.clone(),
            content: new_post_content.clone(),
            imagesUrl: new_post_images_url.clone(),
            fileUrl: new_post_file_url.clone(),
            modType: new_post_mod_type.clone(),
        };

        let result = create_new_post(State(state), Json(new_post_request)).await;
        let inserted_id_json = result.ok().unwrap();
        let inserted_id = inserted_id_json.0.as_object_id().unwrap();

        let new_post = find_post_by_id(test_db, inserted_id).await.unwrap();

        assert_eq!(new_post.title, new_post_title);
        assert_eq!(new_post.images_url, new_post_images_url);
        assert_eq!(new_post.file_url, new_post_file_url);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_edit_post() {
        let docker = clients::Cli::default();
        let port = generate_port_number();
        let mongo_img = get_mongo_image(&port);
        let redis_img = get_redis_image();
        let _c = docker.run(mongo_img);
        let redis_node = docker.run(redis_img);

        let uri = get_db_connection_uri(&port);
        let redis_uri = get_redis_connection_uri(&redis_node.get_host_port_ipv4(REDIS_PORT));
        let client = Client::with_uri_str(uri).await.unwrap();
        let redis_client = redis::Client::open(redis_uri.as_ref()).unwrap();

        let test_db = client.database("test_db");

        let state = create_test_state(test_db.clone(), redis_client);

        let new_post = Post {
            _id: ObjectId::new().to_hex(),
            patreon_post_id: "123123".to_string(),
            title: "test post".to_string(),
            content: "test content".to_string(),
            images_url: vec![],
            file_url: "test file url".to_string(),
            mod_type: "aaa".to_string(),
            created_at: DateTime::now(),
            updated_at: DateTime::now(),
            synced_at: DateTime::now(),
        };

        let updated_title = "updated test post".to_string();
        let updated_content = "updated content".to_string();
        let updated_image_url = vec!["one two three".to_string()];
        let updated_file_url = "updated file url".to_string();
        let updated_mod_type = "aaaaa".to_string();

        let edit_post_request = EditPostRequest {
            title: updated_title.clone(),
            content: updated_content.clone(),
            imagesUrl: updated_image_url.clone(),
            fileUrl: updated_file_url.clone(),
            modType: updated_mod_type.clone(),
        };

        let inserted_post_object_id = insert_test_post(test_db.clone(), new_post).await;
        let object_id_string = inserted_post_object_id.to_hex();
        let result = edit_post(
            State(state),
            Path(object_id_string),
            Json(edit_post_request),
        )
        .await;

        assert!(result.is_ok());

        let updated_post = find_post_by_id(test_db, inserted_post_object_id)
            .await
            .unwrap();

        assert_eq!(updated_post.title, updated_title);
        assert_eq!(updated_post.images_url, updated_image_url);
        assert_eq!(updated_post.file_url, updated_file_url);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_delete_post() {
        let docker = clients::Cli::default();
        let port = generate_port_number();
        let mongo_img = get_mongo_image(&port);
        let redis_img = get_redis_image();
        let _c = docker.run(mongo_img);
        let redis_node = docker.run(redis_img);

        let uri = get_db_connection_uri(&port);
        let redis_uri = get_redis_connection_uri(&redis_node.get_host_port_ipv4(REDIS_PORT));
        let client = Client::with_uri_str(uri).await.unwrap();
        let redis_client = redis::Client::open(redis_uri.as_ref()).unwrap();

        let test_db = client.database("test_db");

        let state = create_test_state(test_db.clone(), redis_client);

        let new_post = Post {
            _id: ObjectId::new().to_hex(),
            patreon_post_id: "123123".to_string(),
            title: "test post".to_string(),
            content: "test content".to_string(),
            images_url: vec![],
            file_url: "test file url".to_string(),
            mod_type: "aaa".to_string(),
            created_at: DateTime::now(),
            updated_at: DateTime::now(),
            synced_at: DateTime::now(),
        };

        let inserted_post_object_id = insert_test_post(test_db.clone(), new_post).await;
        let object_id_string = inserted_post_object_id.to_hex();
        let result = delete_post(State(state), Path(object_id_string)).await;

        assert!(result.is_ok());

        let find_result = find_post_by_id(test_db, inserted_post_object_id).await;

        assert!(find_result.is_none());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_delete_all_posts() {
        let docker = clients::Cli::default();
        let port = generate_port_number();
        let mongo_img = get_mongo_image(&port);
        let redis_img = get_redis_image();
        let _c = docker.run(mongo_img);
        let redis_node = docker.run(redis_img);

        let uri = get_db_connection_uri(&port);
        let redis_uri = get_redis_connection_uri(&redis_node.get_host_port_ipv4(REDIS_PORT));
        let client = Client::with_uri_str(uri).await.unwrap();
        let redis_client = redis::Client::open(redis_uri.as_ref()).unwrap();

        let test_db = client.database("test_db");

        let state = create_test_state(test_db.clone(), redis_client);

        let result = delete_all_posts(State(state)).await;

        assert!(result.is_ok());

        let count_posts = count_all_posts(test_db).await;

        assert_eq!(count_posts, 0);
    }
}
