use crate::dao::{count_docs, delete_one_doc, insert_one_doc};
use anyhow::{Error, Result};
use mongodb::bson::oid::ObjectId;
use mongodb::bson::serde_helpers::{bson_datetime_as_rfc3339_string, hex_string_as_object_id};
use mongodb::bson::{doc, DateTime};
use mongodb::Database;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use tracing::{error, info};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SyncJob {
    #[serde(with = "hex_string_as_object_id")]
    _id: String,
    #[serde(with = "bson_datetime_as_rfc3339_string")]
    started_at: DateTime,
}

impl SyncJob {
    pub fn new() -> Self {
        SyncJob {
            _id: ObjectId::new().to_hex(),
            started_at: DateTime::now(),
        }
    }
}

pub async fn create_sync_job(mongo: Database) -> Result<String> {
    let new_job = SyncJob::new();

    match insert_one_doc::<SyncJob>(mongo, new_job).await {
        Ok(inserted_id) => Ok(inserted_id.as_object_id().unwrap().to_hex()),
        Err(err) => {
            error!("{}", err.to_string());
            Err(Error::from(Error::from(err)))
        }
    }
}

pub async fn delete_sync_job(mongo: Database, object_id: String) -> Result<()> {
    let filter = doc! {
        "_id": ObjectId::from_str(&*object_id).unwrap()
    };

    match delete_one_doc::<SyncJob>(mongo, filter).await {
        Ok(result) => {
            match result {
                Some(job) => {
                    info!("Job ID {} deleted", job._id);
                    Ok(())
                }
                None => {
                    // Err(error::Error::from(error::ErrorKind::InvalidArgument { message: "".to_string() }))
                    Ok(())
                }
            }
        }
        Err(err) => {
            error!("{}", err.to_string());
            Err(Error::from(err))
        }
    }
}

pub async fn count_running_sync_job(mongo: Database) -> Result<u64> {
    count_docs::<SyncJob>(mongo).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dao::find_one_doc;
    use crate::test_util::test_util::{
        generate_port_number, get_db_connection_uri, get_mongo_image,
    };
    use mongodb::Client;
    use testcontainers_modules::testcontainers::clients;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_create_sync_job() {
        let docker = clients::Cli::default();
        let port = generate_port_number();
        let mongo_img = get_mongo_image(&port);
        let _c = docker.run(mongo_img);
        let uri = get_db_connection_uri(&port);
        let client = Client::with_uri_str(uri).await.unwrap();

        let test_db = client.database("test_db");

        let result = create_sync_job(test_db).await;

        assert!(result.is_ok());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_delete_sync_job() {
        let docker = clients::Cli::default();
        let port = generate_port_number();
        let mongo_img = get_mongo_image(&port);
        let _c = docker.run(mongo_img);
        let uri = get_db_connection_uri(&port);
        let client = Client::with_uri_str(uri).await.unwrap();

        let test_db = client.database("test_db");

        let insert_result = create_sync_job(test_db.clone()).await;

        let job_id = insert_result.unwrap();

        let result = delete_sync_job(test_db.clone(), job_id.clone()).await;

        assert!(result.is_ok());

        let filter = doc! {
            "_id": ObjectId::from_str(&*job_id).unwrap()
        };

        let find_result = find_one_doc::<SyncJob>(test_db, filter).await;

        assert!(find_result.is_ok());

        assert!(find_result.unwrap().is_none());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_count_running_sync_job() {
        let docker = clients::Cli::default();
        let port = generate_port_number();
        let mongo_img = get_mongo_image(&port);
        let _c = docker.run(mongo_img);
        let uri = get_db_connection_uri(&port);
        let client = Client::with_uri_str(uri).await.unwrap();

        let test_db = client.database("test_db");

        create_sync_job(test_db.clone())
            .await
            .expect("Fail to create SyncJob");
        create_sync_job(test_db.clone())
            .await
            .expect("Fail to create SyncJob");
        create_sync_job(test_db.clone())
            .await
            .expect("Fail to create SyncJob");

        let result = count_running_sync_job(test_db).await;

        assert!(result.is_ok());

        let count = result.unwrap();

        assert_eq!(count, 3);
    }
}
