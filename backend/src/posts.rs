use std::str::FromStr;
use std::vec;

use crate::AppState;
use axum::response::IntoResponse;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use futures::stream::TryStreamExt;
use mongodb::bson::oid::ObjectId;
use mongodb::bson::{Bson, DateTime, doc};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

#[derive(Deserialize, Serialize, Debug)]
pub struct Post {
    title: String,
    images_url: Vec<String>,
    file_url: String,
    created_at: DateTime,
    updated_at: DateTime,
}

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct NewPostRequest {
    title: String,
    imagesUrl: Vec<String>,
    fileUrl: String,
}

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct EditPostRequest {
    title: String,
    imagesUrl: Vec<String>,
    fileUrl: String,
}

pub async fn get_all_posts(
    State(state): State<AppState>,
) -> Result<Json<Vec<Post>>, impl IntoResponse> {
    let typed_collection = &state.mongo.collection::<Post>("posts");

    // let filter = doc! {};
    // let find_option = FindOptions::builder().build();

    match typed_collection.find(None, None).await {
        Ok(cursor) => {
            // let posts = cursor.try_collect().await.unwrap_or_else(|_| vec![]);
            let posts = cursor.try_collect().await.unwrap_or_else(|err| {
                error!("{}", err.to_string());
                vec![]
            });

            Ok(Json(posts))
        }
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
    let typed_collection = &state.mongo.collection::<Post>("posts");

    let target_post_object_id_result = ObjectId::from_str(&id);

    if target_post_object_id_result.is_err() {
        let error_message = target_post_object_id_result.unwrap_err().to_string();
        error!(error_message);

        return Err((StatusCode::BAD_REQUEST, error_message).into_response());
    }

    let filter = doc! {
        "_id": target_post_object_id_result.unwrap()
    };

    match typed_collection.find_one(filter, None).await {
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
    let typed_collection = &state.mongo.collection::<Post>("posts");

    let new_post = Post {
        title: req.title,
        images_url: req.imagesUrl,
        file_url: req.fileUrl,
        created_at: DateTime::now(),
        updated_at: DateTime::now(),
    };

    match typed_collection.insert_one(new_post, None).await {
        Ok(result) => {
            info!("New Post Created");
            Ok(Json(result.inserted_id))
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
    let typed_collection = &state.mongo.collection::<Post>("posts");

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
        }
    };

    match typed_collection
        .find_one_and_update(filter, update, None)
        .await
    {
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

pub async fn delete_post(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, impl IntoResponse> {
    let typed_collection = &state.mongo.collection::<Post>("posts");

    let target_post_object_id_result = ObjectId::from_str(&id);

    if target_post_object_id_result.is_err() {
        let error_message = target_post_object_id_result.unwrap_err().to_string();
        error!(error_message);

        return Err((StatusCode::BAD_REQUEST, error_message).into_response());
    }

    let filter = doc! {
        "_id": target_post_object_id_result.unwrap()
    };

    match typed_collection.find_one_and_delete(filter, None).await {
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

#[cfg(test)]
use test_env_helpers::*;

#[before_all]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::test_util::{
        generate_port_number, get_db_connection_uri, get_mongo_image, populate_test_data,
    };
    use crate::AppState;
    use mongodb::Client;
    use testcontainers::clients;

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
        let _c = docker.run(mongo_img);
        populate_test_data(&port);
        let uri = get_db_connection_uri(&port);
        let client = Client::with_uri_str(uri).await.unwrap();

        let test_db = client.database("test_db");

        let state = AppState { mongo: test_db };

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
        let _c = docker.run(mongo_img);
        populate_test_data(&port);
        let uri = get_db_connection_uri(&port);
        let client = Client::with_uri_str(uri).await.unwrap();

        let test_db = client.database("test_db");

        let state = AppState { mongo: test_db };

        let result = get_post_by_id(State(state), Path("aaaa".to_string())).await;

        assert!(result.is_err());

        let response = result.unwrap_err();

        assert_eq!(response.into_response().status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_create_new_posts() {
        let docker = clients::Cli::default();
        let port = generate_port_number();
        let mongo_img = get_mongo_image(&port);
        let _c = docker.run(mongo_img);
        populate_test_data(&port);
        let uri = get_db_connection_uri(&port);
        let client = Client::with_uri_str(uri).await.unwrap();

        let test_db = client.database("test_db");

        let state = AppState {
            mongo: test_db.clone(),
        };
        let new_post_title = "aa".to_string();
        let new_post_images_url: Vec<String> = vec![];
        let new_post_file_url = "aa".to_string();

        let new_post_request = NewPostRequest {
            title: new_post_title.clone(),
            imagesUrl: new_post_images_url.clone(),
            fileUrl: new_post_file_url.clone(),
        };

        let result = create_new_post(State(state), Json(new_post_request)).await;
        let inserted_id_json = result.ok().unwrap();
        let inserted_id = inserted_id_json.0.as_object_id().unwrap();

        let typed_collection = test_db.collection::<Post>("posts");

        let new_post = typed_collection
            .find_one(
                doc! {
                    "_id": inserted_id
                },
                None,
            )
            .await
            .unwrap()
            .unwrap();

        assert_eq!(new_post.title, new_post_title);
        assert_eq!(new_post.images_url, new_post_images_url);
        assert_eq!(new_post.file_url, new_post_file_url);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_edit_post() {
        let docker = clients::Cli::default();
        let port = generate_port_number();
        let mongo_img = get_mongo_image(&port);
        let _c = docker.run(mongo_img);
        populate_test_data(&port);
        let uri = get_db_connection_uri(&port);
        let client = Client::with_uri_str(uri).await.unwrap();

        let test_db = client.database("test_db");

        let state = AppState {
            mongo: test_db.clone(),
        };

        let typed_collection = test_db.collection::<Post>("posts");

        let new_post = Post {
            title: "test post".to_string(),
            images_url: vec![],
            file_url: "test file url".to_string(),
            created_at: DateTime::now(),
            updated_at: DateTime::now(),
        };

        let insert_result = typed_collection.insert_one(new_post, None).await.unwrap();
        let updated_title = "updated test post".to_string();
        let updated_image_url = vec!["one two three".to_string()];
        let updated_file_url = "updated file url".to_string();

        let edit_post_request = EditPostRequest {
            title: updated_title.clone(),
            imagesUrl: updated_image_url.clone(),
            fileUrl: updated_file_url.clone(),
        };

        let inserted_post_object_id = insert_result.inserted_id.as_object_id().unwrap();
        let object_id_string = inserted_post_object_id.to_hex();
        let result = edit_post(State(state), Path(object_id_string), Json(edit_post_request)).await;

        assert!(result.is_ok());

        let updated_post = typed_collection
            .find_one(
                doc! {
                    "_id": inserted_post_object_id
                },
                None,
            )
            .await
            .unwrap()
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
        let _c = docker.run(mongo_img);
        let uri = get_db_connection_uri(&port);
        let client = Client::with_uri_str(uri).await.unwrap();

        let test_db = client.database("test_db");

        let state = AppState {
            mongo: test_db.clone(),
        };

        let typed_collection = test_db.collection::<Post>("posts");
        let new_post = Post {
            title: "test post".to_string(),
            images_url: vec![],
            file_url: "test file url".to_string(),
            created_at: DateTime::now(),
            updated_at: DateTime::now(),
        };

        let insert_result = typed_collection.insert_one(new_post, None).await.unwrap();
        let inserted_post_object_id = insert_result.inserted_id.as_object_id().unwrap();
        let object_id_string = inserted_post_object_id.to_hex();
        let result = delete_post(State(state), Path(object_id_string)).await;

        assert!(result.is_ok());

        let find_result = typed_collection.find_one(
            doc! {
                "_id": inserted_post_object_id
            },
            None).await;

        let k = find_result.unwrap();

        assert!(k.is_none());
    }
}
