use crate::redis_pubsub::message::Message;
use crate::redis_pubsub::CHANNEL;
use crate::sync_post::sync_post;
use crate::AppState;
use redis::Commands;
use tracing::debug;

pub fn publish_message(redis: redis::Client, message: Message) -> anyhow::Result<i64> {
    let mut con = redis.get_connection()?;

    let json = serde_json::to_string(&message)?;

    let num: i64 = con.publish(CHANNEL, json)?;
    debug!("Published message. Num of subscriber : {}", num);

    Ok(num)
}

pub fn subscribe(
    state: AppState,
) -> anyhow::Result<()> {
    // let _ = tokio::spawn(async move {
    //     let mut con = redis.get_connection().unwrap();
    //
    //     let _: () = con.subscribe(CHANNEL, |message| {
    //         let received: String = message.get_payload().unwrap();
    //         let message_obj = serde_json::from_str::<Message>(&received).unwrap();
    //
    //         println!("{:?}", message_obj);
    //
    //         return ControlFlow::Continue;
    //     }).unwrap();
    // });
    let mongo = state.mongo;
    let redis = state.redis;
    let patreon_access_token = state.patreon_access_token;

    // TODO: propagate errors
    tokio::spawn(async move {
        let mut con = redis.get_connection().unwrap();
        let mut pubsub = con.as_pubsub();
        pubsub.subscribe(CHANNEL).unwrap();

        loop {
            let msg = pubsub.get_message().unwrap();
            let received: String = msg.get_payload().unwrap();
            let message_obj = serde_json::from_str::<Message>(&received).unwrap_or_default();

            debug!("Message received: {:?}", message_obj);

            if received.eq("Sync") {
                sync_post(mongo.clone(), patreon_access_token.clone()).await;
            }
        }
    });

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::redis_pubsub::message::Message;
    use crate::test_util::test_util::{create_test_state, generate_port_number, get_db_connection_uri, get_mongo_image, get_redis_connection_uri, get_redis_image};
    use mongodb::Client;
    use testcontainers_modules::redis::REDIS_PORT;
    use testcontainers_modules::testcontainers::clients;

    #[tokio::test]
    async fn test_subscribe() {
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

        let result = subscribe(create_test_state(test_db, redis_client));

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_publish_message() {
        let docker = clients::Cli::default();
        let redis_img = get_redis_image();
        let redis_node = docker.run(redis_img);

        let redis_uri = get_redis_connection_uri(&redis_node.get_host_port_ipv4(REDIS_PORT));
        let redis_client = redis::Client::open(redis_uri.as_ref()).unwrap();

        let mut con = redis_client.clone().get_connection().unwrap();
        let mut pubsub = con.as_pubsub();
        pubsub.subscribe(CHANNEL).unwrap();

        let result = publish_message(redis_client, Message::new(String::from("test message")));

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
    }
}
