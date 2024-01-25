#[cfg(test)]
pub mod test_util {
    use mongodb::{
        bson::{doc, oid::ObjectId},
        Database,
    };
    use run_script::run_script;
    use std::net::UdpSocket;
    use testcontainers::{GenericImage, RunnableImage};

    use crate::{posts::Post, AppState};

    pub fn generate_port_number() -> u16 {
        let address = "0.0.0.0:0";
        let socket = UdpSocket::bind(address).expect("Cannot bind to socket");
        let local_addr = socket.local_addr().expect("Cannot get local address");
        local_addr.port()
    }

    pub fn get_mongo_image(&port: &u16) -> RunnableImage<GenericImage> {
        let image = GenericImage::new("mongo".to_string(), "5.0.6".to_string());
        RunnableImage::from(image).with_mapped_port((port, 27017))
    }

    pub fn populate_test_data(&port: &u16) {
        let formatted_command =
            format!(r#" bash ./src/test_data/import.sh {} {}"#, "0.0.0.0", port);
        let (code, output, error) =
            run_script!(formatted_command).expect("Cannot seed MongoDB data");
        println!("---run script---");
        println!("code: {}", code);
        println!("error: {}", error);
        println!("output: {}", output);
    }

    pub fn get_db_connection_uri(&port: &u16) -> String {
        format!("mongodb://{}:{}", "0.0.0.0", port)
    }

    pub async fn insert_test_post(db: Database, new_post: Post) -> ObjectId {
        let typed_collection = db.collection::<Post>("posts");

        let insert_result = typed_collection.insert_one(new_post, None).await.unwrap();
        insert_result.inserted_id.as_object_id().unwrap()
    }

    pub async fn find_post_by_id(db: Database, id: ObjectId) -> Option<Post> {
        let typed_collection = db.collection::<Post>("posts");

        let find_result = typed_collection
            .find_one(
                doc! {
                    "_id": id
                },
                None,
            )
            .await;

        find_result.unwrap()
    }

    pub fn create_test_state(mongo: mongodb::Database) -> AppState {
        AppState {
            mongo,
            jwt_key: "jpPZGyeXTRpQ2rfyyCY/a+eStyVkYrlIhCYe69Ul+lw=".to_string()
        }
    }
}
