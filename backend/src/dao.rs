use anyhow::{Error, Result};
use futures::TryStreamExt;
use mongodb::bson::{doc, Bson, Document};
use mongodb::options::FindOneAndUpdateOptions;
use mongodb::options::ReturnDocument::After;
use mongodb::Database;
use serde::Serialize;
use std::any::type_name;
use tracing::{error, info};

// TODO: error logging??
pub fn get_collection_name<T>() -> String {
    let type_name = type_name::<T>();
    let splitted_type_name: Vec<_> = type_name.split("::").collect();
    let collection_name = splitted_type_name.last().unwrap_or(&"Unknown");
    collection_name.to_string()
}

pub async fn get_all_docs<T>(mongo: Database) -> Result<Vec<T>>
where
    T: serde::de::DeserializeOwned,
{
    let typed_collection = mongo.collection::<T>(&*get_collection_name::<T>());

    match typed_collection.find(None, None).await {
        Ok(cursor) => {
            let docs: Vec<T> = cursor.try_collect().await.unwrap_or_else(|err| {
                error!("{}", err.to_string());
                vec![]
            });

            Ok(docs)
        }
        Err(err) => {
            let error_message = err.to_string();
            error!("{}", error_message.clone());
            Err(Error::from(err))
        }
    }
}

pub async fn insert_one_doc<T>(mongo: Database, new_doc: T) -> Result<Bson>
where
    T: Serialize,
{
    let typed_collection = mongo.collection::<T>(&*get_collection_name::<T>());

    match typed_collection.insert_one(new_doc, None).await {
        Ok(result) => {
            let object_id = result.inserted_id.as_object_id().unwrap();
            info!("New Doc Created {}", object_id.to_hex());
            Ok(result.inserted_id)
        }
        Err(err) => {
            let error_message = err.to_string();
            error!("{}", error_message.clone());
            Err(Error::from(err))
        }
    }
}

pub async fn find_one_doc<T>(mongo: Database, filter: Document) -> Result<Option<T>>
where
    T: serde::de::DeserializeOwned + Unpin + Send + Sync,
{
    let typed_collection = mongo.collection::<T>(&*get_collection_name::<T>());

    match typed_collection.find_one(filter, None).await {
        Ok(result) => Ok(result),
        Err(err) => {
            let error_message = err.to_string();
            error!("{}", error_message.clone());
            Err(Error::from(err))
        }
    }
}

pub async fn edit_one_doc<T>(
    mongo: Database,
    filter: Document,
    update: Document,
) -> Result<Option<T>>
where
    T: serde::de::DeserializeOwned,
{
    let typed_collection = mongo.collection::<T>(&*get_collection_name::<T>());

    let options = FindOneAndUpdateOptions::builder()
        .return_document(After)
        .build();

    match typed_collection
        .find_one_and_update(filter, update, options)
        .await
    {
        Ok(result) => Ok(result),
        Err(err) => {
            let error_message = err.to_string();
            error!("{}", error_message.clone());
            Err(Error::from(err))
        }
    }
}

pub async fn delete_one_doc<T>(mongo: Database, filter: Document) -> Result<Option<T>>
    where
        T: serde::de::DeserializeOwned,
{
    let typed_collection = mongo.collection::<T>(&*get_collection_name::<T>());

    match typed_collection.find_one_and_delete(filter, None).await {
        Ok(result) => Ok(result),
        Err(err) => {
            let error_message = err.to_string();
            error!("{}", error_message.clone());
            Err(Error::from(err))
        }
    }
}

pub async fn delete_all_docs<T>(mongo: Database) -> Result<u64> {
    let typed_collection = mongo.collection::<T>(&*get_collection_name::<T>());

    match typed_collection.delete_many(doc! {}, None).await {
        Ok(result) => Ok(result.deleted_count),
        Err(err) => Err(Error::from(err)),
    }
}

#[cfg(test)]
mod tests {
    use crate::dao::{delete_all_docs, delete_one_doc, edit_one_doc, find_one_doc, get_all_docs, get_collection_name, insert_one_doc};
    use crate::posts::Post;
    use crate::sync_job::SyncJob;
    use crate::sync_post::SyncResult;
    use crate::test_util::test_util::{
        generate_port_number, get_db_connection_uri, get_mongo_image, populate_test_data,
    };
    use mongodb::bson::{doc, to_document};
    use mongodb::Client;
    use testcontainers::clients;

