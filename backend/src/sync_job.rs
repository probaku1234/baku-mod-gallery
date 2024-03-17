use mongodb::bson::oid::ObjectId;
use mongodb::bson::serde_helpers::{bson_datetime_as_rfc3339_string, hex_string_as_object_id};
use mongodb::bson::{doc, DateTime};
use mongodb::error;
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

pub async fn create_sync_job(mongo: Database) -> Result<String, error::Error> {
    let typed_collection = mongo.collection::<SyncJob>("SyncJob");

    let new_job = SyncJob {
        _id: ObjectId::new().to_hex(),
        started_at: DateTime::now(),
    };

    match typed_collection.insert_one(new_job, None).await {
        Ok(result) => Ok(result.inserted_id.as_object_id().unwrap().to_hex()),
        Err(err) => {
            error!("{}", err.to_string());
            Err(err)
        }
    }
}

pub async fn delete_sync_job(mongo: Database, object_id: String) -> Result<(), error::Error> {
    let typed_collection = mongo.collection::<SyncJob>("SyncJob");

    match typed_collection
        .find_one_and_delete(
            doc! {
                "_id": ObjectId::from_str(&*object_id).unwrap()
            },
            None,
        )
        .await
    {
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
            Err(err)
        }
    }
}

pub async fn count_running_sync_job(mongo: Database) -> Result<u64, error::Error> {
    let typed_collection = mongo.collection::<SyncJob>("SyncJob");

    typed_collection.count_documents(None, None).await
}
