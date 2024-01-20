use crate::AppState;
use axum::response::IntoResponse;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use futures::stream::TryStreamExt;
use mongodb::{bson::doc, options::FindOptions};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct Post {
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
        Err(err) => Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response())
    }
}

#[cfg(test)]
mod tests {
    use crate::test_util::test_util::{generate_port_number, get_mongo_image, populate_test_data, get_db_connection_uri};
    use testcontainers::clients;
    use mongodb::Client;
    use crate::AppState;
    use serde_json::{json, Value};
    use super::*;

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

}