    #[test]
    fn test_get_collection_name() {
        assert_eq!(get_collection_name::<Post>(), "Post");
        assert_eq!(get_collection_name::<SyncJob>(), "SyncJob");
        assert_eq!(get_collection_name::<SyncResult>(), "SyncResult");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_get_all_docs() {
        let docker = clients::Cli::default();
        let port = generate_port_number();
        let mongo_img = get_mongo_image(&port);
        let _c = docker.run(mongo_img);
        populate_test_data(&port);
        let uri = get_db_connection_uri(&port);
        let client = Client::with_uri_str(uri).await.unwrap();

        let test_db = client.database("test_db");

        let result = get_all_docs::<Post>(test_db).await;
        assert!(result.is_ok());
        let posts = result.unwrap();
        assert_eq!(posts.len(), 2);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_insert_one_doc() {
        let docker = clients::Cli::default();
        let port = generate_port_number();
        let mongo_img = get_mongo_image(&port);
        let _c = docker.run(mongo_img);
        let uri = get_db_connection_uri(&port);
        let client = Client::with_uri_str(uri).await.unwrap();

        let test_db = client.database("test_db");

        let result = insert_one_doc::<SyncJob>(test_db, SyncJob::new()).await;

        assert!(result.is_ok());

        let inserted_id = result.unwrap();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_find_one_doc() {
        let docker = clients::Cli::default();
        let port = generate_port_number();
        let mongo_img = get_mongo_image(&port);
        let _c = docker.run(mongo_img);
        let uri = get_db_connection_uri(&port);
        let client = Client::with_uri_str(uri).await.unwrap();

        let test_db = client.database("test_db");

        let insert_result = insert_one_doc::<SyncJob>(test_db.clone(), SyncJob::new()).await;

        let inserted_id = insert_result.unwrap();

        let filter = doc! {
            "_id": inserted_id.as_object_id().unwrap()
        };

        let result = find_one_doc::<SyncJob>(test_db, filter).await;

        assert!(result.is_ok());

        let doc = result.unwrap();

        assert!(doc.is_some());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_edit_one_doc() {
        let docker = clients::Cli::default();
        let port = generate_port_number();
        let mongo_img = get_mongo_image(&port);
        let _c = docker.run(mongo_img);
        let uri = get_db_connection_uri(&port);
        let client = Client::with_uri_str(uri).await.unwrap();

        let test_db = client.database("test_db");

        let insert_result = insert_one_doc::<Post>(
            test_db.clone(),
            Post::new_for_sync("asdf", "asdf", "asdf", "asdf"),
        )
        .await;

        let inserted_id = insert_result.unwrap();

        let filter = doc! {
            "_id": inserted_id.as_object_id().unwrap()
        };

        let update = doc! {
            "$set": doc! {
                "title": "test title"
            }
        };

        let result = edit_one_doc::<Post>(test_db, filter, update).await;

        assert!(result.is_ok());

        let doc = result.unwrap();

        assert!(doc.is_some());

        let post = doc.unwrap();
        let post_doc = to_document(&post).unwrap();

        assert_eq!(post_doc.get_str("title").unwrap(), "test title");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_delete_one_doc() {
        let docker = clients::Cli::default();
        let port = generate_port_number();
        let mongo_img = get_mongo_image(&port);
        let _c = docker.run(mongo_img);
        let uri = get_db_connection_uri(&port);
        let client = Client::with_uri_str(uri).await.unwrap();

        let test_db = client.database("test_db");

        let insert_result = insert_one_doc::<Post>(
            test_db.clone(),
            Post::new_for_sync("asdf", "asdf", "asdf", "asdf"),
        )
            .await;

        let inserted_id = insert_result.unwrap();

        let filter = doc! {
            "_id": inserted_id.as_object_id().unwrap()
        };

        let result = delete_one_doc::<Post>(test_db, filter).await;

        assert!(result.is_ok());

        let deleted_post = result.unwrap();

        assert!(deleted_post.is_some());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_delete_all_docs() {
        let docker = clients::Cli::default();
        let port = generate_port_number();
        let mongo_img = get_mongo_image(&port);
        let _c = docker.run(mongo_img);
        populate_test_data(&port);
        let uri = get_db_connection_uri(&port);
        let client = Client::with_uri_str(uri).await.unwrap();

        let test_db = client.database("test_db");

        let result = delete_all_docs::<Post>(test_db).await;

        assert!(result.is_ok());

        let deleted_count = result.unwrap();

        assert_eq!(deleted_count, 2);
    }
}
