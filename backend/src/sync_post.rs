use crate::posts::Post;
use crate::sync_job::{create_sync_job, delete_sync_job};
use crate::util::convert_to_rfc3999_string;
use mongodb::bson::oid::ObjectId;
use mongodb::bson::serde_helpers::{bson_datetime_as_rfc3339_string, hex_string_as_object_id};
use mongodb::bson::{doc, DateTime};
use mongodb::Database;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SyncResult {
    #[serde(with = "hex_string_as_object_id")]
    _id: String,
    is_success: bool,
    message: String,
    sync_count: usize,
    #[serde(with = "bson_datetime_as_rfc3339_string")]
    synced_at: DateTime,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct PatreonPostsApiResult {
    data: Vec<PatreonPostsApiPostResult>,
    links: Option<PatreonPostsApiLinksResult>,
    meta: PatreonPostsApiMetaResult,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct PatreonPostsApiLinksResult {
    next: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct PatreonPostsApiMetaResult {
    pagination: PatreonPostsApiPaginationResult,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct PatreonPostsApiPaginationResult {
    cursors: PatreonPostsApiCursorsResult,
    total: usize,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct PatreonPostsApiCursorsResult {
    next: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct PatreonPostsApiPostResult {
    id: String,
    // type: String,
    attributes: PatreonPostsApiPostAttributesResult,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct PatreonPostsApiPostAttributesResult {
    content: String,
    title: String,
    published_at: String,
}

#[derive(Debug, Clone)]
pub struct PatreonPost {
    id: String,
    content: String,
    title: String,
    published_at: String,
}

pub async fn sync_post(mongo: Database, patreon_access_token: String) {
    let create_job_result = create_sync_job(mongo.clone()).await;

    if create_job_result.is_err() {
        let error = create_job_result.err().unwrap();
        error!("fail to create job {}", error.to_string());
        return;
    }
    let job_id = create_job_result.unwrap();

    let sync_count: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));
    let client = reqwest::Client::new();
    let mut next_link = Some(PatreonPostsApiLinksResult {
        next: "https://www.patreon.com/api/oauth2/v2/campaigns/8365446/posts?fields%5Bpost%5D=content,title,published_at".to_string()
    });

    loop {
        let response_result = client
            .get(next_link.unwrap().next)
            .header(
                reqwest::header::AUTHORIZATION,
                format!("Bearer {}", patreon_access_token),
            )
            .send()
            .await;

        if response_result.is_err() {
            let error = response_result.as_ref().err().unwrap();
            save_sync_result(mongo, error.to_string(), sync_count.lock().await.to_owned()).await;
            return;
        }

        let response = response_result.unwrap();
        let data_result: reqwest::Result<PatreonPostsApiResult> = response.json().await;
        if data_result.is_err() {
            let error = data_result.as_ref().err().unwrap();
            save_sync_result(mongo, error.to_string(), sync_count.lock().await.to_owned()).await;
            return;
        }

        let data = data_result.unwrap();

        next_link = data.links;

        // update db
        let patreon_posts: Vec<PatreonPost> = data
            .data
            .into_iter()
            .map(|post| PatreonPost {
                id: post.id,
                content: post.attributes.content,
                title: post.attributes.title,
                published_at: post.attributes.published_at,
            })
            .collect();
        let upsert_result =
            upsert_posts(mongo.clone(), patreon_posts, Arc::clone(&sync_count)).await;

        if !upsert_result {
            return;
        }

        if next_link.is_none() {
            break;
        }
    }

    save_sync_result(
        mongo.clone(),
        "".to_string(),
        sync_count.lock().await.to_owned(),
    )
    .await;

    let delete_job_result = delete_sync_job(mongo, job_id).await;
    if delete_job_result.is_err() {
        let error = delete_job_result.err().unwrap();
        error!("fail to delete job {}", error.to_string());
    }
}

async fn save_sync_result(mongo: Database, message: String, sync_count: usize) {
    let typed_collection = mongo.collection::<SyncResult>("SyncResult");
    let is_success = if message.is_empty() { true } else { false };
    let new_sync_result = SyncResult {
        _id: ObjectId::new().to_hex(),
        is_success,
        message,
        sync_count,
        synced_at: DateTime::now(),
    };

    match typed_collection.insert_one(new_sync_result, None).await {
        Ok(result) => {
            let x = result.inserted_id.as_object_id().unwrap();
            info!("New sync result created {}", x.to_hex());
        }
        Err(err) => {
            error!("{}", err.to_string());
        }
    }
}

async fn upsert_posts(
    mongo: Database,
    mut patreon_posts: Vec<PatreonPost>,
    sync_count: Arc<Mutex<usize>>,
) -> bool {
    let typed_collection = mongo.collection::<Post>("posts");
    let new_posts = Arc::new(Mutex::new(vec![]));
    let is_update_success = Arc::new(Mutex::new(true));

    while let Some(patreon_post) = patreon_posts.pop() {
        let typed_collection = typed_collection.clone();
        let new_posts = Arc::clone(&new_posts);
        let is_update_success = Arc::clone(&is_update_success);
        let sync_count = Arc::clone(&sync_count);
        let mongo = mongo.clone();

        let x = patreon_post.published_at.clone();

        match typed_collection
            .find_one_and_update(
                doc! { "patreon_post_id": &patreon_post.id },
                doc! {
                    "$set": doc! {
                        "title": &patreon_post.title,
                        "content": &patreon_post.content,
                        "synced_at": convert_to_rfc3999_string(x),
                    }
                },
                None,
            )
            .await
        {
            Ok(result) => match result {
                Some(_) => {
                    info!("Post {} synced", &patreon_post.id);
                    let mut sync_count_lock = sync_count.lock().await;
                    *sync_count_lock += 1;
                }
                None => {
                    let mut new_posts = new_posts.lock().await;
                    let new_post = Post::new_for_sync(
                        &patreon_post.id,
                        &patreon_post.title,
                        &patreon_post.content,
                        &patreon_post.published_at,
                    );
                    new_posts.push(new_post);
                }
            },
            Err(err) => {
                save_sync_result(
                    mongo.clone(),
                    err.to_string(),
                    sync_count.lock().await.to_owned(),
                )
                .await;
                let mut is_update_success = is_update_success.lock().await;
                *is_update_success = false;
                break;
            }
        }
    }

    let is_update_success = is_update_success.lock().await;
    if !*is_update_success {
        return false;
    }

    let new_posts = new_posts.lock().await;
    if !new_posts.is_empty() {
        let x = new_posts.to_vec();
        return insert_posts(mongo.clone(), x, Arc::clone(&sync_count)).await;
    }

    true
}

async fn insert_posts(
    mongo: Database,
    new_posts: Vec<Post>,
    sync_count: Arc<Mutex<usize>>,
) -> bool {
    let typed_collection = mongo.collection::<Post>("posts");

    match typed_collection.insert_many(new_posts, None).await {
        Ok(result) => {
            let inserted_count = result.inserted_ids.len();
            let mut sync_count_lock = sync_count.lock().await;
            *sync_count_lock += inserted_count;
            info!("{} Posts Created during sync", result.inserted_ids.len());
            true
        }
        Err(err) => {
            save_sync_result(mongo, err.to_string(), sync_count.lock().await.to_owned()).await;
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::prelude::*;
    use mongodb::bson;

    #[test]
    fn pika() {
        let chrono_dt: DateTime<Utc> = "2022-03-14T05:23:49.000+00:00".parse().unwrap();
        let bson_dt = bson::DateTime::from_chrono(chrono_dt);
        println!("{}", bson_dt.try_to_rfc3339_string().unwrap());
    }
}
