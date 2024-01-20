use crate::AppState;
use axum::response::IntoResponse;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use futures::stream::TryStreamExt;
use mongodb::bson::{Bson, DateTime};
use mongodb::{bson::doc, options::FindOptions};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};

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
    images_url: Vec<String>,
    file_url: String
}

pub async fn get_all_posts(
    State(state): State<AppState>
) -> Result<Json<Vec<Post>>, impl IntoResponse> {
    let typed_collection = &state.mongo.collection::<Post>("posts");

    let filter = doc! {};
    let find_option = FindOptions::builder().build();

    match typed_collection.find(filter, find_option).await {
        Ok(cursor) => {
            let posts = cursor.try_collect().await.unwrap_or_else(|_| vec![]);
            Ok(Json(posts))
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
    Json(req): Json<NewPostRequest>
) -> Result<Json<Bson>, impl IntoResponse> {
    let typed_collection = &state.mongo.collection::<Post>("posts");

    let new_post = Post {
        title: req.title,
        images_url: req.images_url,
        file_url: req.file_url,
        created_at: DateTime::now(),
        updated_at: DateTime::now(),
    };

    match typed_collection.insert_one(new_post, None).await {
        Ok(result) => {
            info!("New Post Created");
            Ok(Json(result.inserted_id))
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
    use crate::test_util::test_util::{generate_port_number, get_mongo_image, populate_test_data, get_db_connection_uri};
    use testcontainers::clients;
    use mongodb::Client;
    use crate::AppState;
    use serde_json::{json, Value};
    use super::*;

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

        let state = AppState {
            mongo: test_db
        };

        let result = get_all_posts(State(state)).await;

        assert!(result.is_ok());
        // FIXME: how to assert json data
        // let body: Value = serde_json::to_vec(result.unwrap());
        // assert_eq!(result.unwrap()., Json(vec![]));
    }

    #[tokio::test]
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
            mongo: test_db
        };
        let new_post_request = NewPostRequest {
            title: "aa".to_string(),
            images_url: vec![],
            file_url: "aa".to_string(),
        };

        let result = create_new_post(State(state), Json(new_post_request)).await;

        assert!(result.is_ok());
    }
}